//! «My data»: export and import of everything the stores cannot give back (plan 0.3.0, 2.6).
//!
//! The file carries plays and notes, matching decisions, «Keep», dismissed suggestions, play time
//! snapshots and settings, each product with its title and store so a person can read it and the
//! app can relate it again. Never connections, tokens, cookies or `secret_ref`. Cleanup batches
//! are left out: they belong to syncs of one computer.
//!
//! Importing replaces the user data (it does not merge), after a copy of the user file.

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ts_rs::TS;

pub const FORMAT: &str = "crossshelf-user-data";
pub const VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum UserDataError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a CrossShelf user data file")]
    NotOurs,
    #[error("made by a newer version")]
    TooNew,
    #[error("backup: {0}")]
    Backup(#[from] crate::db::DbError),
}

impl UserDataError {
    /// Plain code for the UI, which words it.
    pub fn code(&self) -> &'static str {
        match self {
            UserDataError::NotOurs => "not_ours",
            UserDataError::TooNew => "too_new",
            UserDataError::Io(_) => "io",
            UserDataError::Sqlite(_) | UserDataError::Backup(_) => "internal",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UserData {
    pub format: String,
    pub version: u32,
    /// Unix seconds.
    pub exported_at: i64,
    pub settings: Vec<SettingRow>,
    pub plays: Vec<PlayRow>,
    pub relations: Vec<RelationRow>,
    pub keeps: Vec<KeepRow>,
    pub dismissed: Vec<DismissedRow>,
    pub snapshots: Vec<SnapshotRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SettingRow {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PlayRow {
    pub store: String,
    pub native_id: String,
    /// For reading only; the cache may not know it.
    pub title: Option<String>,
    pub state: String,
    pub position: f64,
    pub since: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub note: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RelationRow {
    pub wanted_store: String,
    pub wanted_id: String,
    pub wanted_title: Option<String>,
    pub held_store: String,
    pub held_id: String,
    pub held_title: Option<String>,
    pub class: String,
    pub origin: String,
    pub evidence: Option<String>,
    pub decided_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct KeepRow {
    pub account_id: String,
    pub store: String,
    pub native_id: String,
    pub title: Option<String>,
    pub reason: Option<String>,
    pub decided_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DismissedRow {
    pub store: String,
    pub native_id: String,
    pub title: Option<String>,
    pub target: String,
    pub dismissed_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SnapshotRow {
    pub account_id: String,
    pub store: String,
    pub native_id: String,
    pub title: Option<String>,
    pub minutes: i64,
    pub observed_at: i64,
}

/// What a file brings, shown before the user confirms an import.
#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct UserDataSummary {
    pub exported_at: i64,
    pub plays: usize,
    pub relations: usize,
    pub keeps: usize,
    pub dismissed: usize,
    pub snapshots: usize,
    pub settings: usize,
}

const TITLE: &str = "(SELECT title FROM product p WHERE p.store = {s} AND p.native_id = {i})";

fn title(store: &str, id: &str) -> String {
    TITLE.replace("{s}", store).replace("{i}", id)
}

fn rows<T>(
    c: &Connection,
    sql: &str,
    f: impl FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
) -> rusqlite::Result<Vec<T>> {
    c.prepare(sql)?.query_map([], f)?.collect()
}

/// Everything the export carries, read from the user file (titles from the cache).
pub fn export(c: &Connection, now: i64) -> Result<UserData, UserDataError> {
    Ok(UserData {
        format: FORMAT.into(),
        version: VERSION,
        exported_at: now,
        settings: rows(c, "SELECT key, value FROM setting ORDER BY key", |r| {
            Ok(SettingRow {
                key: r.get(0)?,
                value: r.get(1)?,
            })
        })?,
        plays: rows(
            c,
            &format!(
                "SELECT store, native_id, {}, state, position, since, started_at, finished_at,
                        note, updated_at
                 FROM play y ORDER BY state, position",
                title("y.store", "y.native_id")
            ),
            |r| {
                Ok(PlayRow {
                    store: r.get(0)?,
                    native_id: r.get(1)?,
                    title: r.get(2)?,
                    state: r.get(3)?,
                    position: r.get(4)?,
                    since: r.get(5)?,
                    started_at: r.get(6)?,
                    finished_at: r.get(7)?,
                    note: r.get(8)?,
                    updated_at: r.get(9)?,
                })
            },
        )?,
        relations: rows(
            c,
            &format!(
                "SELECT wanted_store, wanted_id, {}, held_store, held_id, {}, class, origin,
                        evidence, decided_at
                 FROM product_relation x ORDER BY 1, 2, 4, 5, 8",
                title("x.wanted_store", "x.wanted_id"),
                title("x.held_store", "x.held_id")
            ),
            |r| {
                Ok(RelationRow {
                    wanted_store: r.get(0)?,
                    wanted_id: r.get(1)?,
                    wanted_title: r.get(2)?,
                    held_store: r.get(3)?,
                    held_id: r.get(4)?,
                    held_title: r.get(5)?,
                    class: r.get(6)?,
                    origin: r.get(7)?,
                    evidence: r.get(8)?,
                    decided_at: r.get(9)?,
                })
            },
        )?,
        keeps: rows(
            c,
            &format!(
                "SELECT account_id, store, native_id, {}, reason, decided_at
                 FROM wishlist_keep k ORDER BY 1, 2, 3",
                title("k.store", "k.native_id")
            ),
            |r| {
                Ok(KeepRow {
                    account_id: r.get(0)?,
                    store: r.get(1)?,
                    native_id: r.get(2)?,
                    title: r.get(3)?,
                    reason: r.get(4)?,
                    decided_at: r.get(5)?,
                })
            },
        )?,
        dismissed: rows(
            c,
            &format!(
                "SELECT store, native_id, {}, target, dismissed_at
                 FROM suggestion_dismissed d ORDER BY 1, 2, 4",
                title("d.store", "d.native_id")
            ),
            |r| {
                Ok(DismissedRow {
                    store: r.get(0)?,
                    native_id: r.get(1)?,
                    title: r.get(2)?,
                    target: r.get(3)?,
                    dismissed_at: r.get(4)?,
                })
            },
        )?,
        snapshots: rows(
            c,
            &format!(
                "SELECT account_id, store, native_id, {}, minutes, observed_at
                 FROM playtime_snapshot s ORDER BY 1, 2, 3, 6",
                title("s.store", "s.native_id")
            ),
            |r| {
                Ok(SnapshotRow {
                    account_id: r.get(0)?,
                    store: r.get(1)?,
                    native_id: r.get(2)?,
                    title: r.get(3)?,
                    minutes: r.get(4)?,
                    observed_at: r.get(5)?,
                })
            },
        )?,
    })
}

/// Reads a file's text. Anything that is not a CrossShelf user data file is refused before its
/// version is looked at.
pub fn parse(text: &str) -> Result<UserData, UserDataError> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|_| UserDataError::NotOurs)?;
    if v.get("format").and_then(serde_json::Value::as_str) != Some(FORMAT) {
        return Err(UserDataError::NotOurs);
    }
    match v.get("version").and_then(serde_json::Value::as_u64) {
        Some(n) if n > u64::from(VERSION) => return Err(UserDataError::TooNew),
        Some(_) => {}
        None => return Err(UserDataError::NotOurs),
    }
    serde_json::from_value(v).map_err(|_| UserDataError::NotOurs)
}

pub fn summary(d: &UserData) -> UserDataSummary {
    UserDataSummary {
        exported_at: d.exported_at,
        plays: d.plays.len(),
        relations: d.relations.len(),
        keeps: d.keeps.len(),
        dismissed: d.dismissed.len(),
        snapshots: d.snapshots.len(),
        settings: d.settings.len(),
    }
}

/// Replaces the user data with the file's, in one transaction. Connections, batches and the
/// cache are not touched.
pub fn import(c: &mut Connection, d: &UserData) -> Result<UserDataSummary, UserDataError> {
    if d.format != FORMAT {
        return Err(UserDataError::NotOurs);
    }
    if d.version > VERSION {
        return Err(UserDataError::TooNew);
    }
    let tx = c.transaction()?;
    tx.execute_batch(
        "DELETE FROM setting; DELETE FROM play; DELETE FROM product_relation;
         DELETE FROM wishlist_keep; DELETE FROM suggestion_dismissed; DELETE FROM playtime_snapshot;",
    )?;
    for s in &d.settings {
        tx.execute(
            "INSERT OR REPLACE INTO setting(key, value) VALUES (?1, ?2)",
            params![s.key, s.value],
        )?;
    }
    for p in &d.plays {
        tx.execute(
            "INSERT OR REPLACE INTO play(store, native_id, state, position, since, started_at,
                                         finished_at, note, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                p.store,
                p.native_id,
                p.state,
                p.position,
                p.since,
                p.started_at,
                p.finished_at,
                p.note,
                p.updated_at
            ],
        )?;
    }
    for r in &d.relations {
        tx.execute(
            "INSERT OR REPLACE INTO product_relation(wanted_store, wanted_id, held_store, held_id,
                                                     class, origin, evidence, decided_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                r.wanted_store,
                r.wanted_id,
                r.held_store,
                r.held_id,
                r.class,
                r.origin,
                r.evidence,
                r.decided_at
            ],
        )?;
    }
    for k in &d.keeps {
        tx.execute(
            "INSERT OR REPLACE INTO wishlist_keep(account_id, store, native_id, reason, decided_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![k.account_id, k.store, k.native_id, k.reason, k.decided_at],
        )?;
    }
    for x in &d.dismissed {
        tx.execute(
            "INSERT OR REPLACE INTO suggestion_dismissed(store, native_id, target, dismissed_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![x.store, x.native_id, x.target, x.dismissed_at],
        )?;
    }
    for s in &d.snapshots {
        tx.execute(
            "INSERT OR REPLACE INTO playtime_snapshot(account_id, store, native_id, minutes, observed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![s.account_id, s.store, s.native_id, s.minutes, s.observed_at],
        )?;
    }
    tx.commit()?;
    Ok(summary(d))
}

/// Copies the user file into `backups` (`crossshelf-before-import-<now>.db`), then imports.
/// Returns the summary and the copy's path.
pub fn import_with_backup(
    c: &mut Connection,
    d: &UserData,
    backups: &Path,
    now: i64,
) -> Result<(UserDataSummary, PathBuf), UserDataError> {
    std::fs::create_dir_all(backups)?;
    let copy = backups.join(format!("crossshelf-before-import-{now:012}.db"));
    c.execute("VACUUM main INTO ?1", [copy.to_string_lossy().as_ref()])?;
    Ok((import(c, d)?, copy))
}

#[cfg(test)]
mod tests;
