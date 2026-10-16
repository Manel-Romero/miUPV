//! Academic grades (notas) command handler.

use crate::auth_helper;
use crate::ui;
use chrono::Datelike;
use comfy_table::{Cell, Color};
use miupv_core::services::scores_service;

pub async fn run(
    curso_opt: Option<String>,
    json: bool,
    lang: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    let curso = curso_opt.unwrap_or_else(|| {
        let now = chrono::Local::now();
        let year = if now.month() >= 9 {
            now.year()
        } else {
            now.year() - 1
        };
        year.to_string()
    });

    let spinner = ui::create_spinner("Consultando calificaciones académicas...");
    let scores = match scores_service::get_scores(&token, &curso, lang).await {
        Ok(s) => {
            spinner.finish_and_clear();
            s
        }
        Err(err) => {
            spinner.finish_and_clear();
            eprintln!(
                "{}",
                ui::error(&format!("Error al obtener las notas: {err}"))
            );
            return Ok(());
        }
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&scores)?);
        return Ok(());
    }

    if scores.is_empty() {
        println!(
            "{}",
            ui::info(&format!(
                "No se encontraron calificaciones registradas para el curso {curso}."
            ))
        );
        return Ok(());
    }

    println!(
        "{}",
        console::style(format!("Calificaciones del Curso {curso}:"))
            .bold()
            .underlined()
    );

    let mut table = ui::create_table(&[
        "Código",
        "Asignatura",
        "Convocatoria",
        "Nota",
        "Calificación",
        "Créditos",
        "Fecha",
    ]);

    for a in &scores {
        let nota_cell = match a.nota {
            Some(n) => {
                let text = format!("{:.1}", n);
                if n >= 9.0 {
                    Cell::new(text)
                        .fg(Color::Green)
                        .add_attribute(comfy_table::Attribute::Bold)
                } else if n >= 7.0 {
                    Cell::new(text).fg(Color::Cyan)
                } else if n >= 5.0 {
                    Cell::new(text)
                } else {
                    Cell::new(text)
                        .fg(Color::Red)
                        .add_attribute(comfy_table::Attribute::Bold)
                }
            }
            None => Cell::new("-"),
        };

        let calif_cell = match &a.calificacion {
            Some(c) => Cell::new(c),
            None => Cell::new("-"),
        };

        table.add_row(vec![
            Cell::new(&a.codigo),
            Cell::new(&a.nombre),
            Cell::new(a.convocatoria.as_deref().unwrap_or("-")),
            nota_cell,
            calif_cell,
            Cell::new(
                a.creditos
                    .map(|c| format!("{:.1}", c))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::new(a.fecha.as_deref().unwrap_or("-")),
        ]);
    }

    println!("{table}");
    Ok(())
}
