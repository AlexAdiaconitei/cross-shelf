//! SQLite persistence (spec 4.4, ADR 0004): two files on one connection.
//!
//! - `crossshelf.db` (schema `main`) holds what only the user has: connections, settings,
//!   decisions, cleanup batches. Copied before each migration; the app does not start without it.
//! - `crossshelf-cache.db` (schema `cache`) holds what the stores give back on the next sync. If
//!   it is broken or newer than this app, it is set aside and rebuilt empty.
//!
//! The cache file is attached as `cache`, so queries name tables without a schema and SQLite finds
//! each one in its file. Only the `CREATE` statements of the cache name the schema.

use rusqlite::Connection;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("database schema v{found} is newer than this app supports (v{supported})")]
    TooNew { found: u32, supported: u32 },
}

/// Schema name of the attached cache file.
pub const CACHE: &str = "cache";

/// Tables of the user file. A new table must be added here or to [`CACHE_TABLES`]: the
/// classification test fails otherwise.
#[cfg(test)]
pub const USER_TABLES: &[&str] = &[
    "connection",
    "setting",
    "product_relation",
    "wishlist_keep",
    "action_batch",
    "batch_op",
    "play",
    "playtime_snapshot",
    "suggestion_dismissed",
];

/// Tables of the cache file: everything a sync can read again.
#[cfg(test)]
pub const CACHE_TABLES: &[&str] = &[
    "sync_generation",
    "product",
    "entitlement",
    "wishlist_entry",
    "steam_family_member",
    "playtime",
];

