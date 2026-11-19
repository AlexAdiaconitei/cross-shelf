//! Suggestions ("Sugerencias"): a change of state the play time hints at, which the user accepts
//! or dismisses. Nothing changes without a click. Thresholds are a first guess, to be tuned with
//! real use (plan 0.3.0, 2.3):
//!
//! 1. No play and played in the last 14 days → "playing".
//! 2. In "backlog" and played in the last 14 days → "playing".
//! 3. In "playing" for more than 30 days and not played for more than 30 → "paused".
//!
//! "Played" is Steam's last session or, where the store gives no session (Epic, GOG), hours that
//! rose between two snapshots. A first read is not a session. A dismissed suggestion does not
//! come back for that product and target.

use rusqlite::{Connection, params};
use serde::Serialize;
use std::collections::BTreeMap;
use ts_rs::TS;

use super::PlayError;
use crate::review::ProductRef;

const DAY: i64 = 86_400;
pub const RECENT_DAYS: i64 = 14;
pub const IDLE_DAYS: i64 = 30;
/// States a suggestion can propose.
pub const TARGETS: &[&str] = &["playing", "paused"];

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct Suggestion {
    pub product: ProductRef,
    /// `playing` or `paused`.
    pub target: String,
    /// `played_recently` or `not_played`, worded by the UI.
    pub reason: String,
    /// The play's state today, if there is a play.
    pub current: Option<String>,
    /// Last known session or rise in hours (seconds).
    pub last_activity: Option<i64>,
}

/// The rule that applies to one product, as (target, reason). `play` is its state and since when.
pub fn rule(
    play: Option<(&str, i64)>,
    last_activity: Option<i64>,
    now: i64,
) -> Option<(&'static str, &'static str)> {
    let last = last_activity?;
    let recent = now - last <= RECENT_DAYS * DAY;
    match play {
        None | Some(("backlog", _)) if recent => Some(("playing", "played_recently")),
        Some(("playing", since))
            if now - since > IDLE_DAYS * DAY && now - last > IDLE_DAYS * DAY =>
        {
            Some(("paused", "not_played"))
        }
        _ => None,
    }
}

/// Last activity of every product of the active accounts: the latest of Steam's last session and
/// the last snapshot whose minutes rose over the one before.
pub(crate) const ACTIVITY_SQL: &str = "
    WITH lp AS (
        SELECT t.store, t.native_id, MAX(t.last_played) AS at
        FROM playtime t
        JOIN connection c ON c.provider = t.store AND c.account_id = t.account_id
        GROUP BY t.store, t.native_id),
    steps AS (
        SELECT s.store, s.native_id, s.observed_at,
               s.minutes - LAG(s.minutes) OVER (
                   PARTITION BY s.account_id, s.store, s.native_id ORDER BY s.observed_at) AS rise
        FROM playtime_snapshot s
        JOIN connection c ON c.provider = s.store AND c.account_id = s.account_id),
    inc AS (
        SELECT store, native_id, MAX(observed_at) AS at FROM steps WHERE rise > 0
        GROUP BY store, native_id)
    SELECT store, native_id, MAX(at) FROM (
        SELECT store, native_id, at FROM lp UNION ALL SELECT store, native_id, at FROM inc)
    WHERE at IS NOT NULL
    GROUP BY store, native_id";

pub fn activity(c: &Connection) -> Result<BTreeMap<(String, String), i64>, rusqlite::Error> {
    c.prepare(ACTIVITY_SQL)?
        .query_map([], |r| Ok(((r.get(0)?, r.get(1)?), r.get(2)?)))?
        .collect()
}

/// Every product the rules point at, minus the dismissed ones. Only products an active account
/// holds; sorted by most recent activity first.
pub fn suggestions(c: &Connection, now: i64) -> Result<Vec<Suggestion>, rusqlite::Error> {
    let last = activity(c)?;
    let plays: BTreeMap<(String, String), (String, i64)> = c
        .prepare("SELECT store, native_id, state, since FROM play")?
        .query_map([], |r| Ok(((r.get(0)?, r.get(1)?), (r.get(2)?, r.get(3)?))))?
        .collect::<Result<_, _>>()?;
    let dismissed: std::collections::BTreeSet<(String, String, String)> = c
        .prepare("SELECT store, native_id, target FROM suggestion_dismissed")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    let mut product = c.prepare(
        "SELECT p.title, COALESCE(p.kind,'unknown'), COALESCE(p.windows,'unknown'), p.url,
                p.image_url
         FROM entitlement e
         JOIN connection cn ON cn.provider = e.store AND cn.account_id = e.account_id
         LEFT JOIN product p ON p.store = e.store AND p.native_id = e.native_id
         WHERE e.store = ?1 AND e.native_id = ?2 LIMIT 1",
    )?;

    let mut keys: Vec<&(String, String)> = last.keys().chain(plays.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut out = Vec::new();
    for key in keys {
        let play = plays.get(key);
        let at = last.get(key).copied();
        let Some((target, reason)) = rule(play.map(|(s, t)| (s.as_str(), *t)), at, now) else {
            continue;
        };
        if dismissed.contains(&(key.0.clone(), key.1.clone(), target.to_owned())) {
            continue;
        }
        let found = product
            .query_map(params![key.0, key.1], |r| {
                Ok(ProductRef {
                    store: key.0.clone(),
                    native_id: key.1.clone(),
                    title: r.get(0)?,
                    kind: r.get(1)?,
                    windows: r.get(2)?,
                    url: r.get(3)?,
                    image_url: r.get(4)?,
                })
            })?
            .next()
            .transpose()?;
        let Some(product) = found else { continue };
        out.push(Suggestion {
            product,
            target: target.into(),
            reason: reason.into(),
            current: play.map(|(s, _)| s.clone()),
            last_activity: at,
        });
    }
    out.sort_by_key(|s| std::cmp::Reverse(s.last_activity));
    Ok(out)
}

/// "Descartar": the rule stops proposing `target` for this product.
pub fn dismiss(
    c: &Connection,
    store: &str,
    native_id: &str,
    target: &str,
    now: i64,
) -> Result<(), PlayError> {
    if !TARGETS.contains(&target) {
        return Err(PlayError::BadState);
    }
    c.execute(
        "INSERT INTO suggestion_dismissed(store, native_id, target, dismissed_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(store, native_id, target) DO UPDATE SET dismissed_at = excluded.dismissed_at",
        params![store, native_id, target, now],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
