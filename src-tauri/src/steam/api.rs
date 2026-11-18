use serde::{Deserialize, Serialize};
use url::Url;

const CONFIG_URL: &str = "https://store.steampowered.com/pointssummary/ajaxgetasyncconfig";
const API: &str = "https://api.steampowered.com";

/// Marker the login flow sends the user back to. Exact host, path and query are checked.
pub const RETURN_PATH: &str = "/about/";
pub const RETURN_FLAG: (&str, &str) = ("crossshelf", "1");

pub fn login_url() -> Url {
    let mut u = Url::parse("https://store.steampowered.com/login/").expect("static url");
    u.query_pairs_mut()
        .append_pair("redir", "about/?crossshelf=1")
        .append_pair("redir_ssl", "1");
    u
}

pub fn is_return_url(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("store.steampowered.com")
        && url.port().is_none()
        && url.path() == RETURN_PATH
        && url
            .query_pairs()
            .any(|(k, v)| k == RETURN_FLAG.0 && v == RETURN_FLAG.1)
}

/// Session material kept in the DPAPI store. `cookie_header` holds HttpOnly cookies.
#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub cookie_header: String,
    pub steamid: String,
    pub webapi_token: String,
    pub obtained_at: u64,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("steamid", &self.steamid)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SteamError {
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("session expired or not logged in")]
    NeedsReauth,
    #[error("private or not permitted")]
    Private,
    #[error("provider error (HTTP {0})")]
    Http(u16),
    #[error("unexpected response shape: {0}")]
    Shape(&'static str),
}

impl SteamError {
    /// Plain code stored with a failed generation; never the provider's text.
    pub fn code(&self) -> &'static str {
        match self {
            SteamError::Network(_) => "network",
            SteamError::NeedsReauth => "needs_reauth",
            SteamError::Private => "private",
            SteamError::Http(_) => "provider",
            SteamError::Shape(_) => "shape",
        }
    }
}

/// Parses the store config. `data.webapi_token` is empty when the cookies are not logged in.
/// The real response carries only the token (first real test), so `steamid` is optional here.
pub fn parse_config(body: &str) -> Result<(Option<String>, String), SteamError> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|_| SteamError::Shape("config json"))?;
    let data = v.get("data").ok_or(SteamError::Shape("config data"))?;
    let token = data
        .get("webapi_token")
        .and_then(|t| t.as_str())
        .unwrap_or("");
    if token.is_empty() {
        return Err(SteamError::NeedsReauth);
    }
    // steamid may be a string or a number depending on the endpoint version; keep it text.
    let id = match data.get("steamid") {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        _ => None,
    };
    Ok((id, token.to_owned()))
}

/// SteamID64 from the `steamLoginSecure` cookie, whose value starts with it followed by `%7C%7C`
/// (or `||`) and the token. Returns it only if it looks like a SteamID64.
pub fn steamid_from_cookies(cookie_header: &str) -> Option<String> {
    let value = cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|c| c.strip_prefix("steamLoginSecure="))?;
    let id: String = value.chars().take_while(char::is_ascii_digit).collect();
    (id.len() == 17 && id.starts_with("7656")).then_some(id)
}

pub async fn fetch_session(
    http: &reqwest::Client,
    cookie_header: &str,
) -> Result<Session, SteamError> {
    let resp = http
        .get(CONFIG_URL)
        .header("Cookie", cookie_header)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(SteamError::Http(resp.status().as_u16()));
    }
    let (from_body, webapi_token) = parse_config(&resp.text().await?)?;
    let steamid = from_body
        .or_else(|| steamid_from_cookies(cookie_header))
        .ok_or(SteamError::NeedsReauth)?;
    Ok(Session {
        cookie_header: cookie_header.to_owned(),
        steamid,
        webapi_token,
        obtained_at: crate::now_secs(),
    })
}

/// Describes the config response without values: status, key names, token length. For diagnosing
/// a failed login only; contains no secret.
pub async fn probe_config(http: &reqwest::Client, cookie_header: &str) -> String {
    let resp = match http
        .get(CONFIG_URL)
        .header("Cookie", cookie_header)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return "sin respuesta de la red".into(),
    };
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    describe_config(status, &body)
}

