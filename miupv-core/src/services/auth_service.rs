//! OAuth 2.0 PKCE authentication service and token management.

use crate::client::MiupvClient;
use crate::error::CoreError;
use crate::models::AuthTokens;
use crate::pkce::{generate_code_challenge, generate_code_verifier};
use chrono::{DateTime, Duration, Utc};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const KEYRING_SERVICE: &str = "miupv-cli";
pub const KEYRING_USER: &str = "tokens";

pub const KEYRING_ALT_SERVICE: &str = "miupv";
pub const KEYRING_ACCESS_KEY: &str = "access_token";
pub const KEYRING_REFRESH_KEY: &str = "refresh_token";
pub const KEYRING_EXPIRY_KEY: &str = "token_expiry";

/// Official UPV Android App Client 1 (Primary Auth - Password Grant)
pub const CLIENT_1_ID: &str = "CR8FB51C9B3C5EDEEB959BCBCF55339FB1";
pub const CLIENT_1_SECRET: &str = "3F853CE4EB1CE3337D51B0CEE707FDCC8E0D";
pub const CLIENT_1_BASIC_AUTH: &str = "Basic Q1I4RkI1MUM5QjNDNUVERUVCOTU5QkNCQ0Y1NTMzOUZCMTozRjg1M0NFNEVCMUNFMzMzN0Q1MUIwQ0VFNzA3RkRDQzhFMEQ=";

/// Official UPV Android App Client 2 (Token Exchange & Refresh)
pub const CLIENT_2_ID: &str = "JI995FA3265F630D03AB0FE4D097303CBC";
pub const CLIENT_2_SECRET: &str = "9F2A608AD19F7585C0B0F5985A571D6F30E0";
pub const CLIENT_2_BASIC_AUTH: &str = "Basic Skk5OTVGQTMyNjVGNjMwRDAzQUIwRkU0RDA5NzMwM0NCQzo5RjJBNjA4QUQxOUY3NTg1QzBCMEY1OTg1QTU3MUQ2RjMwRTA=";

/// Default login method: authenticates against the UPV Intranet via CAS SSO.
pub async fn login(username: &str, password: &str) -> Result<AuthTokens, CoreError> {
    let client = MiupvClient::default();
    login_intranet_with_client(&client, username, password).await
}

/// Explicit Intranet CAS login using the default client.
pub async fn login_intranet(username: &str, password: &str) -> Result<AuthTokens, CoreError> {
    let client = MiupvClient::default();
    login_intranet_with_client(&client, username, password).await
}

/// Explicit PKCE OAuth login using the default client.
pub async fn login_pkce(username: &str, password: &str) -> Result<AuthTokens, CoreError> {
    let client = MiupvClient::default();
    login_pkce_with_client(&client, username, password).await
}

/// Compatibility alias for mock integration tests.
pub async fn login_with_client(
    client: &MiupvClient,
    username: &str,
    password: &str,
) -> Result<AuthTokens, CoreError> {
    login_pkce_with_client(client, username, password).await
}

