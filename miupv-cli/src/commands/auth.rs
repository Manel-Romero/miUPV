//! Auth command handler for direct token management and status.

use crate::ui;
use clap::Subcommand;
use comfy_table::Cell;
use miupv_core::services::auth_service;

#[derive(Subcommand, Debug)]
pub enum AuthAction {
    /// Inyecta manualmente un token extraído de la app oficial o de la web
    #[command(name = "set-token")]
    SetToken {
        /// Token de acceso (Bearer token o cookie TDp)
        token: String,

        /// Token de refresco opcional
        #[arg(short, long)]
        refresh_token: Option<String>,

        /// Tiempo de validez en segundos (por defecto 3600 o 28800 según el tipo)
        #[arg(short, long)]
        ttl: Option<i64>,
    },

    /// Fuerza la renovación inmediata del token de acceso y de refresco de 30 días
    Renew,

    /// Muestra el estado actual de la autenticación y tokens almacenados
    Status,

    /// Elimina las credenciales y tokens almacenados
    Clear,
}

pub async fn run(action: AuthAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        AuthAction::SetToken {
            token,
            refresh_token,
            ttl,
        } => match auth_service::set_manual_token(&token, refresh_token.as_deref(), ttl) {
            Ok(tokens) => {
                let tipo = if tokens.is_intranet_session() {
                    "Sesión Intranet UPV (Cookie TDp)"
                } else {
                    "Bearer Token OAuth 2.0 (App Oficial)"
                };
                println!(
                    "{}",
                    ui::success(&format!(
                        "Token guardado correctamente en el almacén seguro.\nTipo detectado: {tipo}"
                    ))
                );
            }
            Err(e) => {
                eprintln!("{}", ui::error(&format!("Error al guardar el token: {e}")));
            }
        },
        AuthAction::Renew => {
            let spinner = ui::create_spinner("Renovando sesión con los servidores de la UPV...");
            if let Some((u, p)) = auth_service::load_credentials() {
                match auth_service::login_pkce(&u, &p).await {
                    Ok(new_tokens) => {
                        auth_service::save_tokens(&new_tokens)?;
                        spinner.finish_and_clear();
                        println!(
                            "{}",
                            ui::success("Sesión renovada con éxito vía OAuth 2.0 (nuevo Refresh Token de 30 días emitido).")
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        spinner.finish_and_clear();
                        eprintln!(
                            "{}",
                            ui::error(&format!("Error renovando con credenciales: {e}"))
                        );
                    }
                }
            } else if let Some(tokens) = auth_service::load_tokens()? {
                if !tokens.refresh_token.is_empty() {
                    match auth_service::refresh(&tokens.refresh_token).await {
                        Ok(new_tokens) => {
                            auth_service::save_tokens(&new_tokens)?;
                            spinner.finish_and_clear();
                            println!(
                                "{}",
                                ui::success("Token de acceso renovado con éxito (24h).")
                            );
                            return Ok(());
                        }
                        Err(e) => {
                            spinner.finish_and_clear();
                            eprintln!("{}", ui::error(&format!("Error al refrescar token: {e}")));
                        }
                    }
                }
            } else {
                spinner.finish_and_clear();
                eprintln!("{}", ui::warning("No hay credenciales ni tokens guardados para renovar. Usa `miupv login` primero."));
            }
        }
        AuthAction::Status => match auth_service::load_tokens()? {
            Some(tokens) => {
                let is_valid = tokens.is_valid();
                let tipo = if tokens.is_intranet_session() {
                    "Sesión Intranet UPV (Cookie TDp)"
                } else {
                    "Bearer Token OAuth 2.0 (App Oficial)"
                };

                let masked = if tokens.access_token.len() > 10 {
                    format!(
                        "{}...{}",
                        &tokens.access_token[..6],
                        &tokens.access_token[tokens.access_token.len() - 4..]
                    )
                } else {
                    "********".to_string()
                };

                let expiry_str = match tokens.expires_at {
                    Some(exp) => exp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    None => "No configurada (indefinida)".to_string(),
                };

                let creds_present = auth_service::load_credentials().is_some();

                let mut table = ui::create_table(&["Propiedad", "Valor"]);
                table.add_row(vec![
                    Cell::new("Estado de la sesión"),
                    Cell::new(if is_valid {
                        "Activa / Válida"
                    } else {
                        "Expirada"
                    }),
                ]);
                table.add_row(vec![Cell::new("Tipo de token"), Cell::new(tipo)]);
                table.add_row(vec![Cell::new("Token (enmascarado)"), Cell::new(&masked)]);
                table.add_row(vec![
                    Cell::new("Fecha de expiración"),
                    Cell::new(&expiry_str),
                ]);
                table.add_row(vec![
                    Cell::new("Token de refresco"),
                    Cell::new(if tokens.refresh_token.is_empty() {
                        "No disponible"
                    } else {
                        "Presente (30 días)"
                    }),
                ]);
                table.add_row(vec![
                    Cell::new("Autorrenovación permanente"),
                    Cell::new(if creds_present {
                        "Activa (Credenciales vinculadas de forma segura)"
                    } else {
                        "Inactiva (usa `miupv login --pkce` para vincularlas)"
                    }),
                ]);

                println!("{table}");
            }
            None => {
                println!(
                    "{}",
                    ui::warning("No hay ninguna sesión activa ni tokens almacenados.")
                );
                println!(
                    "Puedes iniciar sesión con `miupv login` o pasar un token con `miupv auth set-token <TOKEN>`."
                );
            }
        },
        AuthAction::Clear => {
            auth_service::clear_tokens()?;
            let _ = auth_service::clear_credentials();
            println!(
                "{}",
                ui::success("Credenciales y tokens eliminados con éxito.")
            );
        }
    }
    Ok(())
}
