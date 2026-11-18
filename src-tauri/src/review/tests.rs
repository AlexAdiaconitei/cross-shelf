//! Review over the database with synthetic titles only (spec 7.4: no user data in fixtures).

use super::*;
use crate::connections::record_connection;
use crate::sync::{
    EntitlementRow, ProductRow, ReadOutcome, Resource, WishRow, commit_entitlements,
    commit_wishlist,
};

fn db() -> Connection {
    let dir = tempfile::tempdir().unwrap();
    let conn = crate::db::open(&dir.path().join("t.db")).unwrap();
    std::mem::forget(dir);
    record_connection(&conn, "steam", "s1", None).unwrap();
    record_connection(&conn, "gog", "g1", None).unwrap();
    conn
}

fn product(id: &str, title: &str, kind: &str) -> ProductRow {
    ProductRow {
        native_id: id.into(),
        title: Some(title.into()),
        kind: kind.into(),
        windows: "yes".into(),
        url: None,
        image: None,
    }
}

fn own(c: &mut Connection, store: &str, acct: &str, items: &[(&str, &str)]) {
    let ps: Vec<_> = items.iter().map(|(i, t)| product(i, t, "game")).collect();
    let rs: Vec<_> = items
        .iter()
        .map(|(i, _)| EntitlementRow {
            native_id: (*i).into(),
            class: "owned".into(),
            owners: vec![],
            exclude_reason: None,
        })
        .collect();
    commit_entitlements(
        c,
        store,
        acct,
        Resource::Library,
        &ps,
        &rs,
        &ReadOutcome::Complete,
        true,
        1,
    )
    .unwrap();
}

fn wish(c: &mut Connection, store: &str, acct: &str, items: &[(&str, &str)], now: i64) {
    let ps: Vec<_> = items.iter().map(|(i, t)| product(i, t, "game")).collect();
    let rs: Vec<_> = items
        .iter()
        .map(|(i, _)| WishRow {
            native_id: (*i).into(),
            priority: None,
            added_at: None,
            confirmed: true,
        })
        .collect();
    commit_wishlist(c, store, acct, &ps, &rs, &ReadOutcome::Complete, true, now).unwrap();
}

fn item<'a>(list: &'a [ReviewItem], id: &str) -> &'a ReviewItem {
    list.iter().find(|i| i.wanted.native_id == id).unwrap()
}

#[test]
fn a_trademark_variant_in_another_store_is_a_candidate_not_exact() {
    let mut c = db();
    own(&mut c, "gog", "g1", &[("g10", "Lumen Drift™")]);
    wish(&mut c, "steam", "s1", &[("100", "Lumen Drift")], 1);
    let r = build(&c, false).unwrap();
    let m = &item(&r, "100").matches;
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].outcome, "candidate");
    assert_eq!(m[0].evidence.as_deref(), Some("title_candidate"));
    assert!(
        !item(&r, "100").kept,
        "nothing is preselected or kept by default"
    );
}

#[test]
fn sequels_and_unrelated_titles_never_show_up() {
    let mut c = db();
    own(
        &mut c,
        "gog",
        "g1",
        &[("g1", "Lumen Drift"), ("g2", "The Lumen Drift")],
    );
    wish(&mut c, "steam", "s1", &[("200", "Lumen Drift 2")], 1);
    let r = build(&c, false).unwrap();
    assert!(item(&r, "200").matches.is_empty());
}

#[test]
fn manual_decisions_and_keep_survive_a_resync_and_stay_independent() {
    let mut c = db();
    own(&mut c, "gog", "g1", &[("g10", "Lumen Drift")]);
    wish(&mut c, "steam", "s1", &[("100", "Lumen Drift")], 1);
    set_manual(&c, ("steam", "100"), ("gog", "g10"), Some("different"), 5).unwrap();
    set_keep(&c, "s1", "steam", "100", true, Some("logros"), 5).unwrap();

    // Both lists are synced again: decisions must not move.
    own(&mut c, "gog", "g1", &[("g10", "Lumen Drift")]);
    wish(&mut c, "steam", "s1", &[("100", "Lumen Drift")], 9);
    let r = build(&c, false).unwrap();
    let it = item(&r, "100");
    assert!(it.kept);
    assert_eq!(it.keep_reason.as_deref(), Some("logros"));
    assert_eq!(it.matches[0].outcome, "not_covered");
    assert_eq!(it.matches[0].reason.as_deref(), Some("manually_different"));
    assert_eq!(it.matches[0].manual.as_deref(), Some("different"));

    // Changing the relation does not touch "Mantener", and the other way round.
    set_manual(&c, ("steam", "100"), ("gog", "g10"), Some("same"), 10).unwrap();
    let r = build(&c, false).unwrap();
    let it = item(&r, "100");
    assert_eq!(
        (it.matches[0].outcome.as_str(), it.kept),
        ("exact", true),
        "a manual 'same' is hard evidence"
    );
    set_keep(&c, "s1", "steam", "100", false, None, 11).unwrap();
    set_manual(&c, ("steam", "100"), ("gog", "g10"), None, 12).unwrap();
    let r = build(&c, false).unwrap();
    let it = item(&r, "100");
    assert_eq!(
        (it.matches[0].outcome.as_str(), it.kept),
        ("candidate", false)
    );
}

