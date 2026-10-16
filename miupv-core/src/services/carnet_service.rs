//! Virtual university identity card service (TUI / Carnet Virtual).

use crate::client::{parse_json_list, parse_json_single_or_first, MiupvClient};
use crate::error::CoreError;
use crate::models::{CarnetInfo, VCard};
use reqwest::Method;

/// Fetches virtual card information and barcode data using the default client.
pub async fn get_info_carnet(token: &str, lang: &str) -> Result<CarnetInfo, CoreError> {
    let client = MiupvClient::default();
    get_info_carnet_with_client(&client, token, lang).await
}

/// Fetches virtual card information and barcode data using a custom client.
pub async fn get_info_carnet_with_client(
    client: &MiupvClient,
    token: &str,
    lang: &str,
) -> Result<CarnetInfo, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_vcard.datos_vcard?p_idioma={lang}");

    let res = client
        .authed_request(Method::GET, &url, token)
        .send()
        .await?;
    let body = client.handle_response_text(res).await?;

    if let Ok(info) = parse_json_single_or_first::<CarnetInfo>(&body) {
        return Ok(info);
    }

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
        let card_data = val.get("cardData").unwrap_or(&val);
        let nombre = card_data
            .get("name")
            .or_else(|| card_data.get("nombre"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let nip = card_data
            .get("card")
            .or_else(|| card_data.get("dni"))
            .or_else(|| card_data.get("nip"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let foto_url = card_data
            .get("photo")
            .or_else(|| card_data.get("foto_url"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        let tipo_carnet = card_data
            .get("title")
            .or_else(|| card_data.get("tipo_carnet"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let mut qr_data = card_data
            .get("qr_data")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);
        if qr_data.is_none() {
            if let Some(services) = card_data.get("services").and_then(|v| v.as_array()) {
                for s in services {
                    if s.get("id").and_then(|v| v.as_str()) == Some("qr") {
                        qr_data = s
                            .get("value")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string);
                        break;
                    }
                }
            }
        }

        return Ok(CarnetInfo {
            nip,
            nombre,
            foto_url,
            qr_data,
            tipo_carnet,
            estado: Some("ACTIVO".to_string()),
        });
    }

    parse_json_single_or_first(&body)
}

/// Fetches all active virtual cards associated with the user account using the default client.
pub async fn get_cards(token: &str, lang: &str) -> Result<Vec<VCard>, CoreError> {
    let client = MiupvClient::default();
    get_cards_with_client(&client, token, lang).await
}

/// Fetches all active virtual cards using a custom client.
pub async fn get_cards_with_client(
    client: &MiupvClient,
    token: &str,
    lang: &str,
) -> Result<Vec<VCard>, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_vcard.get_vcards?p_idioma={lang}");

    let res = client
        .authed_request(Method::GET, &url, token)
        .send()
        .await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Registers the current device for electronic card access using the default client.
pub async fn register_device(
    token: &str,
    device_id: &str,
    os: &str,
    version: &str,
    model: &str,
    lang: &str,
) -> Result<(), CoreError> {
    let client = MiupvClient::default();
    register_device_with_client(&client, token, device_id, os, version, model, lang).await
}

/// Registers the current device for electronic card access using a custom client.
pub async fn register_device_with_client(
    client: &MiupvClient,
    token: &str,
    device_id: &str,
    os: &str,
    version: &str,
    model: &str,
    lang: &str,
) -> Result<(), CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!(
        "{base}/pls/sorest/sic_app_vcard.alta_vcard?p_device={device_id}&p_so={os}&p_version_so={version}&p_modelo={model}&p_idioma={lang}"
    );

    let res = client
        .authed_request(Method::POST, &url, token)
        .send()
        .await?;
    let status = res.status();
    if status.is_success() || status == reqwest::StatusCode::NO_CONTENT {
        Ok(())
    } else {
        let msg = res.text().await.unwrap_or_default();
        Err(CoreError::Api {
            status,
            message: msg,
        })
    }
}
