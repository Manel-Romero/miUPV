//! Wake-on-LAN service for remotely powering on lab and office computers.

use crate::client::{parse_json_list, parse_json_single_or_first, MiupvClient};
use crate::error::CoreError;
use crate::models::{Equipo, WakeupResult};
use reqwest::Method;

/// Fetches assigned lab/office equipment list using the default client.
pub async fn get_equipos(token: &str, lang: &str) -> Result<Vec<Equipo>, CoreError> {
    let client = MiupvClient::default();
    get_equipos_with_client(&client, token, lang).await
}

/// Fetches assigned lab/office equipment list using a custom client.
pub async fn get_equipos_with_client(
    client: &MiupvClient,
    token: &str,
    lang: &str,
) -> Result<Vec<Equipo>, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_wakeup.equipos?p_idioma={lang}&p_modo=");

    let res = client
        .authed_request(Method::GET, &url, token)
        .send()
        .await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Sends a Wake-on-LAN packet to wake up a computer using the default client.
pub async fn wakeup_equipo(
    token: &str,
    equipo_id: &str,
    lang: &str,
) -> Result<WakeupResult, CoreError> {
    let client = MiupvClient::default();
    wakeup_equipo_with_client(&client, token, equipo_id, lang).await
}

/// Sends a Wake-on-LAN packet to wake up a computer using a custom client.
pub async fn wakeup_equipo_with_client(
    client: &MiupvClient,
    token: &str,
    equipo_id: &str,
    lang: &str,
) -> Result<WakeupResult, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url =
        format!("{base}/pls/sorest/sic_wakeup.wakeup?p_equipo={equipo_id}&p_idioma={lang}&p_modo=");

    let res = client
        .authed_request(Method::GET, &url, token)
        .send()
        .await?;
    let body = client.handle_response_text(res).await?;
    parse_json_single_or_first(&body)
}
