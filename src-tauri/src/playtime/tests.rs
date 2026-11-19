use super::*;

const ME: &str = "76561198000000001";

fn db() -> Connection {
    let dir = tempfile::tempdir().unwrap();
    let conn = crate::db::open(&dir.path().join("t.db")).unwrap();
    std::mem::forget(dir);
    conn
}

fn row(id: &str, minutes: Option<i64>, last: Option<i64>) -> PlaytimeRow {
    PlaytimeRow {
        native_id: id.into(),
        minutes,
        last_played: last,
    }
}

fn snapshots(c: &Connection, id: &str) -> Vec<(i64, i64)> {
    c.prepare(
        "SELECT minutes, observed_at FROM playtime_snapshot WHERE native_id = ?1 ORDER BY observed_at",
    )
    .unwrap()
    .query_map([id], |r| Ok((r.get(0)?, r.get(1)?)))
    .unwrap()
    .map(Result::unwrap)
    .collect()
}

fn cached(c: &Connection, id: &str) -> Option<(Option<i64>, Option<i64>)> {
    c.query_row(
        "SELECT minutes, last_played FROM playtime WHERE native_id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .ok()
}

#[test]
fn a_first_read_keeps_the_time_and_one_snapshot_per_known_value() {
    let mut c = db();
    record(
        &mut c,
        "steam",
        ME,
        &[row("1", Some(120), Some(90)), row("2", None, None)],
        true,
        100,
    )
    .unwrap();
    assert_eq!(cached(&c, "1"), Some((Some(120), Some(90))));
    assert_eq!(
        cached(&c, "2"),
        Some((None, None)),
        "unknown is stored as such"
    );
    assert_eq!(snapshots(&c, "1"), [(120, 100)]);
    assert!(
        snapshots(&c, "2").is_empty(),
        "no snapshot of an unknown time"
    );
}

#[test]
fn a_snapshot_is_added_only_when_the_minutes_change() {
    let mut c = db();
    for (minutes, now) in [(120, 100), (120, 200), (180, 300), (180, 400)] {
        record(
            &mut c,
            "steam",
            ME,
            &[row("1", Some(minutes), None)],
            true,
            now,
        )
        .unwrap();
    }
    assert_eq!(snapshots(&c, "1"), [(120, 100), (180, 300)]);
    let observed: i64 = c
        .query_row("SELECT observed_at FROM playtime", [], |r| r.get(0))
        .unwrap();
    assert_eq!(observed, 400, "the cache row says when it was last read");
}

#[test]
fn a_complete_read_drops_games_that_left_and_a_partial_one_does_not() {
    let mut c = db();
    record(
        &mut c,
        "steam",
        ME,
        &[row("1", Some(1), None), row("2", Some(2), None)],
        true,
        100,
    )
    .unwrap();
    record(&mut c, "steam", ME, &[row("1", Some(1), None)], true, 200).unwrap();
    assert_eq!(cached(&c, "2"), None);
    record(&mut c, "gog", "g", &[row("7", Some(5), None)], false, 300).unwrap();
    record(&mut c, "gog", "g", &[row("8", Some(6), None)], false, 400).unwrap();
    assert!(cached(&c, "7").is_some() && cached(&c, "8").is_some());
    assert!(cached(&c, "1").is_some(), "stores and accounts never mix");
    assert_eq!(snapshots(&c, "2"), [(2, 100)], "history stays");
}

#[test]
fn snapshots_survive_clearing_the_cache() {
    let mut c = db();
    record(&mut c, "steam", ME, &[row("1", Some(60), None)], true, 100).unwrap();
    crate::db::clear_cache(&c).unwrap();
    assert_eq!(cached(&c, "1"), None);
    assert_eq!(snapshots(&c, "1"), [(60, 100)]);
    // After a resync with the same minutes there is nothing new to remember.
    record(&mut c, "steam", ME, &[row("1", Some(60), None)], true, 200).unwrap();
    assert_eq!(snapshots(&c, "1"), [(60, 100)]);
}

#[test]
fn a_recent_read_is_reused_for_an_hour() {
    let mut c = db();
    record(&mut c, "gog", "g", &[row("7", Some(90), None)], false, 1000).unwrap();
    assert_eq!(
        recent(&c, "gog", "g", "7", 1000 + 3599, 3600).unwrap(),
        Some(Some(90))
    );
    assert_eq!(
        recent(&c, "gog", "g", "7", 1000 + 3600, 3600).unwrap(),
        None,
        "stale"
    );
    assert_eq!(
        recent(&c, "gog", "g", "8", 1000, 3600).unwrap(),
        None,
        "never read"
    );
    record(&mut c, "gog", "g", &[row("9", None, None)], false, 1000).unwrap();
    assert_eq!(
        recent(&c, "gog", "g", "9", 1001, 3600).unwrap(),
        Some(None),
        "a recent 'GOG does not give it' is reused too"
    );
}

#[test]
fn the_sheet_gets_each_product_time_from_the_active_accounts_only() {
    let mut c = db();
    crate::connections::record_connection(&c, "steam", ME, None).unwrap();
    record(
        &mut c,
        "steam",
        ME,
        &[row("1", Some(754), Some(900))],
        true,
        1000,
    )
    .unwrap();
    record(
        &mut c,
        "steam",
        "old",
        &[row("2", Some(5), None)],
        true,
        1000,
    )
    .unwrap();
    let got = of_products(&c, &[("steam", "1"), ("steam", "2"), ("epic", "x")]).unwrap();
    assert_eq!(
        got,
        [
            ProductTime {
                store: "steam".into(),
                native_id: "1".into(),
                minutes: Some(754),
                last_played: Some(900)
            },
            ProductTime {
                store: "steam".into(),
                native_id: "2".into(),
                minutes: None,
                last_played: None
            },
            ProductTime {
                store: "epic".into(),
                native_id: "x".into(),
                minutes: None,
                last_played: None
            },
        ]
    );
}
