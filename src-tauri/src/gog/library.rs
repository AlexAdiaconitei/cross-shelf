//! GOG library and wishlist reads. Parsing is separated from HTTP so it can be tested against
//! synthetic responses. An invalid or unexpected response is an error, never an empty list
//! (spec 6.1 and 6.3).

use serde::Deserialize;
use std::collections::BTreeMap;

const WISHLIST_URL: &str = "https://embed.gog.com/user/wishlist.json";
const USERDATA_URL: &str = "https://embed.gog.com/userData.json";

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("session rejected (HTTP {0})")]
    Unauthorized(u16),
    #[error("provider error (HTTP {0})")]
    Http(u16),
    #[error("unexpected response shape: {0}")]
    Shape(&'static str),
    #[error("library pagination did not finish")]
    Incomplete,
}

impl ReadError {
    /// Plain code stored with a failed generation; never the provider's text.
    pub fn code(&self) -> &'static str {
        match self {
            ReadError::Network(_) => "network",
            ReadError::Unauthorized(_) => "needs_reauth",
            ReadError::Http(_) => "provider",
            ReadError::Shape(_) => "shape",
            ReadError::Incomplete => "incomplete",
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LibraryItem {
    pub external_id: String,
    pub platform_id: String,
}

#[derive(Debug, Deserialize)]
struct LibraryPage {
    items: Vec<LibraryItem>,
    #[serde(default)]
    next_page_token: Option<String>,
}

pub fn parse_library_page(body: &str) -> Result<(Vec<LibraryItem>, Option<String>), ReadError> {
    let p: LibraryPage =
        serde_json::from_str(body).map_err(|_| ReadError::Shape("library page"))?;
    Ok((p.items, p.next_page_token.filter(|t| !t.is_empty())))
}

/// Confirmed-empty and unreadable are different: a missing `wishlist` key is an error.
pub fn parse_wishlist(body: &str) -> Result<Vec<String>, ReadError> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|_| ReadError::Shape("wishlist json"))?;
    match v.get("wishlist") {
        Some(serde_json::Value::Object(m)) => Ok(m.keys().cloned().collect()),
        // GOG serves an empty wishlist as an empty array in some responses.
        Some(serde_json::Value::Array(a)) if a.is_empty() => Ok(vec![]),
        _ => Err(ReadError::Shape("wishlist key")),
    }
}

/// How many wishlist entries carry each JSON value (`true`, `false`, ...). Diagnostic only: the
/// endpoint returned an entry the storefront does not show, and its value may be why.
pub fn wishlist_value_kinds(body: &str) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body)
        && let Some(o) = v.get("wishlist").and_then(|w| w.as_object())
    {
        for val in o.values() {
            *m.entry(val.to_string()).or_insert(0) += 1;
        }
    }
    m
}

/// Most ids asked for in one catalog request.
pub const CATALOG_CHUNK: usize = 50;

/// Public catalog records (title, `game_type`, systems) for some product ids. No account data is
/// sent: catalog data is the same for everyone.
pub async fn fetch_catalog(
    http: &reqwest::Client,
    ids: &[String],
) -> Result<Vec<serde_json::Value>, ReadError> {
    let mut url = reqwest::Url::parse("https://api.gog.com/products").expect("static url");
    url.query_pairs_mut()
        .append_pair("ids", &ids.join(","))
        .append_pair("locale", "es-ES");
    let resp = http.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(ReadError::Http(resp.status().as_u16()));
    }
    let v: serde_json::Value = serde_json::from_str(&resp.text().await?)
        .map_err(|_| ReadError::Shape("catalog json"))?;
    v.as_array().cloned().ok_or(ReadError::Shape("catalog array"))
}

/// The wishlist map as GOG sends it, id with its value, for the local raw copy.
pub fn wishlist_raw_entries(body: &str) -> Vec<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("wishlist").and_then(|w| w.as_object()).cloned())
        .map(|m| {
            m.into_iter()
                .map(|(id, value)| serde_json::json!({ "id": id, "value": value }))
                .collect()
        })
        .unwrap_or_default()
}

pub async fn fetch_wishlist_body(
    http: &reqwest::Client,
    bearer: &str,
) -> Result<String, ReadError> {
    let url = reqwest::Url::parse(WISHLIST_URL).expect("static url");
    get_text(http, url, bearer).await
}

pub fn count_by_platform(items: &[LibraryItem]) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for i in items {
        *m.entry(i.platform_id.clone()).or_insert(0) += 1;
    }
    m
}

async fn get_text(
    http: &reqwest::Client,
    url: reqwest::Url,
    bearer: &str,
) -> Result<String, ReadError> {
    let resp = http.get(url).bearer_auth(bearer).send().await?;
    let s = resp.status();
    if s.as_u16() == 401 || s.as_u16() == 403 {
        return Err(ReadError::Unauthorized(s.as_u16()));
    }
    if !s.is_success() {
        return Err(ReadError::Http(s.as_u16()));
    }
    Ok(resp.text().await?)
}

/// Every page or an error. A half-read library is never returned as complete.
pub async fn fetch_library(
    http: &reqwest::Client,
    user_id: &str,
    bearer: &str,
) -> Result<(Vec<LibraryItem>, usize), ReadError> {
    let mut all = Vec::new();
    let mut token: Option<String> = None;
    let mut pages = 0usize;
    loop {
        let mut url = reqwest::Url::parse(&format!(
            "https://galaxy-library.gog.com/users/{user_id}/releases"
        ))
        .map_err(|_| ReadError::Shape("user id"))?;
        if let Some(t) = &token {
            url.query_pairs_mut().append_pair("page_token", t);
        }
        let (items, next) = parse_library_page(&get_text(http, url, bearer).await?)?;
        pages += 1;
        all.extend(items);
        match next {
            Some(n) => {
                if pages > 500 {
                    return Err(ReadError::Incomplete);
                }
                token = Some(n)
            }
            None => return Ok((all, pages)),
        }
    }
}

