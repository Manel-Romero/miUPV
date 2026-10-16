//! Institutional center announcements service (Avisos de Centros).

use crate::client::{parse_json_list, MiupvClient};
use crate::error::CoreError;
use crate::models::Aviso;
use reqwest::Method;

/// Fetches institutional announcements and school notices using the default client.
pub async fn get_avisos(token: &str, lang: &str) -> Result<Vec<Aviso>, CoreError> {
    let client = MiupvClient::default();
    get_avisos_with_client(&client, token, "", lang).await
}

/// Fetches institutional announcements for a specific degree/titulación.
pub async fn get_avisos_for_tit(
    token: &str,
    tit: &str,
    lang: &str,
) -> Result<Vec<Aviso>, CoreError> {
    let client = MiupvClient::default();
    get_avisos_with_client(&client, token, tit, lang).await
}

/// Fetches institutional announcements using a custom client.
pub async fn get_avisos_with_client(
    client: &MiupvClient,
    token: &str,
    tit: &str,
    lang: &str,
) -> Result<Vec<Aviso>, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_centros.get_avisos?p_tit={tit}&p_idioma={lang}");

    let res = client
        .authed_request(Method::GET, &url, token)
        .send()
        .await?;
    if res.status().is_success() {
        let body = client.handle_response_text(res).await?;
        return parse_json_list(&body);
    }

    // Fallback: Query PoliformaT announcements (accessible via Intranet session)
    if let Ok(poliformat_avisos) =
        crate::services::poliformat_service::get_announcements_with_client(client, token, 20, 15)
            .await
    {
        let converted: Vec<Aviso> = poliformat_avisos
            .into_iter()
            .map(|a| Aviso {
                id: a.id,
                titulo: a.title,
                centro: a.site_title,
                fecha: a.date,
                importante: false,
                texto: a.body,
            })
            .collect();
        return Ok(converted);
    }

    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    Err(CoreError::Api {
        status,
        message: body,
    })
}
