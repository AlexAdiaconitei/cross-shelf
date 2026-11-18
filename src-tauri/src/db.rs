//! SQLite persistence: WAL, versioned migrations, backup before each migration (spec 4.4).

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

/// Ordered migrations. Index + 1 is the schema version. Never edit an applied one.
const MIGRATIONS: &[&str] = &[
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
    // v5: portrait cover on the store CDN, when the store gives one.
    "ALTER TABLE product ADD COLUMN image_url TEXT;",
];

#[cfg(test)]
pub const SCHEMA_VERSION: u32 = MIGRATIONS.len() as u32;

pub fn open(path: &Path) -> Result<Connection, DbError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&mut conn, Some(path))?;
    Ok(conn)
}

fn user_version(conn: &Connection) -> Result<u32, DbError> {
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}

fn backup_path(path: &Path, from: u32) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".bak-v{from}"));
    path.with_file_name(name)
}

pub fn migrate(conn: &mut Connection, path: Option<&Path>) -> Result<(), DbError> {
    migrate_with(conn, path, MIGRATIONS)
}

fn migrate_with(
    conn: &mut Connection,
    path: Option<&Path>,
    migrations: &[&str],
) -> Result<(), DbError> {
    let target = migrations.len() as u32;
    let current = user_version(conn)?;
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
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        std::fs::copy(p, backup_path(p, current))?;
    }
    for (i, sql) in migrations.iter().enumerate().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", i as u32 + 1)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_db_reaches_latest_version() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open(&dir.path().join("t.db")).unwrap();
        assert_eq!(user_version(&conn).unwrap(), SCHEMA_VERSION);
        let wal: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(wal, "wal");
    }

    #[test]
    fn reopening_is_idempotent_and_makes_no_backup() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.db");
        drop(open(&p).unwrap());
        drop(open(&p).unwrap());
        assert!(!backup_path(&p, SCHEMA_VERSION).exists());
    }

    #[test]
    fn older_db_is_backed_up_before_migrating() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.db");
        let mut conn = Connection::open(&p).unwrap();
        migrate_with(&mut conn, Some(&p), &["CREATE TABLE a(x);"]).unwrap();
        assert!(!backup_path(&p, 1).exists(), "no backup for a brand-new db");
        conn.execute("INSERT INTO a VALUES (42)", []).unwrap();
        migrate_with(
            &mut conn,
            Some(&p),
            &["CREATE TABLE a(x);", "CREATE TABLE b(y);"],
        )
        .unwrap();
        let bak = backup_path(&p, 1);
        assert!(bak.exists());
        let old = Connection::open(&bak).unwrap();
        assert_eq!(user_version(&old).unwrap(), 1);
        let v: i64 = old.query_row("SELECT x FROM a", [], |r| r.get(0)).unwrap();
        assert_eq!(v, 42);
        assert_eq!(user_version(&conn).unwrap(), 2);
    }

    #[test]
    fn newer_db_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.db");
        {
            let conn = Connection::open(&p).unwrap();
            conn.pragma_update(None, "user_version", SCHEMA_VERSION + 5)
                .unwrap();
        }
        assert!(matches!(open(&p), Err(DbError::TooNew { .. })));
    }

    #[test]
    fn ids_above_js_safe_integer_roundtrip_as_text() {
        let dir = tempfile::tempdir().unwrap();
        let conn = open(&dir.path().join("t.db")).unwrap();
        let id = "76561198000000000"; // > Number.MAX_SAFE_INTEGER
        conn.execute(
            "INSERT INTO connection(provider, account_id) VALUES ('steam', ?1)",
            [id],
        )
        .unwrap();
        let back: String = conn
            .query_row("SELECT account_id FROM connection", [], |r| r.get(0))
            .unwrap();
        assert_eq!(back, id);
    }
}