#[test]
fn a_manual_link_reaches_a_product_with_a_different_title() {
    let mut c = db();
    own(&mut c, "gog", "g7", &[("g7", "Lumen Drift Collection")]);
    wish(&mut c, "steam", "s1", &[("300", "Something Else")], 1);
    set_manual(&c, ("steam", "300"), ("gog", "g7"), Some("same"), 2).unwrap();
    let r = build(&c, false).unwrap();
    let m = &item(&r, "300").matches;
    assert_eq!(m.len(), 0, "g7 belongs to an account that is not active");
    own(&mut c, "gog", "g1", &[("g7", "Lumen Drift Collection")]);
    let r = build(&c, false).unwrap();
    assert_eq!(
        item(&r, "300").matches[0].evidence.as_deref(),
        Some("manual")
    );
}

#[test]
fn family_access_is_excluded_unless_the_user_opts_in() {
    let mut c = db();
    commit_entitlements(
        &mut c,
        "steam",
        "s1",
        Resource::Family,
        &[product("55", "Lumen Drift", "game")],
        &[EntitlementRow {
            native_id: "55".into(),
            class: "family_shared".into(),
            owners: vec!["76561198000000002".into()],
            exclude_reason: None,
        }],
        &ReadOutcome::Complete,
        true,
        1,
    )
    .unwrap();
    wish(&mut c, "gog", "g1", &[("g55", "Lumen Drift")], 1);
    let off = build(&c, false).unwrap();
    assert_eq!(item(&off, "g55").matches[0].outcome, "family_excluded");
    let on = build(&c, true).unwrap();
    let m = &item(&on, "g55").matches[0];
    assert_eq!(
        (m.outcome.as_str(), m.access.as_str()),
        ("candidate", "family_shared")
    );
    assert_eq!(m.family_owners, 1);
}

#[test]
fn the_same_wish_in_two_stores_is_reported_as_repeated() {
    let mut c = db();
    wish(&mut c, "steam", "s1", &[("400", "Quiet Harbor")], 1);
    wish(&mut c, "gog", "g1", &[("g400", "Quiet Harbor")], 1);
    let r = build(&c, false).unwrap();
    assert_eq!(item(&r, "400").repeated_in[0].native_id, "g400");
    assert_eq!(item(&r, "g400").repeated_in[0].native_id, "400");
    assert!(item(&r, "400").matches.is_empty());
}

#[test]
fn a_movie_never_covers_a_game() {
    let mut c = db();
    commit_entitlements(
        &mut c,
        "gog",
        "g1",
        Resource::Library,
        &[product("m1", "Quiet Harbor", "movie")],
        &[EntitlementRow {
            native_id: "m1".into(),
            class: "owned".into(),
            owners: vec![],
            exclude_reason: None,
        }],
        &ReadOutcome::Complete,
        true,
        1,
    )
    .unwrap();
    let mut wanted = product("500", "Quiet Harbor", "game");
    wanted.windows = "unknown".into();
    commit_wishlist(
        &mut c,
        "steam",
        "s1",
        &[wanted],
        &[WishRow {
            native_id: "500".into(),
            priority: None,
            added_at: None,
            confirmed: true,
        }],
        &ReadOutcome::Complete,
        true,
        1,
    )
    .unwrap();
    let r = build(&c, false).unwrap();
    let m = &item(&r, "500").matches[0];
    assert_eq!(
        (m.outcome.as_str(), m.reason.as_deref()),
        ("not_covered", Some("different_kind"))
    );
}
