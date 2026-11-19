//! Mis partidas over a real database, synthetic titles only.

use super::*;
use crate::sync::{EntitlementRow, ProductRow, ReadOutcome, Resource, commit_entitlements};

const ME: &str = "76561198000000001";

fn db() -> Connection {
    let dir = tempfile::tempdir().unwrap();
    let mut conn = crate::db::open(&dir.path().join("t.db")).unwrap();
    std::mem::forget(dir);
    crate::connections::record_connection(&conn, "steam", ME, None).unwrap();
    let ids = ["1", "2", "3", "4"];
    let products: Vec<ProductRow> = ids
        .iter()
        .map(|i| ProductRow {
            native_id: (*i).into(),
            title: Some(format!("Game {i}")),
            kind: "game".into(),
            windows: "yes".into(),
            url: None,
            image: None,
        })
        .collect();
    let rows: Vec<EntitlementRow> = ids
        .iter()
        .map(|i| EntitlementRow {
            native_id: (*i).into(),
            class: "owned".into(),
            owners: vec![],
            exclude_reason: None,
        })
        .collect();
    commit_entitlements(
        &mut conn,
        "steam",
        ME,
        Resource::Library,
        &products,
        &rows,
        &ReadOutcome::Complete,
        true,
        1,
    )
    .unwrap();
    conn
}

fn put(c: &mut Connection, id: &str, state: &str, before: Option<&str>, now: i64) {
    set(c, "steam", id, state, before.map(|b| ("steam", b)), now).unwrap();
}

fn get(c: &Connection, id: &str) -> PlayView {
    list(c)
        .unwrap()
        .into_iter()
        .find(|p| p.product.native_id == id)
        .unwrap()
}

fn order(c: &Connection, state: &str) -> Vec<String> {
    list(c)
        .unwrap()
        .into_iter()
        .filter(|p| p.state == state)
        .map(|p| p.product.native_id)
        .collect()
}

#[test]
fn a_new_play_goes_to_the_top_of_its_state() {
    let mut c = db();
    put(&mut c, "1", "backlog", None, 10);
    put(&mut c, "2", "backlog", None, 11);
    assert_eq!(order(&c, "backlog"), ["2", "1"]);
    let p = get(&c, "2");
    assert_eq!((p.since, p.started_at, p.finished_at), (11, None, None));
    assert_eq!(p.product.title.as_deref(), Some("Game 2"));
    assert!(p.in_library);
}

#[test]
fn entering_playing_sets_the_start_once() {
    let mut c = db();
    put(&mut c, "1", "playing", None, 10);
    put(&mut c, "1", "paused", None, 20);
    put(&mut c, "1", "playing", None, 30);
    let p = get(&c, "1");
    assert_eq!((p.since, p.started_at), (30, Some(10)));
}

#[test]
fn completed_sets_the_finish_and_leaving_it_clears_it() {
    let mut c = db();
    put(&mut c, "1", "playing", None, 10);
    put(&mut c, "1", "completed", None, 20);
    assert_eq!(get(&c, "1").finished_at, Some(20));
    put(&mut c, "1", "playing", None, 30);
    let p = get(&c, "1");
    assert_eq!((p.finished_at, p.started_at), (None, Some(10)));
}

#[test]
fn reordering_within_a_state_keeps_since() {
    let mut c = db();
    put(&mut c, "1", "backlog", None, 10);
    put(&mut c, "2", "backlog", None, 11);
    put(&mut c, "1", "backlog", None, 50);
    assert_eq!(order(&c, "backlog"), ["1", "2"]);
    assert_eq!(get(&c, "1").since, 10);
}

#[test]
fn before_puts_the_card_just_ahead_of_another() {
    let mut c = db();
    put(&mut c, "1", "backlog", None, 10);
    put(&mut c, "2", "backlog", None, 11);
    put(&mut c, "3", "backlog", None, 12);
    assert_eq!(order(&c, "backlog"), ["3", "2", "1"]);
    put(&mut c, "3", "backlog", Some("1"), 13);
    assert_eq!(order(&c, "backlog"), ["2", "3", "1"]);
    // From another state, straight in front of a card.
    put(&mut c, "4", "playing", None, 14);
    put(&mut c, "4", "backlog", Some("2"), 15);
    assert_eq!(order(&c, "backlog"), ["4", "2", "3", "1"]);
}