pub fn describe_config(status: u16, body: &str) -> String {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(body) else {
        return format!("HTTP {status}, cuerpo no JSON ({} bytes)", body.len());
    };
    let keys = |x: &serde_json::Value| {
        x.as_object()
            .map(|o| o.keys().cloned().collect::<Vec<_>>().join(","))
            .unwrap_or_default()
    };
    let data = v.get("data");
    let token_len = data
        .and_then(|d| d.get("webapi_token"))
        .and_then(|t| t.as_str())
        .map_or(0, str::len);
    format!(
        "HTTP {status}, claves raíz [{}], claves data [{}], longitud webapi_token {token_len}, steamid presente {}",
        keys(&v),
        data.map(keys).unwrap_or_default(),
        data.and_then(|d| d.get("steamid")).is_some()
    )
}

async fn api_get(
    http: &reqwest::Client,
    path: &str,
    token: &str,
    input_json: Option<String>,
) -> Result<serde_json::Value, SteamError> {
    let mut url = Url::parse(&format!("{API}/{path}")).map_err(|_| SteamError::Shape("url"))?;
    url.query_pairs_mut().append_pair("access_token", token);
    if let Some(j) = input_json {
        url.query_pairs_mut().append_pair("input_json", &j);
    }
    let resp = http.get(url).send().await?;
    match resp.status().as_u16() {
        200 => resp.json().await.map_err(|_| SteamError::Shape("api json")),
        401 => Err(SteamError::NeedsReauth),
        403 => Err(SteamError::Private),
        s => Err(SteamError::Http(s)),
    }
}

/// Owned games: (total, free-to-play flagged by the API when present).
pub async fn owned_games_count(http: &reqwest::Client, s: &Session) -> Result<usize, SteamError> {
    let input = serde_json::json!({
        "steamid": s.steamid, "include_appinfo": false, "include_played_free_games": true
    })
    .to_string();
    let v = api_get(
        http,
        "IPlayerService/GetOwnedGames/v1/",
        &s.webapi_token,
        Some(input),
    )
    .await?;
    count_games(&v)
}

pub fn count_games(v: &serde_json::Value) -> Result<usize, SteamError> {
    let r = v.get("response").ok_or(SteamError::Shape("response"))?;
    // An empty library still returns game_count: 0; a bare {} means nothing was read.
    r.get("game_count")
        .and_then(|c| c.as_u64())
        .map(|c| c as usize)
        .ok_or(SteamError::Shape("game_count"))
}

pub async fn wishlist_count(http: &reqwest::Client, s: &Session) -> Result<usize, SteamError> {
    let input = serde_json::json!({ "steamid": s.steamid }).to_string();
    let v = api_get(
        http,
        "IWishlistService/GetWishlist/v1/",
        &s.webapi_token,
        Some(input),
    )
    .await?;
    count_wishlist(&v)
}

pub fn count_wishlist(v: &serde_json::Value) -> Result<usize, SteamError> {
    let r = v.get("response").ok_or(SteamError::Shape("response"))?;
    match r.get("items") {
        Some(serde_json::Value::Array(a)) => Ok(a.len()),
        // Observed shape for an empty list is `response: {}`; treat only that as confirmed empty.
        None if r.as_object().is_some_and(|o| o.is_empty()) => Ok(0),
        _ => Err(SteamError::Shape("wishlist items")),
    }
}

/// Raw records for the shape diagnostics: owned games with names.
pub async fn owned_games_raw(
    http: &reqwest::Client,
    s: &Session,
) -> Result<Vec<serde_json::Value>, SteamError> {
    let input = serde_json::json!({
        "steamid": s.steamid, "include_appinfo": true, "include_played_free_games": true
    })
    .to_string();
    let v = api_get(
        http,
        "IPlayerService/GetOwnedGames/v1/",
        &s.webapi_token,
        Some(input),
    )
    .await?;
    owned_games_list(&v)
}

/// The games array. An empty library comes as `game_count: 0` without `games`: confirmed empty.
pub fn owned_games_list(v: &serde_json::Value) -> Result<Vec<serde_json::Value>, SteamError> {
    let r = v.get("response").ok_or(SteamError::Shape("response"))?;
    match r.get("games") {
        Some(serde_json::Value::Array(a)) => Ok(a.clone()),
        None if r.get("game_count").and_then(|c| c.as_u64()) == Some(0) => Ok(vec![]),
        _ => Err(SteamError::Shape("games")),
    }
}

pub async fn wishlist_raw(
    http: &reqwest::Client,
    s: &Session,
) -> Result<Vec<serde_json::Value>, SteamError> {
    let input = serde_json::json!({ "steamid": s.steamid }).to_string();
    let v = api_get(
        http,
        "IWishlistService/GetWishlist/v1/",
        &s.webapi_token,
        Some(input),
    )
    .await?;
    wishlist_list(&v)
}

