//! User profile service for retrieving personal and student details.

use crate::client::{parse_json_single_or_first, MiupvClient};
use crate::error::CoreError;
use crate::models::PersonData;
use reqwest::Method;

/// Fetches personal profile data for the authenticated user using the default client.
pub async fn get_person_data(token: &str, lang: &str) -> Result<PersonData, CoreError> {
    let client = MiupvClient::default();
    get_person_data_with_client(&client, token, lang).await
}

/// Fetches personal profile data using a custom client.
pub async fn get_person_data_with_client(
    client: &MiupvClient,
    token: &str,
    lang: &str,
) -> Result<PersonData, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_miupv.get_person_data?p_idioma={lang}");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    if res.status().is_success() {
        if let Ok(body) = client.handle_response_text(res).await {
            if let Ok(mut data) = parse_json_single_or_first::<PersonData>(&body) {
                if (data.nombre.is_empty() || data.apellidos.is_empty()) && data.full_name.is_some() {
                    let full = data.full_name.as_deref().unwrap_or("").trim();
                    if let Some((aps, nom)) = full.split_once(',') {
                        data.apellidos = aps.trim().to_string();
                        data.nombre = nom.trim().to_string();
                    } else if !full.is_empty() {
                        data.nombre = full.to_string();
                    }
                }
                if data.tipo == "A" {
                    data.tipo = "alumno".to_string();
                }
                if data.foto_url.is_none() && !data.nip.is_empty() {
                    data.foto_url = Some(format!(
                        "https://aplicat.upv.es/restsgen-1.0/directorio/publico/foto/{}",
                        data.nip
                    ));
                }
                if !data.nombre.is_empty() || !data.apellidos.is_empty() {
                    return Ok(data);
                }
            }
        }
    }

    // Fallback: Query the Intranet UPV portal using the session cookie
    let intranet_base = client.config().intranet_base_url.trim_end_matches('/');
    let intra_lang = match lang {
        "ca" | "va" | "v" => "v",
        "en" | "i" => "i",
        _ => "c",
    };
    let menu_url = format!("{intranet_base}/pls/soalu/sic_menu.MiUPV?P_IDIOMA={intra_lang}&P_MODO=alumno");

    if let Ok(res_intra) = client.authed_request(Method::GET, &menu_url, token).send().await {
        if res_intra.status().is_success() {
            let bytes = res_intra.bytes().await.unwrap_or_default();
            // Decode ISO-8859-15 / Latin1 cleanly to preserve Spanish diacritics
            let html: String = bytes.iter().map(|&b| b as char).collect();
            if let Some((nombre, apellidos)) = extract_name_from_intranet_menu(&html) {
                return Ok(PersonData {
                    nip: String::new(),
                    nombre,
                    apellidos,
                    email: String::new(),
                    dni: String::new(),
                    foto_url: None,
                    tipo: "alumno".to_string(),
                    full_name: None,
                });
            }
        }
    }

    Err(CoreError::Auth(
        "No se pudo obtener el perfil de usuario. Comprueba que la sesión o el token sigan vigentes."
            .to_string(),
    ))
}

/// Parses the student's name from the Intranet UPV menu HTML (e.g. "[ García López, Juan ]").
pub fn extract_name_from_intranet_menu(html: &str) -> Option<(String, String)> {
    let clean = html.replace("&nbsp;", " ");

    for part in clean.split('[') {
        if let Some(close_idx) = part.find(']') {
            let candidate = part[..close_idx].trim();
            if candidate.contains(',')
                && !candidate.contains('<')
                && !candidate.contains('>')
                && !candidate.contains('{')
            {
                let mut parts = candidate.splitn(2, ',');
                let apellidos = parts.next()?.trim().to_string();
                let nombre = parts.next()?.trim().to_string();
                if !nombre.is_empty() && !apellidos.is_empty() {
                    return Some((nombre, apellidos));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name_from_intranet_html() {
        let sample = r#"
            <li class="menuAD">[&nbsp; García López, Juan &nbsp;]
            <a class="menuAD2N2" href="/expira" title="Cerrar sesión">Cerrar sesión</a>
        "#;
        let (nombre, apellidos) =
            extract_name_from_intranet_menu(sample).expect("Should parse student name");
        assert_eq!(nombre, "Juan");
        assert_eq!(apellidos, "García López");
    }
}

