//! Virtual university ID card (Carnet Virtual) command handler.

use crate::auth_helper;
use crate::ui;
use clap::Subcommand;
use comfy_table::Cell;
use miupv_core::services::carnet_service;
use qrcode::render::unicode;
use qrcode::QrCode;

#[derive(Subcommand, Debug)]
pub enum CarnetAction {
    /// Muestra la información del carnet universitario virtual
    Info,

    /// Genera y muestra el código QR del carnet en el terminal
    Qr,
}

pub async fn run(
    action: CarnetAction,
    json: bool,
    lang: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = auth_helper::get_valid_token().await?;

    let spinner = ui::create_spinner("Consultando datos del carnet virtual...");
    let carnet = match carnet_service::get_info_carnet(&token, lang).await {
        Ok(c) => {
            spinner.finish_and_clear();
            c
        }
        Err(err) => {
            spinner.finish_and_clear();
            eprintln!("{}", ui::error(&format!("Error al obtener el carnet virtual: {err}")));
            return Ok(());
        }
    };

    match action {
        CarnetAction::Info => {
            if json {
                println!("{}", serde_json::to_string_pretty(&carnet)?);
                return Ok(());
            }

            println!("{}", console::style("Carnet Universitario Virtual (TUI):").bold().underlined());

            let mut table = ui::create_table(&["Campo", "Valor"]);
            table.add_row(vec![Cell::new("Titular"), Cell::new(&carnet.nombre)]);
            table.add_row(vec![Cell::new("NIP"), Cell::new(&carnet.nip)]);
            table.add_row(vec![
                Cell::new("Tipo de Carnet"),
                Cell::new(carnet.tipo_carnet.as_deref().unwrap_or("Universitario")),
            ]);
            table.add_row(vec![
                Cell::new("Estado"),
                Cell::new(carnet.estado.as_deref().unwrap_or("ACTIVO")),
            ]);
            if let Some(foto) = &carnet.foto_url {
                table.add_row(vec![Cell::new("Fotografía"), Cell::new(foto)]);
            }

            println!("{table}");
        }

        CarnetAction::Qr => {
            let qr_payload = carnet
                .qr_data
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(&carnet.nip);

            if json {
                println!("{}", serde_json::json!({ "qr_data": qr_payload }));
                return Ok(());
            }

            println!(
                "{}",
                console::style(format!("Código QR del Carnet Virtual (NIP: {}):", carnet.nip))
                    .bold()
            );

            match QrCode::new(qr_payload.as_bytes()) {
                Ok(code) => {
                    let image = code.render::<unicode::Dense1x2>().build();
                    println!("{image}");
                    println!("{}", console::style("Muestra este código en lectores de tornos o bibliotecas.").dim());
                }
                Err(err) => {
                    eprintln!("{}", ui::error(&format!("Error al renderizar el código QR: {err}")));
                }
            }
        }
    }

    Ok(())
}