/// Ordered migrations of the user file. Index + 1 is its `user_version`. Never edit an applied one.
/// v1 to v7 date from when everything lived in one file; v8 moves the synced tables to the cache.
const USER_MIGRATIONS: &[&str] = &[
    // v1: base tables for the entities of spec 4.5 that phase 0 already needs.
    // IDs of 64 bits are TEXT everywhere.
    "CREATE TABLE connection (
        provider TEXT PRIMARY KEY,
        account_id TEXT,
        display_name TEXT,
        avatar_url TEXT,
        state TEXT NOT NULL DEFAULT 'disconnected',
        secret_ref TEXT,
        last_sync_at TEXT
     );
     CREATE TABLE sync_generation (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        provider TEXT NOT NULL,
        resource TEXT NOT NULL,
        account_id TEXT,
        state TEXT NOT NULL CHECK (state IN ('complete','partial','failed')),
        pages_done INTEGER NOT NULL DEFAULT 0,
        pages_failed INTEGER NOT NULL DEFAULT 0,
        item_count INTEGER NOT NULL DEFAULT 0,
        started_at TEXT NOT NULL,
        finished_at TEXT
     );",
    // v2: synchronised data (spec 4.5). Every row carries its account so accounts never mix.
    "CREATE TABLE product (
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        title TEXT,
        kind TEXT NOT NULL DEFAULT 'unknown',
        windows TEXT NOT NULL DEFAULT 'unknown',
        url TEXT,
        updated_at INTEGER NOT NULL,
        PRIMARY KEY (store, native_id)
     );
     CREATE TABLE entitlement (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        account_id TEXT NOT NULL,
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        class TEXT NOT NULL CHECK (class IN ('owned','family_shared','temporary','unknown')),
        owners TEXT NOT NULL DEFAULT '[]',
        exclude_reason TEXT,
        observed_at INTEGER NOT NULL,
        generation_id INTEGER NOT NULL REFERENCES sync_generation(id),
        UNIQUE (account_id, store, native_id, class)
     );
     CREATE INDEX entitlement_account ON entitlement(account_id, store);
     CREATE TABLE wishlist_entry (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        account_id TEXT NOT NULL,
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        priority INTEGER,
        added_at INTEGER,
        confirmed INTEGER NOT NULL DEFAULT 1,
        observed_at INTEGER NOT NULL,
        generation_id INTEGER NOT NULL REFERENCES sync_generation(id),
        UNIQUE (account_id, store, native_id)
     );
     CREATE INDEX wishlist_account ON wishlist_entry(account_id, store);",
    // v3: why a generation failed (plain code, never provider text) and app settings.
    // `connection.account_id` is the active account of a store; it survives disconnecting so the
    // last copy stays readable, and rows of any other account stay separated (spec 5.5).
    "ALTER TABLE sync_generation ADD COLUMN error_code TEXT;
     CREATE TABLE setting (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
     );",
    // v4: what the user and the evidence say about pairs of products, and "Mantener" (spec 4.5,
    // 7.3). Sync never touches these tables: decisions survive every resync. A rejection is a row.
    "CREATE TABLE product_relation (
        wanted_store TEXT NOT NULL,
        wanted_id TEXT NOT NULL,
        held_store TEXT NOT NULL,
        held_id TEXT NOT NULL,
        class TEXT NOT NULL CHECK (class IN
          ('same','edition_of','contains','remaster_of','sequel_of','dlc_of','different')),
        origin TEXT NOT NULL CHECK (origin IN ('store_id','external_map','manual','heuristic')),
        evidence TEXT,
        decided_at INTEGER NOT NULL,
        PRIMARY KEY (wanted_store, wanted_id, held_store, held_id, origin)
     );
     CREATE TABLE wishlist_keep (
        account_id TEXT NOT NULL,
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        reason TEXT,
        decided_at INTEGER NOT NULL,
        PRIMARY KEY (account_id, store, native_id)
     );",
    // v5: cleanup batches (spec 4.5, 8.2). A batch belongs to one account and one store and is
    // bound to the wishlist generation it was reviewed on. The snapshot holds no secrets.
    "CREATE TABLE action_batch (
        id TEXT PRIMARY KEY,
        store TEXT NOT NULL,
        account_id TEXT NOT NULL,
        state TEXT NOT NULL CHECK (state IN ('draft','confirmed','done','invalidated')),
        revision TEXT NOT NULL,
        generation_id INTEGER NOT NULL,
        snapshot TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        confirmed_at INTEGER
     );
     CREATE TABLE batch_op (
        op_id TEXT PRIMARY KEY,
        batch_id TEXT NOT NULL REFERENCES action_batch(id),
        native_id TEXT NOT NULL,
        desired TEXT NOT NULL CHECK (desired IN ('present','absent')),
        status TEXT NOT NULL,
        finished_at INTEGER
     );
     CREATE INDEX batch_op_batch ON batch_op(batch_id);",
    // v6: portrait cover on the store CDN, when the store gives one.
    "ALTER TABLE product ADD COLUMN image_url TEXT;",
    // v7: public names of the Steam family members who share games, to say who shares what.
    "CREATE TABLE steam_family_member (
        steamid TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        updated_at INTEGER NOT NULL
     );",
    // v8: the synced tables move to the cache file (ADR 0004). Rows are copied as they are, ids
    // included, so nobody has to sync again and batches keep pointing at their generation. The
    // copy first empties the cache tables: across two WAL files a crash can leave the cache
    // committed and this migration not, and running it again must not duplicate rows.
    "PRAGMA defer_foreign_keys = ON;
     DELETE FROM cache.entitlement;
     DELETE FROM cache.wishlist_entry;
     DELETE FROM cache.sync_generation;
     DELETE FROM cache.product;
     DELETE FROM cache.steam_family_member;
     INSERT INTO cache.sync_generation(id, provider, resource, account_id, state, pages_done,
            pages_failed, item_count, started_at, finished_at, error_code)
       SELECT id, provider, resource, account_id, state, pages_done, pages_failed, item_count,
              started_at, finished_at, error_code FROM main.sync_generation;
     INSERT INTO cache.product(store, native_id, title, kind, windows, url, image_url, updated_at)
       SELECT store, native_id, title, kind, windows, url, image_url, updated_at FROM main.product;
     INSERT INTO cache.entitlement(id, account_id, store, native_id, class, owners,
            exclude_reason, observed_at, generation_id)
       SELECT id, account_id, store, native_id, class, owners, exclude_reason, observed_at,
              generation_id FROM main.entitlement;
     INSERT INTO cache.wishlist_entry(id, account_id, store, native_id, priority, added_at,
            confirmed, observed_at, generation_id)
       SELECT id, account_id, store, native_id, priority, added_at, confirmed, observed_at,
              generation_id FROM main.wishlist_entry;
     INSERT INTO cache.steam_family_member(steamid, name, updated_at)
       SELECT steamid, name, updated_at FROM main.steam_family_member;
     DROP TABLE main.entitlement;
     DROP TABLE main.wishlist_entry;
     DROP TABLE main.sync_generation;
     DROP TABLE main.product;
     DROP TABLE main.steam_family_member;",
    // v9: what the user is playing (Mis partidas), play time history and dismissed suggestions.
    // Keyed by product, with no foreign key to the cache: they survive clearing it.
    "CREATE TABLE play (
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        state TEXT NOT NULL CHECK (state IN ('playing','backlog','paused','completed','dropped')),
        position REAL NOT NULL,
        since INTEGER NOT NULL,
        started_at INTEGER,
        finished_at INTEGER,
        note TEXT NOT NULL DEFAULT '',
        updated_at INTEGER NOT NULL,
        PRIMARY KEY (store, native_id)
     ) STRICT;
     CREATE INDEX play_state ON play(state, position);
     CREATE TABLE playtime_snapshot (
        account_id TEXT NOT NULL,
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        minutes INTEGER NOT NULL,
        observed_at INTEGER NOT NULL,
        PRIMARY KEY (account_id, store, native_id, observed_at)
     ) STRICT, WITHOUT ROWID;
     CREATE TABLE suggestion_dismissed (
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        target TEXT NOT NULL,
        dismissed_at INTEGER NOT NULL,
        PRIMARY KEY (store, native_id, target)
     ) STRICT, WITHOUT ROWID;",
];

