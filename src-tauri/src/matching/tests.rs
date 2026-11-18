//! One test per row of spec 7.3 plus the normalisation cases of 7.4. Synthetic titles only.

use super::*;

fn key(store: Store, id: &str) -> ProductKey {
    ProductKey {
        store,
        native_id: id.into(),
    }
}

fn product(store: Store, id: &str, title: Option<&str>, kind: Kind) -> Product {
    Product {
        key: key(store, id),
        title: title.map(Into::into),
        kind,
        windows: Tri::Yes,
        components: vec![],
    }
}

fn wish(title: &str) -> Product {
    product(Store::Steam, "w1", Some(title), Kind::Game)
}

fn owned(title: &str) -> Holding {
    Holding {
        product: product(Store::Gog, "g1", Some(title), Kind::Game),
        access: Access::Owned,
    }
}

fn run(w: &Product, h: &Holding) -> Verdict {
    classify(w, h, &Relations::default(), Options::default())
}

fn is_candidate(v: Verdict) -> bool {
    matches!(v.outcome, Outcome::Candidate { .. })
}

#[test]
fn trademark_variant_with_distinct_store_ids_is_a_candidate_never_exact() {
    let v = run(&wish("Título"), &owned("Título™"));
    assert_eq!(
        v.outcome,
        Outcome::Candidate {
            edition_superior: false,
            platform_unknown: false
        }
    );
    assert_eq!(v.evidence, Some(Evidence::TitleCandidate));
}

#[test]
fn owned_gold_edition_covers_base_wish_as_superior_candidate() {
    let v = run(&wish("Título"), &owned("Título: Gold Edition"));
    assert_eq!(
        v.outcome,
        Outcome::Candidate {
            edition_superior: true,
            platform_unknown: false
        }
    );
}

#[test]
fn wished_deluxe_is_not_covered_by_owned_base() {
    let v = run(&wish("Título: Deluxe"), &owned("Título"));
    assert_eq!(
        v.outcome,
        Outcome::NotCovered(Reason::WantedHasLargerEdition)
    );
}

#[test]
fn remaster_and_ultra_deluxe_are_distinct_products() {
    assert_eq!(
        run(&wish("Título Remastered"), &owned("Título")).outcome,
        Outcome::NotCovered(Reason::Remaster)
    );
    assert_eq!(
        run(&wish("Título"), &owned("Título Remastered")).outcome,
        Outcome::NotCovered(Reason::Remaster)
    );
    assert_eq!(
        run(&wish("Título: Ultra Deluxe"), &owned("Título")).outcome,
        Outcome::NotCovered(Reason::Remaster)
    );
}

#[test]
fn sequels_are_different_whatever_the_numeral() {
    for (a, b) in [
        ("Título II", "Título"),
        ("Título 2", "Título"),
        ("Título", "Título 2"),
        ("Título II", "Título 2"),
    ] {
        assert_eq!(
            run(&wish(a), &owned(b)).outcome,
            Outcome::NotCovered(Reason::DifferentTitle),
            "{a} vs {b}"
        );
    }
}

#[test]
fn prologue_demo_and_soundtrack_are_not_equivalent() {
    assert_eq!(
        run(&wish("Título"), &owned("Título: Prologue")).outcome,
        Outcome::NotCovered(Reason::DifferentTitle)
    );
    let mut demo = owned("Título");
    demo.product.kind = Kind::Demo;
    assert_eq!(
        run(&wish("Título"), &demo).outcome,
        Outcome::NotCovered(Reason::DifferentKind)
    );
    let mut ost = owned("Título");
    ost.product.kind = Kind::Soundtrack;
    assert_eq!(
        run(&wish("Título"), &ost).outcome,
        Outcome::NotCovered(Reason::DifferentKind)
    );
}

#[test]
fn dlc_and_base_game_are_not_equivalent_in_either_direction() {
    let mut dlc_held = owned("Título");
    dlc_held.product.kind = Kind::Dlc;
    assert_eq!(
        run(&wish("Título"), &dlc_held).outcome,
        Outcome::NotCovered(Reason::DifferentKind)
    );
    let mut dlc_wish = wish("Título");
    dlc_wish.kind = Kind::Dlc;
    assert_eq!(
        run(&dlc_wish, &owned("Título")).outcome,
        Outcome::NotCovered(Reason::DifferentKind)
    );
}

#[test]
fn bundle_covers_only_with_accredited_components() {
    let w = wish("Título");
    let mut bundle = owned("Colección Título y Más");
    bundle.product.kind = Kind::Bundle;
    // Unknown components: no coverage, the title is simply different.
    assert_eq!(
        run(&w, &bundle).outcome,
        Outcome::NotCovered(Reason::DifferentTitle)
    );
    bundle.product.components = vec![w.key.clone()];
    let v = run(&w, &bundle);
    assert_eq!(
        (v.outcome, v.evidence),
        (Outcome::Exact, Some(Evidence::BundleComponent))
    );
}

#[test]
fn near_identical_titles_of_different_games_never_match() {
    assert_eq!(
        run(&wish("Outer X"), &owned("The Outer X")).outcome,
        Outcome::NotCovered(Reason::DifferentTitle)
    );
    assert_eq!(
        run(&wish("Farlands"), &owned("Fariands")).outcome,
        Outcome::NotCovered(Reason::DifferentTitle)
    );
    assert_eq!(
        run(&wish("Blade"), &owned("Blades")).outcome,
        Outcome::NotCovered(Reason::DifferentTitle)
    );
}

