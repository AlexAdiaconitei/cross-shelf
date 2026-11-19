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
    for (store, acct) in [("steam", ME), ("epic", "e")] {
        crate::connections::record_connection(&conn, store, acct, None).unwrap();
        let ids = ["1", "2", "3"];
        commit_entitlements(
            &mut conn,
            store,
            acct,
            Resource::Library,
            &ids.iter()
                .map(|i| ProductRow {
                    native_id: (*i).into(),
                    title: Some(format!("{store} {i}")),
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
    }
    conn
}

/// A Steam read: minutes and last session for game `id`.
fn steam(c: &mut Connection, id: &str, minutes: i64, last: Option<i64>, now: i64) {
    let row = PlaytimeRow {
        native_id: id.into(),
        minutes: Some(minutes),
        last_played: last,
    };
    record(c, "steam", ME, &[row], false, now).unwrap();
}

/// An Epic read: minutes only.
fn epic(c: &mut Connection, id: &str, minutes: i64, now: i64) {
    let row = PlaytimeRow {
        native_id: id.into(),
        minutes: Some(minutes),
        last_played: None,
    };
    record(c, "epic", "e", &[row], false, now).unwrap();
}

fn play(c: &mut Connection, store: &str, id: &str, state: &str, at: i64) {
    crate::plays::set(c, store, id, state, None, at).unwrap();
}

fn got(c: &Connection) -> Vec<(String, String, String)> {
    let mut v: Vec<_> = suggestions(c, NOW)
        .unwrap()
        .into_iter()
        .map(|s| (s.product.store, s.product.native_id, s.target))
        .collect();
    v.sort();
    v
}

fn one(store: &str, id: &str, target: &str) -> (String, String, String) {
    (store.into(), id.into(), target.into())
}

#[test]
fn a_game_without_a_play_played_this_fortnight_is_suggested_as_playing() {
    let mut c = db();
    steam(&mut c, "1", 300, Some(NOW - 3 * DAY), NOW);
    steam(&mut c, "2", 300, Some(NOW - 20 * DAY), NOW);
    assert_eq!(got(&c), [one("steam", "1", "playing")]);
    let s = &suggestions(&c, NOW).unwrap()[0];
    assert_eq!(
        (s.reason.as_str(), s.current.as_deref()),
        ("played_recently", None)
    );
    assert_eq!(s.product.title.as_deref(), Some("steam 1"));
}

#[test]
fn a_pending_game_played_this_fortnight_is_suggested_as_playing() {
    let mut c = db();
    play(&mut c, "steam", "1", "backlog", NOW - 50 * DAY);
    steam(&mut c, "1", 300, Some(NOW - DAY), NOW);
    assert_eq!(got(&c), [one("steam", "1", "playing")]);
    assert_eq!(
        suggestions(&c, NOW).unwrap()[0].current.as_deref(),
        Some("backlog")
    );
}

#[test]
fn a_game_in_playing_untouched_for_a_month_is_suggested_as_paused() {
    let mut c = db();
    play(&mut c, "steam", "1", "playing", NOW - 60 * DAY);
    steam(&mut c, "1", 300, Some(NOW - 31 * DAY), NOW);
    play(&mut c, "steam", "2", "playing", NOW - 60 * DAY);
    steam(&mut c, "2", 300, Some(NOW - 29 * DAY), NOW);
    assert_eq!(got(&c), [one("steam", "1", "paused")]);
    assert_eq!(suggestions(&c, NOW).unwrap()[0].reason, "not_played");
}

#[test]
fn without_any_activity_known_nothing_is_suggested_as_paused() {
    let mut c = db();
    play(&mut c, "steam", "1", "playing", NOW - 60 * DAY);
    play(&mut c, "epic", "1", "playing", NOW - 60 * DAY);
    epic(&mut c, "1", 100, NOW - 50 * DAY); // a first read is not a session
    assert!(got(&c).is_empty());
}

#[test]
fn a_game_moved_to_playing_recently_is_not_suggested_as_paused() {
    let mut c = db();
    steam(&mut c, "1", 300, Some(NOW - 90 * DAY), NOW);
    play(&mut c, "steam", "1", "playing", NOW - 5 * DAY);
    assert!(got(&c).is_empty());
}

#[test]
fn rising_hours_between_two_reads_count_as_playing_where_there_is_no_last_session() {
    let mut c = db();
    epic(&mut c, "1", 100, NOW - 30 * DAY);
    epic(&mut c, "1", 160, NOW - 2 * DAY); // played since the last read
    epic(&mut c, "2", 100, NOW - 30 * DAY);
    epic(&mut c, "2", 100, NOW - 2 * DAY); // read again, same hours
    epic(&mut c, "3", 50, NOW - 2 * DAY); // first read only
    assert_eq!(got(&c), [one("epic", "1", "playing")]);
}

#[test]
fn hours_that_rose_long_ago_suggest_a_pause() {
    let mut c = db();
    play(&mut c, "epic", "1", "playing", NOW - 90 * DAY);
    epic(&mut c, "1", 100, NOW - 80 * DAY);
    epic(&mut c, "1", 160, NOW - 45 * DAY);
    epic(&mut c, "1", 160, NOW - DAY);
    assert_eq!(got(&c), [one("epic", "1", "paused")]);
}

#[test]
fn paused_completed_and_dropped_games_are_left_alone() {
    let mut c = db();
    for (id, state) in [("1", "paused"), ("2", "completed"), ("3", "dropped")] {
        play(&mut c, "steam", id, state, NOW - 60 * DAY);
        steam(&mut c, id, 300, Some(NOW - DAY), NOW);
    }
    assert!(got(&c).is_empty());
}

#[test]
fn a_dismissed_suggestion_does_not_come_back_for_that_target() {
    let mut c = db();
    steam(&mut c, "1", 300, Some(NOW - DAY), NOW);
    dismiss(&c, "steam", "1", "playing", NOW).unwrap();
    assert!(got(&c).is_empty());
    // Moved to playing and later forgotten: the pause suggestion is a different one.
    play(&mut c, "steam", "1", "playing", NOW - 60 * DAY);
    steam(&mut c, "1", 300, Some(NOW - 40 * DAY), NOW);
    assert_eq!(got(&c), [one("steam", "1", "paused")]);
    assert!(matches!(
        dismiss(&c, "steam", "1", "completed", NOW),
        Err(crate::plays::PlayError::BadState)
    ));
}

#[test]
fn games_of_an_account_that_is_not_active_are_not_suggested() {
    let mut c = db();
    let row = PlaytimeRow {
        native_id: "1".into(),
        minutes: Some(300),
        last_played: Some(NOW - DAY),
    };
    record(&mut c, "steam", "someone-else", &[row], false, NOW).unwrap();
    assert!(got(&c).is_empty());
}
