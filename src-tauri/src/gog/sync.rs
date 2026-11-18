//! GOG library and wishlist into the database. The library comes from `getFilteredProducts`
//! (titles, game or movie, Windows support): in the first real test it matched the storefront
//! exactly (268 games + 3 movies), whereas the Galaxy library carries no titles and 13 extra
//! entries the storefront does not show.

use crate::sync::{EntitlementRow, ProductRow, WishRow};
use serde_json::Value;

const STORE: &str = "gog";

pub fn store() -> &'static str {
    STORE
}

/// Maps `getFilteredProducts` records. Records without a usable id are skipped, never guessed.
pub fn map_products(records: &[Value]) -> (Vec<ProductRow>, Vec<EntitlementRow>) {
    let mut products = Vec::new();
    let mut rights = Vec::new();
    for r in records {
        let Some(id) = id_text(r.get("id")) else {
            continue;
        };
        let is_movie = r.get("isMovie").and_then(Value::as_bool) == Some(true);
        let is_game = r.get("isGame").and_then(Value::as_bool) == Some(true);
        let kind = if is_movie {
            "movie"
        } else if is_game {
            "game"
        } else {
            "unknown"
        };
        let windows = match r
            .get("worksOn")
            .and_then(|w| w.get("Windows"))
            .and_then(Value::as_bool)
        {
            Some(true) => "yes",
            Some(false) => "no",
            None => "unknown",
        };
        let url = r
            .get("url")
            .and_then(Value::as_str)
            .filter(|u| u.starts_with('/'))
            .map(|u| format!("https://www.gog.com{u}"));
        products.push(ProductRow {
            native_id: id.clone(),
            title: r.get("title").and_then(Value::as_str).map(str::to_owned),
            kind: kind.into(),
            windows: windows.into(),
            url,
            image: r
                .get("image")
                .and_then(Value::as_str)
                .and_then(crate::images::gog_cover),
        });
        rights.push(EntitlementRow {
            native_id: id,
            class: "owned".into(),
            owners: vec![],
            exclude_reason: None,
        });
    }
    (products, rights)
}