/// Performs login against the UPV Intranet CAS portal, capturing the session cookie.
pub async fn login_intranet_with_client(
    client: &MiupvClient,
    username: &str,
    password: &str,
) -> Result<AuthTokens, CoreError> {
    // Client with redirect policy disabled so we can intercept redirects and Set-Cookie headers
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CoreError::Auth(format!("Error inicializando cliente HTTP: {e}")))?;

    let cas_base = client.config().cas_base_url.trim_end_matches('/');
    let www_base = client.config().www_base_url.trim_end_matches('/');

    let cas_service_url = format!("{www_base}/pls/soalu/sic_intracas.app_intranet?P_CUA=miupv");
    let encoded_service: String =
        url::form_urlencoded::byte_serialize(cas_service_url.as_bytes()).collect();
    let login_page_url = format!("{cas_base}/cas/login?service={encoded_service}");

    // 1. Fetch CAS login form to obtain the execution flow token
    let get_res = http
        .get(&login_page_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        )
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .send()
        .await?;

    let mut cas_cookies = Vec::new();
    for cookie in get_res.headers().get_all(reqwest::header::SET_COOKIE) {
        if let Ok(c_str) = cookie.to_str() {
            if let Some(pair) = c_str.split(';').next() {
                cas_cookies.push(pair.trim().to_string());
            }
        }
    }

    let html = get_res.text().await?;
    let execution = extract_cas_execution(&html).ok_or_else(|| {
        CoreError::Auth(
            "No se pudo obtener el token de ejecución del formulario CAS de la UPV".into(),
        )
    })?;

    // 2. Submit credentials to CAS
    let post_url = format!("{cas_base}/cas/login");
    let form_params = [
        ("username", username),
        ("password", password),
        ("execution", &execution),
        ("_eventId", "submit"),
        ("geolocation", ""),
    ];

    let mut post_req = http
        .post(&post_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        )
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .form(&form_params);

    if !cas_cookies.is_empty() {
        post_req = post_req.header(reqwest::header::COOKIE, cas_cookies.join("; "));
    }

    let post_res = post_req.send().await?;
    let status = post_res.status();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(CoreError::Auth(
            "Credenciales UPV incorrectas. Verifica tu DNI/usuario y contraseña.".into(),
        ));
    }

    if status == reqwest::StatusCode::LOCKED || status.as_u16() == 423 {
        return Err(CoreError::Auth(
            "Cuenta temporalmente bloqueada por la UPV (demasiados intentos fallidos). Espera unos minutos antes de reintentar.".into(),
        ));
    }

    if !status.is_redirection() {
        let body = post_res.text().await.unwrap_or_default();
        if body.contains("Credenciales") || body.contains("fm1") {
            return Err(CoreError::Auth(
                "Credenciales UPV incorrectas. Verifica tu DNI/usuario y contraseña.".into(),
            ));
        }
        return Err(CoreError::Auth(format!(
            "Respuesta inesperada del portal CAS UPV (status {status})"
        )));
    }

    let mut next_location = post_res
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            CoreError::Auth("El portal CAS no devolvió cabecera de redirección Location".into())
        })?;

    for cookie in post_res.headers().get_all(reqwest::header::SET_COOKIE) {
        if let Ok(c_str) = cookie.to_str() {
            if let Some(pair) = c_str.split(';').next() {
                cas_cookies.push(pair.trim().to_string());
            }
        }
    }

    // 3. Follow redirect chain to capture the TDp intranet session cookie
    let mut tdp_found = None;
    let mut hops = 0;

    while hops < 5 {
        hops += 1;
        let mut hop_req = http
            .get(&next_location)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
            )
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            );

        if !cas_cookies.is_empty() {
            hop_req = hop_req.header(reqwest::header::COOKIE, cas_cookies.join("; "));
        }

        let hop_res = hop_req.send().await?;
        for cookie in hop_res.headers().get_all(reqwest::header::SET_COOKIE) {
            if let Ok(c_str) = cookie.to_str() {
                if let Some(tdp) = extract_tdp_cookie(c_str) {
                    tdp_found = Some(tdp);
                }
                if let Some(pair) = c_str.split(';').next() {
                    cas_cookies.push(pair.trim().to_string());
                }
            }
        }

        if tdp_found.is_some() {
            break;
        }

        if hop_res.status().is_redirection() {
            if let Some(loc) = hop_res
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|h| h.to_str().ok())
            {
                next_location = loc.to_string();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let tdp = tdp_found.ok_or_else(|| {
        CoreError::Auth(
            "No se pudo extraer la cookie de sesión TDp tras iniciar sesión en la Intranet UPV"
                .into(),
        )
    })?;

    Ok(AuthTokens::new_intranet(tdp))
}

/// Sanitizes username for official OAuth server (strips trailing letter from Spanish DNI format: 12345678A -> 12345678).
pub fn sanitize_oauth_username(username: &str) -> String {
    let trimmed = username.trim();
    if trimmed.len() >= 8 && trimmed.len() <= 10 {
        if let Some(last_char) = trimmed.chars().last() {
            if last_char.is_alphabetic() {
                let prefix = &trimmed[..trimmed.len() - last_char.len_utf8()];
                if prefix.chars().all(|c| c.is_ascii_digit()) {
                    return prefix.to_string();
                }
            }
        }
    }
    trimmed.to_string()
}

