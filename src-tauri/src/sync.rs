//! Persistence of synchronised data by generation (spec 4.4, 4.5, 6.1).
//!
//! The rule that matters: a failed, partial or suspiciously empty read never replaces a valid
//! library or wishlist. Data of different accounts never mixes: every row carries its account.
// The commit functions take the whole read as separate values on purpose: a parameter struct would
// only move the same list elsewhere.
#![allow(clippy::too_many_arguments)]

use rusqlite::{Connection, params};

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// What a provider read finished with. The caller decides, the store only obeys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOutcome {
    /// Every page read and parsed.
    Complete,
    /// Some pages missing: keep what was there, add what arrived.
    #[allow(dead_code)] // the paginated Epic library reports it once its sync exists
    Partial { pages_done: u32, pages_failed: u32 },
    /// Nothing usable (auth, network, parsing).
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Library,
    Family,
    Wishlist,
}

impl Resource {
    fn as_str(self) -> &'static str {
        match self {
            Resource::Library => "library",
            Resource::Family => "family",
            Resource::Wishlist => "wishlist",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProductRow {
    pub native_id: String,
    pub title: Option<String>,
    pub kind: String,
    pub windows: String,
    pub url: Option<String>,
    /// Portrait cover on the store CDN, already checked by `images::checked`.
    pub image: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EntitlementRow {
    pub native_id: String,
    pub class: String,
    pub owners: Vec<String>,
    pub exclude_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WishRow {
    pub native_id: String,
    pub priority: Option<i64>,
    pub added_at: Option<i64>,
    /// `false` when the endpoint returned it but the storefront does not show it.
    pub confirmed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stored {
    Complete,
    Partial,
    Failed,
    /// An empty read that the provider did not confirm: treated as a failure, data kept.
    FailedEmptyUnconfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Committed {
    pub generation_id: i64,
    pub stored: Stored,
    pub item_count: usize,
}

fn record_generation(
    conn: &Connection,
    store: &str,
    account: &str,
    resource: Resource,
    state: &str,
    done: u32,
    failed: u32,
    count: usize,
    now: i64,
) -> Result<i64, SyncError> {
    conn.execute(
        "INSERT INTO sync_generation(provider, resource, account_id, state, pages_done, pages_failed, item_count, started_at, finished_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
        params![store, resource.as_str(), account, state, done, failed, count as i64, now.to_string()],
    )?;
    Ok(conn.last_insert_rowid())
}

fn upsert_products(
    tx: &rusqlite::Transaction,
    store: &str,
    products: &[ProductRow],
    now: i64,
) -> Result<(), SyncError> {
    let mut st = tx.prepare_cached(
        "INSERT INTO product(store, native_id, title, kind, windows, url, image_url, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(store, native_id) DO UPDATE SET
           title = COALESCE(excluded.title, product.title),
           kind = CASE WHEN excluded.kind = 'unknown' THEN product.kind ELSE excluded.kind END,
           windows = CASE WHEN excluded.windows = 'unknown' THEN product.windows ELSE excluded.windows END,
           url = COALESCE(excluded.url, product.url),
           image_url = COALESCE(excluded.image_url, product.image_url),
           updated_at = excluded.updated_at",
    )?;
    for p in products {
        st.execute(params![
            store,
            p.native_id,
            p.title,
            p.kind,
            p.windows,
            p.url,
            p.image,
            now
        ])?;
    }
    Ok(())
}

/// Catalog details (name, type, Windows) learned outside a library or wishlist read. They only
/// fill in or refine products: an `unknown` never overwrites a known value and no right or
/// wishlist entry is touched.
pub fn update_products(
    conn: &mut Connection,
    store: &str,
    products: &[ProductRow],
    now: i64,
) -> Result<(), SyncError> {
    let tx = conn.transaction()?;
    upsert_products(&tx, store, products, now)?;
    tx.commit()?;
    Ok(())
}

/// Most family members whose names are looked up in one sync. Steam families have at most six.
pub const FAMILY_NAMES_MAX: usize = 12;

/// Distinct owners of the family rights, in first-seen order, for the name lookup.
pub fn family_owner_ids(rows: &[EntitlementRow]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    rows.iter()
        .flat_map(|r| r.owners.iter())
        .filter(|o| seen.insert(o.as_str().to_owned()))
        .take(FAMILY_NAMES_MAX)
        .cloned()
        .collect()
}

/// Keeps the public names found for family members. Names not found leave the last known one.
pub fn remember_family_names(
    conn: &Connection,
    names: &[(String, String)],
    now: i64,
) -> Result<(), SyncError> {
    let mut st = conn.prepare_cached(
        "INSERT INTO steam_family_member(steamid, name, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(steamid) DO UPDATE SET name = excluded.name, updated_at = excluded.updated_at",
    )?;
    for (id, name) in names {
        st.execute(params![id, name, now])?;
    }
    Ok(())
}

/// Stores an entitlement read (library or family) for one account.
///
/// `confirmed_empty` is true only when the provider answered validly with zero items.
pub fn commit_entitlements(
    conn: &mut Connection,
    store: &str,
    account: &str,
    resource: Resource,
    products: &[ProductRow],
    rows: &[EntitlementRow],
    outcome: &ReadOutcome,
    confirmed_empty: bool,
    now: i64,
) -> Result<Committed, SyncError> {
    let classes: &[&str] = match resource {
        Resource::Family => &["family_shared"],
        _ => &["owned", "temporary", "unknown"],
    };
    let tx = conn.transaction()?;
    let previous: i64 = tx.query_row(
        "SELECT COUNT(*) FROM entitlement WHERE account_id = ?1 AND store = ?2 AND class IN (SELECT value FROM json_each(?3))",
        params![account, store, serde_json::to_string(classes).unwrap_or_default()],
        |r| r.get(0),
    )?;

    let (state, stored, replace) = match outcome {
        ReadOutcome::Failed => ("failed", Stored::Failed, false),
        ReadOutcome::Partial { .. } => ("partial", Stored::Partial, false),
        ReadOutcome::Complete if rows.is_empty() && !confirmed_empty && previous > 0 => {
            ("failed", Stored::FailedEmptyUnconfirmed, false)
        }
        ReadOutcome::Complete => ("complete", Stored::Complete, true),
    };
    let (done, failed) = match outcome {
        ReadOutcome::Partial {
            pages_done,
            pages_failed,
        } => (*pages_done, *pages_failed),
        ReadOutcome::Complete => (1, 0),
        ReadOutcome::Failed => (0, 1),
    };
    let writes = matches!(stored, Stored::Complete | Stored::Partial);
    let gen_id = record_generation(
        &tx,
        store,
        account,
        resource,
        state,
        done,
        failed,
        if writes { rows.len() } else { 0 },
        now,
    )?;

    if writes {
        upsert_products(&tx, store, products, now)?;
        if replace {
            tx.execute(
                "DELETE FROM entitlement WHERE account_id = ?1 AND store = ?2 AND class IN (SELECT value FROM json_each(?3))",
                params![account, store, serde_json::to_string(classes).unwrap_or_default()],
            )?;
        }
        let mut st = tx.prepare_cached(
            "INSERT INTO entitlement(account_id, store, native_id, class, owners, exclude_reason, observed_at, generation_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(account_id, store, native_id, class) DO UPDATE SET
               owners = excluded.owners, exclude_reason = excluded.exclude_reason,
               observed_at = excluded.observed_at, generation_id = excluded.generation_id",
        )?;
        for r in rows {
            let owners = serde_json::to_string(&r.owners).unwrap_or_else(|_| "[]".into());
            st.execute(params![
                account,
                store,
                r.native_id,
                r.class,
                owners,
                r.exclude_reason,
                now,
                gen_id
            ])?;
        }
        drop(st);
    }
    tx.commit()?;
    Ok(Committed {
        generation_id: gen_id,
        stored,
        item_count: if writes { rows.len() } else { 0 },
    })
}

/// Stores a wishlist read for one account. Same guarantees as `commit_entitlements`.
pub fn commit_wishlist(
    conn: &mut Connection,
    store: &str,
    account: &str,
    products: &[ProductRow],
    rows: &[WishRow],
    outcome: &ReadOutcome,
    confirmed_empty: bool,
    now: i64,
) -> Result<Committed, SyncError> {
    let tx = conn.transaction()?;
    let previous: i64 = tx.query_row(
        "SELECT COUNT(*) FROM wishlist_entry WHERE account_id = ?1 AND store = ?2",
        params![account, store],
        |r| r.get(0),
    )?;
    let (state, stored, replace) = match outcome {
        ReadOutcome::Failed => ("failed", Stored::Failed, false),
        ReadOutcome::Partial { .. } => ("partial", Stored::Partial, false),
        ReadOutcome::Complete if rows.is_empty() && !confirmed_empty && previous > 0 => {
            ("failed", Stored::FailedEmptyUnconfirmed, false)
        }
        ReadOutcome::Complete => ("complete", Stored::Complete, true),
    };
    let (done, failed) = match outcome {
        ReadOutcome::Partial {
            pages_done,
            pages_failed,
        } => (*pages_done, *pages_failed),
        ReadOutcome::Complete => (1, 0),
        ReadOutcome::Failed => (0, 1),
    };
    let writes = matches!(stored, Stored::Complete | Stored::Partial);
    let gen_id = record_generation(
        &tx,
        store,
        account,
        Resource::Wishlist,
        state,
        done,
        failed,
        if writes { rows.len() } else { 0 },
        now,
    )?;
    if writes {
        upsert_products(&tx, store, products, now)?;
        if replace {
            tx.execute(
                "DELETE FROM wishlist_entry WHERE account_id = ?1 AND store = ?2",
                params![account, store],
            )?;
        }
        let mut st = tx.prepare_cached(
            "INSERT INTO wishlist_entry(account_id, store, native_id, priority, added_at, confirmed, observed_at, generation_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(account_id, store, native_id) DO UPDATE SET
               priority = excluded.priority, added_at = excluded.added_at, confirmed = excluded.confirmed,
               observed_at = excluded.observed_at, generation_id = excluded.generation_id",
        )?;
        for r in rows {
            st.execute(params![
                account,
                store,
                r.native_id,
                r.priority,
                r.added_at,
                r.confirmed as i64,
                now,
                gen_id
            ])?;
        }
        drop(st);
    }
    tx.commit()?;
    Ok(Committed {
        generation_id: gen_id,
        stored,
        item_count: if writes { rows.len() } else { 0 },
    })
}

/// Records why a generation failed: a short code (`needs_reauth`, `private`, `network`,
/// `provider`, `shape`), never provider text, which could echo request data.
pub fn set_generation_error(
    conn: &Connection,
    generation_id: i64,
    code: &str,
) -> Result<(), SyncError> {
    conn.execute(
        "UPDATE sync_generation SET error_code = ?2 WHERE id = ?1",
        params![generation_id, code],
    )?;
    Ok(())
}

/// Last generation of a resource, for the "data from <date>" label (spec 9.2, A15).
#[cfg(test)]
pub fn last_generation(
    conn: &Connection,
    store: &str,
    account: &str,
    resource: Resource,
    only_complete: bool,
) -> Result<Option<(i64, String, i64)>, SyncError> {
    let sql = if only_complete {
        "SELECT id, state, CAST(finished_at AS INTEGER) FROM sync_generation
         WHERE provider = ?1 AND account_id = ?2 AND resource = ?3 AND state = 'complete' ORDER BY id DESC LIMIT 1"
    } else {
        "SELECT id, state, CAST(finished_at AS INTEGER) FROM sync_generation
         WHERE provider = ?1 AND account_id = ?2 AND resource = ?3 ORDER BY id DESC LIMIT 1"
    };
    let mut st = conn.prepare(sql)?;
    let mut rows = st.query(params![store, account, resource.as_str()])?;
    Ok(match rows.next()? {
        Some(r) => Some((r.get(0)?, r.get(1)?, r.get(2)?)),
        None => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.db");
        let conn = crate::db::open(&path).unwrap();
        std::mem::forget(dir); // keep the file alive for the connection in this test
        conn
    }

    fn product(id: &str, title: &str) -> ProductRow {
        ProductRow {
            native_id: id.into(),
            title: Some(title.into()),
            kind: "game".into(),
            windows: "unknown".into(),
            url: None,
            image: None,
        }
    }

    fn owned(id: &str) -> EntitlementRow {
        EntitlementRow {
            native_id: id.into(),
            class: "owned".into(),
            owners: vec![],
            exclude_reason: None,
        }
    }

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    fn commit(
        conn: &mut Connection,
        acct: &str,
        ids: &[&str],
        outcome: ReadOutcome,
        empty_ok: bool,
    ) -> Committed {
        let products: Vec<_> = ids
            .iter()
            .map(|i| product(i, &format!("Game {i}")))
            .collect();
        let rows: Vec<_> = ids.iter().map(|i| owned(i)).collect();
        commit_entitlements(
            conn,
            "steam",
            acct,
            Resource::Library,
            &products,
            &rows,
            &outcome,
            empty_ok,
            100,
        )
        .unwrap()
    }

    #[test]
    fn a_complete_read_replaces_the_previous_library() {
        let mut c = db();
        commit(&mut c, "a1", &["1", "2", "3"], ReadOutcome::Complete, false);
        let r = commit(&mut c, "a1", &["2", "4"], ReadOutcome::Complete, false);
        assert_eq!(r.stored, Stored::Complete);
        let ids: Vec<String> = c
            .prepare("SELECT native_id FROM entitlement ORDER BY native_id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(ids, ["2", "4"]);
    }

    #[test]
    fn a_failed_read_keeps_the_library_and_records_the_failure() {
        let mut c = db();
        commit(&mut c, "a1", &["1", "2"], ReadOutcome::Complete, false);
        let r = commit(&mut c, "a1", &[], ReadOutcome::Failed, false);
        assert_eq!(r.stored, Stored::Failed);
        assert_eq!(count(&c, "entitlement"), 2);
        let (_, state, _) = last_generation(&c, "steam", "a1", Resource::Library, false)
            .unwrap()
            .unwrap();
        assert_eq!(state, "failed");
        let (_, state, _) = last_generation(&c, "steam", "a1", Resource::Library, true)
            .unwrap()
            .unwrap();
        assert_eq!(
            state, "complete",
            "the last complete generation is still reachable"
        );
    }

    #[test]
    fn an_unconfirmed_empty_read_never_wipes_a_valid_library() {
        let mut c = db();
        commit(&mut c, "a1", &["1", "2"], ReadOutcome::Complete, false);
        let r = commit(&mut c, "a1", &[], ReadOutcome::Complete, false);
        assert_eq!(r.stored, Stored::FailedEmptyUnconfirmed);
        assert_eq!(count(&c, "entitlement"), 2);
        // The provider confirming an empty library is the only way to empty it.
        let r = commit(&mut c, "a1", &[], ReadOutcome::Complete, true);
        assert_eq!(r.stored, Stored::Complete);
        assert_eq!(count(&c, "entitlement"), 0);
    }

    #[test]
    fn a_partial_read_adds_without_deleting() {
        let mut c = db();
        commit(&mut c, "a1", &["1", "2"], ReadOutcome::Complete, false);
        let r = commit(
            &mut c,
            "a1",
            &["3"],
            ReadOutcome::Partial {
                pages_done: 1,
                pages_failed: 1,
            },
            false,
        );
        assert_eq!(r.stored, Stored::Partial);
        assert_eq!(count(&c, "entitlement"), 3);
        let (_, state, _) = last_generation(&c, "steam", "a1", Resource::Library, false)
            .unwrap()
            .unwrap();
        assert_eq!(state, "partial");
    }

    #[test]
    fn accounts_never_mix() {
        let mut c = db();
        commit(&mut c, "a1", &["1", "2"], ReadOutcome::Complete, false);
        commit(&mut c, "a2", &["9"], ReadOutcome::Complete, false);
        assert_eq!(count(&c, "entitlement"), 3);
        // Re-syncing one account leaves the other untouched.
        commit(&mut c, "a2", &["10"], ReadOutcome::Complete, false);
        let a1: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM entitlement WHERE account_id='a1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(a1, 2);
    }

    #[test]
    fn family_and_own_access_are_separate_rights_on_the_same_product() {
        let mut c = db();
        commit(&mut c, "a1", &["1"], ReadOutcome::Complete, false);
        let fam = EntitlementRow {
            native_id: "1".into(),
            class: "family_shared".into(),
            owners: vec!["76561198000000002".into()],
            exclude_reason: Some("unknown_reason_7".into()),
        };
        commit_entitlements(
            &mut c,
            "steam",
            "a1",
            Resource::Family,
            &[product("1", "Game 1")],
            &[fam],
            &ReadOutcome::Complete,
            false,
            101,
        )
        .unwrap();
        assert_eq!(
            count(&c, "entitlement"),
            2,
            "owned and family_shared coexist"
        );
        // Losing the family removes only the family right.
        commit_entitlements(
            &mut c,
            "steam",
            "a1",
            Resource::Family,
            &[],
            &[],
            &ReadOutcome::Complete,
            true,
            102,
        )
        .unwrap();
        let left: String = c
            .query_row("SELECT class FROM entitlement", [], |r| r.get(0))
            .unwrap();
        assert_eq!(left, "owned");
        assert_eq!(
            count(&c, "product"),
            1,
            "products and decisions survive losing the family"
        );
    }

    #[test]
    fn family_owners_and_unknown_exclusion_reasons_are_preserved_as_text() {
        let mut c = db();
        let fam = EntitlementRow {
            native_id: "5".into(),
            class: "family_shared".into(),
            owners: vec!["76561198000000001".into(), "9007199254740993".into()],
            exclude_reason: Some("some_new_reason".into()),
        };
        commit_entitlements(
            &mut c,
            "steam",
            "a1",
            Resource::Family,
            &[],
            &[fam],
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        let (owners, reason): (String, String) = c
            .query_row("SELECT owners, exclude_reason FROM entitlement", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(owners, r#"["76561198000000001","9007199254740993"]"#);
        assert_eq!(reason, "some_new_reason");
    }

    #[test]
    fn a_later_read_without_a_title_does_not_erase_a_known_one() {
        let mut c = db();
        commit(&mut c, "a1", &["1"], ReadOutcome::Complete, false);
        let untitled = ProductRow {
            native_id: "1".into(),
            title: None,
            kind: "unknown".into(),
            windows: "unknown".into(),
            url: None,
            image: None,
        };
        commit_entitlements(
            &mut c,
            "steam",
            "a1",
            Resource::Library,
            &[untitled],
            &[owned("1")],
            &ReadOutcome::Complete,
            false,
            200,
        )
        .unwrap();
        let title: String = c
            .query_row("SELECT title FROM product", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Game 1");
    }

    #[test]
    fn wishlist_keeps_unconfirmed_entries_and_guards_against_empty_reads() {
        let mut c = db();
        let rows = vec![
            WishRow {
                native_id: "1".into(),
                priority: Some(1),
                added_at: Some(10),
                confirmed: true,
            },
            WishRow {
                native_id: "2".into(),
                priority: None,
                added_at: None,
                confirmed: false,
            },
        ];
        commit_wishlist(
            &mut c,
            "gog",
            "g1",
            &[],
            &rows,
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        assert_eq!(count(&c, "wishlist_entry"), 2);
        let unconfirmed: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM wishlist_entry WHERE confirmed = 0",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unconfirmed, 1);
        let r = commit_wishlist(
            &mut c,
            "gog",
            "g1",
            &[],
            &[],
            &ReadOutcome::Complete,
            false,
            2,
        )
        .unwrap();
        assert_eq!(r.stored, Stored::FailedEmptyUnconfirmed);
        assert_eq!(count(&c, "wishlist_entry"), 2);
        let r = commit_wishlist(
            &mut c,
            "gog",
            "g1",
            &[],
            &[],
            &ReadOutcome::Complete,
            true,
            3,
        )
        .unwrap();
        assert_eq!(r.stored, Stored::Complete);
        assert_eq!(count(&c, "wishlist_entry"), 0);
    }

    #[test]
    fn ids_above_the_js_safe_integer_roundtrip() {
        let mut c = db();
        commit_entitlements(
            &mut c,
            "steam",
            "76561198000000001",
            Resource::Library,
            &[product("9007199254740993", "Big")],
            &[owned("9007199254740993")],
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        let (acct, id): (String, String) = c
            .query_row("SELECT account_id, native_id FROM entitlement", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(
            (acct.as_str(), id.as_str()),
            ("76561198000000001", "9007199254740993")
        );
    }

    #[test]
    fn a_later_read_without_cover_keeps_the_known_one() {
        let mut conn = db();
        let mut with = product("1", "Alpha");
        with.image = Some("https://cdn1.epicgames.com/a".into());
        update_products(&mut conn, "epic", &[with], 1).unwrap();
        update_products(&mut conn, "epic", &[product("1", "Alpha")], 2).unwrap();
        let kept: Option<String> = conn
            .query_row(
                "SELECT image_url FROM product WHERE native_id = '1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kept.as_deref(), Some("https://cdn1.epicgames.com/a"));
    }

    #[test]
    fn family_owner_ids_are_distinct_and_bounded() {
        let row = |owners: &[&str]| EntitlementRow {
            native_id: "x".into(),
            class: "family_shared".into(),
            owners: owners.iter().map(|o| o.to_string()).collect(),
            exclude_reason: None,
        };
        assert_eq!(
            family_owner_ids(&[row(&["a", "b"]), row(&["b", "c"])]),
            ["a", "b", "c"]
        );
        let many: Vec<String> = (0..30).map(|i| i.to_string()).collect();
        let refs: Vec<&str> = many.iter().map(String::as_str).collect();
        assert_eq!(family_owner_ids(&[row(&refs)]).len(), FAMILY_NAMES_MAX);
    }

    #[test]
    fn a_family_name_is_updated_not_duplicated() {
        let conn = db();
        remember_family_names(&conn, &[("a".into(), "Ana".into())], 1).unwrap();
        remember_family_names(&conn, &[("a".into(), "Ana B".into())], 2).unwrap();
        let (n, name): (i64, String) = conn
            .query_row(
                "SELECT COUNT(*), MAX(name) FROM steam_family_member",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((n, name.as_str()), (1, "Ana B"));
    }
}
