//! Announcements and notices (avisos) command handler.

use crate::auth_helper;
use crate::ui;
use comfy_table::{Cell, Color};
use miupv_core::services::announcements_service;

pub async fn run(n: u32, json: bool, lang: &str) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    let spinner = ui::create_spinner("Consultando avisos institucionales...");
    let avisos = match announcements_service::get_avisos(&token, lang).await {
        Ok(a) => {
            spinner.finish_and_clear();
            a
        }
        Err(err) => {
            spinner.finish_and_clear();
            eprintln!("{}", ui::error(&format!("Error al obtener los avisos: {err}")));
            return Ok(());
        }
    };

    let display_avisos: Vec<_> = avisos.into_iter().take(n as usize).collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&display_avisos)?);
        return Ok(());
    }

    if display_avisos.is_empty() {
        println!("{}", ui::info("No se han encontrado avisos recientes."));
        return Ok(());
    }

    println!("{}", console::style(format!("Últimos avisos de la UPV (máx. {n}):")).bold().underlined());

    let mut table = ui::create_table(&["ID", "Fecha", "Centro", "Título", "Importante"]);

    for a in &display_avisos {
        let imp_cell = if a.importante {
            Cell::new("★ URGENTE").fg(Color::Red).add_attribute(comfy_table::Attribute::Bold)
        } else {
            Cell::new("Normal").fg(Color::Grey)
        };

        table.add_row(vec![
            Cell::new(&a.id),
            Cell::new(a.fecha.as_deref().unwrap_or("-")),
            Cell::new(a.centro.as_deref().unwrap_or("-")),
            Cell::new(&a.titulo),
            imp_cell,
        ]);
    }

    println!("{table}");
    Ok(())
}
