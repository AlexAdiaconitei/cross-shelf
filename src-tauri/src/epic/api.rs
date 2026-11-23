use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

// Public launcher client values used by every third-party Epic client (see module docs).
const CLIENT_ID: &str = "34a02cf8f4414e29b15921876da36f9a";
const CLIENT_SECRET: &str = "daafbccc737745039dffe53d94fc76cf";
const OAUTH_HOST: &str = "account-public-service-prod03.ol.epicgames.com";
const LAUNCHER_HOST: &str = "launcher-public-service-prod06.ol.epicgames.com";
const ENTITLEMENTS_HOST: &str = "entitlement-public-service-prod08.ol.epicgames.com";
const LABEL: &str = "Live-EternalKnight";
const SKEW_SECS: u64 = 60;

/// Login page with the launcher's `clientId` in the redirect target. Without it Epic answers
/// "a client id is required" (observed in the first real test). Shape from legendary `egs.py`
/// `get_auth_url`, understood and not copied.
pub fn login_url() -> Url {
    let mut redirect = Url::parse("https://www.epicgames.com/id/api/redirect").expect("static url");
    redirect
        .query_pairs_mut()
        .append_pair("clientId", CLIENT_ID)
        .append_pair("responseType", "code");
    let mut u = Url::parse("https://www.epicgames.com/id/login").expect("static url");
    u.query_pairs_mut()
        .append_pair("redirectUrl", redirect.as_str());
    u
}

/// The page Epic lands on after login. It renders `{"authorizationCode": ...}` as text, so the
/// code is read from the page by the native webview API, never by page JavaScript talking to us.
pub fn is_redirect_page(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("www.epicgames.com")
        && url.port().is_none()
        && url.path() == "/id/api/redirect"
}

/// Extracts the code from the text of the redirect page. `None` while it has not rendered or when
/// it carries an error instead.
pub fn parse_redirect_body(text: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(text.trim()).ok()?;
    v.get("authorizationCode")?
        .as_str()
        .filter(|c| !c.is_empty())
        .map(str::to_owned)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub refresh_expires: u64,
    pub account_id: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub obtained_at: u64,
}

impl std::fmt::Debug for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tokens")
            .field("account_id", &self.account_id)
            .finish_non_exhaustive()
    }
}

impl Tokens {
    pub fn from_response(body: &str, now: u64) -> Result<Self, EpicError> {
        let v: serde_json::Value =
            serde_json::from_str(body).map_err(|_| EpicError::Shape("token json"))?;
        if v.get("errorCode").is_some() {
            return Err(EpicError::Rejected);
        }
        let mut t: Tokens =
            serde_json::from_value(v).map_err(|_| EpicError::Shape("token fields"))?;
        t.obtained_at = now;
        Ok(t)
    }
    pub fn needs_refresh(&self, now: u64) -> bool {
        now + SKEW_SECS >= self.obtained_at + self.expires_in
    }
    /// The refresh token itself may have expired: reconnect instead of retrying.
    pub fn refresh_usable(&self, now: u64) -> bool {
        now < self.obtained_at + self.refresh_expires
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EpicError {
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("Epic rejected the credentials")]
    Rejected,
    #[error("Epic denied the request (HTTP {0}): {1}")]
    Denied(u16, String),
    #[error("provider error (HTTP {0})")]
    Http(u16),
    #[error("unexpected response shape: {0}")]
    Shape(&'static str),
}

impl EpicError {
    /// Plain code stored with a failed generation; never the provider's text.
    pub fn code(&self) -> &'static str {
        match self {
            EpicError::Network(_) => "network",
            EpicError::Rejected => "needs_reauth",
            EpicError::Denied(..) => "private",
            EpicError::Http(_) => "provider",
            EpicError::Shape(_) => "shape",
        }
    }
}

async fn token_request(http: &reqwest::Client, form: &[(&str, &str)]) -> Result<Tokens, EpicError> {
    let resp = http
        .post(format!("https://{OAUTH_HOST}/account/api/oauth/token"))
        .basic_auth(CLIENT_ID, Some(CLIENT_SECRET))
        .form(form)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await?;
    if status.is_server_error() {
        return Err(EpicError::Http(status.as_u16()));
    }
    if status.is_client_error() {
        return Err(EpicError::Rejected);
    }
    Tokens::from_response(&body, crate::now_secs())
}

pub async fn exchange_code(http: &reqwest::Client, code: &str) -> Result<Tokens, EpicError> {
    token_request(
        http,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("token_type", "eg1"),
        ],
    )
    .await
}

