//! Steam library, family and wishlist into the database. Shapes are those seen in the first real
//! test: owned games carry `appid` and `name` but no type; the wishlist carries only `appid`,
//! `priority` and `date_added` (names need a store lookup); family apps carry `app_type`,
//! `owner_steamids` and `exclude_reason`.

use crate::sync::{EntitlementRow, ProductRow, WishRow};
use serde_json::Value;

pub const STORE: &str = "steam";

fn id_text(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

fn app_url(appid: &str) -> String {
    format!("https://store.steampowered.com/app/{appid}")
}

/// Steam's `EAppType` flags as the family endpoint reports them. Anything else stays `unknown`
/// instead of being guessed. Unverified against Steam documentation: the real values seen were
/// 1, 2, 4, 8 and 8192.
pub fn kind_from_app_type(t: u64) -> &'static str {
    match t {
        1 => "game",
        2 => "application",
        4 => "tool",
        8 => "demo",
        32 => "dlc",
        2048 => "video",
        8192 => "soundtrack",
        _ => "unknown",
    }
}

/// Owned games with names. The type is unknown here and is shown as such.
pub fn map_owned(games: &[Value]) -> (Vec<ProductRow>, Vec<EntitlementRow>) {
    let mut products = Vec::new();
    let mut rights = Vec::new();
    for g in games {
        let Some(id) = id_text(g.get("appid")) else {
            continue;
        };
        products.push(ProductRow {
            native_id: id.clone(),
            title: g.get("name").and_then(Value::as_str).map(str::to_owned),
            kind: "unknown".into(),
            windows: "unknown".into(),
            url: Some(app_url(&id)),
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

/// Family-shared rights: only apps that someone other than `me` owns. Own copies stay `owned`.
/// `exclude_reason` is kept as text, known or not.
pub fn map_family(apps: &[Value], me: &str) -> (Vec<ProductRow>, Vec<EntitlementRow>) {
    let mut products = Vec::new();
    let mut rights = Vec::new();
    for a in apps {
        let Some(id) = id_text(a.get("appid")) else {
            continue;
        };
        let owners: Vec<String> = a
            .get("owner_steamids")
            .and_then(Value::as_array)
            .map(|o| o.iter().filter_map(|x| id_text(Some(x))).collect())
            .unwrap_or_default();
        let others: Vec<String> = owners
            .iter()
            .filter(|o| o.as_str() != me)
            .cloned()
            .collect();
        let kind = a
            .get("app_type")
            .and_then(Value::as_u64)
            .map(kind_from_app_type)
            .unwrap_or("unknown");
        products.push(ProductRow {
            native_id: id.clone(),
            title: a.get("name").and_then(Value::as_str).map(str::to_owned),
            kind: kind.into(),
            windows: "unknown".into(),
            url: Some(app_url(&id)),
        });
        if others.is_empty() {
            continue;
        }
        rights.push(EntitlementRow {
            native_id: id,
            class: "family_shared".into(),
            owners: others,
            exclude_reason: a
                .get("exclude_reason")
                .and_then(Value::as_u64)
                .map(|r| r.to_string()),
        });
    }
    (products, rights)
}

/// The store's own `type` in `IStoreBrowseService/GetItems`, a different numbering from
/// `app_type`. Only values seen on real pages on 18 Nov 2026: 0 game (Half-Life 2), 1 demo
/// (Half-Life 2: Demo), 4 DLC (Blood and Wine), 6 software (Blender), 11 soundtrack. Hardware (10)
/// and anything else stay `unknown`.
pub fn kind_from_store_type(t: u64) -> &'static str {
    match t {
        0 => "game",
        1 => "demo",
        4 => "dlc",
        6 => "application",
        11 => "soundtrack",
        _ => "unknown",
    }
}

/// Name, type and Windows support from store records. Records without `success: 1` carry no data
/// and are skipped, so whatever was known stays. `platforms` lists only the systems it supports:
/// with `platforms` present and no `windows`, the app does not run on Windows.
pub fn map_store_items(items: &[Value]) -> Vec<ProductRow> {
    items
        .iter()
        .filter(|i| i.get("success").and_then(Value::as_u64) == Some(1))
        .filter_map(|i| {
            // `id` is the one asked for; `appid` comes back as 0 in records without a page.
            let id = id_text(i.get("id").or_else(|| i.get("appid")))?;
            let windows = match i.get("platforms") {
                Some(p) if p.get("windows").and_then(Value::as_bool) == Some(true) => "yes",
                Some(Value::Object(_)) => "no",
                _ => "unknown",
            };
            Some(ProductRow {
                title: i
                    .get("name")
                    .and_then(Value::as_str)
                    .filter(|n| !n.is_empty())
                    .map(str::to_owned),
                kind: i
                    .get("type")
                    .and_then(Value::as_u64)
                    .map(kind_from_store_type)
                    .unwrap_or("unknown")
                    .into(),
                windows: windows.into(),
                url: Some(app_url(&id)),
                native_id: id,
            })
        })
        .collect()
}

pub fn map_wishlist(items: &[Value]) -> Vec<WishRow> {
    items
        .iter()
        .filter_map(|i| {
            Some(WishRow {
                native_id: id_text(i.get("appid"))?,
                // In the real list (18 Nov 2026) 321 of 344 entries had priority 0 and the rest
                // unique, non-consecutive ranks from 2 up: 0 means "not ranked", not "first".
                priority: i
                    .get("priority")
                    .and_then(Value::as_i64)
                    .filter(|p| *p > 0),
                added_at: i.get("date_added").and_then(Value::as_i64),
                confirmed: true,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ME: &str = "76561198000000001";

    #[test]
    fn owned_games_keep_names_and_leave_the_type_unknown() {
        let (p, r) = map_owned(&[
            json!({"appid": 10, "name": "Alpha", "playtime_forever": 5}),
            json!({"name": "no id"}),
        ]);
        assert_eq!(p.len(), 1);
        assert_eq!(
            (p[0].title.as_deref(), p[0].kind.as_str()),
            (Some("Alpha"), "unknown")
        );
        assert_eq!(r[0].class, "owned");
    }

    #[test]
    fn family_rights_only_for_apps_someone_else_owns() {
        let apps = [
            json!({"appid": 1, "name": "Own only", "app_type": 1, "owner_steamids": [ME], "exclude_reason": 0}),
            json!({"appid": 2, "name": "Shared", "app_type": 1, "owner_steamids": [ME, "76561198000000002"], "exclude_reason": 0}),
            json!({"appid": 3, "name": "Theirs", "app_type": 4, "owner_steamids": ["76561198000000003"], "exclude_reason": 3}),
        ];
        let (products, rights) = map_family(&apps, ME);
        assert_eq!(
            products.len(),
            3,
            "products are recorded for all, rights only when shared"
        );
        assert_eq!(rights.len(), 2);
        let shared = rights.iter().find(|r| r.native_id == "2").unwrap();
        assert_eq!(
            shared.owners,
            ["76561198000000002"],
            "the user is not their own lender"
        );
        let theirs = rights.iter().find(|r| r.native_id == "3").unwrap();
        assert_eq!(theirs.exclude_reason.as_deref(), Some("3"));
        assert_eq!(products[2].kind, "tool");
    }

    #[test]
    fn unknown_app_types_and_exclusion_reasons_survive() {
        let apps = [
            json!({"appid": 9, "name": "X", "app_type": 99999, "owner_steamids": ["76561198000000002"], "exclude_reason": 77}),
        ];
        let (p, r) = map_family(&apps, ME);
        assert_eq!(p[0].kind, "unknown");
        assert_eq!(r[0].exclude_reason.as_deref(), Some("77"));
    }

    #[test]
    fn app_type_values_seen_in_the_real_test() {
        assert_eq!(kind_from_app_type(1), "game");
        assert_eq!(kind_from_app_type(2), "application");
        assert_eq!(kind_from_app_type(4), "tool");
        assert_eq!(kind_from_app_type(8), "demo");
        assert_eq!(kind_from_app_type(8192), "soundtrack");
        assert_eq!(kind_from_app_type(3), "unknown");
    }

    #[test]
    fn store_items_give_name_type_and_windows() {
        // Trimmed copies of real GetItems records (public catalog data).
        let rows = map_store_items(&[
            json!({"item_type":0,"id":220,"success":1,"visible":true,"name":"Half-Life 2","appid":220,"type":0,
                   "platforms":{"windows":true,"steamos_linux":true,"vr_support":{}}}),
            json!({"item_type":0,"id":378648,"success":1,"name":"The Witcher 3: Wild Hunt - Blood and Wine",
                   "appid":378648,"type":4,"related_items":{"parent_appid":292030},"platforms":{"windows":true}}),
            json!({"item_type":0,"id":630380,"success":1,"name":"Castle of Shikigami - Soundtrack","appid":630380,"type":11,
                   "platforms":{"windows":true}}),
            json!({"item_type":0,"id":1675200,"success":1,"name":"Steam Deck","appid":1675200,"type":10}),
            json!({"item_type":0,"id":323370,"success":15}),
            json!({"id":5,"success":1,"name":"Mac only","type":0,"platforms":{"mac":true,"vr_support":{}}}),
        ]);
        let got: Vec<_> = rows
            .iter()
            .map(|r| {
                (
                    r.native_id.as_str(),
                    r.title.as_deref(),
                    r.kind.as_str(),
                    r.windows.as_str(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                ("220", Some("Half-Life 2"), "game", "yes"),
                (
                    "378648",
                    Some("The Witcher 3: Wild Hunt - Blood and Wine"),
                    "dlc",
                    "yes"
                ),
                (
                    "630380",
                    Some("Castle of Shikigami - Soundtrack"),
                    "soundtrack",
                    "yes"
                ),
                ("1675200", Some("Steam Deck"), "unknown", "unknown"),
                ("5", Some("Mac only"), "game", "no"),
            ],
            "a record without data (success 15) is skipped"
        );
    }

    #[test]
    fn store_types_outside_the_seen_ones_stay_unknown() {
        assert_eq!(kind_from_store_type(6), "application");
        assert_eq!(kind_from_store_type(1), "demo");
        for t in [2, 3, 5, 7, 10, 12, 99] {
            assert_eq!(kind_from_store_type(t), "unknown");
        }
    }

    #[test]
    fn wishlist_keeps_priority_and_date_added() {
        let rows = map_wishlist(&[
            json!({"appid": 5, "priority": 3, "date_added": 1700000000}),
            json!({"appid": 6, "priority": 0, "date_added": 1700000001}),
            json!({"priority": 1}),
        ]);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            (
                rows[0].native_id.as_str(),
                rows[0].priority,
                rows[0].added_at
            ),
            ("5", Some(3), Some(1700000000))
        );
        assert_eq!(rows[1].priority, None, "0 is an unranked entry");
    }
}
