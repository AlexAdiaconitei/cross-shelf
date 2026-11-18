use super::*;
use crate::connections::record_connection;
use crate::sync::{ReadOutcome, WishRow, commit_wishlist};

fn db() -> Connection {
    let dir = tempfile::tempdir().unwrap();
    let conn = crate::db::open(&dir.path().join("t.db")).unwrap();
    std::mem::forget(dir);
    record_connection(&conn, "steam", "s1", None).unwrap();
    record_connection(&conn, "gog", "g1", None).unwrap();
    conn
}

fn wishlist(c: &mut Connection, store: &str, acct: &str, ids: &[&str], now: i64) {
    let rows: Vec<_> = ids
        .iter()
        .map(|i| WishRow {
            native_id: (*i).into(),
            priority: None,
            added_at: None,
            confirmed: true,
        })
        .collect();
    commit_wishlist(
        c,
        store,
        acct,
        &[],
        &rows,
        &ReadOutcome::Complete,
        true,
        now,
    )
    .unwrap();
}

fn ids(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn no_write_is_enabled_without_its_validation_record() {
    for store in crate::connections::PROVIDERS {
        let record = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("validation")
            .join(format!("{store}-write.md"));
        assert!(
            !write_validated(store) || record.exists(),
            "{store}: write enabled without {}",
            record.display()
        );
    }
}

#[test]
fn a_reviewed_batch_confirms_and_is_done_only_after_a_reread() {
    let mut c = db();
    wishlist(&mut c, "steam", "s1", &["1", "2", "3"], 1);
    let b = create(&mut c, "steam", "s1", &ids(&["1", "2"]), 10).unwrap();
    assert_eq!(b.state, "draft");
    assert!(b.write_validated);
    assert!(
        b.ops
            .iter()
            .all(|o| o.status == "pending" && o.desired == "absent")
    );
    let b = confirm(&c, &b.id, &b.revision, 11).unwrap();
    assert_eq!(b.state, "confirmed");

    // Opening the store and coming back proves nothing: without a reread, still pending.
    let same = reconcile(&c, &b.id, 12).unwrap();
    assert!(same.ops.iter().all(|o| o.status == "pending"));

    // The user removed "1" in the store; the reread shows it gone, "2" still there.
    wishlist(&mut c, "steam", "s1", &["2", "3"], 20);
    let r = reconcile(&c, &b.id, 21).unwrap();
    let st: Vec<_> = r.ops.iter().map(|o| o.status.as_str()).collect();
    assert_eq!(st, ["already_in_desired_state", "pending"]);
    assert_eq!(r.state, "confirmed");
    wishlist(&mut c, "steam", "s1", &["3"], 30);
    assert_eq!(reconcile(&c, &b.id, 31).unwrap().state, "done");
}

#[test]
fn a_list_edited_since_the_review_invalidates_the_confirmation() {
    let mut c = db();
    wishlist(&mut c, "steam", "s1", &["1", "2"], 1);
    let b = create(&mut c, "steam", "s1", &ids(&["1"]), 10).unwrap();
    wishlist(&mut c, "steam", "s1", &["1", "2", "9"], 15);
    assert_eq!(
        confirm(&c, &b.id, &b.revision, 16),
        Err(PlanError::PlanChanged)
    );
    assert_eq!(get(&c, &b.id).unwrap().unwrap().state, "invalidated");
}

#[test]
fn a_confirmation_for_another_revision_is_refused() {
    let mut c = db();
    wishlist(&mut c, "gog", "g1", &["1"], 1);
    let b = create(&mut c, "gog", "g1", &ids(&["1"]), 10).unwrap();
    assert_eq!(
        confirm(&c, &b.id, "0000000000000000", 11),
        Err(PlanError::PlanChanged)
    );
    // A second click on an invalidated batch does nothing.
    assert_eq!(
        confirm(&c, &b.id, &b.revision, 12),
        Err(PlanError::BadState)
    );
}

#[test]
fn a_changed_account_invalidates_pending_batches() {
    let mut c = db();
    wishlist(&mut c, "steam", "s1", &["1"], 1);
    let b = create(&mut c, "steam", "s1", &ids(&["1"]), 10).unwrap();
    confirm(&c, &b.id, &b.revision, 11).unwrap();
    record_connection(&c, "steam", "s2", None).unwrap();
    let all = list(&c).unwrap();
    assert_eq!(all[0].state, "invalidated");
    assert_eq!(reconcile(&c, &b.id, 12), Err(PlanError::BadState));
}

#[test]
fn kept_missing_and_foreign_entries_cannot_enter_a_batch() {
    let mut c = db();
    wishlist(&mut c, "steam", "s1", &["1", "2"], 1);
    crate::review::set_keep(&c, "s1", "steam", "2", true, None, 2).unwrap();
    assert_eq!(
        create(&mut c, "steam", "s1", &ids(&["2"]), 3).unwrap_err(),
        PlanError::Kept
    );
    assert_eq!(
        create(&mut c, "steam", "s1", &ids(&["7"]), 3).unwrap_err(),
        PlanError::NotInWishlist
    );
    assert_eq!(
        create(&mut c, "steam", "other", &ids(&["1"]), 3).unwrap_err(),
        PlanError::AccountNotActive
    );
    assert_eq!(
        create(&mut c, "epic", "e1", &ids(&["1"]), 3).unwrap_err(),
        PlanError::StoreUnsupported
    );
    assert_eq!(
        create(&mut c, "steam", "s1", &[], 3).unwrap_err(),
        PlanError::Empty
    );
}

#[test]
fn only_confirmed_batches_of_validated_stores_are_executable() {
    let mut c = db();
    wishlist(&mut c, "gog", "g1", &["1", "2"], 1);
    wishlist(&mut c, "steam", "s1", &["9"], 1);
    let g = create(&mut c, "gog", "g1", &ids(&["1", "2"]), 10).unwrap();
    assert_eq!(executable(&c, &g.id), Err(PlanError::BadState), "a draft");
    confirm(&c, &g.id, &g.revision, 11).unwrap();
    let (store, account, pending) = executable(&c, &g.id).unwrap();
    assert_eq!((store.as_str(), account.as_str()), ("gog", "g1"));
    assert_eq!(pending.len(), 2);

    let s = create(&mut c, "steam", "s1", &ids(&["9"]), 12).unwrap();
    confirm(&c, &s.id, &s.revision, 13).unwrap();
    assert_eq!(executable(&c, &s.id).unwrap().2.len(), 1);
    assert!(!write_validated("epic"), "epic has no wishlist to write");
}

#[test]
fn results_are_recorded_once_and_finish_the_batch() {
    let mut c = db();
    wishlist(&mut c, "gog", "g1", &["1", "2"], 1);
    let b = create(&mut c, "gog", "g1", &ids(&["1", "2"]), 10).unwrap();
    confirm(&c, &b.id, &b.revision, 11).unwrap();
    let (_, _, pending) = executable(&c, &b.id).unwrap();
    record_result(&c, &b.id, &pending[0].0, "applied", 12).unwrap();
    assert_eq!(get(&c, &b.id).unwrap().unwrap().state, "confirmed");
    // A finished op is not overwritten by a later result.
    record_result(&c, &b.id, &pending[0].0, "failed", 13).unwrap();
    record_result(&c, &b.id, &pending[1].0, "unverified", 14).unwrap();
    let v = get(&c, &b.id).unwrap().unwrap();
    assert_eq!(v.state, "done");
    let st: Vec<_> = v.ops.iter().map(|o| o.status.as_str()).collect();
    assert_eq!(st, ["applied", "unverified"]);
    assert!(executable(&c, &b.id).is_err());
}

#[test]
fn a_batch_of_an_account_no_longer_active_is_never_executed() {
    let mut c = db();
    wishlist(&mut c, "gog", "g1", &["1"], 1);
    let b = create(&mut c, "gog", "g1", &ids(&["1"]), 10).unwrap();
    confirm(&c, &b.id, &b.revision, 11).unwrap();
    record_connection(&c, "gog", "g2", None).unwrap();
    assert_eq!(executable(&c, &b.id), Err(PlanError::AccountNotActive));
    assert_eq!(get(&c, &b.id).unwrap().unwrap().state, "invalidated");
}

fn own(c: &mut Connection, store: &str, acct: &str, ids: &[&str]) {
    use crate::sync::{EntitlementRow, ProductRow, Resource, commit_entitlements};
    let products: Vec<_> = ids
        .iter()
        .map(|i| ProductRow {
            native_id: (*i).into(),
            title: Some(format!("Juego {i}")),
            kind: "game".into(),
            windows: "yes".into(),
            url: None,
        })
        .collect();
    let rights: Vec<_> = ids
        .iter()
        .map(|i| EntitlementRow {
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
        &products,
        &rights,
        &ReadOutcome::Complete,
        true,
        1,
    )
    .unwrap();
}

#[test]
fn only_entries_with_an_exact_match_can_enter_a_batch() {
    let mut c = db();
    wishlist(&mut c, "steam", "s1", &["1", "2"], 1);
    own(&mut c, "steam", "s1", &["1"]);
    require_covered(&c, "steam", "s1", &ids(&["1"]), false).unwrap();
    assert_eq!(
        require_covered(&c, "steam", "s1", &ids(&["1", "2"]), false),
        Err(PlanError::NotCovered),
        "2 is not held anywhere"
    );
    assert_eq!(
        require_covered(&c, "steam", "other", &ids(&["1"]), false),
        Err(PlanError::NotCovered),
        "another account's entry is not this one"
    );
}

#[test]
fn a_confirmed_batch_can_be_cancelled_and_sends_nothing_more() {
    let mut c = db();
    wishlist(&mut c, "gog", "g1", &["1", "2"], 1);
    let b = create(&mut c, "gog", "g1", &ids(&["1", "2"]), 10).unwrap();
    confirm(&c, &b.id, &b.revision, 11).unwrap();
    let (_, _, pending) = executable(&c, &b.id).unwrap();
    record_result(&c, &b.id, &pending[0].0, "applied", 12).unwrap();
    let v = cancel(&c, &b.id, 13).unwrap();
    assert_eq!(v.state, "done");
    let st: Vec<_> = v.ops.iter().map(|o| o.status.as_str()).collect();
    assert_eq!(st, ["applied", "cancelled_not_sent"]);
    assert!(executable(&c, &b.id).is_err());
    assert_eq!(cancel(&c, &b.id, 14), Err(PlanError::BadState));
}

#[test]
fn only_failed_operations_go_back_to_pending_on_retry() {
    let mut c = db();
    wishlist(&mut c, "gog", "g1", &["1", "2"], 1);
    let b = create(&mut c, "gog", "g1", &ids(&["1", "2"]), 10).unwrap();
    confirm(&c, &b.id, &b.revision, 11).unwrap();
    let (_, _, pending) = executable(&c, &b.id).unwrap();
    record_result(&c, &b.id, &pending[0].0, "applied", 12).unwrap();
    record_result(&c, &b.id, &pending[1].0, "failed", 13).unwrap();
    let v = retry_failed(&c, &b.id).unwrap();
    assert_eq!(v.state, "confirmed");
    let (_, _, again) = executable(&c, &b.id).unwrap();
    assert_eq!(again.len(), 1, "the applied one is not sent again");
    assert_eq!(again[0].0, pending[1].0);
}

#[test]
fn only_drafts_and_invalidated_batches_can_be_discarded() {
    let mut c = db();
    wishlist(&mut c, "steam", "s1", &["1"], 1);
    let b = create(&mut c, "steam", "s1", &ids(&["1"]), 10).unwrap();
    confirm(&c, &b.id, &b.revision, 11).unwrap();
    assert_eq!(discard(&c, &b.id), Err(PlanError::BadState));
    let d = create(&mut c, "steam", "s1", &ids(&["1"]), 12).unwrap();
    discard(&c, &d.id).unwrap();
    assert!(get(&c, &d.id).unwrap().is_none());
}

#[test]
fn the_snapshot_keeps_no_account_secrets() {
    let mut c = db();
    wishlist(&mut c, "steam", "s1", &["1"], 1);
    let b = create(&mut c, "steam", "s1", &ids(&["1"]), 10).unwrap();
    let snap: String = c
        .query_row(
            "SELECT snapshot FROM action_batch WHERE id = ?1",
            [&b.id],
            |r| r.get(0),
        )
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&snap).unwrap();
    let keys: Vec<&str> = v[0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["added_at", "native_id", "observed_at", "priority"]);
}