/// Ordered migrations of the cache file, numbered on their own from 1.
const CACHE_MIGRATIONS: &[&str] = &[
    // v1: the synced tables as they were in user v7, plus play time.
    "CREATE TABLE cache.sync_generation (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        provider TEXT NOT NULL,
        resource TEXT NOT NULL,
        account_id TEXT,
        state TEXT NOT NULL CHECK (state IN ('complete','partial','failed')),
        pages_done INTEGER NOT NULL DEFAULT 0,
        pages_failed INTEGER NOT NULL DEFAULT 0,
        item_count INTEGER NOT NULL DEFAULT 0,
        started_at TEXT NOT NULL,
        finished_at TEXT,
        error_code TEXT
     );
     CREATE TABLE cache.product (
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        title TEXT,
        kind TEXT NOT NULL DEFAULT 'unknown',
        windows TEXT NOT NULL DEFAULT 'unknown',
        url TEXT,
        updated_at INTEGER NOT NULL,
        image_url TEXT,
        PRIMARY KEY (store, native_id)
     );
     CREATE TABLE cache.entitlement (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        account_id TEXT NOT NULL,
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        class TEXT NOT NULL CHECK (class IN ('owned','family_shared','temporary','unknown')),
        owners TEXT NOT NULL DEFAULT '[]',
        exclude_reason TEXT,
        observed_at INTEGER NOT NULL,
        generation_id INTEGER NOT NULL REFERENCES sync_generation(id),
        UNIQUE (account_id, store, native_id, class)
     );
     CREATE INDEX cache.entitlement_account ON entitlement(account_id, store);
     CREATE TABLE cache.wishlist_entry (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        account_id TEXT NOT NULL,
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        priority INTEGER,
        added_at INTEGER,
        confirmed INTEGER NOT NULL DEFAULT 1,
        observed_at INTEGER NOT NULL,
        generation_id INTEGER NOT NULL REFERENCES sync_generation(id),
        UNIQUE (account_id, store, native_id)
     );
     CREATE INDEX cache.wishlist_account ON wishlist_entry(account_id, store);
     CREATE TABLE cache.steam_family_member (
        steamid TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        updated_at INTEGER NOT NULL
     );
     CREATE TABLE cache.playtime (
        account_id TEXT NOT NULL,
        store TEXT NOT NULL,
        native_id TEXT NOT NULL,
        minutes INTEGER,
        last_played INTEGER,
        observed_at INTEGER NOT NULL,
        PRIMARY KEY (account_id, store, native_id)
     ) STRICT, WITHOUT ROWID;",
];