/// Raw `items` of every library page, for the shape diagnostics.
pub async fn fetch_library_raw(
    http: &reqwest::Client,
    user_id: &str,
    bearer: &str,
) -> Result<Vec<serde_json::Value>, ReadError> {
    let mut all = Vec::new();
    let mut token: Option<String> = None;
    for _ in 0..500 {
        let mut url = reqwest::Url::parse(&format!(
            "https://galaxy-library.gog.com/users/{user_id}/releases"
        ))
        .map_err(|_| ReadError::Shape("user id"))?;
        if let Some(t) = &token {
            url.query_pairs_mut().append_pair("page_token", t);
        }
        let v: serde_json::Value = serde_json::from_str(&get_text(http, url, bearer).await?)
            .map_err(|_| ReadError::Shape("library page"))?;
        all.extend(
            v.get("items")
                .and_then(|i| i.as_array())
                .cloned()
                .ok_or(ReadError::Shape("items"))?,
        );
        match v
            .get("next_page_token")
            .and_then(|t| t.as_str())
            .filter(|t| !t.is_empty())
        {
            Some(n) => token = Some(n.to_owned()),
            None => return Ok(all),
        }
    }
    Err(ReadError::Incomplete)
}

/// The account's product list as the storefront serves it (titles and types live here, not in
/// the Galaxy library). Whether the Galaxy bearer is accepted is exactly what this probes.
pub async fn fetch_filtered_products_raw(
    http: &reqwest::Client,
    bearer: &str,
) -> Result<Vec<serde_json::Value>, ReadError> {
    let mut all = Vec::new();
    let mut page = 1u32;
    loop {
        let mut url = reqwest::Url::parse("https://embed.gog.com/account/getFilteredProducts")
            .expect("static url");
        url.query_pairs_mut().append_pair("page", &page.to_string());
        let v: serde_json::Value = serde_json::from_str(&get_text(http, url, bearer).await?)
            .map_err(|_| ReadError::Shape("filtered products json"))?;
        all.extend(
            v.get("products")
                .and_then(|p| p.as_array())
                .cloned()
                .ok_or(ReadError::Shape("products"))?,
        );
        let total_pages = v.get("totalPages").and_then(|t| t.as_u64()).unwrap_or(1) as u32;
        if page >= total_pages || page >= 40 {
            return Ok(all);
        }
        page += 1;
    }
}

pub async fn fetch_wishlist_ids(
    http: &reqwest::Client,
    bearer: &str,
) -> Result<Vec<String>, ReadError> {
    let url = reqwest::Url::parse(WISHLIST_URL).expect("static url");
    parse_wishlist(&get_text(http, url, bearer).await?)
}

/// Display name from `userData.json`; avatar left out of the spike.
pub async fn fetch_username(http: &reqwest::Client, bearer: &str) -> Result<String, ReadError> {
    let url = reqwest::Url::parse(USERDATA_URL).expect("static url");
    let v: serde_json::Value = serde_json::from_str(&get_text(http, url, bearer).await?)
        .map_err(|_| ReadError::Shape("userData json"))?;
    v.get("username")
        .and_then(|u| u.as_str())
        .map(str::to_owned)
        .ok_or(ReadError::Shape("username"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_wishlist_entries_keep_each_value() {
        let e = wishlist_raw_entries(r#"{"wishlist":{"1":true,"2":false},"checksum":"x"}"#);
        assert_eq!(
            e,
            [
                serde_json::json!({"id": "1", "value": true}),
                serde_json::json!({"id": "2", "value": false})
            ]
        );
        assert!(wishlist_raw_entries("not json").is_empty());
    }

    #[test]
    fn library_page_with_and_without_next_token() {
        let (items, next) = parse_library_page(
            r#"{"items":[{"external_id":"1","platform_id":"gog","date_created":1},{"external_id":"2","platform_id":"steam"}],"next_page_token":"abc"}"#,
        )
        .unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(next.as_deref(), Some("abc"));
        let (_, none) = parse_library_page(r#"{"items":[]}"#).unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn broken_library_is_an_error_not_empty() {
        assert!(parse_library_page(r#"{"oops":true}"#).is_err());
        assert!(parse_library_page("<html>").is_err());
    }

    #[test]
    fn wishlist_distinguishes_empty_from_unreadable() {
        assert_eq!(
            parse_wishlist(r#"{"wishlist":{}}"#).unwrap(),
            Vec::<String>::new()
        );
        assert_eq!(
            parse_wishlist(r#"{"wishlist":[]}"#).unwrap(),
            Vec::<String>::new()
        );
        let mut ids = parse_wishlist(r#"{"wishlist":{"10":true,"20":true}}"#).unwrap();
        ids.sort();
        assert_eq!(ids, ["10", "20"]);
        assert!(parse_wishlist(r#"{"error":"nope"}"#).is_err());
        assert!(parse_wishlist("").is_err());
    }

    #[test]
    fn counts_by_platform_separate_gog_from_other_integrations() {
        let mk = |p: &str| LibraryItem {
            external_id: "x".into(),
            platform_id: p.into(),
        };
        let c = count_by_platform(&[mk("gog"), mk("gog"), mk("steam")]);
        assert_eq!(c["gog"], 2);
        assert_eq!(c["steam"], 1);
    }
}
