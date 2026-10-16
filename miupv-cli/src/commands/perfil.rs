//! Perfil command handler: displays student/staff profile information.

use crate::auth_helper;
use crate::ui;
use comfy_table::Cell;
use miupv_core::services::user_service;

pub async fn run(json: bool, lang: &str) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    let spinner = ui::create_spinner("Obteniendo información del perfil...");
    let person = match user_service::get_person_data(&token, lang).await {
        Ok(p) => {
            spinner.finish_and_clear();
            p
        }
        Err(err) => {
            spinner.finish_and_clear();
            eprintln!("{}", ui::error(&format!("Error al obtener el perfil: {err}")));
            return Ok(());
        }
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&person)?);
        return Ok(());
    }

    let mut table = ui::create_table(&["Campo", "Información"]);
    table.add_row(vec![Cell::new("Nombre completo"), Cell::new(format!("{} {}", person.nombre, person.apellidos))]);
    table.add_row(vec![Cell::new("NIP"), Cell::new(&person.nip)]);
    table.add_row(vec![Cell::new("DNI / NIE"), Cell::new(&person.dni)]);
    table.add_row(vec![Cell::new("Email institucional"), Cell::new(&person.email)]);
    table.add_row(vec![Cell::new("Colectivo / Rol"), Cell::new(if person.tipo.is_empty() { "Estudiante" } else { &person.tipo })]);
    if let Some(foto) = &person.foto_url {
        table.add_row(vec![Cell::new("URL Fotografía"), Cell::new(foto)]);
    }

    println!("{table}");
    Ok(())
}
