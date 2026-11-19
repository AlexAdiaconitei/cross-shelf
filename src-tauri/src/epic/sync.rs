//! Epic library into the database: `library-service` records (what the account holds) joined with
//! the public catalog (title, categories, platforms).
//!
//! Built from the real library on 18 Nov 2026: 666 records, 571 distinct catalog items (the
//! Unreal Marketplace lists one record per engine version), every one with a catalog entry.
//! Categories seen: `games` (510, 13 of them add-ons through `mainGameItem`), `addons` (27),
//! Unreal Marketplace content (`assets`, `plugins`, `asset-format`, `engines`) and `hidden` (2,
//! both Fortnite content). An item with `mainGameItem` belongs to another game: map packs,
//! expansions, a soundtrack. It is never the game itself.

use crate::sync::{EntitlementRow, ProductRow};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const STORE: &str = "epic";

fn text<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}

/// `namespace:catalogItemId`, the pair that names one item in Epic's catalog (spec 5).
pub fn native_id(namespace: &str, item: &str) -> String {
    format!("{namespace}:{item}")
}

fn categories(c: &Value) -> BTreeSet<&str> {
    c.get("categories")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|x| text(x, "path")).collect())
        .unwrap_or_default()
}

/// Kind from the catalog record. Unknown categories stay `unknown`.
pub fn kind_of(c: &Value) -> &'static str {
    let cats = categories(c);
    let has = |k: &str| cats.contains(k);
    if c.get("mainGameItem").is_some_and(Value::is_object) || has("addons") {
        "dlc"
    } else if has("games") {
        "game"
    } else if has("engines") || has("software") {
        "application"
    } else if has("assets") || has("plugins") || has("asset-format") {
        "tool"
    } else {
        "unknown"
    }
}

/// Windows support from the catalog's `releaseInfo[].platform` and the library's `platform`.
/// No platform listed at all is `unknown`, never a guess.
fn windows_of(library_platforms: &[&str], c: Option<&Value>) -> &'static str {
    let mut all: BTreeSet<&str> = library_platforms.iter().copied().collect();
    if let Some(c) = c {
        for ri in c
            .get("releaseInfo")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for p in ri
                .get("platform")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(p) = p.as_str() {
                    all.insert(p);
                }
            }
        }
    }
    if all.contains("Windows") || all.contains("Win32") {
        "yes"
    } else if all.is_empty() {
        "unknown"
    } else {
        "no"
    }
}

