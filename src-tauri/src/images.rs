//! Portrait covers. The app stores the CDN address each store publishes and the webview loads
//! it; no image is downloaded or cached here. Only `https` on the stores' own image hosts is
//! kept, so a response can never make the UI load an address of its choosing.

use reqwest::Url;
use serde_json::Value;

/// Image hosts of each store, checked on the parsed host, never on the text.
const HOSTS: &[&str] = &[
    "shared.akamai.steamstatic.com",
    "shared.cloudflare.steamstatic.com",
    "shared.fastly.steamstatic.com",
    "cdn.akamai.steamstatic.com",
    "cdn.cloudflare.steamstatic.com",
    "images.gog-statics.com",
    "images-1.gog-statics.com",
    "images-2.gog-statics.com",
    "images-3.gog-statics.com",
    "images-4.gog-statics.com",
    "cdn1.epicgames.com",
    "cdn2.unrealengine.com",
];

/// The address as given, when it is `https` on a known image host with no credentials or port.
pub fn checked(raw: &str) -> Option<String> {
    let url = Url::parse(raw).ok()?;
    let host = url.host_str()?;
    let ok = url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && HOSTS.contains(&host);
    ok.then(|| url.to_string())
}

const STEAM_ASSETS: &str = "https://shared.akamai.steamstatic.com/store_item_assets/";

/// Steam's usual portrait cover for an app. Newer apps keep it under a hashed path, so this is
/// only a first guess, replaced by the exact one from the store record; the UI falls back to the
/// initials when the guess does not exist.
pub fn steam_guess(appid: &str) -> Option<String> {
    if appid.is_empty() || !appid.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    checked(&format!(
        "{STEAM_ASSETS}steam/apps/{appid}/library_600x900.jpg"
    ))
}

/// Portrait cover from a family record's `capsule_filename`.
pub fn steam_capsule(appid: &str, filename: &str) -> Option<String> {
    if !appid.bytes().all(|b| b.is_ascii_digit()) || !safe_path(filename) {
        return None;
    }
    checked(&format!("{STEAM_ASSETS}steam/apps/{appid}/{filename}"))
}

/// Exact portrait cover from `IStoreBrowseService/GetItems` with `include_assets`:
/// `asset_url_format` (`steam/apps/<id>/${FILENAME}?t=...`) filled with `library_capsule`.
pub fn steam_assets(assets: &Value) -> Option<String> {
    let format = assets.get("asset_url_format")?.as_str()?;
    let file = assets.get("library_capsule")?.as_str()?;
    if !format.contains("${FILENAME}") || !safe_path(file) {
        return None;
    }
    checked(&format!(
        "{STEAM_ASSETS}{}",
        format.replace("${FILENAME}", file)
    ))
}

/// GOG's library `image` (`//images-1.gog-statics.com/<hash>`) as its vertical cover.
pub fn gog_cover(image: &str) -> Option<String> {
    let base = image.strip_prefix("//").map(|r| format!("https://{r}"))?;
    if base.contains('?') || base.contains('#') {
        return None;
    }
    checked(&format!("{base}_glx_vertical_cover.webp"))
}

/// Epic catalog `keyImages`: the tall box art, else the tall offer image, else the thumbnail.
pub fn epic_cover(catalog_item: &Value) -> Option<String> {
    let images = catalog_item.get("keyImages")?.as_array()?;
    ["DieselGameBoxTall", "OfferImageTall", "Thumbnail"]
        .iter()
        .find_map(|kind| {
            images
                .iter()
                .find(|i| i.get("type").and_then(Value::as_str) == Some(kind))
                .and_then(|i| i.get("url")?.as_str())
                .and_then(checked)
        })
}

/// A relative file path made only of safe segments: no `..`, no scheme, no host.
fn safe_path(p: &str) -> bool {
    !p.is_empty()
        && !p.starts_with('/')
        && p.split('/').all(|s| {
            !s.is_empty()
                && s != ".."
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keeps_only_https_on_store_image_hosts() {
        assert!(checked("https://cdn1.epicgames.com/spt-assets/a").is_some());
        assert!(checked("http://cdn1.epicgames.com/spt-assets/a").is_none());
        assert!(checked("https://evil.test/a.jpg").is_none());
        assert!(checked("https://cdn1.epicgames.com.evil.test/a").is_none());
        assert!(checked("https://user@cdn1.epicgames.com/a").is_none());
        assert!(checked("https://cdn1.epicgames.com:8443/a").is_none());
        assert!(checked("javascript:alert(1)").is_none());
    }

    #[test]
    fn steam_exact_cover_from_store_assets() {
        let a = json!({
            "asset_url_format": "steam/apps/1091500/${FILENAME}?t=1794474878",
            "library_capsule": "6399de67cce3cecadce7900e03f6e09ff910a235/library_capsule.jpg"
        });
        assert_eq!(
            steam_assets(&a).as_deref(),
            Some(
                "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1091500/6399de67cce3cecadce7900e03f6e09ff910a235/library_capsule.jpg?t=1794474878"
            )
        );
        let evil =
            json!({"asset_url_format": "steam/apps/1/${FILENAME}", "library_capsule": "../../x"});
        assert!(steam_assets(&evil).is_none());
        assert!(steam_assets(&json!({"library_capsule": "a.jpg"})).is_none());
    }

    #[test]
    fn steam_guess_and_family_capsule() {
        assert!(
            steam_guess("220")
                .unwrap()
                .ends_with("/steam/apps/220/library_600x900.jpg")
        );
        assert!(steam_guess("22a").is_none());
        assert!(steam_capsule("10", "library_600x900.jpg").is_some());
        assert!(steam_capsule("10", "https://evil.test/x.jpg").is_none());
    }

    #[test]
    fn gog_vertical_cover_from_library_image() {
        assert_eq!(
            gog_cover("//images-1.gog-statics.com/c6e2d2").as_deref(),
            Some("https://images-1.gog-statics.com/c6e2d2_glx_vertical_cover.webp")
        );
        assert!(gog_cover("//evil.test/c6e2d2").is_none());
        assert!(gog_cover("https://images-1.gog-statics.com/x").is_none());
    }

    #[test]
    fn epic_prefers_tall_box_art() {
        let item = json!({"keyImages": [
            {"type": "Thumbnail", "url": "https://cdn1.epicgames.com/thumb"},
            {"type": "DieselGameBoxTall", "url": "https://cdn1.epicgames.com/tall"}
        ]});
        assert_eq!(
            epic_cover(&item).as_deref(),
            Some("https://cdn1.epicgames.com/tall")
        );
        let only_bad =
            json!({"keyImages": [{"type": "DieselGameBoxTall", "url": "https://evil.test/x"}]});
        assert!(epic_cover(&only_bad).is_none());
        assert!(epic_cover(&json!({})).is_none());
    }
}