/// User schema version this build writes.
#[cfg(test)]
pub const SCHEMA_VERSION: u32 = USER_MIGRATIONS.len() as u32;
#[cfg(test)]
pub const CACHE_VERSION: u32 = CACHE_MIGRATIONS.len() as u32;

/// The user version from which the synced tables live in the cache file.
const SPLIT_VERSION: u32 = 8;

/// `crossshelf.db` → `crossshelf-cache.db`, next to it.
pub fn cache_path(user: &Path) -> PathBuf {
    let stem = user.file_stem().unwrap_or_default().to_string_lossy();
    let ext = user
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    user.with_file_name(format!("{stem}-cache{ext}"))
}

/// Opens the user file, attaches the cache file and brings both up to date.
///
/// # Errors
///
/// Fails when the user file cannot be read or migrated, or is newer than this build. A cache file
/// in that state is rebuilt instead.
pub fn open(path: &Path) -> Result<Connection, DbError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    // User writes are rare and irreplaceable; the cache trades that for speed on big syncs.
    conn.pragma_update(Some("main"), "synchronous", "FULL")?;
    let rebuilt = attach_cache(&mut conn, &cache_path(path))?;
    let from = version(&conn, "main")?;
    migrate(&mut conn, Some(path))?;
    if rebuilt {
        // The stores read as never synced, so the views ask for a sync instead of showing an
        // empty library as if it were up to date.
        conn.execute("UPDATE main.connection SET last_sync_at = NULL", [])?;
    }
    if from > 0 && from < SPLIT_VERSION {
        // Give back the space the synced tables took in the user file, once.
        conn.execute_batch("VACUUM main;")?;
    }
    Ok(conn)
}

/// Attaches the cache file. Returns true when it had to be set aside and rebuilt empty.
fn attach_cache(conn: &mut Connection, path: &Path) -> Result<bool, DbError> {
    match try_attach(conn, path) {
        Ok(()) => Ok(false),
        Err(e) => {
            log::warn!("cache database unusable, rebuilding it: {e}");
            let _ = conn.execute_batch("DETACH DATABASE cache;");
            set_aside(path)?;
            try_attach(conn, path).map(|()| true)
        }
    }
}

fn try_attach(conn: &mut Connection, path: &Path) -> Result<(), DbError> {
    conn.execute(
        "ATTACH DATABASE ?1 AS cache",
        [path.to_string_lossy().as_ref()],
    )?;
    let cache = CACHE;
    if version(conn, CACHE)? == 0 {
        // Only possible before the first table exists.
        conn.pragma_update(Some(cache), "auto_vacuum", "INCREMENTAL")?;
    }
    conn.pragma_update(Some(cache), "journal_mode", "WAL")?;
    conn.pragma_update(Some(cache), "synchronous", "NORMAL")?;
    migrate_with(conn, CACHE, None, CACHE_MIGRATIONS)
}

/// Moves a broken or too new cache file (and its WAL) to `<name>.broken`, replacing an older one.
fn set_aside(path: &Path) -> Result<(), DbError> {
    for suffix in ["", "-wal", "-shm"] {
        let mut from = path.as_os_str().to_owned();
        from.push(suffix);
        let from = PathBuf::from(from);
        if !from.exists() {
            continue;
        }
        let mut to = from.as_os_str().to_owned();
        to.push(".broken");
        let to = PathBuf::from(to);
        if to.exists() {
            std::fs::remove_file(&to)?;
        }
        std::fs::rename(&from, &to)?;
    }
    Ok(())
}

fn version(conn: &Connection, schema: &str) -> Result<u32, DbError> {
    Ok(conn.query_row(&format!("PRAGMA {schema}.user_version"), [], |r| r.get(0))?)
}

fn backup_path(path: &Path, from: u32) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".bak-v{from}"));
    path.with_file_name(name)
}