pub async fn refresh(http: &reqwest::Client, t: &Tokens) -> Result<Tokens, EpicError> {
    token_request(
        http,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", t.refresh_token.as_str()),
            ("token_type", "eg1"),
        ],
    )
    .await
}

// Identity fields are parsed now and consumed by the phase 2 catalog.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Asset {
    #[serde(rename = "appName")]
    pub app_name: String,
    pub namespace: String,
    #[serde(rename = "catalogItemId")]
    pub catalog_item_id: String,
}

// Identity fields are parsed now and consumed by the phase 2 catalog.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Entitlement {
    pub namespace: String,
    #[serde(rename = "catalogItemId")]
    pub catalog_item_id: String,
    #[serde(rename = "entitlementType", default)]
    pub entitlement_type: String,
}

fn parse_list<T: for<'de> Deserialize<'de>>(
    body: &str,
    what: &'static str,
) -> Result<Vec<T>, EpicError> {
    // An error object ({"errorCode":..}) is not an empty list.
    serde_json::from_str::<Vec<T>>(body).map_err(|_| EpicError::Shape(what))
}

/// Epic's `errorCode` (and nothing else of the body) for diagnosing a denial.
pub fn error_code(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            v.get("errorCode")
                .and_then(|c| c.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "no errorCode".into())
}

/// Items of the launcher library service (the route Legendary uses for non-asset titles).
/// Counts records over all cursor pages; an unreadable page is an error, never a short count.
pub async fn library_service_count(
    http: &reqwest::Client,
    bearer: &str,
) -> Result<usize, EpicError> {
    let mut total = 0usize;
    let mut cursor: Option<String> = None;
    for _ in 0..200 {
        let mut url = Url::parse(
            "https://library-service.live.use1a.on.epicgames.com/library/api/public/items",
        )
        .expect("static url");
        url.query_pairs_mut().append_pair("includeMetadata", "true");
        if let Some(c) = &cursor {
            url.query_pairs_mut().append_pair("cursor", c);
        }
        let body = get_body(http, url.to_string(), bearer).await?;
        let (n, next) = parse_library_service_page(&body)?;
        total += n;
        match next {
            Some(c) => cursor = Some(c),
            None => return Ok(total),
        }
    }
    Err(EpicError::Shape("library pagination did not finish"))
}

/// Raw `records` of the library service over all cursor pages, for the shape diagnostics.
pub async fn library_service_raw(
    http: &reqwest::Client,
    bearer: &str,
) -> Result<Vec<serde_json::Value>, EpicError> {
    let mut all = Vec::new();
    let mut cursor: Option<String> = None;
    for _ in 0..200 {
        let mut url = Url::parse(
            "https://library-service.live.use1a.on.epicgames.com/library/api/public/items",
        )
        .expect("static url");
        url.query_pairs_mut().append_pair("includeMetadata", "true");
        if let Some(c) = &cursor {
            url.query_pairs_mut().append_pair("cursor", c);
        }
        let body = get_body(http, url.to_string(), bearer).await?;
        let v: serde_json::Value =
            serde_json::from_str(&body).map_err(|_| EpicError::Shape("library service json"))?;
        all.extend(
            v.get("records")
                .and_then(|r| r.as_array())
                .cloned()
                .ok_or(EpicError::Shape("library service records"))?,
        );
        match v
            .get("responseMetadata")
            .and_then(|m| m.get("nextCursor"))
            .and_then(|c| c.as_str())
            .filter(|c| !c.is_empty())
        {
            Some(c) => cursor = Some(c.to_owned()),
            None => return Ok(all),
        }
    }
    Err(EpicError::Shape("library pagination did not finish"))
}

/// Most item ids asked for in one catalog request, to keep the URL short.
pub const CATALOG_CHUNK: usize = 40;

