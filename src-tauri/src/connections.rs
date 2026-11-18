//! Connections and their capabilities (spec 5.4, 5.5, 9.1).
//!
//! `connection.account_id` is the active account of each store. Synced rows always carry their
//! account, so when another account connects the previous one's data stays in the database,
//! separated and hidden from the views, never mixed and never silently deleted.

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use ts_rs::TS;

pub const PROVIDERS: [&str; 3] = ["steam", "gog", "epic"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountChange {
    /// First account ever linked for this store.
    New,
    Same,
    /// A different account than the one whose data is stored. Pending plans must be dropped.
    Changed,
}

/// Records the account a store is linked to now.
pub fn record_connection(
    conn: &Connection,
    provider: &str,
    account_id: &str,
    display_name: Option<&str>,
) -> Result<AccountChange, rusqlite::Error> {
    let previous: Option<Option<String>> = conn
        .query_row(
            "SELECT account_id FROM connection WHERE provider = ?1",
            [provider],
            |r| r.get(0),
        )
        .optional()?;
    let change = match previous.flatten() {
        None => AccountChange::New,
        Some(p) if p == account_id => AccountChange::Same,
        Some(_) => AccountChange::Changed,
    };
    conn.execute(
        "INSERT INTO connection(provider, account_id, display_name, state)
         VALUES (?1, ?2, ?3, 'connected')
         ON CONFLICT(provider) DO UPDATE SET
           account_id = excluded.account_id,
           display_name = CASE WHEN excluded.account_id = connection.account_id
                               THEN COALESCE(excluded.display_name, connection.display_name)
                               ELSE excluded.display_name END,
           state = 'connected'",
        params![provider, account_id, display_name],
    )?;
    Ok(change)
}

/// The secret is gone: the store is disconnected, but the last copy stays readable.
pub fn mark_disconnected(conn: &Connection, provider: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE connection SET state = 'disconnected' WHERE provider = ?1",
        [provider],
    )?;
    Ok(())
}

pub fn touch_sync(conn: &Connection, provider: &str, now: i64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE connection SET last_sync_at = ?2 WHERE provider = ?1",
        params![provider, now.to_string()],
    )?;
    Ok(())
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct CapabilityView {
    /// `library`, `family`, `wishlist` or `wishlist_write`.
    pub capability: String,
    /// `not_synced`, `complete`, `partial`, `failed`, `unavailable` (decided out of scope),
    /// `pending` (integration not finished), `validated` (write with its validation record) or
    /// `not_validated` (write without one).
    pub state: String,
    pub error_code: Option<String>,
    pub last_attempt_at: Option<i64>,
    pub last_complete_at: Option<i64>,
    pub item_count: Option<i64>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct ConnectionView {
    pub provider: String,
    /// `connected` or `disconnected`.
    pub state: String,
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    pub last_sync_at: Option<i64>,
    pub capabilities: Vec<CapabilityView>,
    /// Data of an account other than the active one is stored and kept apart.
    pub other_account_data: bool,
}

/// Static description of what each store can do in this version.
fn capability_plan(provider: &str) -> &'static [(&'static str, Plan)] {
    match provider {
        "steam" => &[
            ("library", Plan::Synced),
            ("family", Plan::Synced),
            ("wishlist", Plan::Synced),
            ("wishlist_write", Plan::Write),
        ],
        "gog" => &[
            ("library", Plan::Synced),
            ("wishlist", Plan::Synced),
            ("wishlist_write", Plan::Write),
        ],
        // Epic wishlist dropped by the user (docs/validation/phase-1.md); no reading, no writing.
        "epic" => &[
            ("library", Plan::Synced),
            ("wishlist", Plan::Fixed("unavailable")),
            ("wishlist_write", Plan::Fixed("unavailable")),
        ],
        _ => &[],
    }
}