#[test]
fn android_only_licence_does_not_cover_a_pc_wish() {
    let mut h = owned("Título");
    h.product.windows = Tri::No;
    assert_eq!(
        run(&wish("Título"), &h).outcome,
        Outcome::NotCovered(Reason::PlatformNotPc)
    );
}

#[test]
fn unknown_platform_is_flagged_not_assumed() {
    let mut h = owned("Título");
    h.product.windows = Tri::Unknown;
    assert_eq!(
        run(&wish("Título"), &h).outcome,
        Outcome::Candidate {
            edition_superior: false,
            platform_unknown: true
        }
    );
}

#[test]
fn family_access_is_excluded_by_default_and_included_on_request() {
    let mut h = owned("Título");
    h.access = Access::FamilyShared;
    assert_eq!(
        run(&wish("Título"), &h).outcome,
        Outcome::Ignored(Reason::FamilyExcluded)
    );
    let v = classify(
        &wish("Título"),
        &h,
        &Relations::default(),
        Options {
            include_family: true,
        },
    );
    assert!(is_candidate(v));
}

#[test]
fn temporary_and_unknown_access_never_read_as_owned() {
    for (a, r) in [
        (Access::Temporary, Reason::NotPermanent),
        (Access::Unknown, Reason::AccessUnknown),
    ] {
        let mut h = owned("Título");
        h.access = a;
        assert_eq!(run(&wish("Título"), &h).outcome, Outcome::Unresolved(r));
        // Even with a hard store-ID match.
        let mut same = h.clone();
        same.product.key = wish("Título").key;
        assert_eq!(run(&wish("Título"), &same).outcome, Outcome::Unresolved(r));
    }
}

#[test]
fn tools_and_dedicated_servers_do_not_count() {
    let mut h = owned("Título Dedicated Server");
    h.product.kind = Kind::Tool;
    assert_eq!(
        run(&wish("Título"), &h).outcome,
        Outcome::Ignored(Reason::ToolNotAGame)
    );
}

#[test]
fn unnamed_wishlist_entry_is_kept_for_review_not_marked_owned() {
    let mut w = wish("x");
    w.title = None;
    assert_eq!(
        run(&w, &owned("Título")).outcome,
        Outcome::Unresolved(Reason::UnnamedWishlist)
    );
}

#[test]
fn same_game_wished_in_two_stores_is_a_repeated_wish() {
    let a = product(Store::Steam, "1", Some("Título"), Kind::Game);
    let b = product(Store::Gog, "2", Some("Título™"), Kind::Game);
    let c = product(Store::Gog, "3", Some("Otro"), Kind::Game);
    assert_eq!(
        repeated_wishes(&[a.clone(), b.clone(), c]),
        vec![(a.key, b.key)]
    );
}

#[test]
fn hard_evidence_is_the_only_way_to_exact() {
    let w = product(Store::Steam, "10", Some("Título"), Kind::Game);
    let same_id = Holding {
        product: product(Store::Steam, "10", Some("Cualquier cosa"), Kind::Game),
        access: Access::Owned,
    };
    let v = run(&w, &same_id);
    assert_eq!(
        (v.outcome, v.evidence),
        (Outcome::Exact, Some(Evidence::StoreId))
    );

    let other = owned("Título");
    let mut rels = Relations::default();
    rels.external
        .push((w.key.clone(), other.product.key.clone()));
    let v = classify(&w, &other, &rels, Options::default());
    assert_eq!(
        (v.outcome, v.evidence),
        (Outcome::Exact, Some(Evidence::ExternalMap))
    );
}

#[test]
fn manual_decisions_win_and_a_rejection_persists() {
    let w = wish("Título");
    let h = owned("Título");
    let mut rels = Relations::default();
    rels.manual
        .insert((w.key.clone(), h.product.key.clone()), Manual::Different);
    let v = classify(&w, &h, &rels, Options::default());
    assert_eq!(v.outcome, Outcome::NotCovered(Reason::ManuallyDifferent));

    let w2 = wish("Algo distinto");
    let mut rels = Relations::default();
    rels.manual
        .insert((w2.key.clone(), h.product.key.clone()), Manual::Same);
    let v = classify(&w2, &h, &rels, Options::default());
    assert_eq!(
        (v.outcome, v.evidence),
        (Outcome::Exact, Some(Evidence::Manual))
    );
}

#[test]
fn accents_case_ampersand_and_subtitles_match_as_candidates_only() {
    assert!(is_candidate(run(
        &wish("Épico & Raro"),
        &owned("EPICO and RARO")
    )));
    assert!(is_candidate(run(
        &wish("Título: Subtítulo"),
        &owned("titulo - subtitulo")
    )));
    // A different subtitle is a different title.
    assert_eq!(
        run(&wish("Título: A"), &owned("Título: B")).outcome,
        Outcome::NotCovered(Reason::DifferentTitle)
    );
}

#[test]
fn no_title_only_row_can_ever_be_exact() {
    let wishes = [
        "Título",
        "Título: Deluxe",
        "Título Remastered",
        "Título II",
        "Título 2",
        "Épico & Raro",
    ];
    let helds = [
        "Título™",
        "Título: Gold Edition",
        "Título",
        "Título: Prologue",
        "titulo",
        "EPICO and RARO",
    ];
    for w in wishes {
        for h in helds {
            assert_ne!(
                run(&wish(w), &owned(h)).outcome,
                Outcome::Exact,
                "{w} vs {h}"
            );
        }
    }
}