/// Catalog entries for a few items of one namespace (title, categories, main game...).
/// Protocol read from legendary `get_game_info`; the response is an object keyed by item id.
pub async fn catalog_bulk_raw(
    http: &reqwest::Client,
    bearer: &str,
    namespace: &str,
    ids: &[&str],
) -> Result<Vec<serde_json::Value>, EpicError> {
    let mut url = Url::parse(&format!(
        "https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/{namespace}/bulk/items"
    ))
    .map_err(|_| EpicError::Shape("namespace"))?;
    {
        let mut q = url.query_pairs_mut();
        for id in ids {
            q.append_pair("id", id);
        }
        q.append_pair("includeDLCDetails", "true")
            .append_pair("includeMainGameDetails", "true")
            .append_pair("country", "ES")
            .append_pair("locale", "es-ES");
    }
    let body = get_body(http, url.to_string(), bearer).await?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| EpicError::Shape("catalog json"))?;
    v.as_object()
        .map(|m| m.values().cloned().collect())
        .ok_or(EpicError::Shape("catalog object"))
}

pub fn parse_library_service_page(body: &str) -> Result<(usize, Option<String>), EpicError> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|_| EpicError::Shape("library service json"))?;
    let n = v
        .get("records")
        .and_then(|r| r.as_array())
        .map(Vec::len)
        .ok_or(EpicError::Shape("library service records"))?;
    let next = v
        .get("responseMetadata")
        .and_then(|m| m.get("nextCursor"))
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
        .map(str::to_owned);
    Ok((n, next))
}

pub fn parse_assets(body: &str) -> Result<Vec<Asset>, EpicError> {
    parse_list(body, "assets")
}
pub fn parse_entitlements(body: &str) -> Result<Vec<Entitlement>, EpicError> {
    parse_list(body, "entitlements")
}

async fn get_body(http: &reqwest::Client, url: String, bearer: &str) -> Result<String, EpicError> {
    let resp = http.get(url).bearer_auth(bearer).send().await?;
    let s = resp.status();
    if s.as_u16() == 401 || s.as_u16() == 403 {
        let body = resp.text().await.unwrap_or_default();
        return Err(EpicError::Denied(s.as_u16(), error_code(&body)));
    }
    if !s.is_success() {
        return Err(EpicError::Http(s.as_u16()));
    }
    Ok(resp.text().await?)
}

/// Play time Epic tracked for the account: `[{accountId, artifactId, totalTime}]`, seconds.
/// Route read from the GOG Galaxy Epic plugin (FriendsOfGalaxy/galaxy-integration-epic); it is
/// not documented by Epic. Unvalidated until `docs/validation/epic-playtime.md` exists, so the
/// sync only calls it when [`super::PLAYTIME_VALIDATED`] is set.
pub async fn playtime_raw(
    http: &reqwest::Client,
    bearer: &str,
    account_id: &str,
) -> Result<Vec<serde_json::Value>, EpicError> {
    let mut url =
        Url::parse("https://library-service.live.use1a.on.epicgames.com/").expect("static url");
    url.path_segments_mut()
        .map_err(|_| EpicError::Shape("url"))?
        .extend([
            "library", "api", "public", "playtime", "account", account_id, "all",
        ]);
    parse_playtime(&get_body(http, url.to_string(), bearer).await?)
}

/// The play time list. An object (an error) is not an empty list.
pub fn parse_playtime(body: &str) -> Result<Vec<serde_json::Value>, EpicError> {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(serde_json::Value::Array(a)) => Ok(a),
        _ => Err(EpicError::Shape("playtime")),
    }
}

pub async fn fetch_assets(http: &reqwest::Client, bearer: &str) -> Result<Vec<Asset>, EpicError> {
    let url = format!("https://{LAUNCHER_HOST}/launcher/api/public/assets/Windows?label={LABEL}");
    parse_assets(&get_body(http, url, bearer).await?)
}

pub async fn fetch_entitlements(
    http: &reqwest::Client,
    account_id: &str,
    bearer: &str,
) -> Result<Vec<Entitlement>, EpicError> {
    let url = format!(
        "https://{ENTITLEMENTS_HOST}/entitlement/api/account/{account_id}/entitlements?start=0&count=5000"
    );
    parse_entitlements(&get_body(http, url, bearer).await?)
}

