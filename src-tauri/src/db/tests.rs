use super::*;
use rusqlite::types::Value;

fn tables(conn: &Connection, schema: &str) -> Vec<String> {
    let mut names: Vec<String> = conn
        .prepare(&format!(
            "SELECT name FROM {schema}.sqlite_master
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        ))
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    names.sort();
    names
}

fn sorted(list: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = list.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

fn pragma_i64(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

/// Every row of every user table, as text, to compare the user file before and after.
fn dump_user(conn: &Connection) -> Vec<String> {
    let mut out = Vec::new();
    let present = tables(conn, "main");
    for t in USER_TABLES
        .iter()
        .filter(|t| present.iter().any(|p| p == *t))
    {
        let mut st = conn
            .prepare(&format!("SELECT * FROM main.{t} ORDER BY 1"))
            .unwrap();
        let n = st.column_count();
        let rows = st
            .query_map([], |r| {
                (0..n)
                    .map(|i| r.get::<_, Value>(i))
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap();
        for row in rows {
            out.push(format!("{t}: {:?}", row.unwrap()));
        }
    }
    out
}

/// A user file as v7 left it, everything in one file, with a little of every table.
fn v7_file(path: &Path) {
    let mut conn = Connection::open(path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();
    migrate_with(&mut conn, "main", None, &USER_MIGRATIONS[..7]).unwrap();
    conn.execute_batch(
        "INSERT INTO connection(provider, account_id, state, secret_ref, last_sync_at)
           VALUES ('steam', '76561198000000001', 'connected', 'steam', '100');
         INSERT INTO setting VALUES ('language', 'es');
         INSERT INTO sync_generation(id, provider, resource, account_id, state, started_at,
                finished_at, error_code)
           VALUES (41, 'steam', 'library', '76561198000000001', 'complete', '100', '100', NULL),
                  (42, 'steam', 'wishlist', '76561198000000001', 'failed', '101', '101', 'network');
         INSERT INTO product(store, native_id, title, updated_at, image_url)
           VALUES ('steam', '10', 'Alpha', 100, 'https://cdn/a.jpg');
         INSERT INTO entitlement(id, account_id, store, native_id, class, observed_at, generation_id)
           VALUES (7, '76561198000000001', 'steam', '10', 'owned', 100, 41);
         INSERT INTO wishlist_entry(id, account_id, store, native_id, priority, observed_at,
                generation_id)
           VALUES (9, '76561198000000001', 'steam', '20', 3, 100, 41);
         INSERT INTO steam_family_member VALUES ('76561198000000002', 'Marta', 100);
         INSERT INTO product_relation VALUES ('steam','20','gog','5','same','manual',NULL,100);
         INSERT INTO wishlist_keep VALUES ('76561198000000001','steam','20',NULL,100);
         INSERT INTO action_batch VALUES ('b1','steam','76561198000000001','done','r',42,'{}',100,101);
         INSERT INTO batch_op VALUES ('o1','b1','20','absent','applied',101);",
    )
    .unwrap();
}

#[test]
fn fresh_files_reach_the_latest_versions_in_wal() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("crossshelf.db");
    let conn = open(&p).unwrap();
    assert_eq!(version(&conn, "main").unwrap(), SCHEMA_VERSION);
    assert_eq!(version(&conn, CACHE).unwrap(), CACHE_VERSION);
    assert!(dir.path().join("crossshelf-cache.db").exists());
    for schema in ["main", CACHE] {
        let mode: String = conn
            .query_row(&format!("PRAGMA {schema}.journal_mode"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode, "wal", "{schema}");
    }
}

#[test]
fn each_file_has_its_own_durability_and_vacuum_policy() {
    let dir = tempfile::tempdir().unwrap();
    let conn = open(&dir.path().join("t.db")).unwrap();
    // synchronous: 2 FULL, 1 NORMAL. auto_vacuum: 0 NONE, 2 INCREMENTAL.
    assert_eq!(pragma_i64(&conn, "PRAGMA main.synchronous"), 2);
    assert_eq!(pragma_i64(&conn, "PRAGMA cache.synchronous"), 1);
    assert_eq!(pragma_i64(&conn, "PRAGMA main.auto_vacuum"), 0);
    assert_eq!(pragma_i64(&conn, "PRAGMA cache.auto_vacuum"), 2);
    assert_eq!(pragma_i64(&conn, "PRAGMA busy_timeout"), 5000);
    assert_eq!(pragma_i64(&conn, "PRAGMA temp_store"), 2);
    assert_eq!(pragma_i64(&conn, "PRAGMA foreign_keys"), 1);
}

#[test]
fn cache_path_sits_next_to_the_user_file() {
    assert_eq!(
        cache_path(Path::new("C:/data/crossshelf.db")),
        Path::new("C:/data/crossshelf-cache.db")
    );
}

#[test]
fn every_table_belongs_to_the_file_it_is_listed_in() {
    let dir = tempfile::tempdir().unwrap();
    let conn = open(&dir.path().join("t.db")).unwrap();
    assert_eq!(tables(&conn, "main"), sorted(USER_TABLES));
    assert_eq!(tables(&conn, CACHE), sorted(CACHE_TABLES));
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
    migrate_with(&mut conn, "main", Some(&p), &["CREATE TABLE a(x);"]).unwrap();
    assert!(!backup_path(&p, 1).exists(), "no backup for a brand-new db");
    conn.execute("INSERT INTO a VALUES (42)", []).unwrap();
    migrate_with(
        &mut conn,
        "main",
        Some(&p),
        &["CREATE TABLE a(x);", "CREATE TABLE b(y);"],
    )
    .unwrap();
    let bak = backup_path(&p, 1);
    assert!(bak.exists());
    let old = Connection::open(&bak).unwrap();
    assert_eq!(version(&old, "main").unwrap(), 1);
    let v: i64 = old.query_row("SELECT x FROM a", [], |r| r.get(0)).unwrap();
    assert_eq!(v, 42);
    assert_eq!(version(&conn, "main").unwrap(), 2);
}

#[test]
fn newer_user_db_is_refused() {
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
fn a_newer_cache_is_set_aside_and_rebuilt_empty() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.db");
    let cache = cache_path(&p);
    {
        let c = Connection::open(&cache).unwrap();
        c.execute_batch("CREATE TABLE future(x); INSERT INTO future VALUES (1);")
            .unwrap();
        c.pragma_update(None, "user_version", CACHE_VERSION + 3)
            .unwrap();
    }
    let conn = open(&p).unwrap();
    assert_eq!(version(&conn, CACHE).unwrap(), CACHE_VERSION);
    assert_eq!(tables(&conn, CACHE), sorted(CACHE_TABLES));
    assert!(dir.path().join("t-cache.db.broken").exists());
}

#[test]
fn a_broken_cache_is_set_aside_and_rebuilt_empty() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.db");
    std::fs::write(cache_path(&p), b"this is not a database, just bytes...").unwrap();
    let conn = open(&p).unwrap();
    assert_eq!(version(&conn, CACHE).unwrap(), CACHE_VERSION);
    assert_eq!(count(&conn, "product"), 0);
    assert!(dir.path().join("t-cache.db.broken").exists());
}

#[test]
fn v7_moves_its_synced_tables_to_the_cache_without_a_resync() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("crossshelf.db");
    v7_file(&p);
    let before = {
        let c = Connection::open(&p).unwrap();
        c.query_row("SELECT COUNT(*) FROM product_relation", [], |r| {
            r.get::<_, i64>(0)
        })
        .unwrap()
    };
    let conn = open(&p).unwrap();

    // The copy of the user file was taken before migrating, with everything still in it.
    let bak = Connection::open(backup_path(&p, 7)).unwrap();
    assert_eq!(version(&bak, "main").unwrap(), 7);
    assert_eq!(count(&bak, "entitlement"), 1);

    assert_eq!(tables(&conn, "main"), sorted(USER_TABLES));
    assert_eq!(count(&conn, "main.product_relation"), before);
    for (t, n) in [
        ("cache.sync_generation", 2),
        ("cache.product", 1),
        ("cache.entitlement", 1),
        ("cache.wishlist_entry", 1),
        ("cache.steam_family_member", 1),
    ] {
        assert_eq!(count(&conn, t), n, "{t}");
    }
    let (gen_id, image): (i64, String) = conn
        .query_row(
            "SELECT e.generation_id, p.image_url FROM entitlement e JOIN product p USING (store, native_id)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((gen_id, image.as_str()), (41, "https://cdn/a.jpg"));
    let err: String = conn
        .query_row(
            "SELECT error_code FROM sync_generation WHERE id = 42",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(err, "network");
    // The views read the moved rows as before.
    assert_eq!(crate::queries::list_library(&conn).unwrap().len(), 1);
    assert_eq!(crate::queries::list_wishlist(&conn).unwrap().len(), 1);
}

#[test]
fn a_new_generation_never_reuses_a_moved_id() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.db");
    v7_file(&p);
    let mut conn = open(&p).unwrap();
    let c = crate::sync::commit_wishlist(
        &mut conn,
        "steam",
        "76561198000000001",
        &[],
        &[],
        &crate::sync::ReadOutcome::Failed,
        false,
        200,
    )
    .unwrap();
    assert!(c.generation_id > 42);
}

#[test]
fn clearing_the_cache_leaves_the_user_file_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.db");
    v7_file(&p);
    let conn = open(&p).unwrap();
    conn.execute(
        "INSERT INTO playtime VALUES ('76561198000000001','steam','10',120,100,100)",
        [],
    )
    .unwrap();
    conn.execute_batch(
        "UPDATE connection SET last_sync_at = NULL;
         INSERT INTO play(store, native_id, state, position, since, note, updated_at)
           VALUES ('steam', '10', 'playing', 1, 100, 'boss 3', 100);
         INSERT INTO playtime_snapshot VALUES ('76561198000000001', 'steam', '10', 120, 100);
         INSERT INTO suggestion_dismissed VALUES ('steam', '10', 'paused', 100);",
    )
    .unwrap();
    let before = dump_user(&conn);
    assert!(
        before.len() >= USER_TABLES.len(),
        "every user table has a row"
    );

    clear_cache(&conn).unwrap();

    for t in CACHE_TABLES {
        assert_eq!(count(&conn, &format!("cache.{t}")), 0, "{t}");
    }
    assert_eq!(dump_user(&conn), before);
}

#[test]
fn clearing_the_cache_forgets_the_last_sync_date() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("t.db");
    v7_file(&p);
    let conn = open(&p).unwrap();
    clear_cache(&conn).unwrap();
    let last: Option<String> = conn
        .query_row("SELECT last_sync_at FROM connection", [], |r| r.get(0))
        .unwrap();
    assert_eq!(last, None);
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

/// Tables scanned in full by a query plan.
fn scans(conn: &Connection, sql: &str) -> Vec<String> {
    let mut st = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
    let nulls = vec![Value::Null; st.parameter_count()];
    st.query_map(rusqlite::params_from_iter(nulls), |r| r.get::<_, String>(3))
        .unwrap()
        .map(Result::unwrap)
        .filter_map(|d| {
            let rest = d.strip_prefix("SCAN ")?;
            Some(rest.split_whitespace().next().unwrap_or("").to_string())
        })
        .collect()
}

#[test]
fn list_queries_only_scan_the_table_they_list() {
    let dir = tempfile::tempdir().unwrap();
    let conn = open(&dir.path().join("t.db")).unwrap();
    // (name, sql, tables it may read in full). Everything else must go through a key or index.
    // `connection` has one row per store. Aliases are what EXPLAIN reports.
    let cases: &[(&str, &str, &[&str])] = &[
        ("list_library", crate::queries::LIBRARY_SQL, &["e", "c"]),
        ("list_wishlist", crate::queries::WISHLIST_SQL, &["w", "c"]),
        ("review holdings", crate::review::HOLDINGS_SQL, &["e", "cn"]),
        ("review wishes", crate::review::WISHES_SQL, &["w", "cn"]),
        ("plays_list", crate::plays::LIST_SQL, &["y"]),
        ("diary hours", crate::plays::diary::HOURS_SQL, &["t", "c"]),
    ];
    for (name, sql, allowed) in cases {
        let s = scans(&conn, sql);
        for t in &s {
            assert!(allowed.contains(&t.as_str()), "{name} scans {t}: {s:?}");
        }
    }
}

/// Migrates a copy of a real v7 database. Run by hand:
/// `CROSSSHELF_V7_DB=<copy of crossshelf.db> cargo test real_v7 -- --ignored`.
/// The file is copied first (with its WAL), so the original is never written.
#[test]
#[ignore = "needs a real v7 database"]
fn real_v7_database_migrates_without_losing_rows() {
    let Ok(src) = std::env::var("CROSSSHELF_V7_DB") else {
        panic!("set CROSSSHELF_V7_DB");
    };
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("crossshelf.db");
    for suffix in ["", "-wal"] {
        let from = format!("{src}{suffix}");
        if Path::new(&from).exists() {
            std::fs::copy(&from, format!("{}{suffix}", p.display())).unwrap();
        }
    }
    let synced = [
        "sync_generation",
        "product",
        "entitlement",
        "wishlist_entry",
        "steam_family_member",
    ];
    let user_before;
    let counts_before: Vec<i64> = {
        let c = Connection::open(&p).unwrap();
        assert_eq!(version(&c, "main").unwrap(), 7, "not a v7 database");
        user_before = dump_user(&c);
        synced.iter().map(|t| count(&c, t)).collect()
    };
    let library_before = {
        let c = Connection::open(&p).unwrap();
        crate::queries::list_library(&c).unwrap().len()
    };
    let before_size = {
        let c = Connection::open(&p).unwrap();
        c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
        drop(c);
        std::fs::metadata(&p).unwrap().len()
    };

    let conn = open(&p).unwrap();
    assert!(backup_path(&p, 7).exists());
    assert_eq!(tables(&conn, "main"), sorted(USER_TABLES));
    let counts_after: Vec<i64> = synced
        .iter()
        .map(|t| count(&conn, &format!("cache.{t}")))
        .collect();
    assert_eq!(counts_after, counts_before);
    assert_eq!(dump_user(&conn), user_before);
    assert_eq!(
        crate::queries::list_library(&conn).unwrap().len(),
        library_before
    );
    drop(conn); // checkpoints the WAL, so the size below is the vacuumed one
    let user_size = std::fs::metadata(&p).unwrap().len();
    assert!(user_size < before_size, "the user file gave back the space");
    let bak_size = std::fs::metadata(backup_path(&p, 7)).unwrap().len();
    println!(
        "rows moved {counts_after:?}; user file {bak_size} -> {user_size} bytes; library {library_before}"
    );
}

const DAY: i64 = 86_400;

fn backups(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .map(|r| {
            r.map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[test]
fn the_first_daily_backup_is_a_readable_copy_of_the_user_file_only() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("crossshelf.db");
    v7_file(&p);
    let conn = open(&p).unwrap();
    let to = dir.path().join("backups");
    let made = daily_backup(&conn, &to, 10 * DAY).unwrap().unwrap();
    assert_eq!(backups(&to).len(), 1);
    let copy = Connection::open(&made).unwrap();
    assert_eq!(version(&copy, "main").unwrap(), SCHEMA_VERSION);
    assert_eq!(tables(&copy, "main"), sorted(USER_TABLES));
    assert_eq!(dump_user(&copy), dump_user(&conn));
}

#[test]
fn a_backup_less_than_a_day_old_is_enough() {
    let dir = tempfile::tempdir().unwrap();
    let conn = open(&dir.path().join("t.db")).unwrap();
    let to = dir.path().join("backups");
    assert!(daily_backup(&conn, &to, 10 * DAY).unwrap().is_some());
    assert!(daily_backup(&conn, &to, 11 * DAY - 1).unwrap().is_none());
    assert!(daily_backup(&conn, &to, 11 * DAY).unwrap().is_some());
    assert_eq!(backups(&to).len(), 2);
}

#[test]
fn only_the_seven_newest_backups_are_kept() {
    let dir = tempfile::tempdir().unwrap();
    let conn = open(&dir.path().join("t.db")).unwrap();
    let to = dir.path().join("backups");
    std::fs::create_dir_all(&to).unwrap();
    std::fs::write(to.join("notes.txt"), b"not ours").unwrap();
    for day in 1..=9 {
        daily_backup(&conn, &to, day * DAY).unwrap();
    }
    let left = backups(&to);
    assert_eq!(left.len(), BACKUPS_KEPT + 1, "{left:?}");
    assert!(
        left.contains(&"notes.txt".to_string()),
        "foreign files stay"
    );
    assert!(!left.iter().any(|n| n.contains(&(DAY * 2).to_string())));
    assert!(left.iter().any(|n| n.contains(&(DAY * 9).to_string())));
}
