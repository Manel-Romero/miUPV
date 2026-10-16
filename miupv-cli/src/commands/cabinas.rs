//! Library study cabin commands handler (Biblioteca UPV).

use crate::auth_helper;
use crate::ui;
use chrono::Local;
use clap::Subcommand;
use comfy_table::{Cell, Color};
use miupv_core::services::{cabinas_service, user_service};

#[derive(Subcommand, Debug)]
pub enum CabinasAction {
    /// Reserva una franja de cabina por su identificador
    Reservar {
        /// Identificador de la franja a reservar
        #[arg(short, long)]
        id: String,
    },

    /// Muestra las reservas de cabina activas del usuario
    MisReservas,

    /// Cancela o libera una reserva de cabina activa
    Liberar {
        /// Identificador de la franja o reserva a cancelar
        #[arg(short, long)]
        id: String,
    },
}

pub async fn run(
    fecha_opt: Option<String>,
    action: Option<CabinasAction>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    match action {
        None => {
            // Default: list available cabins
            let fecha = fecha_opt.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
            let spinner = ui::create_spinner("Consultando cabinas libres en biblioteca...");
            let cabinas = match cabinas_service::get_cabinas_disponibles(&token, &fecha).await {
                Ok(c) => {
                    spinner.finish_and_clear();
                    c
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al consultar cabinas: {err}")));
                    return Ok(());
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&cabinas)?);
                return Ok(());
            }

            if cabinas.is_empty() {
                println!("{}", ui::info(&format!("No hay cabinas libres disponibles para la fecha {fecha}.")));
                return Ok(());
            }

            println!("{}", console::style(format!("Cabinas disponibles para el {fecha}:")).bold().underlined());

            let mut table = ui::create_table(&[
                "ID Franja",
                "Espacio / Cabina",
                "Biblioteca",
                "Horario",
                "Aforo Máx.",
                "Estado",
            ]);

            for c in &cabinas {
                let horario = match (&c.hora_ini, &c.hora_fin) {
                    (Some(ini), Some(fin)) => format!("{ini} - {fin}"),
                    (Some(ini), None) => ini.to_string(),
                    _ => "-".to_string(),
                };

                table.add_row(vec![
                    Cell::new(&c.id),
                    Cell::new(&c.nombre),
                    Cell::new(c.biblioteca.as_deref().unwrap_or("-")),
                    Cell::new(&horario),
                    Cell::new(c.max_personas.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string())),
                    Cell::new("LIBRE").fg(Color::Green),
                ]);
            }

            println!("{table}");
        }

        Some(CabinasAction::Reservar { id }) => {
            let spinner = ui::create_spinner("Efectuando reserva de cabina...");
            match cabinas_service::reservar_cabina(&token, &id).await {
                Ok(res) => {
                    spinner.finish_and_clear();
                    println!("{}", ui::success(&format!("Reserva confirmada con éxito. ID de Reserva: {}", res.id)));
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("No se pudo reservar la cabina: {err}")));
                }
            }
        }

        Some(CabinasAction::MisReservas) => {
            let spinner = ui::create_spinner("Obteniendo tus reservas de cabina...");
            let person = match user_service::get_person_data(&token, "es").await {
                Ok(p) => p,
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al obtener tus datos de usuario: {err}")));
                    return Ok(());
                }
            };
            let reservas = match cabinas_service::get_mis_reservas(&token, &person.nip).await {
                Ok(r) => {
                    spinner.finish_and_clear();
                    r
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al consultar tus reservas: {err}")));
                    return Ok(());
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&reservas)?);
                return Ok(());
            }

            if reservas.is_empty() {
                println!("{}", ui::info("No tienes reservas de cabina activas actualmente."));
                return Ok(());
            }

            println!("{}", console::style("Tus reservas de cabina activas:").bold().underlined());

            let mut table = ui::create_table(&["ID Reserva", "Espacio", "Biblioteca", "Fecha", "Horario"]);
            for r in &reservas {
                let horario = match (&r.hora_ini, &r.hora_fin) {
                    (Some(ini), Some(fin)) => format!("{ini} - {fin}"),
                    (Some(ini), None) => ini.to_string(),
                    _ => "-".to_string(),
                };

                table.add_row(vec![
                    Cell::new(&r.id),
                    Cell::new(r.nombre.as_deref().unwrap_or("-")),
                    Cell::new(r.biblioteca.as_deref().unwrap_or("-")),
                    Cell::new(r.fecha.as_deref().unwrap_or("-")),
                    Cell::new(&horario),
                ]);
            }

            println!("{table}");
        }

        Some(CabinasAction::Liberar { id }) => {
            let spinner = ui::create_spinner("Cancelando reserva de cabina...");
            match cabinas_service::liberar_cabina(&token, &id).await {
                Ok(()) => {
                    spinner.finish_and_clear();
                    println!("{}", ui::success(&format!("Reserva {id} liberada con éxito.")));
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("No se pudo liberar la reserva: {err}")));
                }
            }
        }
    }

    Ok(())
}
