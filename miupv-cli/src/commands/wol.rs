//! Wake-on-LAN remote computer awakening command handler.

use crate::auth_helper;
use crate::ui;
use clap::Subcommand;
use comfy_table::{Cell, Color};
use miupv_core::services::wol_service;

#[derive(Subcommand, Debug)]
pub enum WolAction {
    /// Muestra los ordenadores asignados al usuario en laboratorios o despachos
    Lista,

    /// Envía el paquete Wake-on-LAN para encender el equipo remoto
    Despertar {
        /// Identificador o código del equipo a despertar (ej: LAB-01)
        #[arg(short, long)]
        equipo: String,
    },
}

pub async fn run(
    action: WolAction,
    json: bool,
    lang: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    match action {
        WolAction::Lista => {
            let spinner = ui::create_spinner("Consultando equipos asignados...");
            let equipos = match wol_service::get_equipos(&token, lang).await {
                Ok(eq) => {
                    spinner.finish_and_clear();
                    eq
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al obtener equipos: {err}")));
                    return Ok(());
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&equipos)?);
                return Ok(());
            }

            if equipos.is_empty() {
                println!(
                    "{}",
                    ui::info("No tienes equipos asociados para encendido remoto.")
                );
                return Ok(());
            }

            println!(
                "{}",
                console::style("Equipos disponibles para Wake-on-LAN:")
                    .bold()
                    .underlined()
            );

            let mut table =
                ui::create_table(&["ID", "Nombre / Host", "Estado", "IP", "MAC", "Campus"]);

            for eq in &equipos {
                let estado_cell = match eq.estado.as_deref() {
                    Some(s) if s.to_uppercase() == "ON" => Cell::new("ENCENDIDO").fg(Color::Green),
                    Some(s) if s.to_uppercase() == "OFF" => Cell::new("APAGADO").fg(Color::Red),
                    Some(s) => Cell::new(s),
                    None => Cell::new("DESCONOCIDO"),
                };

                table.add_row(vec![
                    Cell::new(&eq.id),
                    Cell::new(&eq.nombre),
                    estado_cell,
                    Cell::new(eq.ip.as_deref().unwrap_or("-")),
                    Cell::new(eq.mac.as_deref().unwrap_or("-")),
                    Cell::new(eq.campus.as_deref().unwrap_or("-")),
                ]);
            }

            println!("{table}");
        }

        WolAction::Despertar { equipo } => {
            let spinner = ui::create_spinner("Enviando paquete Wake-on-LAN...");
            match wol_service::wakeup_equipo(&token, &equipo, lang).await {
                Ok(result) => {
                    spinner.finish_and_clear();
                    if result.success {
                        let msg = result.mensaje.as_deref().unwrap_or("Petición aceptada");
                        println!(
                            "{}",
                            ui::success(&format!(
                                "Comando Wake-on-LAN enviado a '{equipo}': {msg}"
                            ))
                        );
                    } else {
                        let msg = result
                            .mensaje
                            .as_deref()
                            .unwrap_or("No se pudo encender el equipo");
                        eprintln!(
                            "{}",
                            ui::error(&format!("Fallo al encender '{equipo}': {msg}"))
                        );
                    }
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!(
                        "{}",
                        ui::error(&format!("Error al enviar Wake-on-LAN: {err}"))
                    );
                }
            }
        }
    }

    Ok(())
}
