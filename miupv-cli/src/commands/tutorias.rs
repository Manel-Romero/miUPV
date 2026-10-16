//! Academic tutoring appointments command handler (Tutorías UPV).

use crate::auth_helper;
use crate::ui;
use clap::Subcommand;
use comfy_table::{Cell, Color};
use miupv_core::services::tutorias_service;

#[derive(Subcommand, Debug)]
pub enum TutoriasAction {
    /// Muestra las asignaturas y profesorado disponibles para tutorías
    Asignaturas,
}

pub async fn run(
    action: Option<TutoriasAction>,
    json: bool,
    lang: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    match action {
        None => {
            // Default: list tutoring sessions
            let spinner = ui::create_spinner("Consultando tus tutorías programadas...");
            let tutorias = match tutorias_service::get_tutorias(&token, lang).await {
                Ok(t) => {
                    spinner.finish_and_clear();
                    t
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al obtener las tutorías: {err}")));
                    return Ok(());
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&tutorias)?);
                return Ok(());
            }

            if tutorias.is_empty() {
                println!("{}", ui::info("No tienes tutorías solicitadas ni programadas."));
                return Ok(());
            }

            println!("{}", console::style("Tus tutorías docentes:").bold().underlined());

            let mut table = ui::create_table(&["ID", "Asignatura", "Profesor", "Fecha", "Hora", "Lugar", "Estado"]);

            for t in &tutorias {
                let estado_cell = match t.estado.as_deref() {
                    Some(s) if s.to_uppercase().contains("ACEPT") => Cell::new(s).fg(Color::Green),
                    Some(s) if s.to_uppercase().contains("SOLIC") => Cell::new(s).fg(Color::Yellow),
                    Some(s) if s.to_uppercase().contains("ANUL") || s.to_uppercase().contains("CANCEL") => Cell::new(s).fg(Color::Red),
                    Some(s) => Cell::new(s),
                    None => Cell::new("PROGRAMADA").fg(Color::Green),
                };

                table.add_row(vec![
                    Cell::new(&t.id),
                    Cell::new(t.asignatura.as_deref().unwrap_or("-")),
                    Cell::new(t.profesor.as_deref().unwrap_or("-")),
                    Cell::new(t.fecha.as_deref().unwrap_or("-")),
                    Cell::new(t.hora.as_deref().unwrap_or("-")),
                    Cell::new(t.lugar.as_deref().unwrap_or("-")),
                    estado_cell,
                ]);
            }

            println!("{table}");
        }

        Some(TutoriasAction::Asignaturas) => {
            let spinner = ui::create_spinner("Consultando asignaturas y profesores para tutorías...");
            let asignaturas = match tutorias_service::get_asignaturas(&token, lang).await {
                Ok(a) => {
                    spinner.finish_and_clear();
                    a
                }
                Err(err) => {
                    spinner.finish_and_clear();
                    eprintln!("{}", ui::error(&format!("Error al obtener asignaturas: {err}")));
                    return Ok(());
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&asignaturas)?);
                return Ok(());
            }

            if asignaturas.is_empty() {
                println!("{}", ui::info("No se encontraron asignaturas disponibles para tutorías."));
                return Ok(());
            }

            println!("{}", console::style("Asignaturas disponibles para solicitar tutorías:").bold().underlined());

            let mut table = ui::create_table(&["Código", "Asignatura", "Profesorado"]);

            for a in &asignaturas {
                let profs = if a.profesores.is_empty() {
                    "-".to_string()
                } else {
                    a.profesores
                        .iter()
                        .map(|p| p.nombre.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                table.add_row(vec![
                    Cell::new(&a.codigo),
                    Cell::new(&a.nombre),
                    Cell::new(&profs),
                ]);
            }

            println!("{table}");
        }
    }

    Ok(())
}
