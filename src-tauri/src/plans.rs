//! Cleanup plans and batches (spec 8.1 steps 5-7, 8.2, 8.3).
//!
//! A batch is a reviewed list of wishlist entries of one account and one store, each with a
//! desired state (`absent`), never a toggle. It is bound to a revision: the account, the wishlist
//! generation it was built on and the entries. Confirming re-checks that revision; anything that
//! moved invalidates the batch and asks for a new review. For a store whose write is validated
//! (`write_validated`) the app sends the confirmed changes itself (`executable`,
//! `record_result`); for the others the user removes entries in the store and the batch is
//! reconciled by reading the wishlist again. Either way an entry is done only after a reread.

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use ts_rs::TS;

/// A write capability is enabled only with its record in `docs/validation/<store>-write.md`
/// (spec 8.3). A test fails if this says yes without the file.
pub fn write_validated(store: &str) -> bool {
    VALIDATED_WRITES.contains(&store)
}

/// Stores whose wishlist writing has a validation record, both since 18 Nov 2026
/// (`docs/validation/gog-write.md`, `docs/validation/steam-write.md`). Epic has no wishlist.
const VALIDATED_WRITES: &[&str] = &["gog", "steam"];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlanError {
    #[error("no entries selected")]
    Empty,
    #[error("store has no wishlist in this version")]
    StoreUnsupported,
    #[error("account is not the active one of this store")]
    AccountNotActive,
    #[error("entry is not in the wishlist any more")]
    NotInWishlist,
    #[error("entry is marked to keep")]
    Kept,
    #[error("the plan changed since it was reviewed")]
    PlanChanged,
    #[error("entry is not covered by anything the user holds")]
    NotCovered,
    #[error("batch not found or not in a state that allows this")]
    BadState,
    #[error("sqlite: {0}")]
    Sqlite(String),
}

impl From<rusqlite::Error> for PlanError {
    fn from(e: rusqlite::Error) -> Self {
        PlanError::Sqlite(e.to_string())
    }
}

