//! Cleanup review (spec 7, 8.1 steps 2-4): every wishlist entry of the active accounts against
//! everything held in any store, explained. Nothing is preselected: this module only describes.
//!
//! Manual decisions and "Mantener" live in their own tables, which no sync ever touches, so they
//! survive every resync (acceptance A10).

use crate::matching::{
    self, Access, Evidence, Holding, Kind, Manual, Outcome, Product, ProductKey, Reason, Relations,
    Store, Tri,
};
use rusqlite::{Connection, params};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use ts_rs::TS;

fn store_of(s: &str) -> Option<Store> {
    match s {
        "steam" => Some(Store::Steam),
        "gog" => Some(Store::Gog),
        "epic" => Some(Store::Epic),
        _ => None,
    }
}

fn store_name(s: Store) -> &'static str {
    match s {
        Store::Steam => "steam",
        Store::Gog => "gog",
        Store::Epic => "epic",
    }
}

fn kind_of(k: &str) -> Kind {
    match k {
        "game" => Kind::Game,
        "dlc" => Kind::Dlc,
        "soundtrack" => Kind::Soundtrack,
        "demo" => Kind::Demo,
        // An application or a tool never accredits owning a game (spec 7.3).
        "tool" | "application" => Kind::Tool,
        "bundle" => Kind::Bundle,
        "movie" | "video" => Kind::Video,
        _ => Kind::Unknown,
    }
}

fn tri_of(w: &str) -> Tri {
    match w {
        "yes" => Tri::Yes,
        "no" => Tri::No,
        _ => Tri::Unknown,
    }
}

fn access_of(c: &str) -> Access {
    match c {
        "owned" => Access::Owned,
        "family_shared" => Access::FamilyShared,
        "temporary" => Access::Temporary,
        _ => Access::Unknown,
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct ProductRef {
    pub store: String,
    pub native_id: String,
    pub title: Option<String>,
    pub kind: String,
    pub windows: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct MatchView {
    pub held: ProductRef,
    /// `owned`, `family_shared`, `temporary` or `unknown`.
    pub access: String,
    pub family_owners: usize,
    /// `exact`, `candidate`, `unresolved`, `not_covered` or `family_excluded`.
    pub outcome: String,
    pub edition_superior: bool,
    pub platform_unknown: bool,
    /// Why it is not covered or unresolved, as a code the UI words.
    pub reason: Option<String>,
    /// `store_id`, `external_map`, `manual`, `bundle_component` or `title_candidate`.
    pub evidence: Option<String>,
    /// The user's own decision on this pair, if any: `same` or `different`.
    pub manual: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct ReviewItem {
    pub account_id: String,
    pub wanted: ProductRef,
    pub priority: Option<i64>,
    pub added_at: Option<i64>,
    pub observed_at: i64,
    pub kept: bool,
    pub keep_reason: Option<String>,
    pub matches: Vec<MatchView>,
    /// The same title wished in another store, owned in none: "deseado repetido entre tiendas".
    pub repeated_in: Vec<ProductRef>,
}

struct Held {
    product: Product,
    reference: ProductRef,
    access: String,
    owners: usize,
}

fn product_ref(
    store: &str,
    id: &str,
    r: &rusqlite::Row,
    base: usize,
) -> rusqlite::Result<ProductRef> {
    Ok(ProductRef {
        store: store.to_owned(),
        native_id: id.to_owned(),
        title: r.get(base)?,
        kind: r.get(base + 1)?,
        windows: r.get(base + 2)?,
        url: r.get(base + 3)?,
    })
}

fn to_product(p: &ProductRef) -> Option<Product> {
    Some(Product {
        key: ProductKey {
            store: store_of(&p.store)?,
            native_id: p.native_id.clone(),
        },
        title: p.title.clone(),
        kind: kind_of(&p.kind),
        windows: tri_of(&p.windows),
        components: vec![],
    })
}

fn load_holdings(c: &Connection) -> rusqlite::Result<Vec<Held>> {
    let mut st = c.prepare(
        "SELECT e.store, e.native_id, p.title, COALESCE(p.kind,'unknown'), COALESCE(p.windows,'unknown'),
                p.url, e.class, e.owners
         FROM entitlement e
         JOIN connection cn ON cn.provider = e.store AND cn.account_id = e.account_id
         LEFT JOIN product p ON p.store = e.store AND p.native_id = e.native_id",
    )?;
    let rows = st.query_map([], |r| {
        let store: String = r.get(0)?;
        let id: String = r.get(1)?;
        let reference = product_ref(&store, &id, r, 2)?;
        let owners: String = r.get(7)?;
        Ok((reference, r.get::<_, String>(6)?, owners))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (reference, access, owners) = row?;
        let Some(product) = to_product(&reference) else {
            continue;
        };
        let owners = serde_json::from_str::<Vec<String>>(&owners).map_or(0, |v| v.len());
        out.push(Held {
            product,
            reference,
            access,
            owners,
        });
    }
    Ok(out)
}

fn load_relations(c: &Connection) -> rusqlite::Result<Relations> {
    let mut rels = Relations::default();
    let mut st = c.prepare(
        "SELECT wanted_store, wanted_id, held_store, held_id, class, origin FROM product_relation",
    )?;
    let rows = st.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
        ))
    })?;
    for row in rows {
        let (ws, wi, hs, hi, class, origin) = row?;
        let (Some(ws), Some(hs)) = (store_of(&ws), store_of(&hs)) else {
            continue;
        };
        let w = ProductKey {
            store: ws,
            native_id: wi,
        };
        let h = ProductKey {
            store: hs,
            native_id: hi,
        };
        match (origin.as_str(), class.as_str()) {
            ("manual", "same") => {
                rels.manual.insert((w, h), Manual::Same);
            }
            ("manual", "different") => {
                rels.manual.insert((w, h), Manual::Different);
            }
            ("external_map", "same") => rels.external.push((w, h)),
            _ => {}
        }
    }
    Ok(rels)
}

fn reason_code(r: Reason) -> &'static str {
    match r {
        Reason::DifferentTitle => "different_title",
        Reason::DifferentKind => "different_kind",
        Reason::Remaster => "remaster",
        Reason::WantedHasLargerEdition => "wanted_larger_edition",
        Reason::EditionsDiffer => "editions_differ",
        Reason::PlatformNotPc => "platform_not_pc",
        Reason::NotPermanent => "not_permanent",
        Reason::AccessUnknown => "access_unknown",
        Reason::FamilyExcluded => "family_excluded",
        Reason::ToolNotAGame => "tool_not_a_game",
        Reason::UnnamedWishlist => "unnamed_wishlist",
        Reason::ManuallyDifferent => "manually_different",
    }
}

