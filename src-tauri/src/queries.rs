//! Read side of the synchronised data, for the views. Rows come from the database only: with no
//! network the last copy is shown, with its date (spec 9.2, acceptance A15).

use rusqlite::Connection;
use serde::Serialize;
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Debug, Serialize, PartialEq, Eq, Clone, TS)]
#[ts(export)]
pub struct Access {
    /// `owned`, `family_shared`, `temporary` or `unknown`. Never reworded as "bought".
    pub class: String,
    pub owners: Vec<String>,
    pub exclude_reason: Option<String>,
    pub observed_at: i64,
}

#[derive(Debug, Serialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct LibraryEntry {
    pub store: String,
    pub account_id: String,
    pub native_id: String,
    pub title: Option<String>,
    pub kind: String,
    pub windows: String,
    pub url: Option<String>,
    /// One product can carry several rights at once (own copy and family access).
    pub access: Vec<Access>,
}

/// Every product with at least one right, grouped, ordered by title then id.
pub fn list_library(conn: &Connection) -> Result<Vec<LibraryEntry>, rusqlite::Error> {
    let mut st = conn.prepare(
        "SELECT e.store, e.account_id, e.native_id, p.title, COALESCE(p.kind,'unknown'),
                COALESCE(p.windows,'unknown'), p.url, e.class, e.owners, e.exclude_reason, e.observed_at
         FROM entitlement e
         JOIN connection c ON c.provider = e.store AND c.account_id = e.account_id
         LEFT JOIN product p ON p.store = e.store AND p.native_id = e.native_id
         ORDER BY e.store, e.account_id, e.native_id, e.class",
    )?;
    let rows = st.query_map([], |r| {
        let owners: String = r.get(8)?;
        Ok((
            (
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ),
            (
                r.get::<_, Option<String>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
            ),
            Access {
                class: r.get(7)?,
                owners: serde_json::from_str(&owners).unwrap_or_default(),
                exclude_reason: r.get(9)?,
                observed_at: r.get(10)?,
            },
        ))
    })?;
    let mut grouped: BTreeMap<(String, String, String), LibraryEntry> = BTreeMap::new();
    for row in rows {
        let (key, (title, kind, windows, url), access) = row?;
        grouped
            .entry(key.clone())
            .or_insert_with(|| LibraryEntry {
                store: key.0.clone(),
                account_id: key.1.clone(),
                native_id: key.2.clone(),
                title,
                kind,
                windows,
                url,
                access: vec![],
            })
            .access
            .push(access);
    }
    let mut out: Vec<LibraryEntry> = grouped.into_values().collect();
    out.sort_by(|a, b| {
        a.title
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&b.title.as_deref().unwrap_or("").to_lowercase())
            .then_with(|| a.native_id.cmp(&b.native_id))
    });
    Ok(out)
}

#[derive(Debug, Serialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct WishlistEntry {
    pub store: String,
    pub account_id: String,
    pub native_id: String,
    pub title: Option<String>,
    pub url: Option<String>,
    pub priority: Option<i64>,
    pub added_at: Option<i64>,
    pub confirmed: bool,
    /// True when the same account holds a right on this product: a hint, never a decision.
    pub already_held_here: bool,
}

pub fn list_wishlist(conn: &Connection) -> Result<Vec<WishlistEntry>, rusqlite::Error> {
    let mut st = conn.prepare(
        "SELECT w.store, w.account_id, w.native_id, p.title, p.url, w.priority, w.added_at, w.confirmed,
                EXISTS(SELECT 1 FROM entitlement e WHERE e.account_id = w.account_id
                       AND e.store = w.store AND e.native_id = w.native_id AND e.class = 'owned')
         FROM wishlist_entry w
         JOIN connection c ON c.provider = w.store AND c.account_id = w.account_id
         LEFT JOIN product p ON p.store = w.store AND p.native_id = w.native_id
         ORDER BY w.store, w.priority IS NULL, w.priority, w.native_id",
    )?;
    let rows = st.query_map([], |r| {
        Ok(WishlistEntry {
            store: r.get(0)?,
            account_id: r.get(1)?,
            native_id: r.get(2)?,
            title: r.get(3)?,
            url: r.get(4)?,
            priority: r.get(5)?,
            added_at: r.get(6)?,
            confirmed: r.get::<_, i64>(7)? != 0,
            already_held_here: r.get::<_, i64>(8)? != 0,
        })
    })?;
    rows.collect()
}

#[derive(Debug, Serialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct SyncStatus {
    pub store: String,
    pub account_id: String,
    pub resource: String,
    /// Last attempt, whatever its outcome.
    pub last_state: String,
    pub last_at: i64,
    /// Last complete generation: the date the shown data really comes from.
    pub last_complete_at: Option<i64>,
}

