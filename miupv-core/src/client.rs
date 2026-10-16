//! HTTP client implementation for communicating with UPV services.

use crate::error::CoreError;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, Method, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Default base URLs for UPV backend systems.
pub const DEFAULT_AUTH_BASE_URL: &str = "https://siupv.upv.es";
pub const DEFAULT_WWW_BASE_URL: &str = "https://www.upv.es";
pub const DEFAULT_POLIFORMAT_BASE_URL: &str = "https://poliformat.upv.es";
pub const DEFAULT_APLICAT_BASE_URL: &str = "https://aplicat.upv.es";
pub const DEFAULT_CAS_BASE_URL: &str = "https://cas.upv.es";
pub const DEFAULT_INTRANET_BASE_URL: &str = "https://intranet.upv.es";

/// UPV OAuth client ID extracted from the official Android APK.
pub const DEFAULT_CLIENT_ID: &str = "miUpv";

/// Configuration settings for the MiUPV HTTP client.
#[derive(Debug, Clone)]
pub struct MiupvConfig {
    pub auth_base_url: String,
    pub www_base_url: String,
    pub poliformat_base_url: String,
    pub aplicat_base_url: String,
    pub cas_base_url: String,
    pub intranet_base_url: String,
    pub client_id: String,
    pub default_lang: String,
}

impl Default for MiupvConfig {
    fn default() -> Self {
        Self {
            auth_base_url: DEFAULT_AUTH_BASE_URL.to_string(),
            www_base_url: DEFAULT_WWW_BASE_URL.to_string(),
            poliformat_base_url: DEFAULT_POLIFORMAT_BASE_URL.to_string(),
            aplicat_base_url: DEFAULT_APLICAT_BASE_URL.to_string(),
            cas_base_url: DEFAULT_CAS_BASE_URL.to_string(),
            intranet_base_url: DEFAULT_INTRANET_BASE_URL.to_string(),
            client_id: DEFAULT_CLIENT_ID.to_string(),
            default_lang: "es".to_string(),
        }
    }
}

/// Builder for constructing a custom [`MiupvClient`].
#[derive(Debug, Default)]
pub struct MiupvClientBuilder {
    config: MiupvConfig,
    client: Option<Client>,
}

impl MiupvClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets all base URLs simultaneously (useful for mock servers in integration tests).
    pub fn all_base_urls(mut self, base_url: impl Into<String>) -> Self {
        let url = base_url.into();
        self.config.auth_base_url = url.clone();
        self.config.www_base_url = url.clone();
        self.config.poliformat_base_url = url.clone();
        self.config.aplicat_base_url = url.clone();
        self.config.cas_base_url = url.clone();
        self.config.intranet_base_url = url;
        self
    }

    pub fn auth_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.auth_base_url = url.into();
        self
    }

    pub fn www_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.www_base_url = url.into();
        self
    }

    pub fn poliformat_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.poliformat_base_url = url.into();
        self
    }

    pub fn aplicat_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.aplicat_base_url = url.into();
        self
    }

    pub fn cas_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.cas_base_url = url.into();
        self
    }

    pub fn intranet_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.intranet_base_url = url.into();
        self
    }

    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.config.client_id = client_id.into();
        self
    }

    pub fn default_lang(mut self, lang: impl Into<String>) -> Self {
        self.config.default_lang = lang.into();
        self
    }

    pub fn reqwest_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> MiupvClient {
        let http = self.client.unwrap_or_else(|| {
            Client::builder()
                .build()
                .expect("Failed to initialize reqwest HTTP client")
        });
        MiupvClient {
            http,
            config: self.config,
        }
    }
}

/// Client for making API calls to UPV endpoints.
#[derive(Debug, Clone)]
pub struct MiupvClient {
    http: Client,
    config: MiupvConfig,
}

impl Default for MiupvClient {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl MiupvClient {
    pub fn builder() -> MiupvClientBuilder {
        MiupvClientBuilder::new()
    }

    pub fn config(&self) -> &MiupvConfig {
        &self.config
    }

    pub fn http(&self) -> &Client {
        &self.http
    }

