//! Sports facility bookings command handler (Servei d'Esports UPV).

use crate::auth_helper;
use crate::ui;
use chrono::Local;
use clap::Subcommand;
use comfy_table::{Cell, Color};
use miupv_core::services::sports_service;

#[derive(Subcommand, Debug)]
pub enum DeportesAction {
    /// Muestra las reservas deportivas activas del usuario
    MisReservas,

    /// Lista las disciplinas deportivas disponibles en un campus
    Lista {
        /// Campus universitario (ej: vera, gandia, alcoi)
        #[arg(short, long, default_value = "vera")]
        campus: String,
    },

    /// Consulta pistas e instalaciones deportivas disponibles
    Disponibles {
        /// Campus universitario (ej: vera, gandia, alcoi)
        #[arg(short, long, default_value = "vera")]
        campus: String,

        /// Modalidad deportiva (ej: baloncesto, padel, tenis, futbol, o ID numérico)
        #[arg(short, long)]
        deporte: String,

        /// Fecha a consultar (YYYY-MM-DD o YYYYMMDD, por defecto hoy)
        #[arg(short, long)]
        fecha: Option<String>,
    },
}

pub async fn run(
    action: DeportesAction,
    json: bool,
    lang: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    match action {
        DeportesAction::MisReservas => {
            let spinner = ui::create_spinner_opt("Consultando tus reservas deportivas...", !json);
            let reservas = match sports_service::get_mis_reservas(&token, lang).await {
                Ok(r) => {
                    spinner.finish_and_clear();
                    r
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al obtener reservas deportivas: {err}")));
                    return Ok(());
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&reservas)?);
                return Ok(());
            }

            if reservas.is_empty() {
                println!("{}", ui::info("No tienes reservas de instalaciones deportivas actualmente."));
                return Ok(());
            }

            println!("{}", console::style("Tus reservas deportivas:").bold().underlined());

            let mut table = ui::create_table(&["ID", "Campus", "Deporte", "Pista", "Fecha", "Horario", "Estado"]);

            for r in &reservas {
                let horario = match (&r.hora_ini, &r.hora_fin) {
                    (Some(ini), Some(fin)) => format!("{ini} - {fin}"),
                    (Some(ini), None) => ini.to_string(),
                    _ => "-".to_string(),
                };

                let estado_cell = match r.estado.as_deref() {
                    Some(s) if s.to_uppercase().contains("CONF") => Cell::new(s).fg(Color::Green),
                    Some(s) => Cell::new(s).fg(Color::Yellow),
                    None => Cell::new("CONFIRMADA").fg(Color::Green),
                };

                table.add_row(vec![
                    Cell::new(r.id.as_deref().unwrap_or("-")),
                    Cell::new(r.campus.as_deref().unwrap_or("-")),
                    Cell::new(r.deporte.as_deref().unwrap_or("-")),
                    Cell::new(r.pista.as_deref().unwrap_or("-")),
                    Cell::new(r.fecha.as_deref().unwrap_or("-")),
                    Cell::new(&horario),
                    estado_cell,
                ]);
            }

            println!("{table}");
        }

        DeportesAction::Lista { campus } => {
            let spinner = ui::create_spinner_opt("Consultando catálogo de deportes...", !json);
            let deportes = match sports_service::get_deportes(&token, &campus, lang).await {
                Ok(d) => {
                    spinner.finish_and_clear();
                    d
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al obtener lista de deportes: {err}")));
                    return Ok(());
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&deportes)?);
                return Ok(());
            }

            if deportes.is_empty() {
                println!("{}", ui::info(&format!("No se encontraron deportes para el campus {campus}.")));
                return Ok(());
            }

            println!(
                "{}",
                console::style(format!("Deportes e instalaciones UPV en campus {campus}:"))
                    .bold()
                    .underlined()
            );

            let mut table = ui::create_table(&["ID", "Deporte"]);
            for d in &deportes {
                table.add_row(vec![
                    Cell::new(&d.id),
                    Cell::new(&d.nombre),
                ]);
            }

            println!("{table}");
        }

        DeportesAction::Disponibles { campus, deporte, fecha } => {
            let fecha_str = fecha.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
            let spinner = ui::create_spinner_opt("Consultando pistas disponibles...", !json);

            let disponibles = match sports_service::get_reservas_disponibles(&token, &campus, &deporte, &fecha_str, lang).await {
                Ok(d) => {
                    spinner.finish_and_clear();
                    d
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al consultar disponibilidad: {err}")));
                    return Ok(());
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&disponibles)?);
                return Ok(());
            }

            if disponibles.is_empty() {
                println!("{}", ui::info(&format!("No hay pistas disponibles de {deporte} en campus {campus} para el {fecha_str}.")));
                return Ok(());
            }

            println!(
                "{}",
                console::style(format!("Disponibilidad de {deporte} ({campus}) para el {fecha_str}:"))
                    .bold()
                    .underlined()
            );

            let mut table = ui::create_table(&["ID Pista", "Instalación", "Fecha", "Horario", "Estado"]);

            for d in &disponibles {
                let horario = match (&d.hora_ini, &d.hora_fin) {
                    (Some(ini), Some(fin)) => format!("{ini} - {fin}"),
                    (Some(ini), None) => ini.to_string(),
                    _ => "-".to_string(),
                };

                let estado_cell = match d.estado.as_deref() {
                    Some("DISPONIBLE") => Cell::new("DISPONIBLE").fg(Color::Green),
                    Some("OCUPADO") => Cell::new("OCUPADO").fg(Color::Red),
                    Some(s) => Cell::new(s).fg(Color::Yellow),
                    None => Cell::new("DISPONIBLE").fg(Color::Green),
                };

                table.add_row(vec![
                    Cell::new(d.id.as_deref().unwrap_or("-")),
                    Cell::new(d.pista.as_deref().unwrap_or("-")),
                    Cell::new(d.fecha.as_deref().unwrap_or(&fecha_str)),
                    Cell::new(&horario),
                    estado_cell,
                ]);
            }

            println!("{table}");
        }
    }

    Ok(())
}