pub fn sync_status(conn: &Connection) -> Result<Vec<SyncStatus>, rusqlite::Error> {
    let mut st = conn.prepare(
        "SELECT g.provider, g.account_id, g.resource, g.state, CAST(g.finished_at AS INTEGER),
                (SELECT CAST(MAX(c.finished_at) AS INTEGER) FROM sync_generation c
                 WHERE c.provider = g.provider AND c.account_id = g.account_id
                   AND c.resource = g.resource AND c.state = 'complete')
         FROM sync_generation g
         JOIN connection cn ON cn.provider = g.provider AND cn.account_id = g.account_id
         WHERE g.id = (SELECT MAX(id) FROM sync_generation x
                       WHERE x.provider = g.provider AND x.account_id = g.account_id AND x.resource = g.resource)
         ORDER BY g.provider, g.account_id, g.resource",
    )?;
    let rows = st.query_map([], |r| {
        Ok(SyncStatus {
            store: r.get(0)?,
            account_id: r.get(1)?,
            resource: r.get(2)?,
            last_state: r.get(3)?,
            last_at: r.get(4)?,
            last_complete_at: r.get(5)?,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::{
        EntitlementRow, ProductRow, ReadOutcome, Resource, WishRow, commit_entitlements,
        commit_wishlist,
    };

    fn db() -> Connection {
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::db::open(&dir.path().join("t.db")).unwrap();
        std::mem::forget(dir);
        for (store, acct) in [("steam", "me"), ("gog", "g")] {
            crate::connections::record_connection(&conn, store, acct, None).unwrap();
        }
        conn
    }

    fn product(id: &str, title: &str) -> ProductRow {
        ProductRow {
            native_id: id.into(),
            title: Some(title.into()),
            kind: "game".into(),
            windows: "yes".into(),
            url: None,
        }
    }

    fn right(id: &str, class: &str, owners: &[&str]) -> EntitlementRow {
        EntitlementRow {
            native_id: id.into(),
            class: class.into(),
            owners: owners.iter().map(|s| s.to_string()).collect(),
            exclude_reason: None,
        }
    }

    #[test]
    fn one_product_with_two_rights_is_one_entry_with_both_accesses() {
        let mut c = db();
        commit_entitlements(
            &mut c,
            "steam",
            "me",
            Resource::Library,
            &[product("1", "Beta")],
            &[right("1", "owned", &[])],
            &ReadOutcome::Complete,
            false,
            10,
        )
        .unwrap();
        commit_entitlements(
            &mut c,
            "steam",
            "me",
            Resource::Family,
            &[product("1", "Beta")],
            &[right("1", "family_shared", &["other"])],
            &ReadOutcome::Complete,
            true,
            11,
        )
        .unwrap();
        let list = list_library(&c).unwrap();
        assert_eq!(list.len(), 1);
        let classes: Vec<&str> = list[0].access.iter().map(|a| a.class.as_str()).collect();
        assert_eq!(classes, ["family_shared", "owned"]);
        assert_eq!(list[0].access[0].owners, ["other"]);
    }

    #[test]
    fn entries_are_ordered_by_title_ignoring_case_and_keep_untitled_ones() {
        let mut c = db();
        let ps = [
            product("1", "banana"),
            product("2", "Apple"),
            ProductRow {
                native_id: "3".into(),
                title: None,
                kind: "unknown".into(),
                windows: "unknown".into(),
                url: None,
            },
        ];
        let rs = [
            right("1", "owned", &[]),
            right("2", "owned", &[]),
            right("3", "owned", &[]),
        ];
        commit_entitlements(
            &mut c,
            "gog",
            "g",
            Resource::Library,
            &ps,
            &rs,
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        let titles: Vec<Option<String>> = list_library(&c)
            .unwrap()
            .into_iter()
            .map(|e| e.title)
            .collect();
        assert_eq!(titles, [None, Some("Apple".into()), Some("banana".into())]);
    }

    #[test]
    fn wishlist_hints_when_the_same_account_already_owns_the_product() {
        let mut c = db();
        commit_entitlements(
            &mut c,
            "gog",
            "g",
            Resource::Library,
            &[product("7", "Seven")],
            &[right("7", "owned", &[])],
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        let wl = [
            WishRow {
                native_id: "7".into(),
                priority: None,
                added_at: None,
                confirmed: false,
            },
            WishRow {
                native_id: "8".into(),
                priority: Some(1),
                added_at: None,
                confirmed: true,
            },
        ];
        commit_wishlist(
            &mut c,
            "gog",
            "g",
            &[],
            &wl,
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        let list = list_wishlist(&c).unwrap();
        let by = |id: &str| list.iter().find(|w| w.native_id == id).unwrap();
        assert!(by("7").already_held_here && !by("7").confirmed);
        assert!(!by("8").already_held_here);
    }

    #[test]
    fn sync_status_reports_the_last_attempt_and_the_last_complete_date() {
        let mut c = db();
        commit_entitlements(
            &mut c,
            "steam",
            "me",
            Resource::Library,
            &[product("1", "A")],
            &[right("1", "owned", &[])],
            &ReadOutcome::Complete,
            false,
            100,
        )
        .unwrap();
        commit_entitlements(
            &mut c,
            "steam",
            "me",
            Resource::Library,
            &[],
            &[],
            &ReadOutcome::Failed,
            false,
            200,
        )
        .unwrap();
        let s = sync_status(&c).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(
            (
                s[0].last_state.as_str(),
                s[0].last_at,
                s[0].last_complete_at
            ),
            ("failed", 200, Some(100))
        );
        // The data is still the one from 100: the view must say so.
        assert_eq!(list_library(&c).unwrap().len(), 1);
    }

    #[test]
    fn rows_of_an_account_that_is_not_the_active_one_are_kept_out_of_the_views() {
        let mut c = db();
        commit_entitlements(
            &mut c,
            "gog",
            "previous",
            Resource::Library,
            &[product("1", "Old")],
            &[right("1", "owned", &[])],
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        commit_wishlist(
            &mut c,
            "gog",
            "previous",
            &[],
            &[WishRow {
                native_id: "2".into(),
                priority: None,
                added_at: None,
                confirmed: true,
            }],
            &ReadOutcome::Complete,
            true,
            1,
        )
        .unwrap();
        assert!(list_library(&c).unwrap().is_empty());
        assert!(list_wishlist(&c).unwrap().is_empty());
        assert!(sync_status(&c).unwrap().is_empty());
    }
}
