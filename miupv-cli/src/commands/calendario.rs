//! Calendar and iCal export command handler with interactive TUI.

use crate::auth_helper;
use crate::commands::calendario_tui;
use crate::ui;
use chrono::{Duration, Local};
use clap::Subcommand;
use miupv_core::services::calendar_service;

#[derive(Subcommand, Debug)]
pub enum CalendarioAction {
    /// Abre la interfaz interactiva (TUI) del calendario académico
    Ver,

    /// Exporta eventos académicos a un archivo .ics estándar
    Export {
        /// Ruta del archivo .ics de salida
        #[arg(short, long, default_value = "horario.ics")]
        out: String,

        /// Fecha de inicio (YYYY-MM-DD, por defecto 30 días antes de hoy)
        #[arg(long)]
        ini: Option<String>,

        /// Fecha de fin (YYYY-MM-DD, por defecto 120 días después de hoy)
        #[arg(long)]
        fin: Option<String>,
    },
}

pub async fn run(
    action: Option<CalendarioAction>,
    json: bool,
    lang: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    if json {
        // Direct JSON dump: use cache or fetch
        let today = Local::now().date_naive();
        let fecha_ini = (today - Duration::days(30)).format("%Y-%m-%d").to_string();
        let fecha_fin = (today + Duration::days(120)).format("%Y-%m-%d").to_string();

        let events = match calendar_service::load_cached_calendar() {
            Some(cache) => cache.events,
            None => {
                let ev = calendar_service::get_ical_events(&token, &fecha_ini, &fecha_fin, lang).await?;
                let _ = calendar_service::save_cached_calendar(&ev);
                ev
            }
        };

        println!("{}", serde_json::to_string_pretty(&events)?);
        return Ok(());
    }

    match action {
        None | Some(CalendarioAction::Ver) => {
            // Interactive Ratatui TUI with Stale-While-Revalidate caching
            calendario_tui::run_calendar_tui(&token, lang).await?;
        }
        Some(CalendarioAction::Export { out, ini, fin }) => {
            let now = Local::now().date_naive();
            let fecha_ini = ini.unwrap_or_else(|| (now - Duration::days(30)).format("%Y-%m-%d").to_string());
            let fecha_fin = fin.unwrap_or_else(|| (now + Duration::days(120)).format("%Y-%m-%d").to_string());

            let spinner = ui::create_spinner("Descargando eventos del calendario...");
            let events = match calendar_service::get_ical_events(&token, &fecha_ini, &fecha_fin, lang).await {
                Ok(ev) => {
                    spinner.finish_and_clear();
                    let _ = calendar_service::save_cached_calendar(&ev);
                    ev
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al obtener eventos del calendario: {err}")));
                    return Ok(());
                }
            };

            let count = events.len();
            calendar_service::export_to_ics(&events, &out)?;
            println!(
                "{}",
                ui::success(&format!(
                    "Se han exportado {count} eventos académicos al archivo '{out}' correctamente."
                ))
            );
        }
    }

    Ok(())
}