/// The wishlist items. `response: {}` is the observed shape of an empty list: confirmed empty.
pub fn wishlist_list(v: &serde_json::Value) -> Result<Vec<serde_json::Value>, SteamError> {
    let r = v.get("response").ok_or(SteamError::Shape("response"))?;
    match r.get("items") {
        Some(serde_json::Value::Array(a)) => Ok(a.clone()),
        None if r.as_object().is_some_and(|o| o.is_empty()) => Ok(vec![]),
        _ => Err(SteamError::Shape("wishlist items")),
    }
}

/// Public persona name of the account, for the Connections view. Unverified with the web API
/// token: any failure just leaves the name unknown and the opaque SteamID is shown instead.
pub async fn persona_name(http: &reqwest::Client, s: &Session) -> Option<String> {
    if let Some(n) = summaries_name(http, s).await {
        return Some(n);
    }
    // Seen on 18 Nov 2026: the summaries call gave no name with the web token, while the public
    // profile XML did. A private profile may hide it; then the name stays unknown.
    if !s.steamid.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let url = format!("https://steamcommunity.com/profiles/{}/?xml=1", s.steamid);
    let body = http.get(url).send().await.ok()?.text().await.ok()?;
    parse_profile_xml_name(&body)
}

async fn summaries_name(http: &reqwest::Client, s: &Session) -> Option<String> {
    let mut url = Url::parse(&format!("{API}/ISteamUser/GetPlayerSummaries/v2/")).ok()?;
    url.query_pairs_mut()
        .append_pair("access_token", &s.webapi_token)
        .append_pair("steamids", &s.steamid);
    let v: serde_json::Value = http.get(url).send().await.ok()?.json().await.ok()?;
    parse_persona_name(&v, &s.steamid)
}

/// `<steamID><![CDATA[name]]></steamID>` from the community profile XML.
pub fn parse_profile_xml_name(body: &str) -> Option<String> {
    let start = body.find("<steamID><![CDATA[")? + "<steamID><![CDATA[".len();
    let len = body[start..].find("]]></steamID>")?;
    let name = body[start..start + len].trim();
    (!name.is_empty()).then(|| name.to_owned())
}

pub fn parse_persona_name(v: &serde_json::Value, steamid: &str) -> Option<String> {
    v.get("response")?
        .get("players")?
        .as_array()?
        .iter()
        .find(|p| p.get("steamid").and_then(|x| x.as_str()) == Some(steamid))?
        .get("personaname")?
        .as_str()
        .filter(|n| !n.is_empty())
        .map(str::to_owned)
}

pub async fn shared_apps_raw(
    http: &reqwest::Client,
    s: &Session,
    group: &str,
) -> Result<Vec<serde_json::Value>, SteamError> {
    let input = serde_json::json!({
        "family_groupid": group, "steamid": s.steamid, "language": "spanish",
        "include_own": true, "include_excluded": true, "include_non_games": true, "max_apps": 100000
    })
    .to_string();
    let v = api_get(
        http,
        "IFamilyGroupsService/GetSharedLibraryApps/v1/",
        &s.webapi_token,
        Some(input),
    )
    .await?;
    array_at(&v, &["response", "apps"])
}

fn array_at(v: &serde_json::Value, path: &[&str]) -> Result<Vec<serde_json::Value>, SteamError> {
    let mut cur = v;
    for k in path {
        cur = cur.get(k).ok_or(SteamError::Shape("array path"))?;
    }
    cur.as_array().cloned().ok_or(SteamError::Shape("array"))
}

/// Most apps asked for in one store lookup.
pub const STORE_ITEMS_CHUNK: usize = 100;

/// Public store records (name, `type`, platforms) for up to `STORE_ITEMS_CHUNK` apps. Catalog
/// data needs no account, so no token is sent. Seen on 18 Nov 2026: it answers anonymously,
/// one record per id, `success: 1` with the data or `success: 15` for an app without a page.
pub async fn store_items_raw(
    http: &reqwest::Client,
    appids: &[u64],
) -> Result<Vec<serde_json::Value>, SteamError> {
    let input = serde_json::json!({
        "ids": appids.iter().map(|a| serde_json::json!({ "appid": a })).collect::<Vec<_>>(),
        "context": { "language": "spanish", "country_code": "ES", "steam_realm": 1 },
        "data_request": { "include_platforms": true, "include_assets": true }
    })
    .to_string();
    let mut url = Url::parse(&format!("{API}/IStoreBrowseService/GetItems/v1/"))
        .map_err(|_| SteamError::Shape("url"))?;
    url.query_pairs_mut().append_pair("input_json", &input);
    let resp = http.get(url).send().await?;
    let v: serde_json::Value = match resp.status().as_u16() {
        200 => resp
            .json()
            .await
            .map_err(|_| SteamError::Shape("api json"))?,
        s => return Err(SteamError::Http(s)),
    };
    array_at(&v, &["response", "store_items"])
}

