//! Sports facilities and court reservation service (Servei d'Esports UPV).

use crate::client::{parse_json_list, MiupvClient};
use crate::error::CoreError;
use crate::models::{DeporteItem, PistaConReservas, ReservaDeporte};
use chrono::Local;
use reqwest::Method;

/// Normalizes campus string to UPV single-letter code ('V', 'G', 'A').
pub fn normalize_campus(campus: &str) -> &'static str {
    match campus.trim().to_lowercase().as_str() {
        "v" | "vera" => "V",
        "g" | "gandia" | "gandía" => "G",
        "a" | "alcoi" | "alcoy" => "A",
        _ => "V",
    }
}

/// Normalizes date to YYYYMMDD format required by UPV Oracle backend.
pub fn normalize_fecha(fecha: &str) -> String {
    let trimmed = fecha.trim();
    if trimmed.is_empty() {
        return Local::now().format("%Y%m%d").to_string();
    }
    // Remove dashes or slashes: e.g. "2026-10-16" -> "20261016"
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 8 {
        // If in DDMMYYYY format (starts with day <= 31 and ends with year >= 2020)
        if trimmed.contains('/') || trimmed.contains('-') {
            let parts: Vec<&str> = trimmed.split(|c| c == '-' || c == '/').collect();
            if parts.len() == 3 && parts[0].len() == 4 {
                // YYYY-MM-DD
                return format!("{}{}{}", parts[0], parts[1], parts[2]);
            } else if parts.len() == 3 && parts[2].len() == 4 {
                // DD-MM-YYYY
                return format!("{}{}{}", parts[2], parts[1], parts[0]);
            }
        }
        digits
    } else {
        Local::now().format("%Y%m%d").to_string()
    }
}

/// Resolves sport name or alias to official UPV numeric ID.
pub async fn resolve_deporte(
    client: &MiupvClient,
    token: &str,
    campus: &str,
    deporte: &str,
    lang: &str,
) -> String {
    let trimmed = deporte.trim();
    if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        return trimmed.to_string();
    }

    let lower = trimmed.to_lowercase();
    match lower.as_str() {
        "tenis" | "tennis" => "2".to_string(),
        "futbol" | "fútbol" | "soccer" => "6".to_string(),
        "futbol sala" | "fútbol sala" | "futsal" => "12".to_string(),
        "baloncesto" | "basket" | "basketball" => "14".to_string(),
        "squash" => "22".to_string(),
        "frontenis" | "fronton" | "frontón" => "28".to_string(),
        "tenis mesa" | "tenis de mesa" | "ping pong" | "pingpong" => "42".to_string(),
        "balonmano" | "handball" => "46".to_string(),
        "voleibol" | "voley" | "volleyball" => "49".to_string(),
        "pilota" | "pilota valenciana" => "121".to_string(),
        "badminton" | "bádminton" => "257".to_string(),
        "voley-playa" | "voley playa" | "vóley playa" => "278".to_string(),
        "padel" | "pádel" => "279".to_string(),
        "futbol 7" | "fútbol 7" => "311".to_string(),
        "pickleball" => "384".to_string(),
        _ => {
            // Attempt to look up in the live list of sports
            if let Ok(list) = get_deportes_with_client(client, token, campus, lang).await {
                for item in list {
                    if item.nombre.to_lowercase().contains(&lower) || lower.contains(&item.nombre.to_lowercase()) {
                        return item.id;
                    }
                }
            }
            trimmed.to_string()
        }
    }
}

/// Fetches list of available sports in a campus using the default client.
pub async fn get_deportes(token: &str, campus: &str, lang: &str) -> Result<Vec<DeporteItem>, CoreError> {
    let client = MiupvClient::default();
    get_deportes_with_client(&client, token, campus, lang).await
}

/// Fetches list of available sports in a campus using a custom client.
pub async fn get_deportes_with_client(
    client: &MiupvClient,
    token: &str,
    campus: &str,
    lang: &str,
) -> Result<Vec<DeporteItem>, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let c_code = normalize_campus(campus);
    let url = format!("{base}/pls/sorest/sic_app_deportes.lista_deportes?p_campus={c_code}&p_idioma={lang}&p_solo_compartidas=N");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Fetches current user's active sports reservations using the default client.
pub async fn get_mis_reservas(token: &str, lang: &str) -> Result<Vec<ReservaDeporte>, CoreError> {
    let client = MiupvClient::default();
    get_mis_reservas_with_client(&client, token, lang).await
}

/// Fetches current user's active sports reservations using a custom client.
pub async fn get_mis_reservas_with_client(
    client: &MiupvClient,
    token: &str,
    lang: &str,
) -> Result<Vec<ReservaDeporte>, CoreError> {
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!("{base}/pls/sorest/sic_app_deportes.mis_reservas?p_idioma={lang}");

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Fetches available sports bookings for a campus, sport, and date using the default client.
pub async fn get_reservas_disponibles(
    token: &str,
    campus: &str,
    deporte: &str,
    fecha: &str,
    lang: &str,
) -> Result<Vec<ReservaDeporte>, CoreError> {
    let client = MiupvClient::default();
    get_reservas_disponibles_with_client(&client, token, campus, deporte, fecha, lang).await
}

/// Fetches available sports bookings for a campus, sport, and date using a custom client.
pub async fn get_reservas_disponibles_with_client(
    client: &MiupvClient,
    token: &str,
    campus: &str,
    deporte: &str,
    fecha: &str,
    lang: &str,
) -> Result<Vec<ReservaDeporte>, CoreError> {
    let campus_code = normalize_campus(campus);
    let fecha_code = normalize_fecha(fecha);
    let dep_id = resolve_deporte(client, token, campus_code, deporte, lang).await;

    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!(
        "{base}/pls/sorest/sic_app_deportes.reservas_disponibles?p_campus={campus_code}&p_deporte={dep_id}&p_fecha={fecha_code}&p_pista=-1&p_mantar=&p_nivel_min=&p_nivel_max=&p_tipo_partida=&p_idioma={lang}"
    );

    let res = client.authed_request(Method::GET, &url, token).send().await?;
    let body = client.handle_response_text(res).await?;

    let pistas: Vec<PistaConReservas> = parse_json_list(&body)?;
    let mut reservas = Vec::new();

    for pista in pistas {
        for slot in pista.reservas {
            let estado_desc = match slot.estado.as_deref() {
                Some("L") => "DISPONIBLE",
                Some("O") => "OCUPADO",
                Some(other) => other,
                None => "DISPONIBLE",
            };

            reservas.push(ReservaDeporte {
                id: Some(format!("{}_{}", pista.pista, slot.hora_ini.replace(':', ""))),
                campus: Some(campus_code.to_string()),
                deporte: Some(deporte.to_string()),
                pista: Some(pista.nompista.clone()),
                fecha: Some(fecha.to_string()),
                hora_ini: Some(slot.hora_ini),
                hora_fin: Some(slot.hora_fin),
                estado: Some(estado_desc.to_string()),
            });
        }
    }

    Ok(reservas)
}
