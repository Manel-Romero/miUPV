//! Tutoring sessions and office hours service (Tutorías UPV).

use crate::client::{parse_json_list, MiupvClient};
use crate::error::CoreError;
use crate::models::{AsignaturaTutoria, SolicitudTutoria, Tutoria};
use reqwest::Method;

/// Fetches subjects available for tutoring using the default client.
pub async fn get_asignaturas(token: &str, lang: &str) -> Result<Vec<AsignaturaTutoria>, CoreError> {
    let client = MiupvClient::default();
    get_asignaturas_with_client(&client, token, lang).await
}

/// Fetches subjects available for tutoring using a custom client.
pub async fn get_asignaturas_with_client(
    client: &MiupvClient,
    token: &str,
    lang: &str,
) -> Result<Vec<AsignaturaTutoria>, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_tutorias.alu_tut_asignaturas?p_idioma={lang}");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Fetches current scheduled or requested tutoring sessions using the default client.
pub async fn get_tutorias(token: &str, lang: &str) -> Result<Vec<Tutoria>, CoreError> {
    let client = MiupvClient::default();
    get_tutorias_with_client(&client, token, lang).await
}

/// Fetches current scheduled or requested tutoring sessions using a custom client.
pub async fn get_tutorias_with_client(
    client: &MiupvClient,
    token: &str,
    lang: &str,
) -> Result<Vec<Tutoria>, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_tutorias.alu_get_tutorias?p_idioma={lang}&p_modo=");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Submits a request for a new tutoring appointment using the default client.
pub async fn solicitar_tutoria(
    token: &str,
    payload: &SolicitudTutoria,
) -> Result<(), CoreError> {
    let client = MiupvClient::default();
    solicitar_tutoria_with_client(&client, token, payload).await
}

/// Submits a request for a new tutoring appointment using a custom client.
pub async fn solicitar_tutoria_with_client(
    client: &MiupvClient,
    token: &str,
    payload: &SolicitudTutoria,
) -> Result<(), CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_tutorias.alu_ins_tutoria");

    let res = client
        .authed_request(Method::POST, &url, token)
        .json(payload)
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