pub fn count_by_type(e: &[Entitlement]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for x in e {
        *m.entry(x.entitlement_type.clone()).or_insert(0) += 1;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_url_carries_the_client_id_in_the_redirect() {
        let u = login_url();
        assert_eq!(u.host_str(), Some("www.epicgames.com"));
        let redirect = u
            .query_pairs()
            .find(|(k, _)| k == "redirectUrl")
            .unwrap()
            .1
            .into_owned();
        let r = Url::parse(&redirect).unwrap();
        assert!(is_redirect_page(&r));
        let q: std::collections::HashMap<_, _> = r.query_pairs().into_owned().collect();
        assert_eq!(q["clientId"], CLIENT_ID);
        assert_eq!(q["responseType"], "code");
    }

    #[test]
    fn redirect_page_is_exact() {
        for s in [
            "http://www.epicgames.com/id/api/redirect",
            "https://www.epicgames.com.evil.test/id/api/redirect",
            "https://www.epicgames.com/id/api/redirect/x",
            "https://store.epicgames.com/id/api/redirect",
            "https://www.epicgames.com/id/login",
        ] {
            assert!(!is_redirect_page(&Url::parse(s).unwrap()), "accepted {s}");
        }
    }

    #[test]
    fn redirect_body_gives_code_only_when_present() {
        assert_eq!(
            parse_redirect_body(r#" {"redirectUrl":"x","authorizationCode":"abc123","sid":null} "#)
                .as_deref(),
            Some("abc123")
        );
        assert!(parse_redirect_body(r#"{"authorizationCode":""}"#).is_none());
        assert!(parse_redirect_body(r#"{"errorCode":"x"}"#).is_none());
        assert!(parse_redirect_body("").is_none());
    }

    #[test]
    fn library_service_page_counts_records_and_follows_the_cursor() {
        let (n, next) = parse_library_service_page(
            r#"{"records":[{},{}],"responseMetadata":{"nextCursor":"c2"}}"#,
        )
        .unwrap();
        assert_eq!((n, next.as_deref()), (2, Some("c2")));
        let (n, next) =
            parse_library_service_page(r#"{"records":[],"responseMetadata":{}}"#).unwrap();
        assert_eq!((n, next), (0, None));
        assert!(parse_library_service_page(r#"{"errorCode":"x"}"#).is_err());
    }

    #[test]
    fn error_code_is_extracted_without_the_rest_of_the_body() {
        assert_eq!(
            error_code(r#"{"errorCode":"errors.com.x.missing","errorMessage":"secret"}"#),
            "errors.com.x.missing"
        );
        assert_eq!(error_code("<html>"), "no errorCode");
    }

    #[test]
    fn token_parse_and_expiry() {
        let body = r#"{"access_token":"a","expires_in":7200,"refresh_token":"r","refresh_expires":28800,"account_id":"0123456789abcdef0123456789abcdef","displayName":"Test"}"#;
        let t = Tokens::from_response(body, 100).unwrap();
        assert!(!t.needs_refresh(100));
        assert!(t.needs_refresh(100 + 7200 - 59));
        assert!(t.refresh_usable(100 + 28799));
        assert!(!t.refresh_usable(100 + 28800));
    }

    #[test]
    fn error_object_is_rejection_not_tokens() {
        assert!(matches!(
            Tokens::from_response(
                r#"{"errorCode":"errors.com.epicgames.oauth.invalid_grant"}"#,
                0
            ),
            Err(EpicError::Rejected)
        ));
    }

    #[test]
    fn error_object_is_not_an_empty_library() {
        assert!(parse_assets(r#"{"errorCode":"x"}"#).is_err());
        assert_eq!(parse_assets("[]").unwrap().len(), 0);
        assert!(parse_entitlements("<html>").is_err());
    }

    #[test]
    fn debug_hides_tokens() {
        let t = Tokens::from_response(
            r#"{"access_token":"SENTINEL_A","expires_in":1,"refresh_token":"SENTINEL_R","refresh_expires":1,"account_id":"x"}"#,
            0,
        )
        .unwrap();
        assert!(!format!("{t:?}").contains("SENTINEL"));
    }
}