    /// Prepares an authenticated request builder with Bearer token, Cookie headers, and JSON headers.
    pub fn authed_request(&self, method: Method, url: &str, token: &str) -> RequestBuilder {
        let clean_token = token
            .trim()
            .strip_prefix("Bearer ")
            .or_else(|| token.trim().strip_prefix("bearer "))
            .or_else(|| token.trim().strip_prefix("TDp="))
            .unwrap_or(token.trim());

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/html, */*"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("miUPV-CLI/0.1.0 (Linux)"),
        );
        let auth_val = format!("Bearer {clean_token}");
        if let Ok(hv) = HeaderValue::from_str(&auth_val) {
            headers.insert(AUTHORIZATION, hv);
        }

        let cookie_val = format!("TDp={clean_token}; UPV_IDIOMA={}", self.config.default_lang);
        if let Ok(cv) = HeaderValue::from_str(&cookie_val) {
            headers.insert(reqwest::header::COOKIE, cv);
        }

        self.http.request(method, url).headers(headers)
    }

    /// Sends an authenticated GET request and deserializes the response.
    pub async fn get_authed<T: DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
    ) -> Result<T, CoreError> {
        let res = self
            .authed_request(Method::GET, url, token)
            .send()
            .await?;
        self.handle_response(res).await
    }

    /// Sends an authenticated GET request expecting a list (handles raw list or collection wrappers).
    pub async fn get_authed_list<T: DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
    ) -> Result<Vec<T>, CoreError> {
        let res = self
            .authed_request(Method::GET, url, token)
            .send()
            .await?;
        let text = self.handle_response_text(res).await?;
        parse_json_list(&text)
    }

    /// Sends an authenticated POST request with a JSON body.
    pub async fn post_authed_json<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        token: &str,
        body: &B,
    ) -> Result<T, CoreError> {
        let res = self
            .authed_request(Method::POST, url, token)
            .json(body)
            .send()
            .await?;
        self.handle_response(res).await
    }

    /// Sends an authenticated POST request without body (or query params already in URL).
    pub async fn post_authed_empty<T: DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
    ) -> Result<T, CoreError> {
        let res = self
            .authed_request(Method::POST, url, token)
            .send()
            .await?;
        self.handle_response(res).await
    }

    /// Sends an authenticated DELETE request.
    pub async fn delete_authed<T: DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
    ) -> Result<T, CoreError> {
        let res = self
            .authed_request(Method::DELETE, url, token)
            .send()
            .await?;
        self.handle_response(res).await
    }

    /// Inspects HTTP status and returns the body string or a clean API error.
    pub async fn handle_response_text(&self, res: Response) -> Result<String, CoreError> {
        let status = res.status();
        let bytes = res.bytes().await?;
        let text = match String::from_utf8(bytes.to_vec()) {
            Ok(s) => s,
            Err(e) => e.into_bytes().into_iter().map(|b| b as char).collect(),
        };
        if status.is_success() {
            Ok(text)
        } else {
            Err(CoreError::Api {
                status,
                message: clean_api_error_message(status, &text),
            })
        }
    }

    /// Inspects HTTP status and deserializes JSON response.
    pub async fn handle_response<T: DeserializeOwned>(
        &self,
        res: Response,
    ) -> Result<T, CoreError> {
        let text = self.handle_response_text(res).await?;
        // Check if response is empty string and T is unit or null
        if text.trim().is_empty() {
            if let Ok(val) = serde_json::from_str::<T>("null") {
                return Ok(val);
            }
        }
        serde_json::from_str::<T>(&text).map_err(|e| {
            tracing::error!("Failed to parse response JSON: {e}. Body: {text}");
            CoreError::Json(e)
        })
    }
}

fn clean_api_error_message(status: reqwest::StatusCode, text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return "No autorizado (401 Unauthorized). Este servicio requiere un Bearer Token OAuth 2.0 de la app oficial.".to_string();
        }
        return format!("El servidor respondió con código HTTP {status}");
    }

    if trimmed.starts_with('<') || trimmed.contains("<!DOCTYPE") || trimmed.contains("<html") {
        if status == reqwest::StatusCode::NOT_FOUND || trimmed.contains("404") {
            return "Recurso no encontrado en el servidor (404 Not Found). Puede requerir parámetros específicos o estar temporalmente inactivo.".to_string();
        } else if status == reqwest::StatusCode::UNAUTHORIZED || trimmed.contains("401") {
            return "No autorizado (401 Unauthorized). Este servicio requiere un Bearer Token OAuth 2.0 de la app oficial.".to_string();
        } else if status == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
            return "Error interno del servidor UPV (500 Internal Server Error)".to_string();
        } else {
            return format!("Respuesta HTML devuelta por el servidor (HTTP {status})");
        }
    }

    if trimmed.contains("Invalid accessTokenValue") || trimmed.contains("invalid_token") {
        return "Token no válido para este servicio. Requiere un Token Bearer OAuth 2.0 de la app oficial (incompatible con sesión de Intranet web). Usa 'miupv auth set-token <TOKEN>' para configurarlo.".to_string();
    }

    trimmed.to_string()
}

/// Helper function to parse JSON lists that might be flat arrays or wrapped in Sakai/Oracle keys.
pub fn parse_json_list<T: DeserializeOwned>(body: &str) -> Result<Vec<T>, CoreError> {
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }

    // 1. Direct array parse attempt
    if let Ok(items) = serde_json::from_str::<Vec<T>>(body) {
        return Ok(items);
    }

    // 2. Inspection of wrapped JSON objects / collections
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(arr) = val.as_array() {
            return serde_json::from_value(serde_json::Value::Array(arr.clone())).map_err(Into::into);
        }
        for key in [
            "items",
            "data",
            "padrino",
            "misNotas",
            "actas",
            "assignment_collection",
            "site_collection",
            "announcement_collection",
            "asignaturas",
            "cabinas",
            "franjas",
            "deportes",
            "DEPORTES",
            "reservas",
            "RESERVAS",
            "pistas",
            "PISTAS",
            "equipos",
            "tutorias",
            "avisos",
        ] {
            if let Some(sub) = val.get(key) {
                if let Ok(res) = serde_json::from_value::<Vec<T>>(sub.clone()) {
                    return Ok(res);
                }
            }
        }
    }

    // 3. Single item parse attempt (wrap in vec)
    if let Ok(item) = serde_json::from_str::<T>(body) {
        return Ok(vec![item]);
    }

    Err(CoreError::Parse(format!(
        "Could not parse payload as list or known collection. Payload: {body}"
    )))
}

/// Helper to parse an object that might be returned directly or as a single-element list.
pub fn parse_json_single_or_first<T: DeserializeOwned>(body: &str) -> Result<T, CoreError> {
    if let Ok(item) = serde_json::from_str::<T>(body) {
        return Ok(item);
    }
    if let Ok(mut items) = serde_json::from_str::<Vec<T>>(body) {
        if !items.is_empty() {
            return Ok(items.remove(0));
        }
    }
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        for key in ["cardData", "data", "item", "result"] {
            if let Some(sub) = val.get(key) {
                if let Ok(item) = serde_json::from_value::<T>(sub.clone()) {
                    return Ok(item);
                }
            }
        }
    }
    Err(CoreError::Parse(format!(
        "Could not parse payload as single item or list. Payload: {body}"
    )))
}