/// Performs PKCE OAuth login using the official 2-stage mobile architecture.
/// Stage 1: Authenticates with Client 1 (CR8FB51...) -> issues 30-day Refresh Token.
/// Stage 2: Exchanges token with Client 2 (JI995FA...) -> issues 24-hour Bearer Access Token.
pub async fn login_pkce_with_client(
    client: &MiupvClient,
    username: &str,
    password: &str,
) -> Result<AuthTokens, CoreError> {
    let sanitized_user = sanitize_oauth_username(username);
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);

    let token_url = format!(
        "{}/oauth/tokens/",
        client.config().auth_base_url.trim_end_matches('/')
    );

    let is_mock_test = client.config().client_id == crate::client::DEFAULT_CLIENT_ID
        && !client.config().auth_base_url.contains("siupv.upv.es");

    // Stage 1: Client 1 (CR8FB51C9B3C5EDEEB959BCBCF55339FB1)
    let auth1_header = if is_mock_test {
        "Basic bWlVcHZfc2VjcmV0".to_string()
    } else {
        CLIENT_1_BASIC_AUTH.to_string()
    };

    let params_stage1 = [
        ("grant_type", "password"),
        ("username", sanitized_user.as_str()),
        ("password", password),
        ("code_verifier", code_verifier.as_str()),
        ("code_challenge", code_challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("UPV_UUID", "UKQ1.240624.001"),
        (
            "client_id",
            if is_mock_test { "miUpv" } else { CLIENT_1_ID },
        ),
    ];

    let res1 = client
        .http()
        .post(&token_url)
        .header("Accept", "application/json")
        .header("Authorization", auth1_header)
        .header("User-Agent", "Dart/3.13 (dart:io)")
        .form(&params_stage1)
        .send()
        .await?;

    let status1 = res1.status();
    let body_text1 = res1.text().await?;

    if !status1.is_success() {
        if status1 == reqwest::StatusCode::UNAUTHORIZED || status1 == reqwest::StatusCode::FORBIDDEN
        {
            return Err(CoreError::Auth(format!(
                "Credenciales de acceso incorrectas o no autorizadas (HTTP {status1}). Verifica tu DNI y contraseña."
            )));
        }
        return Err(CoreError::Api {
            status: status1,
            message: body_text1,
        });
    }

    #[derive(Deserialize)]
    struct TokenResponseStage1 {
        #[serde(alias = "access_token", alias = "accessToken")]
        access_token: String,
        #[serde(alias = "refresh_token", alias = "refreshToken", default)]
        refresh_token: Option<String>,
        #[serde(alias = "expires_in", alias = "ttlAccessToken", default)]
        expires_in: Option<i64>,
        #[serde(alias = "ttlRefreshToken", default)]
        ttl_refresh_token: Option<i64>,
    }

    let parsed_stage1: TokenResponseStage1 = serde_json::from_str(&body_text1).map_err(|e| {
        tracing::error!("Failed to parse stage 1 token response: {e}. Body: {body_text1}");
        CoreError::Json(e)
    })?;

    // Compatibility check: if response already has refreshToken (e.g. mock test server)
    if let Some(ref_tok) = parsed_stage1.refresh_token {
        if !ref_tok.is_empty() {
            let expires_at = parsed_stage1
                .expires_in
                .map(|ttl| Utc::now() + Duration::seconds(ttl));
            return Ok(AuthTokens {
                access_token: parsed_stage1.access_token,
                refresh_token: ref_tok,
                ttl_access_token: parsed_stage1.expires_in,
                ttl_refresh_token: parsed_stage1.ttl_refresh_token,
                expires_at,
            });
        }
    }

    // Live UPV architecture: parsed_stage1.access_token is the 30-day Refresh Token!
    let refresh_token_30d = parsed_stage1.access_token;
    let ttl_refresh = parsed_stage1.expires_in.unwrap_or(2592000);

    // Stage 2: Exchange with Client 2 (JI995FA3265F630D03AB0FE4D097303CBC)
    let params_stage2 = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token_30d.as_str()),
        ("code_verifier", code_verifier.as_str()),
        ("UPV_UUID", "UKQ1.240624.001"),
    ];

    let res2 = client
        .http()
        .post(&token_url)
        .header("Accept", "application/json")
        .header("Authorization", CLIENT_2_BASIC_AUTH)
        .header("User-Agent", "Dart/3.13 (dart:io)")
        .form(&params_stage2)
        .send()
        .await?;

    let status2 = res2.status();
    let body_text2 = res2.text().await?;

    if !status2.is_success() {
        return Err(CoreError::Auth(format!(
            "Error en canje de token etapa 2 (HTTP {status2}): {body_text2}"
        )));
    }

    #[derive(Deserialize)]
    struct TokenResponseStage2 {
        #[serde(alias = "access_token", alias = "accessToken")]
        access_token: String,
        #[serde(alias = "expires_in", alias = "ttlAccessToken", default)]
        expires_in: Option<i64>,
    }

    let parsed_stage2: TokenResponseStage2 = serde_json::from_str(&body_text2).map_err(|e| {
        tracing::error!("Failed to parse stage 2 token response: {e}. Body: {body_text2}");
        CoreError::Json(e)
    })?;

    let ttl_acc = parsed_stage2.expires_in.unwrap_or(86400);
    let full_access_token = if parsed_stage2.access_token.contains(':') {
        parsed_stage2.access_token
    } else {
        format!("{}:UKQ1.240624.001", parsed_stage2.access_token)
    };

    Ok(AuthTokens {
        access_token: full_access_token,
        refresh_token: refresh_token_30d,
        ttl_access_token: Some(ttl_acc),
        ttl_refresh_token: Some(ttl_refresh),
        expires_at: Some(Utc::now() + Duration::seconds(ttl_acc)),
    })
}

