//! The diary ("Diario"): games with a play or with hours, by last activity, with what was played
//! in the last week. A game without a play shows the state the suggestion rules deduce, which the
//! user confirms with a click.

use rusqlite::{Connection, params};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use ts_rs::TS;

use super::suggestions::{activity, rule};
use crate::review::ProductRef;

const WEEK: i64 = 7 * 86_400;

#[derive(Debug, Serialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct DiaryRow {
    pub product: ProductRef,
    /// Minutes played by the active accounts; `None` when no store gives them.
    pub minutes: Option<i64>,
    /// Last session or last rise in hours (seconds).
    pub last_activity: Option<i64>,
    /// Minutes played since a week ago; `None` when there is no read from back then.
    pub week_minutes: Option<i64>,
    /// The play's state, when there is a play.
    pub state: Option<String>,
    /// Without a play: the state the rules point at (`playing` or `paused`), unless dismissed.
    pub deduced: Option<String>,
}

/// Minutes now and a week ago of every product of the active accounts, summed over accounts.
/// A week-ago value exists only when some account has a snapshot from then.
pub(crate) const HOURS_SQL: &str = "
    SELECT t.store, t.native_id, SUM(t.minutes),
           SUM((SELECT s.minutes FROM playtime_snapshot s
                WHERE s.account_id = t.account_id AND s.store = t.store
                  AND s.native_id = t.native_id AND s.observed_at <= ?1
                ORDER BY s.observed_at DESC LIMIT 1))
    FROM playtime t
    JOIN connection c ON c.provider = t.store AND c.account_id = t.account_id
    GROUP BY t.store, t.native_id";

pub fn diary(c: &Connection, now: i64) -> Result<Vec<DiaryRow>, rusqlite::Error> {
    type Key = (String, String);
    let last = activity(c)?;
    let hours: BTreeMap<Key, (Option<i64>, Option<i64>)> = c
        .prepare(HOURS_SQL)?
        .query_map([now - WEEK], |r| {
            Ok(((r.get(0)?, r.get(1)?), (r.get(2)?, r.get(3)?)))
        })?
        .collect::<Result<_, _>>()?;
    let plays: BTreeMap<Key, (String, i64)> = c
        .prepare("SELECT store, native_id, state, since FROM play")?
        .query_map([], |r| Ok(((r.get(0)?, r.get(1)?), (r.get(2)?, r.get(3)?))))?
        .collect::<Result<_, _>>()?;
    let dismissed: BTreeSet<(String, String, String)> = c
        .prepare("SELECT store, native_id, target FROM suggestion_dismissed")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    let mut held = c.prepare(
        "SELECT EXISTS(SELECT 1 FROM entitlement e
                JOIN connection cn ON cn.provider = e.store AND cn.account_id = e.account_id
                WHERE e.store = ?1 AND e.native_id = ?2)",
    )?;
    let mut product = c.prepare(
        "SELECT title, COALESCE(kind,'unknown'), COALESCE(windows,'unknown'), url, image_url
         FROM product WHERE store = ?1 AND native_id = ?2",
    )?;

    let played = hours
        .iter()
        .filter(|(_, (m, _))| m.is_some_and(|m| m > 0))
        .map(|(k, _)| k);
    let mut keys: BTreeSet<&Key> = plays.keys().collect();
    for k in played {
        if held.query_row(params![k.0, k.1], |r| r.get::<_, bool>(0))? {
            keys.insert(k);
        }
    }

    let mut out = Vec::with_capacity(keys.len());
    for key in keys {
        let (minutes, before) = hours.get(key).copied().unwrap_or((None, None));
        let at = last.get(key).copied();
        let play = plays.get(key);
        let deduced = match play {
            Some(_) => None,
            None => rule(None, at, now)
                .map(|(t, _)| t)
                .filter(|t| !dismissed.contains(&(key.0.clone(), key.1.clone(), (*t).to_owned())))
                .map(str::to_owned),
        };
        let details = product
            .query_map(params![key.0, key.1], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .next()
            .transpose()?;
        let (title, kind, windows, url, image_url) =
            details.unwrap_or((None, "unknown".into(), "unknown".into(), None, None));
        out.push(DiaryRow {
            product: ProductRef {
                store: key.0.clone(),
                native_id: key.1.clone(),
                title,
                kind,
                windows,
                url,
                image_url,
            },
            minutes,
            last_activity: at,
            week_minutes: minutes.zip(before).map(|(m, b)| (m - b).max(0)),
            state: play.map(|(s, _)| s.clone()),
            deduced,
        });
    }
    out.sort_by(|a, b| {
        b.last_activity
            .cmp(&a.last_activity)
            .then(b.minutes.cmp(&a.minutes))
    });
    Ok(out)
}

#[cfg(test)]
mod tests;
