//! Mis partidas: what the user is playing, has pending or has finished, on any product of their
//! library. A play belongs to the person, not to an account, and lives in the user file: it
//! survives clearing the cache and the product leaving the library.

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use ts_rs::TS;

use crate::review::ProductRef;

pub mod diary;
pub mod suggestions;

/// States of a play, in board order.
pub const STATES: &[&str] = &["playing", "backlog", "paused", "completed", "dropped"];
/// Longest note, in characters.
pub const NOTE_MAX: usize = 280;
/// Below this gap between two neighbours the whole state is renumbered 1, 2, 3…
pub const MIN_GAP: f64 = 1e-6;

#[derive(Debug, thiserror::Error)]
pub enum PlayError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("unknown state")]
    BadState,
    #[error("unknown store")]
    BadStore,
    #[error("note too long")]
    NoteTooLong,
    #[error("no such play")]
    NotFound,
}

impl PlayError {
    /// Plain code for the UI, which words it. Never SQLite text.
    pub fn code(&self) -> &'static str {
        match self {
            PlayError::Sqlite(_) => "internal",
            PlayError::BadState => "bad_state",
            PlayError::BadStore => "bad_store",
            PlayError::NoteTooLong => "note_too_long",
            PlayError::NotFound => "not_found",
        }
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, TS)]
#[ts(export)]
pub struct PlayView {
    /// The chosen product. Title and cover are missing while the cache is empty.
    pub product: ProductRef,
    /// `playing`, `backlog`, `paused`, `completed` or `dropped`.
    pub state: String,
    /// Order inside the state, ascending.
    pub position: f64,
    /// When it entered its current state (seconds).
    pub since: i64,
    /// First time it entered `playing`.
    pub started_at: Option<i64>,
    /// Last time it entered `completed`; cleared when it leaves.
    pub finished_at: Option<i64>,
    pub note: String,
    /// False when no active account holds the product any more.
    pub in_library: bool,
    /// Minutes played by the active accounts; `None` when no store gives them (not 0).
    pub minutes: Option<i64>,
    /// Last session, or last rise in hours where the store gives no session (seconds).
    pub last_activity: Option<i64>,
}

pub(crate) const LIST_SQL: &str =
    "SELECT y.store, y.native_id, p.title, COALESCE(p.kind,'unknown'),
            COALESCE(p.windows,'unknown'), p.url, p.image_url, y.state, y.position, y.since,
            y.started_at, y.finished_at, y.note,
            EXISTS(SELECT 1 FROM entitlement e
                   JOIN connection cn ON cn.provider = e.store AND cn.account_id = e.account_id
                   WHERE e.store = y.store AND e.native_id = y.native_id),
            (SELECT SUM(t.minutes) FROM playtime t
                   JOIN connection cn ON cn.provider = t.store AND cn.account_id = t.account_id
                   WHERE t.store = y.store AND t.native_id = y.native_id)
     FROM play y
     LEFT JOIN product p ON p.store = y.store AND p.native_id = y.native_id
     ORDER BY y.state, y.position";

/// Every play, grouped by state and in the user's order.
pub fn list(c: &Connection) -> Result<Vec<PlayView>, rusqlite::Error> {
    let activity = suggestions::activity(c)?;
    let mut st = c.prepare(LIST_SQL)?;
    let rows = st.query_map([], |r| {
        Ok(PlayView {
            product: ProductRef {
                store: r.get(0)?,
                native_id: r.get(1)?,
                title: r.get(2)?,
                kind: r.get(3)?,
                windows: r.get(4)?,
                url: r.get(5)?,
                image_url: r.get(6)?,
            },
            state: r.get(7)?,
            position: r.get(8)?,
            since: r.get(9)?,
            started_at: r.get(10)?,
            finished_at: r.get(11)?,
            note: r.get(12)?,
            in_library: r.get::<_, i64>(13)? != 0,
            minutes: r.get(14)?,
            last_activity: None,
        })
    })?;
    let rows = rows.map(|p| {
        p.map(|mut p| {
            p.last_activity = activity
                .get(&(p.product.store.clone(), p.product.native_id.clone()))
                .copied();
            p
        })
    });
    rows.collect()
}

fn check(store: &str, state: &str) -> Result<(), PlayError> {
    if !crate::connections::PROVIDERS.contains(&store) {
        return Err(PlayError::BadStore);
    }
    if !STATES.contains(&state) {
        return Err(PlayError::BadState);
    }
    Ok(())
}