impl PlanError {
    pub fn code(&self) -> &'static str {
        match self {
            PlanError::Empty => "empty",
            PlanError::StoreUnsupported => "store_unsupported",
            PlanError::AccountNotActive => "account_not_active",
            PlanError::NotInWishlist => "not_in_wishlist",
            PlanError::Kept => "kept",
            PlanError::PlanChanged => "plan_changed",
            PlanError::NotCovered => "not_covered",
            PlanError::BadState => "bad_state",
            PlanError::Sqlite(_) => "internal",
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct BatchOp {
    pub op_id: String,
    pub native_id: String,
    pub title: Option<String>,
    /// Always `absent` in v1: cleanup only removes.
    pub desired: String,
    /// `pending`, `already_in_desired_state`, `applied`, `failed`, `needs_reauth`,
    /// `not_supported`, `unverified` or `cancelled_not_sent`.
    pub status: String,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct BatchView {
    pub id: String,
    pub store: String,
    pub account_id: String,
    /// `draft`, `confirmed`, `done` or `invalidated`.
    pub state: String,
    pub revision: String,
    pub created_at: i64,
    pub confirmed_at: Option<i64>,
    /// Whether the app can send the changes itself; otherwise "Abrir en la tienda".
    pub write_validated: bool,
    pub ops: Vec<BatchOp>,
}

/// FNV-1a, 64 bits: stable across builds, enough to bind a confirmation to what was reviewed.
fn fnv(parts: &[&str]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        for b in p.bytes().chain(std::iter::once(0)) {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

fn active_account(c: &Connection, store: &str) -> rusqlite::Result<Option<String>> {
    Ok(c.query_row(
        "SELECT account_id FROM connection WHERE provider = ?1",
        [store],
        |r| r.get::<_, Option<String>>(0),
    )
    .optional()?
    .flatten())
}

fn wishlist_generation(c: &Connection, store: &str, account: &str) -> rusqlite::Result<i64> {
    Ok(c.query_row(
        "SELECT MAX(id) FROM sync_generation
         WHERE provider = ?1 AND account_id = ?2 AND resource = 'wishlist' AND state = 'complete'",
        params![store, account],
        |r| r.get::<_, Option<i64>>(0),
    )?
    .unwrap_or(0))
}

/// Checks every precondition against the database as it is now and returns the revision.
fn revision(
    c: &Connection,
    store: &str,
    account: &str,
    ids: &[String],
) -> Result<(String, i64), PlanError> {
    if store == "epic" {
        return Err(PlanError::StoreUnsupported);
    }
    if ids.is_empty() {
        return Err(PlanError::Empty);
    }
    if active_account(c, store)?.as_deref() != Some(account) {
        return Err(PlanError::AccountNotActive);
    }
    for id in ids {
        let (present, kept): (bool, bool) = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM wishlist_entry WHERE account_id = ?1 AND store = ?2 AND native_id = ?3),
                    EXISTS(SELECT 1 FROM wishlist_keep WHERE account_id = ?1 AND store = ?2 AND native_id = ?3)",
            params![account, store, id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if !present {
            return Err(PlanError::NotInWishlist);
        }
        if kept {
            return Err(PlanError::Kept);
        }
    }
    let generation = wishlist_generation(c, store, account)?;
    let mut sorted: Vec<&str> = ids.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    sorted.dedup();
    let gen_text = generation.to_string();
    let mut parts = vec![store, account, gen_text.as_str()];
    parts.extend(sorted);
    Ok((fnv(&parts), generation))
}

/// Every entry must be covered by an exact match in the review: the same store ID, a decision
/// «Es el mismo» or a pack's credited component. A title that only looks alike is a candidate and
/// never enough on its own (spec 7.2, «nunca borrado por similitud»).
pub fn require_covered(
    c: &Connection,
    store: &str,
    account: &str,
    ids: &[String],
    include_family: bool,
) -> Result<(), PlanError> {
    let review = crate::review::build(c, include_family)?;
    for id in ids {
        let covered = review.iter().any(|it| {
            it.account_id == account
                && it.wanted.store == store
                && &it.wanted.native_id == id
                && it.matches.iter().any(|m| m.outcome == "exact")
        });
        if !covered {
            return Err(PlanError::NotCovered);
        }
    }
    Ok(())
}

/// Cancels what a confirmed batch has not sent yet. Finished operations keep their result; the
/// rest are marked `cancelled_not_sent` and nothing more is sent for this batch.
pub fn cancel(c: &Connection, batch: &str, now: i64) -> Result<BatchView, PlanError> {
    let b = get(c, batch)?.ok_or(PlanError::BadState)?;
    if b.state != "confirmed" {
        return Err(PlanError::BadState);
    }
    c.execute(
        "UPDATE batch_op SET status = 'cancelled_not_sent', finished_at = ?2
         WHERE batch_id = ?1 AND status = 'pending'",
        params![batch, now],
    )?;
    c.execute(
        "UPDATE action_batch SET state = 'done' WHERE id = ?1",
        [batch],
    )?;
    get(c, batch)?.ok_or(PlanError::BadState)
}

/// Puts the operations that did not finish well (`failed`, `unverified`, `needs_reauth`) back to
/// pending, so they can be sent again. Sending reads the wishlist first, so an entry that did go
/// away is recorded as already removed instead of being sent twice.
pub fn retry_failed(c: &Connection, batch: &str) -> Result<BatchView, PlanError> {
    let b = get(c, batch)?.ok_or(PlanError::BadState)?;
    if b.state != "done" || !write_validated(&b.store) {
        return Err(PlanError::BadState);
    }
    if active_account(c, &b.store)?.as_deref() != Some(b.account_id.as_str()) {
        return Err(PlanError::AccountNotActive);
    }
    let n = c.execute(
        "UPDATE batch_op SET status = 'pending', finished_at = NULL
         WHERE batch_id = ?1 AND status IN ('failed', 'unverified', 'needs_reauth')",
        [batch],
    )?;
    if n == 0 {
        return Err(PlanError::BadState);
    }
    c.execute(
        "UPDATE action_batch SET state = 'confirmed' WHERE id = ?1",
        [batch],
    )?;
    get(c, batch)?.ok_or(PlanError::BadState)
}

/// Builds a draft batch. The snapshot keeps what the entries looked like, without secrets.
pub fn create(
    c: &mut Connection,
    store: &str,
    account: &str,
    ids: &[String],
    now: i64,
) -> Result<BatchView, PlanError> {
    let (rev, generation) = revision(c, store, account, ids)?;
    let mut snapshot = Vec::new();
    for id in ids {
        let row: (Option<i64>, Option<i64>, i64) = c.query_row(
            "SELECT priority, added_at, observed_at FROM wishlist_entry
             WHERE account_id = ?1 AND store = ?2 AND native_id = ?3",
            params![account, store, id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        snapshot.push(serde_json::json!({
            "native_id": id, "priority": row.0, "added_at": row.1, "observed_at": row.2
        }));
    }
    let id = format!("b{now}-{}", &fnv(&[&rev, &now.to_string()])[..8]);
    let tx = c.transaction()?;
    tx.execute(
        "INSERT INTO action_batch(id, store, account_id, state, revision, generation_id, snapshot, created_at)
         VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7)",
        params![
            id,
            store,
            account,
            rev,
            generation,
            serde_json::Value::Array(snapshot).to_string(),
            now
        ],
    )?;
    for (i, nid) in ids.iter().enumerate() {
        tx.execute(
            "INSERT INTO batch_op(op_id, batch_id, native_id, desired, status) VALUES (?1, ?2, ?3, 'absent', 'pending')",
            params![format!("{id}-{i}"), id, nid],
        )?;
    }
    tx.commit()?;
    get(c, &id)?.ok_or(PlanError::BadState)
}

fn op_ids(c: &Connection, batch: &str) -> rusqlite::Result<Vec<String>> {
    let mut st = c.prepare("SELECT native_id FROM batch_op WHERE batch_id = ?1 ORDER BY op_id")?;
    st.query_map([batch], |r| r.get(0))?.collect()
}

/// Confirmation bound to the batch, its revision and its account. If anything moved since the
/// review, the batch is invalidated and nothing is confirmed (spec 8.2).
pub fn confirm(
    c: &Connection,
    batch: &str,
    reviewed: &str,
    now: i64,
) -> Result<BatchView, PlanError> {
    let b = get(c, batch)?.ok_or(PlanError::BadState)?;
    if b.state != "draft" {
        return Err(PlanError::BadState);
    }
    let ids = op_ids(c, batch)?;
    let current = revision(c, &b.store, &b.account_id, &ids).map(|(r, _)| r);
    if reviewed != b.revision || current.as_deref().ok() != Some(b.revision.as_str()) {
        c.execute(
            "UPDATE action_batch SET state = 'invalidated' WHERE id = ?1",
            [batch],
        )?;
        return Err(PlanError::PlanChanged);
    }
    c.execute(
        "UPDATE action_batch SET state = 'confirmed', confirmed_at = ?2 WHERE id = ?1",
        params![batch, now],
    )?;
    get(c, batch)?.ok_or(PlanError::BadState)
}

/// After the wishlist was read again: an entry that is gone is done, verified by that reread.
/// Without a newer complete read nothing changes: opening the store proves nothing.
pub fn reconcile(c: &Connection, batch: &str, now: i64) -> Result<BatchView, PlanError> {
    let (store, account, state, generation): (String, String, String, i64) = c
        .query_row(
            "SELECT store, account_id, state, generation_id FROM action_batch WHERE id = ?1",
            [batch],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .ok_or(PlanError::BadState)?;
    if state != "confirmed" {
        return Err(PlanError::BadState);
    }
    if active_account(c, &store)?.as_deref() != Some(account.as_str()) {
        c.execute(
            "UPDATE action_batch SET state = 'invalidated' WHERE id = ?1",
            [batch],
        )?;
        return Err(PlanError::AccountNotActive);
    }
    if wishlist_generation(c, &store, &account)? > generation {
        c.execute(
            "UPDATE batch_op SET status = 'already_in_desired_state', finished_at = ?2
             WHERE batch_id = ?1 AND status = 'pending'
               AND NOT EXISTS (SELECT 1 FROM wishlist_entry w
                               WHERE w.account_id = ?3 AND w.store = ?4 AND w.native_id = batch_op.native_id)",
            params![batch, now, account, store],
        )?;
        let pending: i64 = c.query_row(
            "SELECT COUNT(*) FROM batch_op WHERE batch_id = ?1 AND status = 'pending'",
            [batch],
            |r| r.get(0),
        )?;
        if pending == 0 {
            c.execute(
                "UPDATE action_batch SET state = 'done' WHERE id = ?1",
                [batch],
            )?;
        }
    }
    get(c, batch)?.ok_or(PlanError::BadState)
}

/// (op_id, native_id) of an operation still to send.
pub type PendingOp = (String, String);

/// What a confirmed batch still has to send: (store, account, pending ops). Only for a store with
/// a validated write and the account that is active now; everything else is refused, so nothing
/// is ever sent for a batch the user did not confirm or for another account.
pub fn executable(
    c: &Connection,
    batch: &str,
) -> Result<(String, String, Vec<PendingOp>), PlanError> {
    let b = get(c, batch)?.ok_or(PlanError::BadState)?;
    if b.state != "confirmed" {
        return Err(PlanError::BadState);
    }
    if !write_validated(&b.store) {
        return Err(PlanError::StoreUnsupported);
    }
    if active_account(c, &b.store)?.as_deref() != Some(b.account_id.as_str()) {
        c.execute(
            "UPDATE action_batch SET state = 'invalidated' WHERE id = ?1",
            [batch],
        )?;
        return Err(PlanError::AccountNotActive);
    }
    let pending = b
        .ops
        .into_iter()
        .filter(|o| o.status == "pending")
        .map(|o| (o.op_id, o.native_id))
        .collect();
    Ok((b.store, b.account_id, pending))
}

/// Stores the result of one sent operation. The writer only reports `applied` after reading the
/// wishlist back, so that result is verified. The batch is done when nothing is pending.
pub fn record_result(
    c: &Connection,
    batch: &str,
    op_id: &str,
    status: &str,
    now: i64,
) -> Result<(), PlanError> {
    c.execute(
        "UPDATE batch_op SET status = ?3, finished_at = ?4
         WHERE batch_id = ?1 AND op_id = ?2 AND status = 'pending'",
        params![batch, op_id, status, now],
    )?;
    let pending: i64 = c.query_row(
        "SELECT COUNT(*) FROM batch_op WHERE batch_id = ?1 AND status = 'pending'",
        [batch],
        |r| r.get(0),
    )?;
    if pending == 0 {
        c.execute(
            "UPDATE action_batch SET state = 'done' WHERE id = ?1 AND state = 'confirmed'",
            [batch],
        )?;
    }
    Ok(())
}

/// Drops a draft that was never confirmed.
pub fn discard(c: &Connection, batch: &str) -> Result<(), PlanError> {
    let n = c.execute(
        "DELETE FROM batch_op WHERE batch_id = ?1
           AND EXISTS (SELECT 1 FROM action_batch WHERE id = ?1 AND state IN ('draft','invalidated'))",
        [batch],
    )?;
    let m = c.execute(
        "DELETE FROM action_batch WHERE id = ?1 AND state IN ('draft','invalidated')",
        [batch],
    )?;
    if n + m == 0 {
        return Err(PlanError::BadState);
    }
    Ok(())
}

/// Batches of accounts that are no longer active are invalidated, never run (spec 5.5).
pub fn invalidate_inactive(c: &Connection) -> rusqlite::Result<()> {
    c.execute(
        "UPDATE action_batch SET state = 'invalidated'
         WHERE state IN ('draft','confirmed')
           AND NOT EXISTS (SELECT 1 FROM connection cn
                           WHERE cn.provider = action_batch.store AND cn.account_id = action_batch.account_id)",
        [],
    )?;
    Ok(())
}

fn ops(c: &Connection, batch: &str, store: &str) -> rusqlite::Result<Vec<BatchOp>> {
    let mut st = c.prepare(
        "SELECT o.op_id, o.native_id, p.title, o.desired, o.status, o.finished_at
         FROM batch_op o LEFT JOIN product p ON p.store = ?2 AND p.native_id = o.native_id
         WHERE o.batch_id = ?1 ORDER BY o.op_id",
    )?;
    st.query_map(params![batch, store], |r| {
        Ok(BatchOp {
            op_id: r.get(0)?,
            native_id: r.get(1)?,
            title: r.get(2)?,
            desired: r.get(3)?,
            status: r.get(4)?,
            finished_at: r.get(5)?,
        })
    })?
    .collect()
}

pub fn get(c: &Connection, batch: &str) -> rusqlite::Result<Option<BatchView>> {
    let head = c
        .query_row(
            "SELECT id, store, account_id, state, revision, created_at, confirmed_at
             FROM action_batch WHERE id = ?1",
            [batch],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, Option<i64>>(6)?,
                ))
            },
        )
        .optional()?;
    let Some((id, store, account_id, state, revision, created_at, confirmed_at)) = head else {
        return Ok(None);
    };
    Ok(Some(BatchView {
        ops: ops(c, &id, &store)?,
        write_validated: write_validated(&store),
        id,
        store,
        account_id,
        state,
        revision,
        created_at,
        confirmed_at,
    }))
}

pub fn list(c: &Connection) -> rusqlite::Result<Vec<BatchView>> {
    invalidate_inactive(c)?;
    let ids: Vec<String> = c
        .prepare("SELECT id FROM action_batch ORDER BY created_at DESC, id DESC")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = Vec::new();
    for id in ids {
        if let Some(b) = get(c, &id)? {
            out.push(b);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
