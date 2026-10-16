//! Shell tab-completion generation and installation command.

use crate::ui;
use clap::Command;
use clap_complete::{generate, Shell};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Detects the current user shell from the environment or $SHELL variable.
pub fn detect_shell() -> Shell {
    if let Ok(shell_path) = env::var("SHELL") {
        let lower = shell_path.to_lowercase();
        if lower.contains("zsh") {
            return Shell::Zsh;
        } else if lower.contains("bash") {
            return Shell::Bash;
        } else if lower.contains("fish") {
            return Shell::Fish;
        } else if lower.contains("elvish") {
            return Shell::Elvish;
        } else if lower.contains("powershell") || lower.contains("pwsh") {
            return Shell::PowerShell;
        }
    }
    Shell::Zsh
}

/// Runs the completions command to generate or install completion scripts.
pub fn run(
    mut cmd: Command,
    shell_opt: Option<Shell>,
    install: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let target_shell = shell_opt.unwrap_or_else(detect_shell);

    if !install {
        // Output raw completion script to stdout (for piping / redirection)
        generate(target_shell, &mut cmd, "miupv", &mut io::stdout());
        return Ok(());
    }

    // Auto-install mode
    let home = env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| "HOME environment variable not set")?;
    let mut script_bytes = Vec::new();
    generate(target_shell, &mut cmd, "miupv", &mut script_bytes);

    match target_shell {
        Shell::Zsh => {
            install_zsh(&home, &script_bytes)?;
        }
        Shell::Bash => {
            install_bash(&home, &script_bytes)?;
        }
        Shell::Fish => {
            install_fish(&home, &script_bytes)?;
        }
        Shell::PowerShell => {
            install_powershell(&home, &script_bytes)?;
        }
        Shell::Elvish => {
            install_elvish(&home, &script_bytes)?;
        }
        _ => {
            println!(
                "{}",
                ui::warning("Instalación automática no soportada para este tipo de shell.")
            );
            println!(
                "Puedes redirigir la salida manualmente: `miupv completions <shell> > archivo`"
            );
        }
    }

    Ok(())
}

fn install_zsh(home: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut installed_paths = Vec::new();

    // 1. Oh-My-Zsh directory (if oh-my-zsh exists or is configured in fpath)
    let omz_completions = home.join(".oh-my-zsh/completions");
    let omz_custom_completions = home.join(".oh-my-zsh/custom/completions");

    if home.join(".oh-my-zsh").is_dir() {
        let target_dir = if omz_custom_completions.exists() {
            omz_custom_completions
        } else {
            omz_completions
        };
        fs::create_dir_all(&target_dir)?;
        let file_path = target_dir.join("_miupv");
        fs::write(&file_path, bytes)?;
        installed_paths.push(file_path);
    }

    // 2. Standard ~/.zfunc directory
    let zfunc = home.join(".zfunc");
    fs::create_dir_all(&zfunc)?;
    let zfunc_path = zfunc.join("_miupv");
    fs::write(&zfunc_path, bytes)?;
    installed_paths.push(zfunc_path);

    // 3. Clear zcompdump cache so changes take effect immediately
    if let Ok(entries) = fs::read_dir(home) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(".zcompdump") {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    println!(
        "{}",
        ui::success("Autocompletado para Zsh instalado con éxito en:")
    );
    for p in installed_paths {
        println!("   {}", console::style(p.display()).cyan());
    }

    println!(
        "\n{}",
        console::style("Para activarlo en tu sesión actual de terminal, ejecuta:").bold()
    );
    println!("   {}", console::style("source ~/.zshrc").green().bold());
    println!(
        "   {} (o simplemente abre una nueva pestaña de terminal)\n",
        console::style("o").dim()
    );
    println!(
        "{}",
        ui::info("Prueba pulsando Tab: `miupv <Tab>` o `miupv deportes <Tab>`")
    );

    Ok(())
}

fn install_bash(home: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = home.join(".local/share/bash-completion/completions");
    fs::create_dir_all(&target_dir)?;
    let file_path = target_dir.join("miupv");
    fs::write(&file_path, bytes)?;

    println!(
        "{}",
        ui::success("Autocompletado para Bash instalado con éxito en:")
    );
    println!("   {}", console::style(file_path.display()).cyan());
    println!(
        "\n{}",
        console::style("Para activarlo en tu terminal, ejecuta:").bold()
    );
    println!("   source ~/.local/share/bash-completion/completions/miupv\n");

    Ok(())
}

fn install_fish(home: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = home.join(".config/fish/completions");
    fs::create_dir_all(&target_dir)?;
    let file_path = target_dir.join("miupv.fish");
    fs::write(&file_path, bytes)?;

    println!(
        "{}",
        ui::success("Autocompletado para Fish instalado con éxito en:")
    );
    println!("   {}", console::style(file_path.display()).cyan());
    println!("Fish cargará el autocompletado automáticamente.\n");

    Ok(())
}

fn install_powershell(home: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = home.join(".config/powershell");
    fs::create_dir_all(&target_dir)?;
    let file_path = target_dir.join("miupv.ps1");
    fs::write(&file_path, bytes)?;

    println!(
        "{}",
        ui::success("Script de autocompletado para PowerShell guardado en:")
    );
    println!("   {}", console::style(file_path.display()).cyan());
    println!("Añade `.` seguido de la ruta en tu `$PROFILE`.\n");

    Ok(())
}

fn install_elvish(home: &Path, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = home.join(".config/elvish/lib");
    fs::create_dir_all(&target_dir)?;
    let file_path = target_dir.join("miupv.elv");
    fs::write(&file_path, bytes)?;

    println!(
        "{}",
        ui::success("Script de autocompletado para Elvish guardado en:")
    );
    println!("   {}", console::style(file_path.display()).cyan());

    Ok(())
}
