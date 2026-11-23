use super::*;
use crate::playtime::{PlaytimeRow, record};
use crate::sync::{EntitlementRow, ProductRow, ReadOutcome, Resource, commit_entitlements};

const ME: &str = "76561198000000001";
const SECRET: &str = "SENTINEL_SECRET_REF_9f1c";
const BATCH: &str = "SENTINEL_BATCH_SNAPSHOT_77aa";

fn db() -> Connection {
    let dir = tempfile::tempdir().unwrap();
    let conn = crate::db::open(&dir.path().join("t.db")).unwrap();
    std::mem::forget(dir);
    conn
}

/// A user file with something in every table the export carries, plus secrets and batches it
/// must leave out.
fn filled() -> Connection {
    let mut c = db();
    crate::connections::record_connection(&c, "steam", ME, Some("Ana")).unwrap();
    c.execute(
        "UPDATE connection SET secret_ref = ?1 WHERE provider = 'steam'",
        [SECRET],
    )
    .unwrap();
    commit_entitlements(
        &mut c,
        "steam",
        ME,
        Resource::Library,
        &[ProductRow {
            native_id: "10".into(),
            title: Some("Celeste".into()),
            kind: "game".into(),
            windows: "yes".into(),
            url: None,
            image: None,
        }],
        &[EntitlementRow {
            native_id: "10".into(),
            class: "owned".into(),
            owners: vec![],
            exclude_reason: None,
        }],
        &ReadOutcome::Complete,
        true,
        1,
    )
    .unwrap();
    crate::plays::set(&mut c, "steam", "10", "playing", None, 100).unwrap();
    crate::plays::note(&c, "steam", "10", "chapter 7", 101).unwrap();
    crate::plays::suggestions::dismiss(&c, "gog", "5", "paused", 102).unwrap();
    let row = PlaytimeRow {
        native_id: "10".into(),
        minutes: Some(754),
        last_played: Some(90),
    };
    record(&mut c, "steam", ME, &[row], true, 103).unwrap();
    crate::review::set_manual(&c, ("steam", "20"), ("gog", "5"), Some("same"), 104).unwrap();
    crate::review::set_keep(&c, ME, "steam", "20", true, Some("gift"), 105).unwrap();
    c.execute_batch(&format!(
        "INSERT INTO setting VALUES ('language', 'es');
         INSERT INTO action_batch VALUES ('b1','steam','{ME}','done','r',1,'{BATCH}',1,2);
         INSERT INTO batch_op VALUES ('o1','b1','20','absent','applied',2);"
    ))
    .unwrap();
    c
}

#[test]
fn the_export_carries_what_the_stores_cannot_give_back_with_titles() {
    let c = filled();
    let d = export(&c, 1_800_000_000).unwrap();
    assert_eq!((d.format.as_str(), d.version), (FORMAT, VERSION));
    assert_eq!(d.plays.len(), 1);
    assert_eq!(d.plays[0].title.as_deref(), Some("Celeste"));
    assert_eq!(d.plays[0].note, "chapter 7");
    assert_eq!(d.relations.len(), 1);
    assert_eq!(d.keeps[0].reason.as_deref(), Some("gift"));
    assert_eq!(d.dismissed.len(), 1);
    assert_eq!(d.snapshots[0].minutes, 754);
    assert!(
        d.settings
            .iter()
            .any(|s| s.key == "language" && s.value == "es")
    );
}

#[test]
fn the_export_never_holds_secrets_connections_or_batches() {
    let c = filled();
    let text = serde_json::to_string(&export(&c, 1).unwrap()).unwrap();
    assert!(!text.contains(SECRET), "secret_ref leaked");
    assert!(!text.contains(BATCH), "a batch leaked");
    assert!(!text.contains("secret_ref"));
    assert!(!text.contains("\"Ana\""), "connections are not exported");
}

#[test]
fn importing_replaces_the_user_data_and_leaves_the_rest() {
    let src = filled();
    let data = export(&src, 1).unwrap();

    let mut dst = db();
    crate::plays::set(&mut dst, "gog", "999", "dropped", None, 1).unwrap();
    crate::connections::record_connection(&dst, "gog", "g", None).unwrap();
    let summary = import(&mut dst, &data).unwrap();
    assert_eq!(summary.plays, 1);

    let plays = crate::plays::list(&dst).unwrap();
    assert_eq!(plays.len(), 1, "replaced, not merged");
    assert_eq!(plays[0].product.native_id, "10");
    assert_eq!(plays[0].note, "chapter 7");
    // Titles come from the cache, which this computer has not synced yet: compare without them.
    let back = export(&dst, 1).unwrap();
    let untitled = |s: &[SnapshotRow]| -> Vec<SnapshotRow> {
        s.iter()
            .cloned()
            .map(|r| SnapshotRow { title: None, ..r })
            .collect()
    };
    assert_eq!(untitled(&back.snapshots), untitled(&data.snapshots));
    assert_eq!(back.relations, data.relations);
    assert_eq!(back.keeps, data.keeps);
    assert_eq!(back.dismissed, data.dismissed);
    assert_eq!(back.settings, data.settings);
    assert!(
        crate::connections::view(&dst, "gog")
            .unwrap()
            .account_id
            .is_some(),
        "connections stay"
    );
}

#[test]
fn other_formats_and_newer_versions_are_refused() {
    let mut c = db();
    let mut d = export(&filled(), 1).unwrap();
    d.version = VERSION + 1;
    assert!(matches!(import(&mut c, &d), Err(UserDataError::TooNew)));
    assert!(matches!(
        parse(r#"{"format":"something-else","version":1}"#),
        Err(UserDataError::NotOurs)
    ));
    assert!(matches!(parse("not json"), Err(UserDataError::NotOurs)));
    let ok = serde_json::to_string(&export(&filled(), 1).unwrap()).unwrap();
    assert_eq!(parse(&ok).unwrap().plays.len(), 1);
}

#[test]
fn a_summary_counts_what_will_come_in() {
    let d = export(&filled(), 1_800_000_000).unwrap();
    let s = summary(&d);
    assert_eq!(
        (
            s.plays,
            s.relations,
            s.keeps,
            s.dismissed,
            s.snapshots,
            s.exported_at
        ),
        (1, 1, 1, 1, 1, 1_800_000_000)
    );
}

#[test]
fn importing_copies_the_user_file_first() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::db::open(&dir.path().join("t.db")).unwrap();
    crate::plays::set(&mut c, "steam", "1", "playing", None, 1).unwrap();
    let data = export(&filled(), 1).unwrap();
    let backups = dir.path().join("backups");
    let copy = import_with_backup(&mut c, &data, &backups, 500).unwrap().1;
    let old = Connection::open(&copy).unwrap();
    let n: i64 = old
        .query_row("SELECT COUNT(*) FROM play WHERE native_id = '1'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(n, 1, "the copy holds the data from before the import");
}

#[test]
fn two_imports_in_the_same_second_both_keep_their_copy() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = crate::db::open(&dir.path().join("t.db")).unwrap();
    let data = export(&filled(), 1).unwrap();
    let backups = dir.path().join("backups");
    let a = import_with_backup(&mut c, &data, &backups, 500).unwrap().1;
    let b = import_with_backup(&mut c, &data, &backups, 500).unwrap().1;
    assert_ne!(a, b);
    assert!(a.exists() && b.exists());
}
