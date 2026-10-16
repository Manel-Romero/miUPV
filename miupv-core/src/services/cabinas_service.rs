//! Library study cabin reservation service (Biblioteca UPV).

use crate::client::{parse_json_list, parse_json_single_or_first, MiupvClient};
use crate::error::CoreError;
use crate::models::{Cabina, Reserva};
use reqwest::Method;
use serde_json::json;

/// Fetches available study cabins for a given date (YYYY-MM-DD) using the default client.
pub async fn get_cabinas_disponibles(token: &str, fecha: &str) -> Result<Vec<Cabina>, CoreError> {
    let client = MiupvClient::default();
    get_cabinas_disponibles_with_client(&client, token, fecha).await
}

/// Fetches available study cabins for a given date using a custom client.
pub async fn get_cabinas_disponibles_with_client(
    client: &MiupvClient,
    token: &str,
    fecha: &str,
) -> Result<Vec<Cabina>, CoreError> {
    let base = client.config().aplicat_base_url.trim_end_matches('/');
    let url = format!("{base}/restreservas-1.0/franjas?idEntidad=105&fecha={fecha}&sololibres=S");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Fetches active bookings for a specific user NIP using the default client.
pub async fn get_mis_reservas(token: &str, nip: &str) -> Result<Vec<Reserva>, CoreError> {
    let client = MiupvClient::default();
    get_mis_reservas_with_client(&client, token, nip).await
}

/// Fetches active bookings for a specific user NIP using a custom client.
pub async fn get_mis_reservas_with_client(
    client: &MiupvClient,
    token: &str,
    nip: &str,
) -> Result<Vec<Reserva>, CoreError> {
    let base = client.config().aplicat_base_url.trim_end_matches('/');
    let url = format!("{base}/restreservas-1.0/franjas?idEntidad=105&nip={nip}");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Reserves a study cabin time slot by franja ID using the default client.
pub async fn reservar_cabina(token: &str, franja_id: &str) -> Result<Reserva, CoreError> {
    let client = MiupvClient::default();
    reservar_cabina_with_client(&client, token, franja_id).await
}

/// Reserves a study cabin time slot by franja ID using a custom client.
pub async fn reservar_cabina_with_client(
    client: &MiupvClient,
    token: &str,
    franja_id: &str,
) -> Result<Reserva, CoreError> {
    let base = client.config().aplicat_base_url.trim_end_matches('/');
    let url = format!("{base}/restreservas-1.0/franjas");

    let payload = json!({
        "idFranja": franja_id,
        "id": franja_id
    });

    let res = client
        .authed_request(Method::POST, &url, token)
        .json(&payload)
        .send()
        .await?;

    let body = client.handle_response_text(res).await?;
    parse_json_single_or_first(&body)
}

/// Releases / cancels an active cabin reservation using the default client.
pub async fn liberar_cabina(token: &str, franja_id: &str) -> Result<(), CoreError> {
    let client = MiupvClient::default();
    liberar_cabina_with_client(&client, token, franja_id).await
}

/// Releases / cancels an active cabin reservation using a custom client.
pub async fn liberar_cabina_with_client(
    client: &MiupvClient,
    token: &str,
    franja_id: &str,
) -> Result<(), CoreError> {
    let base = client.config().aplicat_base_url.trim_end_matches('/');
    let url = format!("{base}/restreservas-1.0/franjas?idFranja={franja_id}&id={franja_id}");

    let res = client
        .authed_request(Method::DELETE, &url, token)
        .send()
        .await?;

    let status = res.status();
    if status.is_success() || status == reqwest::StatusCode::NO_CONTENT {
        Ok(())
    } else {
        let msg = res.text().await.unwrap_or_default();
        Err(CoreError::Api { status, message: msg })
    }
}
