//! miUPV - CLI tool for Universitat Politècnica de València services.

mod auth_helper;
mod commands;
mod ui;

use clap::{CommandFactory, Parser, Subcommand};
use commands::{
    auth, avisos, cabinas, calendario, carnet, completions, deportes, login, logout, notas,
    perfil, tareas, tutorias, wol,
};

#[derive(Parser, Debug)]
#[command(
    name = "miupv",
    version,
    about = "Herramienta de línea de comandos para los servicios de la Universitat Politècnica de València (UPV)",
    long_about = "miUPV CLI permite consultar notas, tareas de PoliformaT, avisos, carnet virtual con QR, reservar cabinas de estudio, pistas de deportes y encender equipos remotos por Wake-on-LAN."
)]
struct Cli {
    /// Formato de salida en JSON (para scripts e integraciones)
    #[arg(short, long, global = true)]
    json: bool,

    /// Idioma preferido para las respuestas del servidor (es, ca, en)
    #[arg(short, long, global = true, default_value = "es")]
    lang: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inicia sesión con tus credenciales UPV (Intranet CAS o token de app oficial)
    Login {
        /// Nombre de usuario o DNI
        #[arg(short, long)]
        username: Option<String>,

        /// Contraseña institucional
        #[arg(short, long)]
        password: Option<String>,

        /// Iniciar sesión pasando directamente un token extraído de la app oficial o web
        #[arg(short, long)]
        token: Option<String>,

        /// Forzar autenticación vía OAuth PKCE oficial (siupv.upv.es)
        #[arg(long)]
        pkce: bool,
    },

    /// Gestión directa de autenticación y tokens (set-token, status, clear)
    Auth {
        #[command(subcommand)]
        action: auth::AuthAction,
    },

    /// Cierra la sesión activa y elimina las credenciales almacenadas
    Logout,

    /// Muestra la información personal del usuario autenticado
    Perfil,

    /// Consulta las calificaciones académicas de las asignaturas
    Notas {
        /// Año académico a consultar (ej: 2024 para el curso 2024-2025)
        #[arg(short, long)]
        curso: Option<String>,

        /// Formato de salida en JSON
        #[arg(long)]
        json: bool,
    },

    /// Consulta las tareas y entregas de PoliformaT (Sakai)
    Tareas {
        /// Filtrar por asignatura o identificador de sitio
        #[arg(short, long)]
        site: Option<String>,

        /// Incluir tareas ya entregadas o calificadas
        #[arg(short, long)]
        all: bool,
    },

    /// Consulta los avisos y comunicaciones oficiales de escuelas y facultades
    Avisos {
        /// Número máximo de avisos a mostrar
        #[arg(short = 'n', long, default_value = "20")]
        n: u32,
    },

    /// Gestiona y visualiza interactivamente (TUI) el calendario o exporta a .ics
    Calendario {
        #[command(subcommand)]
        action: Option<calendario::CalendarioAction>,
    },

    /// Consulta y reserva cabinas de estudio en bibliotecas (Biblioteca Central, etc.)
    Cabinas {
        /// Fecha a consultar (YYYY-MM-DD, por defecto hoy)
        #[arg(short, long)]
        fecha: Option<String>,

        #[command(subcommand)]
        action: Option<cabinas::CabinasAction>,
    },

    /// Consulta y reserva instalaciones deportivas (Servei d'Esports)
    Deportes {
        #[command(subcommand)]
        action: deportes::DeportesAction,
    },

    /// Consulta tutorías docentes y asignaturas disponibles
    #[command(name = "tutorias", alias = "tutorías")]
    Tutorias {
        #[command(subcommand)]
        action: Option<tutorias::TutoriasAction>,
    },

    /// Carnet universitario virtual (TUI) y código QR para tornos
    Carnet {
        #[command(subcommand)]
        action: carnet::CarnetAction,
    },

    /// Encendido remoto de ordenadores de laboratorio por Wake-on-LAN
    Wol {
        #[command(subcommand)]
        action: wol::WolAction,
    },

    /// Genera o instala el autocompletado en terminal (Tab)
    Completions {
        /// Shell para la que generar el autocompletado (bash, zsh, fish, powershell, elvish)
        #[arg(value_enum)]
        shell: Option<clap_complete::Shell>,

        /// Instalar automáticamente en los directorios de autocompletado del sistema
        #[arg(short, long)]
        install: bool,
    },
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_target(false)
        .init();

    let cli = Cli::parse();
    if let Err(_err) = run_app(cli).await {
        std::process::exit(1);
    }
}

async fn run_app(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let global_json = cli.json;
    let lang = &cli.lang;

    match cli.command {
        Commands::Login {
            username,
            password,
            token,
            pkce,
        } => {
            login::run(username, password, token, pkce).await?;
        }
        Commands::Auth { action } => {
            auth::run(action).await?;
        }
        Commands::Logout => {
            logout::run().await?;
        }
        Commands::Perfil => {
            perfil::run(global_json, lang).await?;
        }
        Commands::Notas { curso, json } => {
            notas::run(curso, global_json || json, lang).await?;
        }
        Commands::Tareas { site, all } => {
            tareas::run(site, all, global_json).await?;
        }
        Commands::Avisos { n } => {
            avisos::run(n, global_json, lang).await?;
        }
        Commands::Calendario { action } => {
            calendario::run(action, global_json, lang).await?;
        }
        Commands::Cabinas { fecha, action } => {
            cabinas::run(fecha, action, global_json).await?;
        }
        Commands::Deportes { action } => {
            deportes::run(action, global_json, lang).await?;
        }
        Commands::Tutorias { action } => {
            tutorias::run(action, global_json, lang).await?;
        }
        Commands::Carnet { action } => {
            carnet::run(action, global_json, lang).await?;
        }
        Commands::Wol { action } => {
            wol::run(action, global_json, lang).await?;
        }
        Commands::Completions { shell, install } => {
            completions::run(Cli::command(), shell, install)?;
        }
    }

    Ok(())
}
