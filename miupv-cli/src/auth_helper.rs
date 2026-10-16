//! Authentication helpers for CLI execution.

use crate::ui;
use miupv_core::services::auth_service;

/// Retrieves a valid access token from storage, automatically refreshing it if expired.
pub async fn get_valid_token() -> Result<String, Box<dyn std::error::Error>> {
    let tokens = match auth_service::load_tokens()? {
        Some(t) => t,
        None => {
            if let Some((u, p)) = auth_service::load_credentials() {
                tracing::info!(
                    "No active token, but stored credentials found. Performing automatic login..."
                );
                if let Ok(new_tokens) = auth_service::login_pkce(&u, &p).await {
                    let _ = auth_service::save_tokens(&new_tokens);
                    return Ok(new_tokens.access_token);
                }
            }
            eprintln!(
                "{}",
                ui::warning("No has iniciado sesión. Ejecuta `miupv login` para autenticarte.")
            );
            return Err("Not authenticated".into());
        }
    };

    if auth_service::is_token_valid(&tokens) {
        return Ok(tokens.access_token);
    }

    // Attempt token refresh
    tracing::info!("Access token expired, attempting refresh...");
    match auth_service::refresh(&tokens.refresh_token).await {
        Ok(new_tokens) => {
            auth_service::save_tokens(&new_tokens)?;
            tracing::info!("Token successfully refreshed");
            Ok(new_tokens.access_token)
        }
        Err(err) => {
            tracing::warn!("Token refresh failed ({err}). Checking stored credentials for automatic renewal...");
            if let Some((u, p)) = auth_service::load_credentials() {
                tracing::info!("Auto-renewing 30-day session with UPV OAuth server...");
                match auth_service::login_pkce(&u, &p).await {
                    Ok(new_tokens) => {
                        let _ = auth_service::save_tokens(&new_tokens);
                        return Ok(new_tokens.access_token);
                    }
                    Err(renew_err) => {
                        tracing::error!("Auto-renewal failed: {renew_err}");
                    }
                }
            }

            eprintln!(
                "{}",
                ui::error(&format!(
                    "La sesión ha expirado y no se pudo renovar: {err}. Ejecuta `miupv login`."
                ))
            );
            let _ = auth_service::clear_tokens();
            Err("Session expired".into())
        }
    }
}