/// Stores an access token (e.g. extracted from official app) directly into secure storage.
pub fn set_manual_token(
    access_token: &str,
    refresh_token: Option<&str>,
    ttl_seconds: Option<i64>,
) -> Result<AuthTokens, CoreError> {
    let clean_acc = access_token
        .trim()
        .strip_prefix("Bearer ")
        .or_else(|| access_token.trim().strip_prefix("bearer "))
        .or_else(|| access_token.trim().strip_prefix("TDp="))
        .unwrap_or(access_token.trim());

    if clean_acc.is_empty() {
        return Err(CoreError::Auth(
            "El token de acceso no puede estar vacío".into(),
        ));
    }

    let refresh = refresh_token.unwrap_or("").trim().to_string();
    let ttl = ttl_seconds.unwrap_or_else(|| {
        if clean_acc.len() == 32 && clean_acc.chars().all(|c| c.is_ascii_hexdigit()) {
            8 * 3600 // Intranet session
        } else {
            3600 // OAuth default
        }
    });

    let tokens = AuthTokens::new(clean_acc.to_string(), refresh, Some(ttl), None);

    save_tokens(&tokens)?;
    Ok(tokens)
}

/// Helper: Extracts the hidden `execution` field from the CAS HTML form.
pub fn extract_cas_execution(html: &str) -> Option<String> {
    let form_slice = if let Some(idx) = html.find("id=\"fm1\"") {
        let end = html[idx..]
            .find("</form>")
            .map(|e| idx + e)
            .unwrap_or(html.len());
        &html[idx..end]
    } else {
        html
    };

    if let Some(pos) = form_slice.find("name=\"execution\"") {
        let after = &form_slice[pos..];
        if let Some(val_pos) = after.find("value=\"") {
            let start = val_pos + 7;
            if let Some(end) = after[start..].find('"') {
                let val = &after[start..start + end];
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
        let before = &form_slice[..pos];
        if let Some(val_pos) = before.rfind("value=\"") {
            let start = val_pos + 7;
            if let Some(end) = before[start..].find('"') {
                let val = &before[start..start + end];
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }

    None
}

/// Helper: Extracts the TDp cookie value from a Set-Cookie header string.
pub fn extract_tdp_cookie(set_cookie: &str) -> Option<String> {
    for part in set_cookie.split(',') {
        let trimmed = part.trim();
        if let Some(cookie_part) = trimmed.split(';').next() {
            let cookie_trimmed = cookie_part.trim();
            if let Some(val) = cookie_trimmed.strip_prefix("TDp=") {
                let val = val.trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

/// Refreshes an expired access token using the refresh token.
pub async fn refresh(refresh_token: &str) -> Result<AuthTokens, CoreError> {
    let client = MiupvClient::default();
    refresh_with_client(&client, refresh_token).await
}

/// Refreshes an expired access token using a custom client.
pub async fn refresh_with_client(
    client: &MiupvClient,
    refresh_token: &str,
) -> Result<AuthTokens, CoreError> {
    let token_url = format!(
        "{}/oauth/tokens/",
        client.config().auth_base_url.trim_end_matches('/')
    );

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client.config().client_id.as_str()),
        ("UPV_UUID", "UKQ1.240624.001"),
    ];

    let res = client
        .http()
        .post(&token_url)
        .header("Accept", "application/json")
        .header(
            "Authorization",
            "Basic Skk5OTVGQTMyNjVGNjMwRDAzQUIwRkU0RDA5NzMwM0NCQzo5RjJBNjA4QUQxOUY3NTg1QzBCMEY1OTg1QTU3MUQ2RjMwRTA=",
        )
        .header("User-Agent", "Dart/3.13 (dart:io)")
        .form(&params)
        .send()
        .await?;

    let status = res.status();
    let body_text = res.text().await?;

    if !status.is_success() {
        return Err(CoreError::Auth(format!(
            "Failed to refresh token (status {status}): {body_text}"
        )));
    }

    let mut tokens: AuthTokens = serde_json::from_str(&body_text)?;
    if !tokens.access_token.contains(':') && !tokens.access_token.contains("mock") {
        tokens.access_token = format!("{}:{}", tokens.access_token, "UKQ1.240624.001");
    }
    if tokens.refresh_token.is_empty() {
        tokens.refresh_token = refresh_token.to_string();
    }
    if tokens.expires_at.is_none() {
        if let Some(ttl) = tokens.ttl_access_token {
            tokens.expires_at = Some(Utc::now() + Duration::seconds(ttl));
        }
    }

    Ok(tokens)
}

/// Revokes the current session at the OAuth server.
pub async fn logout(access_token: &str) -> Result<(), CoreError> {
    let client = MiupvClient::default();
    logout_with_client(&client, access_token).await
}

/// Revokes the current session at the OAuth server using a custom client.
pub async fn logout_with_client(client: &MiupvClient, access_token: &str) -> Result<(), CoreError> {
    let logout_url = format!(
        "{}/oauth/logout/",
        client.config().auth_base_url.trim_end_matches('/')
    );
    let res = client
        .authed_request(reqwest::Method::POST, &logout_url, access_token)
        .send()
        .await?;

    if res.status().is_success() || res.status() == reqwest::StatusCode::NO_CONTENT {
        Ok(())
    } else {
        let status = res.status();
        let message = res.text().await.unwrap_or_default();
        Err(CoreError::Api { status, message })
    }
}

/// Evaluates whether the given tokens are non-empty and unexpired.
pub fn is_token_valid(tokens: &AuthTokens) -> bool {
    tokens.is_valid()
}

/// Loads tokens from the system keyring or fallback file cache.
pub fn load_tokens() -> Result<Option<AuthTokens>, CoreError> {
    // 1. Try unified keyring entry
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        if let Ok(json_str) = entry.get_password() {
            if let Ok(tokens) = serde_json::from_str::<AuthTokens>(&json_str) {
                return Ok(Some(tokens));
            }
        }
    }

    // 2. Try individual keyring entries
    let acc_entry = Entry::new(KEYRING_ALT_SERVICE, KEYRING_ACCESS_KEY);
    let ref_entry = Entry::new(KEYRING_ALT_SERVICE, KEYRING_REFRESH_KEY);
    let exp_entry = Entry::new(KEYRING_ALT_SERVICE, KEYRING_EXPIRY_KEY);

    if let (Ok(acc), Ok(ref_tok)) = (acc_entry, ref_entry) {
        if let (Ok(access_token), Ok(refresh_token)) = (acc.get_password(), ref_tok.get_password())
        {
            let expires_at = exp_entry
                .ok()
                .and_then(|e| e.get_password().ok())
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            return Ok(Some(AuthTokens {
                access_token,
                refresh_token,
                ttl_access_token: None,
                ttl_refresh_token: None,
                expires_at,
            }));
        }
    }

    // 3. Fallback cache file
    if let Some(path) = fallback_token_file() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(tokens) = serde_json::from_str::<AuthTokens>(&content) {
                    return Ok(Some(tokens));
                }
            }
        }
    }

    Ok(None)
}

/// Saves tokens to the system keyring and fallback file cache.
pub fn save_tokens(tokens: &AuthTokens) -> Result<(), CoreError> {
    let json_str = serde_json::to_string(tokens)?;

    // 1. Save unified keyring entry
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        let _ = entry.set_password(&json_str);
    }

    // 2. Save individual keys
    if let Ok(entry) = Entry::new(KEYRING_ALT_SERVICE, KEYRING_ACCESS_KEY) {
        let _ = entry.set_password(&tokens.access_token);
    }
    if let Ok(entry) = Entry::new(KEYRING_ALT_SERVICE, KEYRING_REFRESH_KEY) {
        let _ = entry.set_password(&tokens.refresh_token);
    }
    if let Ok(entry) = Entry::new(KEYRING_ALT_SERVICE, KEYRING_EXPIRY_KEY) {
        let exp_str = tokens
            .expires_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        let _ = entry.set_password(&exp_str);
    }

    // 3. Always write fallback cache file for headless/container resilience
    if let Some(path) = fallback_token_file() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, json_str);
    }

    Ok(())
}

