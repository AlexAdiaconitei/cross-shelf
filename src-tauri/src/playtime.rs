//! Play time ("Tiempo jugado"): what each store says about the minutes played and the last session,
//! and the history of changes ("fotos de horas") that the diary and the suggestions read.
//!
//! `playtime` is cache: the next sync reads it again. `playtime_snapshot` is the user's: a row only
//! when the minutes change, so it grows with what is played, not with each sync.

use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug, thiserror::Error)]
pub enum PlaytimeError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// One product's play time as a store reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaytimeRow {
    pub native_id: String,
    /// `None`: the store does not say. Different from `Some(0)`, never played.
    pub minutes: Option<i64>,
    /// Unix seconds of the last session, when the store gives it (Steam only).
    pub last_played: Option<i64>,
}

/// Stores a play time read for one account and store. `complete` reads replace the account's
/// rows of that store (a game that left the library leaves); partial ones (GOG, one game at a
/// time) only add or update. Snapshots are written first, in their own transaction, because they
/// live in the user file (ADR 0004).
pub fn record(
    conn: &mut Connection,
    store: &str,
    account: &str,
    rows: &[PlaytimeRow],
    complete: bool,
    now: i64,
) -> Result<(), PlaytimeError> {
    let tx = conn.transaction()?;
    {
        let mut last = tx.prepare_cached(
            "SELECT minutes FROM playtime_snapshot
             WHERE account_id = ?1 AND store = ?2 AND native_id = ?3
             ORDER BY observed_at DESC LIMIT 1",
        )?;
        let mut add = tx.prepare_cached(
            "INSERT OR REPLACE INTO playtime_snapshot(account_id, store, native_id, minutes, observed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for r in rows {
            let Some(minutes) = r.minutes else { continue };
            let before: Option<i64> = last
                .query_map(params![account, store, r.native_id], |x| x.get(0))?
                .next()
                .transpose()?;
            if before != Some(minutes) {
                add.execute(params![account, store, r.native_id, minutes, now])?;
            }
        }
    }
    tx.commit()?;

    let tx = conn.transaction()?;
    if complete {
        tx.execute(
            "DELETE FROM playtime WHERE account_id = ?1 AND store = ?2",
            params![account, store],
        )?;
    }
    {
        let mut put = tx.prepare_cached(
            "INSERT INTO playtime(account_id, store, native_id, minutes, last_played, observed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(account_id, store, native_id) DO UPDATE SET
               minutes = excluded.minutes, last_played = excluded.last_played,
               observed_at = excluded.observed_at",
        )?;
        for r in rows {
            put.execute(params![
                account,
                store,
                r.native_id,
                r.minutes,
                r.last_played,
                now
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// The cached minutes of one product when they were read less than `max_age` seconds ago:
/// `Some(minutes)` (itself `None` when the store did not give them), or `None` to ask again.
pub fn recent(
    conn: &Connection,
    store: &str,
    account: &str,
    native_id: &str,
    now: i64,
    max_age: i64,
) -> Result<Option<Option<i64>>, rusqlite::Error> {
    conn.query_row(
        "SELECT minutes FROM playtime
         WHERE account_id = ?1 AND store = ?2 AND native_id = ?3 AND observed_at > ?4",
        params![account, store, native_id, now - max_age],
        |r| r.get(0),
    )
    .optional()
}

/// One product's time for the game sheet.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ProductTime {
    pub store: String,
    pub native_id: String,
    /// `None`: the store does not give it (or has not been read). Never shown as 0.
    pub minutes: Option<i64>,
    /// Last session, when the store gives it (Steam).
    pub last_played: Option<i64>,
}

/// What the cache knows of each product, for the active accounts. One answer per product asked,
/// in the same order.
pub fn of_products(
    conn: &Connection,
    products: &[(&str, &str)],
) -> Result<Vec<ProductTime>, rusqlite::Error> {
    let mut st = conn.prepare_cached(
        "SELECT SUM(t.minutes), MAX(t.last_played) FROM playtime t
         JOIN connection c ON c.provider = t.store AND c.account_id = t.account_id
         WHERE t.store = ?1 AND t.native_id = ?2",
    )?;
    products
        .iter()
        .map(|(store, id)| {
            let (minutes, last_played) =
                st.query_row(params![store, id], |r| Ok((r.get(0)?, r.get(1)?)))?;
            Ok(ProductTime {
                store: (*store).to_owned(),
                native_id: (*id).to_owned(),
                minutes,
                last_played,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
