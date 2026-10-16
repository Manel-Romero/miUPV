//! Logout command handler.

use crate::ui;
use miupv_core::services::auth_service;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let spinner = ui::create_spinner("Cerrando sesión...");

    if let Ok(Some(tokens)) = auth_service::load_tokens() {
        let _ = auth_service::logout(&tokens.access_token).await;
    }

    if let Err(e) = auth_service::clear_tokens() {
        spinner.finish_and_clear();
        eprintln!("{}", ui::warning(&format!("Sesión borrada localmente con advertencia: {e}")));
    } else {
        spinner.finish_and_clear();
        println!("{}", ui::success("Sesión cerrada. Tokens eliminados del almacenamiento seguro."));
    }

    Ok(())
}
