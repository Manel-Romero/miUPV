//! Academic scores and statistics service.

use crate::client::{parse_json_list, parse_json_single_or_first, MiupvClient};
use crate::error::CoreError;
use crate::models::{Asignatura, Statistics};
use reqwest::Method;

/// Fetches course grades for the specified academic year using the default client.
pub async fn get_scores(token: &str, curso: &str, lang: &str) -> Result<Vec<Asignatura>, CoreError> {
    let client = MiupvClient::default();
    get_scores_with_client(&client, token, curso, lang).await
}

/// Fetches course grades using a custom client.
pub async fn get_scores_with_client(
    client: &MiupvClient,
    token: &str,
    curso: &str,
    lang: &str,
) -> Result<Vec<Asignatura>, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_notas.notas_Alu?p_curso={curso}&p_idioma={lang}");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Fetches grade statistics for a specific course and subject using the default client.
pub async fn get_statistics(
    token: &str,
    curso: &str,
    tit: &str,
    asi: &str,
    lang: &str,
) -> Result<Statistics, CoreError> {
    let client = MiupvClient::default();
    get_statistics_with_client(&client, token, curso, tit, asi, lang).await
}

/// Fetches grade statistics for a specific course and subject using a custom client.
pub async fn get_statistics_with_client(
    client: &MiupvClient,
    token: &str,
    curso: &str,
    tit: &str,
    asi: &str,
    lang: &str,
) -> Result<Statistics, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!(
        "{base}/pls/sorest/sic_app_notas.estadisticas?p_curso={curso}&p_tit={tit}&p_asi={asi}&p_idioma={lang}"
    );

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_single_or_first(&body)
}
