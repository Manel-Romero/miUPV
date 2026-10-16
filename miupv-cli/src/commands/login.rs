//! Login command handler.

use crate::ui;
use console::Term;
use miupv_core::services::auth_service;
use std::io::{self, Write};

pub async fn run(
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    pkce: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Direct token injection from official app or external extraction
    if let Some(t) = token {
        match auth_service::set_manual_token(&t, None, None) {
            Ok(tokens) => {
                let tipo = if tokens.is_intranet_session() {
                    "Sesión Intranet UPV (Cookie TDp)"
                } else {
                    "Bearer Token OAuth 2.0 (App Oficial)"
                };
                println!(
                    "{}",
                    ui::success(&format!(
                        "Token guardado correctamente.\nTipo detectado: {tipo}"
                    ))
                );
                return Ok(());
            }
            Err(e) => {
                eprintln!("{}", ui::error(&format!("Error al registrar token: {e}")));
                return Ok(());
            }
        }
    }

    let term = Term::stdout();

    let user = match username {
        Some(u) => u,
        None => {
            print!("Usuario / DNI: ");
            io::stdout().flush()?;
            term.read_line()?.trim().to_string()
        }
    };

    let pass = match password {
        Some(p) => p,
        None => {
            print!("Contraseña: ");
            io::stdout().flush()?;
            term.read_secure_line()?.trim().to_string()
        }
    };

    if user.is_empty() || pass.is_empty() {
        eprintln!("{}", ui::error("El usuario y la contraseña no pueden estar vacíos."));
        return Ok(());
    }

    if pkce || user.chars().any(|c| c.is_ascii_digit()) {
        let spinner = ui::create_spinner("Iniciando sesión en la UPV vía OAuth 2.0 oficial...");
        match auth_service::login_pkce(&user, &pass).await {
            Ok(tokens) => {
                let _ = auth_service::save_credentials(&user, &pass);
                if let Err(e) = auth_service::save_tokens(&tokens) {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::warning(&format!("Sesión iniciada, pero hubo un error guardando en el keyring: {e}")));
                } else {
                    spinner.finish_and_clear();
                    println!("{}", ui::success("Sesión iniciada correctamente vía OAuth 2.0 (App Oficial UPV)."));
                    println!("{}", ui::info("Credenciales guardadas con seguridad para autorrenovación permanente de tokens."));
                }
                return Ok(());
            }
            Err(err) => {
                spinner.finish_and_clear();
                if pkce {
                    eprintln!("{}", ui::error(&format!("Error de autenticación PKCE: {err}")));
                    return Ok(());
                }
                tracing::warn!("Intento OAuth falló ({err}), probando portal Intranet CAS...");
            }
        }
    }

    let spinner = ui::create_spinner("Iniciando sesión en la UPV vía Intranet CAS SSO...");
    match auth_service::login_intranet(&user, &pass).await {
        Ok(tokens) => {
            let _ = auth_service::save_credentials(&user, &pass);
            if let Err(e) = auth_service::save_tokens(&tokens) {
                spinner.finish_and_clear();
                eprintln!("{}", ui::warning(&format!("Sesión iniciada, pero hubo un error guardando en el keyring: {e}")));
            } else {
                spinner.finish_and_clear();
                println!("{}", ui::success("Sesión iniciada correctamente en la Intranet UPV. Cookie de sesión capturada."));
            }
        }
        Err(err) => {
            spinner.finish_and_clear();
            eprintln!("{}", ui::error(&format!("Error de autenticación Intranet: {err}")));
        }
    }

    Ok(())
}
