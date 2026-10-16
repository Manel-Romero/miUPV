//! Academic and personal calendar service with iCalendar (.ics) export.

use crate::client::{parse_json_list, MiupvClient};
use crate::error::CoreError;
use crate::models::CalEvent;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Fetches calendar events between the given start and end dates using the default client.
pub async fn get_ical_events(
    token: &str,
    fecha_ini: &str,
    fecha_fin: &str,
    lang: &str,
) -> Result<Vec<CalEvent>, CoreError> {
    let client = MiupvClient::default();
    get_ical_events_with_client(&client, token, fecha_ini, fecha_fin, lang).await
}

/// Normalizes standard date formats (e.g. YYYY-MM-DD) to DD/MM/YYYY required by Oracle PL/SQL.
pub fn normalize_oracle_date(date_str: &str) -> String {
    let trimmed = date_str.trim();
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return d.format("%d/%m/%Y").to_string();
    }
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%d/%m/%Y") {
        return d.format("%d/%m/%Y").to_string();
    }
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%Y/%m/%d") {
        return d.format("%d/%m/%Y").to_string();
    }
    trimmed.to_string()
}

/// Fetches calendar events between the given start and end dates using a custom client.
pub async fn get_ical_events_with_client(
    client: &MiupvClient,
    token: &str,
    fecha_ini: &str,
    fecha_fin: &str,
    lang: &str,
) -> Result<Vec<CalEvent>, CoreError> {
    let ini_fmt = normalize_oracle_date(fecha_ini);
    let fin_fmt = normalize_oracle_date(fecha_fin);
    let base = client.config().www_base_url.trim_end_matches('/');
    let url = format!(
        "{base}/pls/soalu/sic_app_ical.geteventos?p_fecha_ini={ini_fmt}&p_fecha_fin={fin_fmt}&p_idioma={lang}&p_modo="
    );

    let res = client
        .authed_request(Method::GET, &url, token)
        .send()
        .await?;
    let body = client.handle_response_text(res).await?;
    parse_json_list(&body)
}

/// Formats a raw date/time string from UPV into standard iCalendar DTSTART / DTEND value.
pub fn format_ics_date(raw: &str, all_day: bool) -> String {
    let trimmed = raw.trim();
    if all_day {
        // Look for YYYY-MM-DD or DD/MM/YYYY
        if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            return format!(";VALUE=DATE:{}", d.format("%Y%m%d"));
        }
        if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%d/%m/%Y") {
            return format!(";VALUE=DATE:{}", d.format("%Y%m%d"));
        }
        let cleaned: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if cleaned.len() >= 8 {
            return format!(";VALUE=DATE:{}", &cleaned[..8]);
        }
    }

    // Try common datetime formats
    for fmt in &[
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%d/%m/%Y %H:%M:%S",
        "%d/%m/%Y %H:%M",
    ] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            return format!(":{}", dt.format("%Y%m%dT%H%M%S"));
        }
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return format!(":{}", dt.format("%Y%m%dT%H%M%SZ"));
    }

    // Fallback: clean alphanumerics
    let cleaned: String = trimmed
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if !cleaned.is_empty() {
        format!(":{cleaned}")
    } else {
        format!(":{}", Utc::now().format("%Y%m%dT%H%M%SZ"))
    }
}

/// Generates a standard RFC 5545 iCalendar string from a list of [`CalEvent`] records.
pub fn generate_ics_content(events: &[CalEvent]) -> String {
    let now_str = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let mut out = String::new();
    out.push_str("BEGIN:VCALENDAR\r\n");
    out.push_str("VERSION:2.0\r\n");
    out.push_str("PRODID:-//miUPV//miUPV Calendar Generator 0.1.0//ES\r\n");
    out.push_str("CALSCALE:GREGORIAN\r\n");

    for ev in events {
        out.push_str("BEGIN:VEVENT\r\n");
        let uid = if ev.id.is_empty() {
            format!("event-{now_str}@miupv.upv.es")
        } else {
            format!("{}@miupv.upv.es", ev.id)
        };
        out.push_str(&format!("UID:{uid}\r\n"));
        out.push_str(&format!("DTSTAMP:{now_str}\r\n"));

        let dtstart = format_ics_date(&ev.fecha_ini, ev.todo_dia);
        out.push_str(&format!("DTSTART{dtstart}\r\n"));

        let dtend = format_ics_date(&ev.fecha_fin, ev.todo_dia);
        out.push_str(&format!("DTEND{dtend}\r\n"));

        // Sanitize summary and description (escape newlines, semicolons, commas)
        let summary = ev
            .titulo
            .replace('\n', "\\n")
            .replace('\r', "")
            .replace(',', "\\,")
            .replace(';', "\\;");
        out.push_str(&format!("SUMMARY:{summary}\r\n"));

        if let Some(desc) = &ev.descripcion {
            let escaped = desc
                .replace('\n', "\\n")
                .replace('\r', "")
                .replace(',', "\\,")
                .replace(';', "\\;");
            out.push_str(&format!("DESCRIPTION:{escaped}\r\n"));
        }

        if let Some(lugar) = &ev.lugar {
            let escaped = lugar
                .replace('\n', "\\n")
                .replace('\r', "")
                .replace(',', "\\,")
                .replace(';', "\\;");
            out.push_str(&format!("LOCATION:{escaped}\r\n"));
        }

        if let Some(tipo) = &ev.tipo {
            out.push_str(&format!("CATEGORIES:{tipo}\r\n"));
        }

        out.push_str("END:VEVENT\r\n");
    }

    out.push_str("END:VCALENDAR\r\n");
    out
}