#[test]
fn a_before_in_another_state_falls_back_to_the_top() {
    let mut c = db();
    put(&mut c, "1", "backlog", None, 10);
    put(&mut c, "2", "playing", None, 11);
    put(&mut c, "3", "backlog", Some("2"), 12);
    assert_eq!(order(&c, "backlog"), ["3", "1"]);
}

#[test]
fn positions_are_renumbered_when_they_get_too_close() {
    let mut c = db();
    put(&mut c, "1", "backlog", None, 10);
    put(&mut c, "2", "backlog", None, 11);
    // Keep inserting between the top two until the gap is under the threshold.
    for i in 0..60 {
        let (a, b) = if i % 2 == 0 { ("3", "1") } else { ("4", "1") };
        put(&mut c, a, "backlog", Some(b), 20 + i);
    }
    let positions: Vec<f64> = list(&c)
        .unwrap()
        .into_iter()
        .filter(|p| p.state == "backlog")
        .map(|p| p.position)
        .collect();
    for w in positions.windows(2) {
        assert!(w[1] - w[0] >= MIN_GAP, "{positions:?}");
    }
}

#[test]
fn unknown_states_and_stores_are_refused() {
    let mut c = db();
    assert!(matches!(
        set(&mut c, "steam", "1", "finished", None, 1),
        Err(PlayError::BadState)
    ));
    assert!(matches!(
        set(&mut c, "origin", "1", "playing", None, 1),
        Err(PlayError::BadStore)
    ));
}

#[test]
fn a_note_is_kept_up_to_its_limit() {
    let mut c = db();
    put(&mut c, "1", "playing", None, 10);
    note(&c, "steam", "1", "  chapter 3  ", 11).unwrap();
    assert_eq!(get(&c, "1").note, "chapter 3");
    let long = "é".repeat(NOTE_MAX + 1);
    assert!(matches!(
        note(&c, "steam", "1", &long, 12),
        Err(PlayError::NoteTooLong)
    ));
    note(&c, "steam", "1", &"é".repeat(NOTE_MAX), 13).unwrap();
    assert!(matches!(
        note(&c, "steam", "2", "x", 14),
        Err(PlayError::NotFound)
    ));
}

#[test]
fn removing_a_play_forgets_it() {
    let mut c = db();
    put(&mut c, "1", "playing", None, 10);
    remove(&c, "steam", "1").unwrap();
    assert!(list(&c).unwrap().is_empty());
}

#[test]
fn a_play_survives_clearing_the_cache_and_leaving_the_library() {
    let mut c = db();
    put(&mut c, "1", "playing", None, 10);
    crate::db::clear_cache(&c).unwrap();
    let p = get(&c, "1");
    assert_eq!(p.state, "playing");
    assert!(!p.in_library);
    assert_eq!(p.product.title, None);
}

#[test]
fn a_play_carries_its_hours_and_last_session_when_known() {
    let mut c = db();
    put(&mut c, "1", "playing", None, 10);
    put(&mut c, "2", "backlog", None, 11);
    let row = crate::playtime::PlaytimeRow {
        native_id: "1".into(),
        minutes: Some(754),
        last_played: Some(900),
    };
    crate::playtime::record(&mut c, "steam", ME, &[row], true, 1000).unwrap();
    let p = get(&c, "1");
    assert_eq!((p.minutes, p.last_activity), (Some(754), Some(900)));
    let q = get(&c, "2");
    assert_eq!(
        (q.minutes, q.last_activity),
        (None, None),
        "unknown, not zero"
    );
}

#[test]
fn before_names_a_card_by_store_and_id() {
    let mut c = db();
    set(&mut c, "steam", "1", "backlog", None, 10).unwrap();
    set(&mut c, "gog", "1", "backlog", None, 11).unwrap();
    set(&mut c, "steam", "2", "backlog", Some(("steam", "1")), 12).unwrap();
    let keys: Vec<String> = list(&c)
        .unwrap()
        .into_iter()
        .map(|p| format!("{}:{}", p.product.store, p.product.native_id))
        .collect();
    assert_eq!(keys, ["gog:1", "steam:2", "steam:1"]);
}
