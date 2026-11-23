use super::*;
use crate::playtime::{PlaytimeRow, record};
use crate::sync::{EntitlementRow, ProductRow, ReadOutcome, Resource, commit_entitlements};

const ME: &str = "76561198000000001";
const DAY: i64 = 86_400;
const NOW: i64 = 1_000 * DAY;

fn db() -> Connection {
    let dir = tempfile::tempdir().unwrap();
    let mut conn = crate::db::open(&dir.path().join("t.db")).unwrap();
    std::mem::forget(dir);
    crate::connections::record_connection(&conn, "steam", ME, None).unwrap();
    let ids = ["1", "2", "3", "4"];
    commit_entitlements(
        &mut conn,
        "steam",
        ME,
        Resource::Library,
        &ids.iter()
            .map(|i| ProductRow {
                native_id: (*i).into(),
                title: Some(format!("Game {i}")),
                kind: "game".into(),
                windows: "yes".into(),
                url: None,
                image: None,
            })
            .collect::<Vec<_>>(),
        &ids.iter()
            .map(|i| EntitlementRow {
                native_id: (*i).into(),
                class: "owned".into(),
                owners: vec![],
                exclude_reason: None,
            })
            .collect::<Vec<_>>(),
        &ReadOutcome::Complete,
        true,
        1,
    )
    .unwrap();
    conn
}

fn read(c: &mut Connection, id: &str, minutes: i64, last: Option<i64>, at: i64) {
    let row = PlaytimeRow {
        native_id: id.into(),
        minutes: Some(minutes),
        last_played: last,
    };
    record(c, "steam", ME, &[row], false, at).unwrap();
}

fn ids(rows: &[DiaryRow]) -> Vec<&str> {
    rows.iter().map(|r| r.product.native_id.as_str()).collect()
}

#[test]
fn lists_games_with_a_play_or_hours_by_last_activity() {
    let mut c = db();
    read(&mut c, "1", 120, Some(NOW - 10 * DAY), NOW);
    read(&mut c, "2", 300, Some(NOW - DAY), NOW);
    read(&mut c, "3", 0, None, NOW); // never played: not in the diary
    crate::plays::set(&mut c, "steam", "4", "backlog", None, NOW).unwrap();
    let rows = diary(&c, NOW).unwrap();
    assert_eq!(ids(&rows), ["2", "1", "4"]);
    assert_eq!(rows[0].minutes, Some(300));
    assert_eq!(
        rows[2].minutes, None,
        "a play without hours: unknown, not zero"
    );
    assert_eq!(rows[2].state.as_deref(), Some("backlog"));
}

#[test]
fn the_week_delta_compares_with_the_hours_a_week_ago() {
    let mut c = db();
    read(&mut c, "1", 100, None, NOW - 20 * DAY);
    read(&mut c, "1", 160, None, NOW - 3 * DAY);
    read(&mut c, "2", 50, None, NOW - DAY); // first read this week: nothing to compare with
    let rows = diary(&c, NOW).unwrap();
    let by = |id: &str| rows.iter().find(|r| r.product.native_id == id).unwrap();
    assert_eq!(by("1").week_minutes, Some(60));
    assert_eq!(by("2").week_minutes, None);
}

#[test]
fn a_game_without_a_play_gets_the_deduced_state_unless_dismissed() {
    let mut c = db();
    read(&mut c, "1", 120, Some(NOW - DAY), NOW);
    read(&mut c, "2", 120, Some(NOW - DAY), NOW);
    crate::plays::set(&mut c, "steam", "2", "completed", None, NOW).unwrap();
    read(&mut c, "3", 120, Some(NOW - 2 * DAY), NOW);
    crate::plays::suggestions::dismiss(&c, "steam", "3", "playing", NOW).unwrap();
    let rows = diary(&c, NOW).unwrap();
    let by = |id: &str| rows.iter().find(|r| r.product.native_id == id).unwrap();
    assert_eq!(by("1").deduced.as_deref(), Some("playing"));
    assert_eq!(by("2").deduced, None, "a play is never second-guessed here");
    assert_eq!(by("3").deduced, None);
}
