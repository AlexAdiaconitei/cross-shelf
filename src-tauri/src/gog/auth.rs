use serde::{Deserialize, Serialize};
use url::Url;

pub const CLIENT_ID: &str = "46899977096215655";
// Public Galaxy client value shipped by every third-party GOG client (see module docs).
pub const CLIENT_SECRET: &str = "9d85c43b1482497dbbce61f6e4aa173a433796eeae2ca8c5f6129f2dc4de46d9";
pub const REDIRECT_URI: &str = "https://embed.gog.com/on_login_success?origin=client";
const TOKEN_URL: &str = "https://auth.gog.com/token";
const AUTH_URL: &str = "https://auth.gog.com/auth";
/// Refresh this many seconds before the server-declared expiry.
const SKEW_SECS: u64 = 60;

/// Full token response plus the time we obtained it. Never leaves Rust.
#[derive(Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    pub login_time: u64,
}

// Manual Debug so tokens can never be printed by accident.
impl std::fmt::Debug for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tokens")
            .field("user_id", &self.user_id)
            .field("expires_in", &self.expires_in)
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    user_id: String,
    #[serde(default)]
    session_id: Option<String>,
}

impl Tokens {
    pub fn from_response(body: &str, now: u64) -> Result<Self, serde_json::Error> {
        let r: TokenResponse = serde_json::from_str(body)?;
        Ok(Self {
            access_token: r.access_token,
            refresh_token: r.refresh_token,
            expires_in: r.expires_in,
            user_id: r.user_id,
            session_id: r.session_id,
            login_time: now,
        })
    }

    pub fn expires_at(&self) -> u64 {
        self.login_time + self.expires_in
    }

    pub fn needs_refresh(&self, now: u64) -> bool {
        now + SKEW_SECS >= self.expires_at()
    }
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn login_url() -> Url {
    let mut u = Url::parse(AUTH_URL).expect("static url");
    u.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("response_type", "code")
        .append_pair("layout", "galaxy");
    u
}

/// Returns the authorization code only if `url` is exactly the GOG login-success page:
/// https scheme, host `embed.gog.com`, path `/on_login_success`, with a non-empty `code`.
/// No regexes over the full URL (spec 5.2).
pub fn parse_return_url(url: &Url) -> Option<String> {
    if url.scheme() != "https" || url.host_str() != Some("embed.gog.com") {
        return None;
    }
    if url.path() != "/on_login_success" || url.port().is_some() {
        return None;
    }
    url.query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.into_owned())
        .filter(|c| !c.is_empty())
}

pub async fn exchange_code(http: &reqwest::Client, code: &str) -> Result<Tokens, AuthError> {
    let resp = http
        .get(TOKEN_URL)
        .query(&[
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
            ("code", code),
        ])
        .send()
        .await?;
    parse_token_http(resp).await
}

pub async fn refresh(http: &reqwest::Client, t: &Tokens) -> Result<Tokens, AuthError> {
    let resp = http
        .get(TOKEN_URL)
        .query(&[
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("grant_type", "refresh_token"),
            ("refresh_token", t.refresh_token.as_str()),
        ])
        .send()
        .await?;
    parse_token_http(resp).await
}

async fn parse_token_http(resp: reqwest::Response) -> Result<Tokens, AuthError> {
    let status = resp.status();
    let body = resp.text().await?;
    if status.is_client_error() {
        // 400/401 on refresh means the grant is dead: ask the user to reconnect.
        return Err(AuthError::Rejected(status.as_u16()));
    }
    if !status.is_success() {
        return Err(AuthError::Server(status.as_u16()));
    }
    Tokens::from_response(&body, now_secs()).map_err(|_| AuthError::BadResponse)
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("GOG rejected the credentials (HTTP {0})")]
    Rejected(u16),
    #[error("GOG server error (HTTP {0})")]
    Server(u16),
    #[error("unexpected token response")]
    BadResponse,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn accepts_the_exact_return_url() {
        let url = u("https://embed.gog.com/on_login_success?origin=client&code=ABC123");
        assert_eq!(parse_return_url(&url).as_deref(), Some("ABC123"));
    }

    #[test]
    fn rejects_lookalikes() {
        for s in [
            "http://embed.gog.com/on_login_success?code=A",
            "https://embed.gog.com.evil.test/on_login_success?code=A",
            "https://evil.test/embed.gog.com/on_login_success?code=A",
            "https://embed.gog.com/on_login_success/extra?code=A",
            "https://embed.gog.com:8443/on_login_success?code=A",
            "https://embed.gog.com/other?code=A",
            "https://embed.gog.com/on_login_success?code=",
            "https://embed.gog.com/on_login_success",
            "https://www.gog.com/on_login_success?code=A",
        ] {
            assert!(parse_return_url(&u(s)).is_none(), "accepted {s}");
        }
    }

    #[test]
    fn login_url_carries_client_and_exact_redirect() {
        let url = login_url();
        assert_eq!(url.host_str(), Some("auth.gog.com"));
        let q: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(q["client_id"], CLIENT_ID);
        assert_eq!(q["redirect_uri"], REDIRECT_URI);
        assert_eq!(q["response_type"], "code");
    }

    #[test]
    fn token_response_parses_and_expiry_uses_server_value() {
        let body = r#"{"access_token":"a","refresh_token":"r","expires_in":3600,"user_id":"51234567890123456","token_type":"bearer","scope":"","session_id":"s"}"#;
        let t = Tokens::from_response(body, 1000).unwrap();
        assert_eq!(t.user_id, "51234567890123456");
        assert_eq!(t.expires_at(), 4600);
        assert!(!t.needs_refresh(1000));
        assert!(t.needs_refresh(4600 - 59));
        assert!(t.needs_refresh(9999));
    }

    #[test]
    fn user_id_survives_as_a_string() {
        let body = r#"{"access_token":"a","refresh_token":"r","expires_in":1,"user_id":"9007199254740993"}"#;
        assert_eq!(
            Tokens::from_response(body, 0).unwrap().user_id,
            "9007199254740993"
        );
    }

    #[test]
    fn debug_never_prints_tokens() {
        let t = Tokens::from_response(
            r#"{"access_token":"SENTINEL_ACCESS","refresh_token":"SENTINEL_REFRESH","expires_in":1,"user_id":"1"}"#,
            0,
        )
        .unwrap();
        assert!(!format!("{t:?}").contains("SENTINEL"));
    }
}