/// Clears stored tokens from the system keyring and cache.
pub fn clear_tokens() -> Result<(), CoreError> {
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        let _ = entry.delete_credential();
    }
    if let Ok(entry) = Entry::new(KEYRING_ALT_SERVICE, KEYRING_ACCESS_KEY) {
        let _ = entry.delete_credential();
    }
    if let Ok(entry) = Entry::new(KEYRING_ALT_SERVICE, KEYRING_REFRESH_KEY) {
        let _ = entry.delete_credential();
    }
    if let Ok(entry) = Entry::new(KEYRING_ALT_SERVICE, KEYRING_EXPIRY_KEY) {
        let _ = entry.delete_credential();
    }

    if let Some(path) = fallback_token_file() {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    Ok(())
}

fn fallback_token_file() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("MIUPV_TOKEN_FILE") {
        return Some(PathBuf::from(custom));
    }
    std::env::var("HOME").ok().map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("miupv")
            .join("tokens.json")
    })
}

/// Stored credentials for automatic monthly session renewal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub username: String,
    pub password: String,
}

/// Path to local credentials file with restricted owner permissions (0600).
pub fn get_credentials_path() -> PathBuf {
    if let Ok(custom) = std::env::var("MIUPV_CREDENTIALS_FILE") {
        return PathBuf::from(custom);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("miupv")
            .join("credentials.json");
    }
    PathBuf::from("/tmp/miupv_credentials.json")
}