/// Cached calendar data with last synchronization timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarCache {
    pub updated_at: DateTime<Utc>,
    pub events: Vec<CalEvent>,
}

/// Resolves the filesystem path for the calendar cache file.
pub fn get_calendar_cache_path() -> PathBuf {
    if let Ok(custom) = std::env::var("MIUPV_CALENDAR_CACHE_FILE") {
        return PathBuf::from(custom);
    }
    if let Ok(cache_home) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(cache_home)
            .join("miupv")
            .join("calendar_cache.json");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".cache")
            .join("miupv")
            .join("calendar_cache.json");
    }
    PathBuf::from("/tmp/miupv_calendar_cache.json")
}

/// Loads cached calendar events from disk, if available.
pub fn load_cached_calendar() -> Option<CalendarCache> {
    let path = get_calendar_cache_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cache) = serde_json::from_str::<CalendarCache>(&content) {
                return Some(cache);
            }
        }
    }
    None
}

/// Saves calendar events to local disk cache with current timestamp.
pub fn save_cached_calendar(events: &[CalEvent]) -> Result<(), CoreError> {
    let path = get_calendar_cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let cache = CalendarCache {
        updated_at: Utc::now(),
        events: events.to_vec(),
    };
    let json = serde_json::to_string(&cache)?;
    fs::write(path, json)?;
    Ok(())
}

/// Exports calendar events to an RFC 5545 standard `.ics` file.
pub fn export_to_ics<P: AsRef<Path>>(events: &[CalEvent], output_path: P) -> Result<(), CoreError> {
    let content = generate_ics_content(events);
    if let Some(parent) = output_path.as_ref().parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(output_path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ics_generation_format() {
        let events = vec![
            CalEvent {
                id: "EV-001".to_string(),
                titulo: "Arquitectura del Software".to_string(),
                descripcion: Some("Aula 1G 1.2 - Prof. Gomez".to_string()),
                fecha_ini: "2025-03-15 08:30:00".to_string(),
                fecha_fin: "2025-03-15 10:30:00".to_string(),
                tipo: Some("TEORIA".to_string()),
                lugar: None,
                color: Some("#336699".to_string()),
                todo_dia: false,
            },
            CalEvent {
                id: "EV-002".to_string(),
                titulo: "Fallas de Valencia".to_string(),
                descripcion: None,
                fecha_ini: "2025-03-19".to_string(),
                fecha_fin: "2025-03-19".to_string(),
                tipo: Some("FESTIVO".to_string()),
                lugar: None,
                color: None,
                todo_dia: true,
            },
        ];

        let ics = generate_ics_content(&events);

        assert!(ics.starts_with("BEGIN:VCALENDAR"));
        assert!(ics.ends_with("END:VCALENDAR\r\n"));
        assert!(ics.contains("VERSION:2.0"));
        assert!(ics.contains("UID:EV-001@miupv.upv.es"));
        assert!(ics.contains("SUMMARY:Arquitectura del Software"));
        assert!(ics.contains("DESCRIPTION:Aula 1G 1.2 - Prof. Gomez"));
        assert!(ics.contains("DTSTART:20250315T083000"));
        assert!(ics.contains("DTSTART;VALUE=DATE:20250319"));
        assert!(ics.contains("CATEGORIES:TEORIA"));
        assert!(ics.contains("CATEGORIES:FESTIVO"));
    }

    #[test]
    fn test_calendar_cache_save_and_load() {
        let temp_file = "/tmp/miupv_test_cal_cache.json";
        std::env::set_var("MIUPV_CALENDAR_CACHE_FILE", temp_file);

        let events = vec![CalEvent {
            id: "EV-TEST".to_string(),
            titulo: "Redes".to_string(),
            descripcion: None,
            fecha_ini: "2026-10-16T08:00:00.000Z".to_string(),
            fecha_fin: "2026-10-16T09:30:00.000Z".to_string(),
            tipo: Some("TEORIA".to_string()),
            lugar: Some("AULA 1E".to_string()),
            color: None,
            todo_dia: false,
        }];

        save_cached_calendar(&events).expect("Should save cache");
        let loaded = load_cached_calendar().expect("Should load cache");
        assert_eq!(loaded.events.len(), 1);
        assert_eq!(loaded.events[0].id, "EV-TEST");

        let _ = std::fs::remove_file(temp_file);
    }
}