/// Brings the user schema up to date. The cache must be attached first (v8 writes into it).
pub fn migrate(conn: &mut Connection, path: Option<&Path>) -> Result<(), DbError> {
    migrate_with(conn, "main", path, USER_MIGRATIONS)
}

fn migrate_with(
    conn: &mut Connection,
    schema: &str,
    path: Option<&Path>,
    migrations: &[&str],
) -> Result<(), DbError> {
    let target = migrations.len() as u32;
    let current = version(conn, schema)?;
    if current > target {
        return Err(DbError::TooNew {
            found: current,
            supported: target,
        });
    }
    if current == target {
        return Ok(());
    }
    // Copy before touching an existing database (version 0 is a brand-new file).
    if let (true, Some(p)) = (current > 0, path) {
        conn.execute_batch(&format!("PRAGMA {schema}.wal_checkpoint(TRUNCATE);"))?;
        std::fs::copy(p, backup_path(p, current))?;
    }
    for (i, sql) in migrations.iter().enumerate().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(Some(schema), "user_version", i as u32 + 1)?;
        tx.commit()?;
    }
    Ok(())
}

/// Empties every table of the cache file. The user file is not touched, except that connections
/// forget their last sync date, which described the emptied data.
pub fn clear_cache(conn: &Connection) -> Result<(), rusqlite::Error> {
    let tables: Vec<String> = conn
        .prepare(
            "SELECT name FROM cache.sqlite_master
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    // `sqlite_sequence` stays: batches keep the generation ids they were reviewed on, and a new
    // generation must never reuse one.
    let mut sql = String::from("BEGIN; PRAGMA defer_foreign_keys = ON;");
    for t in &tables {
        sql.push_str(&format!(
            "DELETE FROM cache.\"{}\";",
            t.replace('"', "\"\"")
        ));
    }
    sql.push_str("UPDATE main.connection SET last_sync_at = NULL; COMMIT;");
    conn.execute_batch(&sql)?;
    conn.execute_batch("PRAGMA cache.incremental_vacuum;")
}

/// Daily copies of the user file kept in the backups folder.
pub const BACKUPS_KEPT: usize = 7;
const BACKUP_PREFIX: &str = "crossshelf-";
const BACKUP_EVERY: i64 = 86_400;

/// Seconds in a backup's name, if it is one of ours.
fn backup_time(name: &str) -> Option<i64> {
    name.strip_prefix(BACKUP_PREFIX)?
        .strip_suffix(".db")?
        .parse()
        .ok()
}

/// Copies the user file into `dir` with `VACUUM INTO` when the newest copy there is a day old or
/// more, then drops the oldest beyond [`BACKUPS_KEPT`]. Returns the new copy, if one was made.
/// The cache is not copied: a sync rebuilds it.
///
/// # Errors
///
/// Fails when the folder cannot be read or written, or SQLite cannot write the copy.
pub fn daily_backup(conn: &Connection, dir: &Path, now: i64) -> Result<Option<PathBuf>, DbError> {
    std::fs::create_dir_all(dir)?;
    let mut ours: Vec<(i64, PathBuf)> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|e| Some((backup_time(&e.file_name().to_string_lossy())?, e.path())))
        .collect();
    ours.sort();
    if ours
        .last()
        .is_some_and(|(at, _)| now.saturating_sub(*at) < BACKUP_EVERY)
    {
        return Ok(None);
    }
    let to = dir.join(format!("{BACKUP_PREFIX}{now:012}.db"));
    conn.execute("VACUUM main INTO ?1", [to.to_string_lossy().as_ref()])?;
    ours.push((now, to.clone()));
    let extra = ours.len().saturating_sub(BACKUPS_KEPT);
    for (_, old) in ours.drain(..extra) {
        std::fs::remove_file(old)?;
    }
    Ok(Some(to))
}

/// Lets SQLite refresh its statistics. Cheap; run after a complete sync and when the app closes.
pub fn optimize(conn: &Connection) {
    if let Err(e) = conn.execute_batch("PRAGMA optimize;") {
        log::warn!("optimize: {e}");
    }
}

#[cfg(test)]
mod tests;