/// Family group, or `None` when the user belongs to none (a normal state, spec 6.2).
pub async fn family_group(
    http: &reqwest::Client,
    s: &Session,
) -> Result<Option<String>, SteamError> {
    let input = serde_json::json!({ "steamid": s.steamid }).to_string();
    let v = api_get(
        http,
        "IFamilyGroupsService/GetFamilyGroupForUser/v1/",
        &s.webapi_token,
        Some(input),
    )
    .await?;
    parse_family_group(&v)
}

pub fn parse_family_group(v: &serde_json::Value) -> Result<Option<String>, SteamError> {
    let r = v.get("response").ok_or(SteamError::Shape("response"))?;
    if r.get("is_not_member_of_any_group")
        .and_then(|b| b.as_bool())
        == Some(true)
    {
        return Ok(None);
    }
    match r.get("family_groupid") {
        Some(serde_json::Value::String(s)) => Ok(Some(s.clone())),
        Some(serde_json::Value::Number(n)) => Ok(Some(n.to_string())),
        _ => Err(SteamError::Shape("family_groupid")),
    }
}

/// Shared apps for the family: (count, apps with several owners).
pub async fn shared_library_counts(
    http: &reqwest::Client,
    s: &Session,
    group: &str,
) -> Result<(usize, usize), SteamError> {
    let input = serde_json::json!({
        "family_groupid": group, "steamid": s.steamid, "language": "spanish",
        "include_own": true, "include_excluded": true, "include_non_games": false, "max_apps": 100000
    })
    .to_string();
    let v = api_get(
        http,
        "IFamilyGroupsService/GetSharedLibraryApps/v1/",
        &s.webapi_token,
        Some(input),
    )
    .await?;
    count_shared(&v)
}