/// Creates a play or moves it: to the top of `state`, or just ahead of `before` (store, id) when that card is
/// in `state`. Dates follow the move: entering `playing` sets `started_at` once, entering
/// `completed` sets `finished_at`, leaving it clears it; `since` changes only with the state.
pub fn set(
    c: &mut Connection,
    store: &str,
    native_id: &str,
    state: &str,
    before: Option<(&str, &str)>,
    now: i64,
) -> Result<(), PlayError> {
    check(store, state)?;
    let tx = c.transaction()?;
    let old: Option<(String, i64, Option<i64>, Option<i64>)> = tx
        .query_row(
            "SELECT state, since, started_at, finished_at FROM play
             WHERE store = ?1 AND native_id = ?2",
            params![store, native_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let changed = old.as_ref().is_none_or(|o| o.0 != state);
    let since = match &old {
        Some((_, s, _, _)) if !changed => *s,
        _ => now,
    };
    let started_at = old
        .as_ref()
        .and_then(|o| o.2)
        .or((state == "playing").then_some(now));
    let finished_at = match (state, &old) {
        ("completed", Some((s, _, _, f))) if s == "completed" => *f,
        ("completed", _) => Some(now),
        _ => None,
    };

    // The other cards of the target state, in order, and where this one goes among them.
    let others: Vec<(String, String, f64)> = tx
        .prepare(
            "SELECT store, native_id, position FROM play
             WHERE state = ?1 AND NOT (store = ?2 AND native_id = ?3)
             ORDER BY position",
        )?
        .query_map(params![state, store, native_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<Result<_, _>>()?;
    let at = before
        .and_then(|(bs, bi)| others.iter().position(|(s, id, _)| s == bs && id == bi))
        .unwrap_or(0);
    let position = match (at.checked_sub(1).map(|i| others[i].2), others.get(at)) {
        (Some(prev), Some((_, _, next))) => (prev + next) / 2.0,
        (None, Some((_, _, next))) => next - 1.0,
        (_, None) => 1.0,
    };

    tx.execute(
        "INSERT INTO play(store, native_id, state, position, since, started_at, finished_at,
                          note, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '', ?8)
         ON CONFLICT(store, native_id) DO UPDATE SET
           state = excluded.state, position = excluded.position, since = excluded.since,
           started_at = excluded.started_at, finished_at = excluded.finished_at,
           updated_at = excluded.updated_at",
        params![
            store,
            native_id,
            state,
            position,
            since,
            started_at,
            finished_at,
            now
        ],
    )?;
    let neighbours = [
        at.checked_sub(1).map(|i| others[i].2),
        others.get(at).map(|o| o.2),
    ];
    if neighbours
        .iter()
        .flatten()
        .any(|n| (n - position).abs() < MIN_GAP)
    {
        renumber(&tx, state)?;
    }
    tx.commit()?;
    Ok(())
}

fn renumber(c: &Connection, state: &str) -> Result<(), rusqlite::Error> {
    let keys: Vec<(String, String)> = c
        .prepare("SELECT store, native_id FROM play WHERE state = ?1 ORDER BY position")?
        .query_map([state], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    let mut st = c.prepare("UPDATE play SET position = ?3 WHERE store = ?1 AND native_id = ?2")?;
    for (i, (store, id)) in keys.iter().enumerate() {
        st.execute(params![store, id, (i + 1) as f64])?;
    }
    Ok(())
}

/// Saves the note of a play, trimmed. At most [`NOTE_MAX`] characters.
pub fn note(
    c: &Connection,
    store: &str,
    native_id: &str,
    text: &str,
    now: i64,
) -> Result<(), PlayError> {
    let text = text.trim();
    if text.chars().count() > NOTE_MAX {
        return Err(PlayError::NoteTooLong);
    }
    let n = c.execute(
        "UPDATE play SET note = ?3, updated_at = ?4 WHERE store = ?1 AND native_id = ?2",
        params![store, native_id, text, now],
    )?;
    if n == 0 {
        return Err(PlayError::NotFound);
    }
    Ok(())
}

/// Forgets a play.
pub fn remove(c: &Connection, store: &str, native_id: &str) -> Result<(), PlayError> {
    c.execute(
        "DELETE FROM play WHERE store = ?1 AND native_id = ?2",
        params![store, native_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