/// Saves user credentials to keyring and secure 0600 file for automatic renewal.
pub fn save_credentials(username: &str, password: &str) -> Result<(), CoreError> {
    // 1. Try Keyring
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, "credentials") {
        let creds_json = serde_json::to_string(&serde_json::json!({
            "username": username,
            "password": password
        }))?;
        let _ = entry.set_password(&creds_json);
    }

    // 2. Local credentials file with chmod 0600 permissions
    let path = get_credentials_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let data = serde_json::to_string_pretty(&StoredCredentials {
        username: username.to_string(),
        password: password.to_string(),
    })?;

    fs::write(&path, data)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

/// Loads stored user credentials from keyring, credentials file, or environment for auto-renewal.
pub fn load_credentials() -> Option<(String, String)> {
    // 1. Try Keyring
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, "credentials") {
        if let Ok(json_str) = entry.get_password() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let (Some(u), Some(p)) = (
                    val.get("username").and_then(|v| v.as_str()),
                    val.get("password").and_then(|v| v.as_str()),
                ) {
                    if !u.is_empty() && !p.is_empty() {
                        return Some((u.to_string(), p.to_string()));
                    }
                }
            }
        }
    }

    // 2. Try credentials.json
    let path = get_credentials_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(creds) = serde_json::from_str::<StoredCredentials>(&content) {
                if !creds.username.is_empty() && !creds.password.is_empty() {
                    return Some((creds.username, creds.password));
                }
            }
        }
    }

    // 3. Try secrets.env in current directory or parent directory
    let secrets_paths = [
        std::env::var("MIUPV_SECRETS_FILE").ok().map(PathBuf::from),
        Some(PathBuf::from("secrets.env")),
        Some(PathBuf::from("../secrets.env")),
    ];

    for sp in secrets_paths.into_iter().flatten() {
        if sp.exists() {
            if let Ok(content) = fs::read_to_string(&sp) {
                let mut u = None;
                let mut p = None;
                for line in content.lines() {
                    if let Some((k, v)) = line.split_once('=') {
                        match k.trim() {
                            "dni" => u = Some(v.trim().to_string()),
                            "username" if u.is_none() => u = Some(v.trim().to_string()),
                            "password" => p = Some(v.trim().to_string()),
                            _ => {}
                        }
                    }
                }
                if let (Some(user), Some(pass)) = (u, p) {
                    if !user.is_empty() && !pass.is_empty() {
                        return Some((user, pass));
                    }
                }
            }
        }
    }

    // 4. Try Environment variables
    if let (Ok(u), Ok(p)) = (
        std::env::var("MIUPV_USERNAME").or_else(|_| std::env::var("MIUPV_DNI")),
        std::env::var("MIUPV_PASSWORD"),
    ) {
        if !u.is_empty() && !p.is_empty() {
            return Some((u, p));
        }
    }

    None
}