pub fn count_shared(v: &serde_json::Value) -> Result<(usize, usize), SteamError> {
    let apps = v
        .get("response")
        .and_then(|r| r.get("apps"))
        .and_then(|a| a.as_array())
        .ok_or(SteamError::Shape("apps"))?;
    let multi = apps
        .iter()
        .filter(|a| {
            a.get("owner_steamids")
                .and_then(|o| o.as_array())
                .is_some_and(|o| o.len() > 1)
        })
        .count();
    Ok((apps.len(), multi))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_url_is_exact() {
        let ok = Url::parse("https://store.steampowered.com/about/?crossshelf=1").unwrap();
        assert!(is_return_url(&ok));
        for s in [
            "http://store.steampowered.com/about/?crossshelf=1",
            "https://store.steampowered.com.evil.test/about/?crossshelf=1",
            "https://store.steampowered.com/about/",
            "https://store.steampowered.com/about/?crossshelf=2",
            "https://store.steampowered.com/other/?crossshelf=1",
            "https://steamcommunity.com/about/?crossshelf=1",
        ] {
            assert!(!is_return_url(&Url::parse(s).unwrap()), "accepted {s}");
        }
    }

    #[test]
    fn describe_config_never_includes_values() {
        let d = describe_config(
            200,
            r#"{"data":{"webapi_token":"SENTINEL_TOKEN","steamid":"1"}}"#,
        );
        assert!(!d.contains("SENTINEL"));
        assert!(d.contains("webapi_token") && d.contains("longitud webapi_token 14"));
    }

    #[test]
    fn config_requires_a_token_but_not_a_steamid() {
        let (id, t) =
            parse_config(r#"{"data":{"webapi_token":"tok","steamid":"76561198000000001"}}"#)
                .unwrap();
        assert_eq!(
            (id.as_deref(), t.as_str()),
            (Some("76561198000000001"), "tok")
        );
        // The shape actually observed: token only.
        let (id, t) = parse_config(r#"{"data":{"webapi_token":"tok"},"success":1}"#).unwrap();
        assert_eq!((id, t.as_str()), (None, "tok"));
        assert!(matches!(
            parse_config(r#"{"data":{"webapi_token":""}}"#),
            Err(SteamError::NeedsReauth)
        ));
        assert!(parse_config("<html>").is_err());
    }

    #[test]
    fn numeric_steamid_stays_exact_text() {
        let (id, _) =
            parse_config(r#"{"data":{"webapi_token":"t","steamid":76561198000000001}}"#).unwrap();
        assert_eq!(id.as_deref(), Some("76561198000000001"));
    }

    #[test]
    fn steamid_comes_from_the_login_cookie() {
        let h = "sessionid=abc; steamLoginSecure=76561198000000001%7C%7CTOKENPART; other=1";
        assert_eq!(
            steamid_from_cookies(h).as_deref(),
            Some("76561198000000001")
        );
        assert_eq!(
            steamid_from_cookies("steamLoginSecure=76561198000000001||x").as_deref(),
            Some("76561198000000001")
        );
        for bad in [
            "sessionid=abc",
            "steamLoginSecure=12345%7C%7Cx",
            "steamLoginSecure=%7C%7Cx",
        ] {
            assert!(steamid_from_cookies(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn family_not_member_is_not_an_error() {
        let none = serde_json::json!({"response":{"is_not_member_of_any_group":true}});
        assert_eq!(parse_family_group(&none).unwrap(), None);
        let some = serde_json::json!({"response":{"family_groupid":"90071992547409930"}});
        assert_eq!(
            parse_family_group(&some).unwrap().as_deref(),
            Some("90071992547409930")
        );
        assert!(parse_family_group(&serde_json::json!({"response":{}})).is_err());
    }

    #[test]
    fn counts_distinguish_empty_from_unreadable() {
        assert_eq!(
            count_games(&serde_json::json!({"response":{"game_count":0}})).unwrap(),
            0
        );
        assert!(count_games(&serde_json::json!({"response":{}})).is_err());
        assert_eq!(
            count_wishlist(&serde_json::json!({"response":{}})).unwrap(),
            0
        );
        assert_eq!(
            count_wishlist(&serde_json::json!({"response":{"items":[{},{}]}})).unwrap(),
            2
        );
        assert!(count_wishlist(&serde_json::json!({"nope":1})).is_err());
    }

    #[test]
    fn shared_counts_multi_owner_apps() {
        let v = serde_json::json!({"response":{"apps":[
            {"appid":1,"owner_steamids":["1"]},
            {"appid":2,"owner_steamids":["1","2"]}]}});
        assert_eq!(count_shared(&v).unwrap(), (2, 1));
    }

    #[test]
    fn empty_library_and_empty_wishlist_are_confirmed_not_errors() {
        use serde_json::json;
        assert!(
            owned_games_list(&json!({"response": {"game_count": 0}}))
                .unwrap()
                .is_empty()
        );
        assert!(owned_games_list(&json!({"response": {}})).is_err());
        assert_eq!(
            owned_games_list(&json!({"response": {"game_count": 1, "games": [{"appid": 1}]}}))
                .unwrap()
                .len(),
            1
        );
        assert!(wishlist_list(&json!({"response": {}})).unwrap().is_empty());
        assert!(wishlist_list(&json!({"response": {"items": "x"}})).is_err());
        assert!(wishlist_list(&json!({})).is_err());
    }

    #[test]
    fn persona_name_only_for_the_requested_account() {
        use serde_json::json;
        let v = json!({"response": {"players": [
            {"steamid": "76561198000000002", "personaname": "Other"},
            {"steamid": "76561198000000001", "personaname": "Me"}
        ]}});
        assert_eq!(
            parse_persona_name(&v, "76561198000000001").as_deref(),
            Some("Me")
        );
        assert_eq!(parse_persona_name(&v, "76561198000000009"), None);
        assert_eq!(parse_persona_name(&json!({}), "76561198000000001"), None);
    }

    #[test]
    fn persona_name_from_the_profile_xml() {
        let xml = "<?xml version=\"1.0\"?><profile>\n\t<steamID64>76561198000000001</steamID64>\n\t<steamID><![CDATA[Some Name]]></steamID>\n\t<onlineState>offline</onlineState>";
        assert_eq!(parse_profile_xml_name(xml).as_deref(), Some("Some Name"));
        assert_eq!(
            parse_profile_xml_name("<steamID><![CDATA[ ]]></steamID>"),
            None
        );
        assert_eq!(
            parse_profile_xml_name("<response><error/></response>"),
            None
        );
    }
}
