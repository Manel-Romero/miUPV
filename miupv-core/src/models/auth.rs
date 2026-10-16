//! Authentication and OAuth token data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OAuth 2.0 PKCE tokens returned by the authentication service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthTokens {
    /// Bearer access token used to authenticate API requests.
    #[serde(alias = "accessToken", alias = "access_token")]
    pub access_token: String,

    /// Refresh token used to obtain a new access token without re-entering credentials.
    #[serde(alias = "refreshToken", alias = "refresh_token", default)]
    pub refresh_token: String,

    /// Time to live for the access token in seconds.
    #[serde(alias = "ttlAccessToken", alias = "expires_in", alias = "ttl_access_token", default)]
    pub ttl_access_token: Option<i64>,

    /// Time to live for the refresh token in seconds.
    #[serde(alias = "ttlRefreshToken", alias = "ttl_refresh_token", default)]
    pub ttl_refresh_token: Option<i64>,

    /// Calculated absolute expiration timestamp in UTC.
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

impl AuthTokens {
    /// Creates a new AuthTokens instance, initializing expires_at from ttl_access_token if present.
    pub fn new(
        access_token: String,
        refresh_token: String,
        ttl_access_token: Option<i64>,
        ttl_refresh_token: Option<i64>,
    ) -> Self {
        let expires_at = ttl_access_token.map(|ttl| Utc::now() + chrono::Duration::seconds(ttl));
        Self {
            access_token,
            refresh_token,
            ttl_access_token,
            ttl_refresh_token,
            expires_at,
        }
    }

    /// Creates an AuthTokens instance from an existing access token (e.g. extracted from official app).
    pub fn from_access_token(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: String::new(),
            ttl_access_token: None,
            ttl_refresh_token: None,
            expires_at: None,
        }
    }

    /// Creates an AuthTokens instance representing an authenticated Intranet UPV session (TDp cookie).
    pub fn new_intranet(tdp_cookie: impl Into<String>) -> Self {
        let ttl = 8 * 3600; // 8 hours typical CAS intranet session
        let expires_at = Some(Utc::now() + chrono::Duration::seconds(ttl));
        Self {
            access_token: tdp_cookie.into(),
            refresh_token: String::new(),
            ttl_access_token: Some(ttl),
            ttl_refresh_token: None,
            expires_at,
        }
    }

    /// Checks whether the token appears to be an Intranet session cookie (TDp: 32 hex characters).
    pub fn is_intranet_session(&self) -> bool {
        self.access_token.len() == 32 && self.access_token.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Checks whether the access token is present and not expired.
    pub fn is_valid(&self) -> bool {
        if self.access_token.is_empty() {
            return false;
        }
        match self.expires_at {
            Some(exp) => Utc::now() < exp,
            None => true,
        }
    }
}
