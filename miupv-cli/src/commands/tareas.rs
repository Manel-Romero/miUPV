//! PoliformaT assignments and tasks command handler.

use crate::auth_helper;
use crate::ui;
use comfy_table::{Cell, Color};
use miupv_core::services::poliformat_service;

pub async fn run(
    site_opt: Option<String>,
    all: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    let spinner = ui::create_spinner("Consultando tareas en PoliformaT...");
    let all_tasks = match poliformat_service::get_tasks(&token).await {
        Ok(t) => {
            spinner.finish_and_clear();
            t
        }
        Err(err) => {
            spinner.finish_and_clear();
            eprintln!("{}", ui::error(&format!("Error al obtener las tareas: {err}")));
            return Ok(());
        }
    };

    let filtered: Vec<_> = all_tasks
        .into_iter()
        .filter(|task| {
            // Filter by site if specified
            if let Some(site) = &site_opt {
                if let Some(st) = &task.site_title {
                    if !st.to_lowercase().contains(&site.to_lowercase()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            // Filter by submission status unless --all is set
            if !all {
                if let Some(status) = &task.status {
                    let s = status.to_uppercase();
                    if s.contains("SUBMITTED") || s.contains("ENTREGADA") || s.contains("GRADED") {
                        return false;
                    }
                }
            }
            true
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
        return Ok(());
    }

    if filtered.is_empty() {
        println!(
            "{}",
            ui::info(if all {
                "No hay tareas registradas en PoliformaT."
            } else {
                "No tienes tareas pendientes de entrega."
            })
        );
        return Ok(());
    }

    let title_prefix = if all { "Todas las tareas" } else { "Tareas pendientes" };
    println!("{}", console::style(format!("{title_prefix} de PoliformaT:")).bold().underlined());

    let mut table = ui::create_table(&["ID", "Título", "Asignatura / Sitio", "Fecha Límite", "Estado"]);

    for t in &filtered {
        let status_str = t.status.as_deref().unwrap_or("PENDIENTE");
        let status_cell = if status_str.to_uppercase().contains("SUBMITTED")
            || status_str.to_uppercase().contains("ENTREGADA")
        {
            Cell::new(status_str).fg(Color::Green)
        } else {
            Cell::new(status_str).fg(Color::Yellow).add_attribute(comfy_table::Attribute::Bold)
        };

        table.add_row(vec![
            Cell::new(&t.id),
            Cell::new(&t.title),
            Cell::new(t.site_title.as_deref().unwrap_or("-")),
            Cell::new(t.due_date.as_deref().unwrap_or("-")),
            status_cell,
        ]);
    }

    println!("{table}");
    Ok(())
}