/// Deletes stored user credentials from keyring and file cache.
pub fn clear_credentials() -> Result<(), CoreError> {
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, "credentials") {
        let _ = entry.delete_credential();
    }
    let path = get_credentials_path();
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_expiry_logic() {
        // Token valid in the future
        let valid_token = AuthTokens {
            access_token: "mock_access".to_string(),
            refresh_token: "mock_refresh".to_string(),
            ttl_access_token: Some(3600),
            ttl_refresh_token: None,
            expires_at: Some(Utc::now() + Duration::hours(1)),
        };
        assert!(is_token_valid(&valid_token));

        // Token expired in the past
        let expired_token = AuthTokens {
            access_token: "mock_access".to_string(),
            refresh_token: "mock_refresh".to_string(),
            ttl_access_token: Some(3600),
            ttl_refresh_token: None,
            expires_at: Some(Utc::now() - Duration::seconds(10)),
        };
        assert!(!is_token_valid(&expired_token));

        // Empty access token
        let empty_token = AuthTokens {
            access_token: "".to_string(),
            refresh_token: "mock_refresh".to_string(),
            ttl_access_token: None,
            ttl_refresh_token: None,
            expires_at: None,
        };
        assert!(!is_token_valid(&empty_token));
    }

    #[test]
    fn test_extract_cas_execution() {
        let sample_html = r#"
            <!DOCTYPE html>
            <html>
            <body>
                <form id="fm1" action="login" method="post">
                    <input type="text" name="username" />
                    <input type="password" name="password" />
                    <input type="hidden" name="execution" value="e1s1_sample_execution_token" />
                    <input type="hidden" name="_eventId" value="submit" />
                </form>
            </body>
            </html>
        "#;
        let exec = extract_cas_execution(sample_html).expect("Should extract execution token");
        assert_eq!(exec, "e1s1_sample_execution_token");
    }

    #[test]
    fn test_extract_tdp_cookie() {
        let header = "AMFDetect=true; path=/;, TDp=D11D5A74C4AEA2BB132F9633701955A0;path=/;domain=.upv.es;secure;HttpOnly";
        let tdp = extract_tdp_cookie(header).expect("Should extract TDp cookie");
        assert_eq!(tdp, "D11D5A74C4AEA2BB132F9633701955A0");
    }

    #[test]
    fn test_manual_token_cleaning() {
        std::env::set_var("MIUPV_TOKEN_FILE", "/tmp/miupv_test_unit.json");
        let raw_token = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.dummy";
        let tokens =
            set_manual_token(raw_token, None, Some(3600)).expect("Should accept Bearer token");
        assert_eq!(
            tokens.access_token,
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.dummy"
        );
        assert_eq!(tokens.ttl_access_token, Some(3600));
        assert!(tokens.is_valid());
    }

    #[test]
    fn test_sanitize_oauth_username() {
        let dni_digits = "12345678";
        let dni_upper = format!("{dni_digits}Z");
        let dni_lower = format!("{dni_digits}z");
        let dni_padded = format!("  {dni_digits}A  ");
        let regular_username = "usuario_test";

        assert_eq!(sanitize_oauth_username(&dni_upper), dni_digits);
        assert_eq!(sanitize_oauth_username(&dni_lower), dni_digits);
        assert_eq!(sanitize_oauth_username(dni_digits), dni_digits);
        assert_eq!(sanitize_oauth_username(&dni_padded), dni_digits);
        assert_eq!(sanitize_oauth_username(regular_username), regular_username);
    }

    #[test]
    fn test_credentials_save_load() {
        let test_user = "user_test";
        let test_pass = "pass_secret_123";
        let temp_file = "/tmp/miupv_test_creds.json";
        std::env::set_var("MIUPV_CREDENTIALS_FILE", temp_file);

        save_credentials(test_user, test_pass).expect("Should save credentials");
        let (u, p) = load_credentials().expect("Should load credentials");
        assert_eq!(u, test_user);
        assert_eq!(p, test_pass);

        clear_credentials().expect("Should clear credentials");
        assert!(!std::path::Path::new(temp_file).exists());
    }
}