/// One product and one owned right per distinct catalog item. Records without namespace or item
/// id are skipped. An item missing from the catalog keeps an unknown title and kind.
pub fn map_library(records: &[Value], catalog: &[Value]) -> (Vec<ProductRow>, Vec<EntitlementRow>) {
    let by_id: BTreeMap<&str, &Value> = catalog
        .iter()
        .filter_map(|c| Some((text(c, "id")?, c)))
        .collect();
    let mut seen = BTreeSet::new();
    let mut products = Vec::new();
    let mut rights = Vec::new();
    for r in records {
        let (Some(ns), Some(item)) = (text(r, "namespace"), text(r, "catalogItemId")) else {
            continue;
        };
        let id = native_id(ns, item);
        if !seen.insert(id.clone()) {
            continue;
        }
        let c = by_id.get(item).copied();
        let platforms: Vec<&str> = r
            .get("platform")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        products.push(ProductRow {
            native_id: id.clone(),
            title: c.and_then(|c| text(c, "title")).map(str::to_owned),
            kind: c.map(kind_of).unwrap_or("unknown").into(),
            windows: windows_of(&platforms, c).into(),
            url: None,
            image: c.and_then(crate::images::epic_cover),
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

/// Item ids per namespace, deduplicated, for the catalog lookup.
pub fn catalog_requests(records: &[Value]) -> BTreeMap<String, Vec<String>> {
    let mut by_ns: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for r in records {
        if let (Some(ns), Some(item)) = (text(r, "namespace"), text(r, "catalogItemId")) {
            by_ns
                .entry(ns.to_owned())
                .or_default()
                .insert(item.to_owned());
        }
    }
    by_ns
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect()
}

/// Play time from Epic's tracking, related to library items through `appName` (Epic calls it
/// `artifactId` there). `totalTime` is in seconds. Epic gives no last session. Artifacts that are
/// not in the library, and entries without a time, are left out: unknown, never 0.
pub fn map_playtime(records: &[Value], played: &[Value]) -> Vec<crate::playtime::PlaytimeRow> {
    let mut by_app: BTreeMap<&str, String> = BTreeMap::new();
    for r in records {
        if let (Some(app), Some(ns), Some(item)) = (
            text(r, "appName"),
            text(r, "namespace"),
            text(r, "catalogItemId"),
        ) {
            by_app.entry(app).or_insert_with(|| native_id(ns, item));
        }
    }
    played
        .iter()
        .filter_map(|p| {
            let id = by_app.get(text(p, "artifactId")?)?;
            let seconds = p.get("totalTime").and_then(Value::as_i64)?;
            Some(crate::playtime::PlaytimeRow {
                native_id: id.clone(),
                minutes: Some(seconds.max(0) / 60),
                last_played: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rec(ns: &str, item: &str, platform: &[&str]) -> Value {
        json!({"namespace": ns, "catalogItemId": item, "appName": "x", "productId": "p",
               "recordType": "APPLICATION", "sandboxType": "PUBLIC", "platform": platform})
    }

    fn cat(id: &str, title: &str, cats: &[&str], main: bool, platforms: &[&str]) -> Value {
        let mut v = json!({
            "id": id, "title": title, "entitlementType": "EXECUTABLE",
            "categories": cats.iter().map(|c| json!({"path": c})).collect::<Vec<_>>(),
            "releaseInfo": [{"platform": platforms}],
            "mainGameItemList": []
        });
        if main {
            v["mainGameItem"] = json!({"id": "base", "title": "Base game"});
        }
        v
    }

    fn record(ns: &str, item: &str, app: &str) -> Value {
        json!({"namespace": ns, "catalogItemId": item, "appName": app, "recordType": "APPLICATION"})
    }

    #[test]
    fn playtime_relates_artifacts_to_library_items_by_app_name() {
        let records = [
            record("ns1", "i1", "Alpha"),
            record("ns1", "i1b", "Alpha"), // a second record of the same app: first one wins
            record("ns2", "i2", "Beta"),
            record("ns3", "i3", "Gamma"),
        ];
        let played = [
            json!({"accountId": "a", "artifactId": "Alpha", "totalTime": 7265}),
            json!({"accountId": "a", "artifactId": "Beta", "totalTime": 30}),
            json!({"accountId": "a", "artifactId": "NotInLibrary", "totalTime": 999}),
            json!({"accountId": "a", "artifactId": "Gamma"}),
        ];
        let rows = map_playtime(&records, &played);
        let got: Vec<_> = rows
            .iter()
            .map(|r| (r.native_id.as_str(), r.minutes, r.last_played))
            .collect();
        assert_eq!(
            got,
            [("ns1:i1", Some(121), None), ("ns2:i2", Some(0), None)],
            "seconds round down to minutes; unknown artifacts and missing times are left out"
        );
    }

    #[test]
    fn a_playtime_error_object_is_not_an_empty_list() {
        use crate::epic::api::parse_playtime;
        assert_eq!(parse_playtime("[]").unwrap().len(), 0);
        assert!(
            parse_playtime(r#"{"errorCode":"errors.com.epicgames.common.not_found"}"#).is_err()
        );
        assert!(parse_playtime("not json").is_err());
    }

    #[test]
    fn categories_seen_in_the_real_library_map_to_kinds() {
        let games = ["applications", "games", "public"];
        assert_eq!(kind_of(&cat("1", "A", &games, false, &[])), "game");
        assert_eq!(
            kind_of(&cat("2", "Map pack", &games, true, &[])),
            "dlc",
            "a game-category item tied to a main game is an add-on"
        );
        assert_eq!(kind_of(&cat("3", "B", &["addons"], true, &[])), "dlc");
        assert_eq!(kind_of(&cat("4", "C", &["hidden"], true, &[])), "dlc");
        assert_eq!(kind_of(&cat("5", "D", &["hidden"], false, &[])), "unknown");
        assert_eq!(
            kind_of(&cat("6", "E", &["applications", "engines"], false, &[])),
            "application"
        );
        assert_eq!(
            kind_of(&cat(
                "7",
                "F",
                &["assets", "assets/environments"],
                false,
                &[]
            )),
            "tool"
        );
    }

    #[test]
    fn library_records_are_deduplicated_and_joined_with_the_catalog() {
        let records = [
            rec("ns1", "a", &["Windows"]),
            rec("ns1", "a", &["Windows"]),
            rec("ns2", "b", &["Mac"]),
            rec("ns3", "c", &[]),
            json!({"namespace": "ns4"}),
        ];
        let catalog = [
            cat(
                "a",
                "Alpha",
                &["applications", "games"],
                false,
                &["Windows"],
            ),
            cat("b", "Beta", &["applications", "games"], false, &["Mac"]),
        ];
        let (p, r) = map_library(&records, &catalog);
        let got: Vec<_> = p
            .iter()
            .map(|x| {
                (
                    x.native_id.as_str(),
                    x.title.as_deref(),
                    x.kind.as_str(),
                    x.windows.as_str(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                ("ns1:a", Some("Alpha"), "game", "yes"),
                ("ns2:b", Some("Beta"), "game", "no"),
                ("ns3:c", None, "unknown", "unknown"),
            ]
        );
        assert_eq!(r.len(), 3);
        assert!(r.iter().all(|x| x.class == "owned"));
    }

    #[test]
    fn win32_counts_as_windows() {
        assert_eq!(windows_of(&["Win32"], None), "yes");
        assert_eq!(windows_of(&[], None), "unknown");
    }

    #[test]
    fn catalog_requests_group_unique_items_by_namespace() {
        let req = catalog_requests(&[
            rec("ns1", "b", &[]),
            rec("ns1", "a", &[]),
            rec("ns1", "a", &[]),
            rec("ns2", "c", &[]),
        ]);
        assert_eq!(req["ns1"], ["a", "b"]);
        assert_eq!(req["ns2"], ["c"]);
    }

    #[test]
    fn catalog_key_images_give_the_tall_cover() {
        let records = [json!({"namespace": "ns1", "catalogItemId": "a", "platform": ["Windows"]})];
        let catalog = [json!({"id": "a", "title": "Alpha", "keyImages": [
            {"type": "DieselGameBox", "url": "https://cdn1.epicgames.com/wide"},
            {"type": "DieselGameBoxTall", "url": "https://cdn1.epicgames.com/tall"}
        ]})];
        let (p, _) = map_library(&records, &catalog);
        assert_eq!(
            p[0].image.as_deref(),
            Some("https://cdn1.epicgames.com/tall")
        );
        let (q, _) = map_library(&records, &[]);
        assert_eq!(q[0].image, None);
    }
}