#[derive(Clone, Copy)]
enum Plan {
    Synced,
    Fixed(&'static str),
    /// `validated` with a record in docs/validation (plans::write_validated), else `not_validated`.
    Write,
}

fn synced_capability(
    conn: &Connection,
    provider: &str,
    account: Option<&str>,
    resource: &str,
) -> Result<CapabilityView, rusqlite::Error> {
    let mut view = CapabilityView {
        capability: resource.into(),
        state: "not_synced".into(),
        error_code: None,
        last_attempt_at: None,
        last_complete_at: None,
        item_count: None,
    };
    let Some(account) = account else {
        return Ok(view);
    };
    let last = conn
        .query_row(
            "SELECT state, error_code, CAST(finished_at AS INTEGER) FROM sync_generation
             WHERE provider = ?1 AND account_id = ?2 AND resource = ?3 ORDER BY id DESC LIMIT 1",
            params![provider, account, resource],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?;
    if let Some((state, code, at)) = last {
        view.state = state;
        view.error_code = code;
        view.last_attempt_at = Some(at);
    }
    let complete = conn
        .query_row(
            "SELECT CAST(finished_at AS INTEGER), item_count FROM sync_generation
             WHERE provider = ?1 AND account_id = ?2 AND resource = ?3 AND state = 'complete'
             ORDER BY id DESC LIMIT 1",
            params![provider, account, resource],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        )
        .optional()?;
    if let Some((at, n)) = complete {
        view.last_complete_at = Some(at);
        view.item_count = Some(n);
    }
    Ok(view)
}

pub fn view(conn: &Connection, provider: &str) -> Result<ConnectionView, rusqlite::Error> {
    let row = conn
        .query_row(
            "SELECT state, account_id, display_name, CAST(last_sync_at AS INTEGER)
             FROM connection WHERE provider = ?1",
            [provider],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                ))
            },
        )
        .optional()?;
    let (state, account, name, last_sync) =
        row.unwrap_or_else(|| ("disconnected".into(), None, None, None));
    let mut capabilities = Vec::new();
    for (cap, plan) in capability_plan(provider) {
        capabilities.push(match plan {
            Plan::Synced => synced_capability(conn, provider, account.as_deref(), cap)?,
            Plan::Fixed(_) | Plan::Write => CapabilityView {
                capability: (*cap).into(),
                state: match plan {
                    Plan::Fixed(s) => (*s).into(),
                    _ if crate::plans::write_validated(provider) => "validated".into(),
                    _ => "not_validated".into(),
                },
                error_code: None,
                last_attempt_at: None,
                last_complete_at: None,
                item_count: None,
            },
        });
    }
    let other_account_data: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM entitlement WHERE store = ?1 AND account_id IS NOT ?2)
             OR EXISTS(SELECT 1 FROM wishlist_entry WHERE store = ?1 AND account_id IS NOT ?2)",
        params![provider, account],
        |r| r.get(0),
    )?;
    Ok(ConnectionView {
        provider: provider.into(),
        state,
        account_id: account,
        display_name: name,
        last_sync_at: last_sync,
        capabilities,
        other_account_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::{EntitlementRow, ReadOutcome, Resource, commit_entitlements};

    fn db() -> Connection {
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::db::open(&dir.path().join("t.db")).unwrap();
        std::mem::forget(dir);
        conn
    }

    fn owned(id: &str) -> EntitlementRow {
        EntitlementRow {
            native_id: id.into(),
            class: "owned".into(),
            owners: vec![],
            exclude_reason: None,
        }
    }

    #[test]
    fn linking_reports_new_same_and_changed_accounts() {
        let c = db();
        assert_eq!(
            record_connection(&c, "gog", "a", Some("Ana")).unwrap(),
            AccountChange::New
        );
        assert_eq!(
            record_connection(&c, "gog", "a", None).unwrap(),
            AccountChange::Same
        );
        let v = view(&c, "gog").unwrap();
        assert_eq!(
            v.display_name.as_deref(),
            Some("Ana"),
            "a missing name does not erase it"
        );
        assert_eq!(
            record_connection(&c, "gog", "b", None).unwrap(),
            AccountChange::Changed
        );
        let v = view(&c, "gog").unwrap();
        assert_eq!(v.display_name, None, "the old account's name is not kept");
        assert_eq!(v.account_id.as_deref(), Some("b"));
    }

    #[test]
    fn disconnecting_keeps_the_account_so_the_last_copy_stays_readable() {
        let c = db();
        record_connection(&c, "steam", "76561198000000001", None).unwrap();
        mark_disconnected(&c, "steam").unwrap();
        let v = view(&c, "steam").unwrap();
        assert_eq!(v.state, "disconnected");
        assert_eq!(v.account_id.as_deref(), Some("76561198000000001"));
    }

    #[test]
    fn capabilities_show_the_last_attempt_and_the_last_complete_copy() {
        let mut c = db();
        record_connection(&c, "gog", "a", None).unwrap();
        let v = view(&c, "gog").unwrap();
        let states: Vec<_> = v
            .capabilities
            .iter()
            .map(|x| (x.capability.as_str(), x.state.as_str()))
            .collect();
        assert_eq!(
            states,
            [
                ("library", "not_synced"),
                ("wishlist", "not_synced"),
                ("wishlist_write", "validated")
            ]
        );
        let epic = view(&c, "epic").unwrap();
        assert_eq!(
            epic.capabilities.last().map(|x| x.state.as_str()),
            Some("unavailable"),
            "epic has no wishlist to write"
        );
        commit_entitlements(
            &mut c,
            "gog",
            "a",
            Resource::Library,
            &[],
            &[owned("1")],
            &ReadOutcome::Complete,
            false,
            100,
        )
        .unwrap();
        let failed = commit_entitlements(
            &mut c,
            "gog",
            "a",
            Resource::Library,
            &[],
            &[],
            &ReadOutcome::Failed,
            false,
            200,
        )
        .unwrap();
        crate::sync::set_generation_error(&c, failed.generation_id, "network").unwrap();
        let lib = view(&c, "gog").unwrap().capabilities.remove(0);
        assert_eq!(lib.state, "failed");
        assert_eq!(lib.error_code.as_deref(), Some("network"));
        assert_eq!(
            (lib.last_attempt_at, lib.last_complete_at, lib.item_count),
            (Some(200), Some(100), Some(1))
        );
    }

    #[test]
    fn epic_wishlist_is_shown_as_unavailable_not_as_empty() {
        let c = db();
        let v = view(&c, "epic").unwrap();
        let wl = v
            .capabilities
            .iter()
            .find(|x| x.capability == "wishlist")
            .unwrap();
        assert_eq!(wl.state, "unavailable");
        assert_eq!(wl.item_count, None);
    }

    #[test]
    fn data_of_a_previous_account_is_flagged_and_kept() {
        let mut c = db();
        record_connection(&c, "gog", "old", None).unwrap();
        commit_entitlements(
            &mut c,
            "gog",
            "old",
            Resource::Library,
            &[],
            &[owned("1")],
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        assert!(!view(&c, "gog").unwrap().other_account_data);
        record_connection(&c, "gog", "new", None).unwrap();
        assert!(view(&c, "gog").unwrap().other_account_data);
        let kept: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM entitlement WHERE account_id = 'old'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kept, 1);
    }
}