fn id_text(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

/// Public catalog records from `api.gog.com/products?ids=...`. Values of `game_type` seen on
/// 18 Nov 2026: `game` and `pack` (a bundle such as a Complete Edition). Anything else stays
/// `unknown`. Ids GOG does not know are simply absent from the answer.
pub fn map_catalog(records: &[Value]) -> Vec<ProductRow> {
    records
        .iter()
        .filter_map(|r| {
            let id = id_text(r.get("id"))?;
            let kind = match r.get("game_type").and_then(Value::as_str) {
                Some("game") => "game",
                Some("pack") => "bundle",
                _ => "unknown",
            };
            let windows = match r
                .get("content_system_compatibility")
                .and_then(|c| c.get("windows"))
                .and_then(Value::as_bool)
            {
                Some(true) => "yes",
                Some(false) => "no",
                None => "unknown",
            };
            let url = r
                .get("links")
                .and_then(|l| l.get("product_card"))
                .and_then(Value::as_str)
                .filter(|u| u.starts_with("https://www.gog.com/"))
                .map(str::to_owned);
            Some(ProductRow {
                native_id: id,
                title: r
                    .get("title")
                    .and_then(Value::as_str)
                    .filter(|t| !t.is_empty())
                    .map(str::to_owned),
                kind: kind.into(),
                windows: windows.into(),
                url,
                image: None,
            })
        })
        .collect()
}

pub fn map_wishlist(ids: &[String]) -> Vec<WishRow> {
    ids.iter()
        .map(|id| WishRow {
            native_id: id.clone(),
            priority: None,
            added_at: None,
            confirmed: true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record(id: Value, title: &str, game: bool, movie: bool, windows: Option<bool>) -> Value {
        let mut works = serde_json::Map::new();
        if let Some(w) = windows {
            works.insert("Windows".into(), json!(w));
        }
        json!({ "id": id, "title": title, "isGame": game, "isMovie": movie,
                "url": "/game/some_slug", "worksOn": works })
    }

    #[test]
    fn maps_games_movies_and_platform_support() {
        let recs = [
            record(json!(1207664663), "A", true, false, Some(true)),
            record(json!(2), "B", false, true, Some(false)),
            record(json!(3), "C", false, false, None),
        ];
        let (p, r) = map_products(&recs);
        assert_eq!(p.len(), 3);
        assert_eq!((p[0].kind.as_str(), p[0].windows.as_str()), ("game", "yes"));
        assert_eq!((p[1].kind.as_str(), p[1].windows.as_str()), ("movie", "no"));
        assert_eq!(
            (p[2].kind.as_str(), p[2].windows.as_str()),
            ("unknown", "unknown")
        );
        assert_eq!(
            p[0].url.as_deref(),
            Some("https://www.gog.com/game/some_slug")
        );
        assert!(r.iter().all(|x| x.class == "owned"));
    }

    #[test]
    fn numeric_ids_become_exact_text_and_unusable_ids_are_skipped() {
        let recs = [
            record(json!(9007199254740993_u64), "Big", true, false, Some(true)),
            json!({ "title": "no id" }),
            json!({ "id": "", "title": "empty id" }),
        ];
        let (p, _) = map_products(&recs);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].native_id, "9007199254740993");
    }

    #[test]
    fn urls_that_are_not_site_paths_are_dropped() {
        let mut rec = record(json!(1), "A", true, false, Some(true));
        rec["url"] = json!("https://evil.test/x");
        assert!(map_products(&[rec]).0[0].url.is_none());
    }

    #[test]
    fn catalog_records_give_title_type_and_windows() {
        // Trimmed copies of real public catalog records.
        let rows = map_catalog(&[
            json!({"id":1207664663,"title":"The Witcher 3: Wild Hunt - Complete Edition","slug":"the_witcher_3_wild_hunt",
                   "content_system_compatibility":{"windows":true,"osx":false,"linux":false},
                   "links":{"product_card":"https://www.gog.com/game/the_witcher_3_wild_hunt"},"game_type":"pack"}),
            json!({"id":1,"title":"Fallout Classic","content_system_compatibility":{"windows":true,"osx":true,"linux":false},
                   "links":{"product_card":"https://www.gog.com/game/fallout_classic"},"game_type":"game"}),
            json!({"id":7,"title":"Mac only","content_system_compatibility":{"windows":false,"osx":true},
                   "links":{"product_card":"https://evil.test/x"},"game_type":"dlc"}),
            json!({"title":"no id"}),
        ]);
        let got: Vec<_> = rows
            .iter()
            .map(|r| {
                (
                    r.native_id.as_str(),
                    r.kind.as_str(),
                    r.windows.as_str(),
                    r.url.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                (
                    "1207664663",
                    "bundle",
                    "yes",
                    Some("https://www.gog.com/game/the_witcher_3_wild_hunt")
                ),
                (
                    "1",
                    "game",
                    "yes",
                    Some("https://www.gog.com/game/fallout_classic")
                ),
                ("7", "unknown", "no", None),
            ]
        );
        assert_eq!(
            rows[0].title.as_deref(),
            Some("The Witcher 3: Wild Hunt - Complete Edition")
        );
    }

    #[test]
    fn wishlist_rows_keep_ids_as_text() {
        let rows = map_wishlist(&["1207664663".into()]);
        assert_eq!(rows[0].native_id, "1207664663");
        assert!(rows[0].confirmed);
    }

    #[test]
    fn library_image_becomes_the_vertical_cover() {
        let (p, _) = map_products(&[
            json!({"id": 1, "title": "Alpha", "isGame": true,
                   "image": "//images-1.gog-statics.com/abc123"}),
            json!({"id": 2, "title": "Beta", "isGame": true, "image": "//evil.test/abc"}),
        ]);
        assert_eq!(
            p[0].image.as_deref(),
            Some("https://images-1.gog-statics.com/abc123_glx_vertical_cover.webp")
        );
        assert_eq!(p[1].image, None);
    }
}