fn evidence_code(e: Evidence) -> &'static str {
    match e {
        Evidence::StoreId => "store_id",
        Evidence::ExternalMap => "external_map",
        Evidence::Manual => "manual",
        Evidence::BundleComponent => "bundle_component",
        Evidence::TitleCandidate => "title_candidate",
    }
}

/// Builds the review for every wishlist entry of the active accounts.
pub fn build(c: &Connection, include_family: bool) -> rusqlite::Result<Vec<ReviewItem>> {
    let holdings = load_holdings(c)?;
    let rels = load_relations(c)?;
    let opts = matching::Options { include_family };

    // Candidates are only looked for among holdings sharing the edition-stripped title, or tied
    // by hard evidence. Similarity alone never proposes anything (spec 7.2).
    let mut by_base: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut by_key: BTreeMap<ProductKey, Vec<usize>> = BTreeMap::new();
    for (i, h) in holdings.iter().enumerate() {
        if let Some(t) = &h.product.title {
            let base = matching::title::strip_editions(&matching::title::normalize(t)).base;
            by_base.entry(base).or_default().push(i);
        }
        by_key.entry(h.product.key.clone()).or_default().push(i);
    }

    let mut st = c.prepare(
        "SELECT w.account_id, w.store, w.native_id, p.title, COALESCE(p.kind,'unknown'),
                COALESCE(p.windows,'unknown'), p.url, w.priority, w.added_at, w.observed_at,
                k.native_id IS NOT NULL, k.reason
         FROM wishlist_entry w
         JOIN connection cn ON cn.provider = w.store AND cn.account_id = w.account_id
         LEFT JOIN product p ON p.store = w.store AND p.native_id = w.native_id
         LEFT JOIN wishlist_keep k ON k.account_id = w.account_id AND k.store = w.store
                                   AND k.native_id = w.native_id
         ORDER BY w.store, w.priority IS NULL, w.priority, w.native_id",
    )?;
    let wishes = st
        .query_map([], |r| {
            let store: String = r.get(1)?;
            let id: String = r.get(2)?;
            Ok(ReviewItem {
                account_id: r.get(0)?,
                wanted: product_ref(&store, &id, r, 3)?,
                priority: r.get(7)?,
                added_at: r.get(8)?,
                observed_at: r.get(9)?,
                kept: r.get(10)?,
                keep_reason: r.get(11)?,
                matches: vec![],
                repeated_in: vec![],
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let wanted_products: Vec<Option<Product>> =
        wishes.iter().map(|w| to_product(&w.wanted)).collect();
    let mut out = Vec::with_capacity(wishes.len());
    for (mut item, wanted) in wishes.into_iter().zip(&wanted_products) {
        let Some(wanted) = wanted else {
            continue;
        };
        let mut pool: BTreeSet<usize> = BTreeSet::new();
        if let Some(t) = &wanted.title {
            let base = matching::title::strip_editions(&matching::title::normalize(t)).base;
            pool.extend(by_base.get(&base).into_iter().flatten());
        }
        pool.extend(by_key.get(&wanted.key).into_iter().flatten());
        for (w, h) in rels.manual.keys() {
            if w == &wanted.key {
                pool.extend(by_key.get(h).into_iter().flatten());
            }
        }
        for (a, b) in &rels.external {
            let other = if a == &wanted.key {
                Some(b)
            } else if b == &wanted.key {
                Some(a)
            } else {
                None
            };
            if let Some(o) = other {
                pool.extend(by_key.get(o).into_iter().flatten());
            }
        }
        for i in pool {
            let h = &holdings[i];
            let holding = Holding {
                product: h.product.clone(),
                access: access_of(&h.access),
            };
            let verdict = matching::classify(wanted, &holding, &rels, opts);
            let (outcome, reason, sup, unk) = match verdict.outcome {
                Outcome::Exact => ("exact", None, false, false),
                Outcome::Candidate {
                    edition_superior,
                    platform_unknown,
                } => ("candidate", None, edition_superior, platform_unknown),
                Outcome::Unresolved(r) => ("unresolved", Some(r), false, false),
                Outcome::NotCovered(Reason::DifferentTitle) => continue,
                Outcome::NotCovered(r) => ("not_covered", Some(r), false, false),
                Outcome::Ignored(Reason::FamilyExcluded) => (
                    "family_excluded",
                    Some(Reason::FamilyExcluded),
                    false,
                    false,
                ),
                Outcome::Ignored(_) => continue,
            };
            item.matches.push(MatchView {
                held: h.reference.clone(),
                access: h.access.clone(),
                family_owners: h.owners,
                outcome: outcome.into(),
                edition_superior: sup,
                platform_unknown: unk,
                reason: reason.map(|r| reason_code(r).to_owned()),
                evidence: verdict.evidence.map(|e| evidence_code(e).to_owned()),
                manual: rels
                    .manual
                    .get(&(wanted.key.clone(), h.product.key.clone()))
                    .map(|m| {
                        match m {
                            Manual::Same => "same",
                            Manual::Different => "different",
                        }
                        .to_owned()
                    }),
            });
        }
        out.push(item);
    }

    // Repeated wishes across stores.
    let named: Vec<Product> = wanted_products.iter().flatten().cloned().collect();
    for (a, b) in matching::repeated_wishes(&named) {
        for item in out.iter_mut() {
            let me = (item.wanted.store.as_str(), item.wanted.native_id.as_str());
            let other = if me == (store_name(a.store), a.native_id.as_str()) {
                &b
            } else if me == (store_name(b.store), b.native_id.as_str()) {
                &a
            } else {
                continue;
            };
            if let Some(p) = named.iter().find(|p| &p.key == other) {
                let src = out_ref(p);
                item.repeated_in.push(src);
            }
        }
    }
    Ok(out)
}

fn out_ref(p: &Product) -> ProductRef {
    ProductRef {
        store: store_name(p.key.store).into(),
        native_id: p.key.native_id.clone(),
        title: p.title.clone(),
        kind: "unknown".into(),
        windows: "unknown".into(),
        url: None,
    }
}

/// The user's decision on a pair: `same`, `different`, or `None` to forget it.
pub fn set_manual(
    c: &Connection,
    wanted: (&str, &str),
    held: (&str, &str),
    decision: Option<&str>,
    now: i64,
) -> rusqlite::Result<()> {
    c.execute(
        "DELETE FROM product_relation WHERE wanted_store = ?1 AND wanted_id = ?2
           AND held_store = ?3 AND held_id = ?4 AND origin = 'manual'",
        params![wanted.0, wanted.1, held.0, held.1],
    )?;
    if let Some(d) = decision {
        c.execute(
            "INSERT INTO product_relation(wanted_store, wanted_id, held_store, held_id, class, origin, decided_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'manual', ?6)",
            params![wanted.0, wanted.1, held.0, held.1, d, now],
        )?;
    }
    Ok(())
}

/// "Mantener en esta wishlist": a preference on one entry, independent of any relation.
pub fn set_keep(
    c: &Connection,
    account: &str,
    store: &str,
    native_id: &str,
    keep: bool,
    reason: Option<&str>,
    now: i64,
) -> rusqlite::Result<()> {
    if keep {
        c.execute(
            "INSERT INTO wishlist_keep(account_id, store, native_id, reason, decided_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(account_id, store, native_id) DO UPDATE SET
               reason = excluded.reason, decided_at = excluded.decided_at",
            params![account, store, native_id, reason, now],
        )?;
    } else {
        c.execute(
            "DELETE FROM wishlist_keep WHERE account_id = ?1 AND store = ?2 AND native_id = ?3",
            params![account, store, native_id],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
