//! Cross-store equivalence engine (spec section 7). Pure logic, no I/O.
//!
//! Three questions stay separate: identity (this module's `Evidence`), coverage (`Outcome`) and
//! preference ("Mantener", stored elsewhere). Only store IDs, external maps, manual decisions and
//! accredited bundle components can produce `Exact`. Title matches are at most `Candidate`.
//! There is no fuzzy matching: similarity alone never proposes anything (spec 7.2, 7.3).

pub mod title;

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Store {
    Steam,
    Gog,
    Epic,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProductKey {
    pub store: Store,
    pub native_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Game,
    Dlc,
    Soundtrack,
    Demo,
    Tool,
    Bundle,
    /// Films and videos (GOG movies, Steam videos): content, never a game.
    Video,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    Yes,
    No,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Product {
    pub key: ProductKey,
    /// `None`: a wishlist entry whose listing was withdrawn (kept by ID, spec 7.3).
    pub title: Option<String>,
    pub kind: Kind,
    pub windows: Tri,
    /// Accredited components of a bundle. Empty means unknown, not "none".
    pub components: Vec<ProductKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Owned,
    FamilyShared,
    Temporary,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Holding {
    pub product: Product,
    pub access: Access,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manual {
    Same,
    Different,
}

/// Persistent knowledge: external ID maps and the user's own decisions. A rejection is a fact.
#[derive(Debug, Default)]
pub struct Relations {
    pub external: Vec<(ProductKey, ProductKey)>,
    pub manual: BTreeMap<(ProductKey, ProductKey), Manual>,
}

impl Relations {
    fn linked(&self, a: &ProductKey, b: &ProductKey) -> bool {
        self.external
            .iter()
            .any(|(x, y)| (x == a && y == b) || (x == b && y == a))
    }
    fn manual(&self, wanted: &ProductKey, held: &ProductKey) -> Option<Manual> {
        self.manual.get(&(wanted.clone(), held.clone())).copied()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Family access is excluded from "you already have it" unless the user opts in (spec 6.2).
    pub include_family: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evidence {
    StoreId,
    ExternalMap,
    Manual,
    BundleComponent,
    TitleCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    DifferentTitle,
    DifferentKind,
    Remaster,
    WantedHasLargerEdition,
    EditionsDiffer,
    PlatformNotPc,
    NotPermanent,
    AccessUnknown,
    FamilyExcluded,
    ToolNotAGame,
    UnnamedWishlist,
    ManuallyDifferent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Same product by hard evidence. Still subject to edition, platform and access checks.
    Exact,
    /// A title match: needs the user's choice. Never labelled "exact".
    Candidate {
        edition_superior: bool,
        platform_unknown: bool,
    },
    NotCovered(Reason),
    /// Cannot tell: show it for review, never as "you already own it".
    Unresolved(Reason),
    /// This holding does not count towards the question at all.
    Ignored(Reason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verdict {
    pub outcome: Outcome,
    pub evidence: Option<Evidence>,
}

fn v(outcome: Outcome, evidence: Option<Evidence>) -> Verdict {
    Verdict { outcome, evidence }
}

pub fn classify(wanted: &Product, held: &Holding, rels: &Relations, opts: Options) -> Verdict {
    let h = &held.product;

    if h.kind == Kind::Tool {
        return v(Outcome::Ignored(Reason::ToolNotAGame), None);
    }
    if held.access == Access::FamilyShared && !opts.include_family {
        return v(Outcome::Ignored(Reason::FamilyExcluded), None);
    }

    // Hard evidence first.
    let manual = rels.manual(&wanted.key, &h.key);
    if manual == Some(Manual::Different) {
        return v(
            Outcome::NotCovered(Reason::ManuallyDifferent),
            Some(Evidence::Manual),
        );
    }
    let hard = if manual == Some(Manual::Same) {
        Some(Evidence::Manual)
    } else if wanted.key == h.key {
        Some(Evidence::StoreId)
    } else if rels.linked(&wanted.key, &h.key) {
        Some(Evidence::ExternalMap)
    } else if h.kind == Kind::Bundle && h.components.contains(&wanted.key) {
        Some(Evidence::BundleComponent)
    } else {
        None
    };

    if let Some(ev) = hard {
        return match access_gate(held.access) {
            Some(r) => v(Outcome::Unresolved(r), Some(ev)),
            None if h.windows == Tri::No => v(Outcome::NotCovered(Reason::PlatformNotPc), Some(ev)),
            None => v(Outcome::Exact, Some(ev)),
        };
    }

    // From here on only titles are available: at best a candidate.
    let Some(wt) = wanted.title.as_deref() else {
        return v(Outcome::Unresolved(Reason::UnnamedWishlist), None);
    };
    let Some(ht) = h.title.as_deref() else {
        return v(Outcome::NotCovered(Reason::DifferentTitle), None);
    };

    if kinds_conflict(wanted.kind, h.kind) {
        return v(Outcome::NotCovered(Reason::DifferentKind), None);
    }
    if h.windows == Tri::No {
        return v(Outcome::NotCovered(Reason::PlatformNotPc), None);
    }

    let (nw, nh) = (title::normalize(wt), title::normalize(ht));
    let (sw, sh) = (title::strip_editions(&nw), title::strip_editions(&nh));
    let candidate = |edition_superior| Outcome::Candidate {
        edition_superior,
        platform_unknown: h.windows == Tri::Unknown,
    };

    let outcome = if nw == nh {
        candidate(false)
    } else if sw.base != sh.base {
        Outcome::NotCovered(Reason::DifferentTitle)
    } else if sw.remaster_words != sh.remaster_words {
        Outcome::NotCovered(Reason::Remaster)
    } else {
        let (ew, eh) = (
            significant(&sw.edition_words),
            significant(&sh.edition_words),
        );
        match (ew.is_empty(), eh.is_empty()) {
            (true, true) => candidate(false),
            (true, false) => candidate(true),
            (false, true) => Outcome::NotCovered(Reason::WantedHasLargerEdition),
            (false, false) if ew == eh => candidate(false),
            (false, false) => Outcome::Unresolved(Reason::EditionsDiffer),
        }
    };

    // A title candidate on a non-permanent licence is not "you own it".
    match (outcome, access_gate(held.access)) {
        (Outcome::Candidate { .. }, Some(r)) => {
            v(Outcome::Unresolved(r), Some(Evidence::TitleCandidate))
        }
        (Outcome::Candidate { .. }, None) => v(outcome, Some(Evidence::TitleCandidate)),
        (o, _) => v(o, None),
    }
}

fn access_gate(a: Access) -> Option<Reason> {
    match a {
        Access::Owned | Access::FamilyShared => None, // family already passed the opt-in check
        Access::Temporary => Some(Reason::NotPermanent),
        Access::Unknown => Some(Reason::AccessUnknown),
    }
}

fn kinds_conflict(wanted: Kind, held: Kind) -> bool {
    use Kind::*;
    let content = |k| matches!(k, Dlc | Soundtrack | Demo | Video);
    match (wanted, held) {
        (Unknown, _) | (_, Unknown) => false,
        (a, b) if content(a) || content(b) => a != b,
        _ => false,
    }
}

/// "standard" and a bare "edition" name no extra content.
fn significant(words: &[String]) -> Vec<&str> {
    let mut w: Vec<&str> = words
        .iter()
        .map(String::as_str)
        .filter(|w| !matches!(*w, "edition" | "standard"))
        .collect();
    w.sort_unstable();
    w
}

/// Same title wanted in two stores, with no ownership involved: "deseado repetido entre tiendas".
pub fn repeated_wishes(wishes: &[Product]) -> Vec<(ProductKey, ProductKey)> {
    let mut out = Vec::new();
    for (i, a) in wishes.iter().enumerate() {
        for b in &wishes[i + 1..] {
            if a.key.store == b.key.store {
                continue;
            }
            if let (Some(x), Some(y)) = (&a.title, &b.title)
                && title::normalize(x) == title::normalize(y)
            {
                out.push((a.key.clone(), b.key.clone()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
