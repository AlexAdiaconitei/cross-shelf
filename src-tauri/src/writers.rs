//! Real wishlist writers for Steam and GOG (spec 6.4), driven by the state machine in `ops`.
//!
//! They exist so the user can validate them one product at a time from the diagnostics page.
//! A batch only uses them once `plans::write_validated` says so, which needs the record in
//! `docs/validation/<store>-write.md` (spec 8.3). Until then no batch sends anything.
//!
//! The `ops::Writer` trait is synchronous; these run on a blocking thread and wait on the async
//! HTTP calls there.

use crate::ops::{Desired, Operation, SendResult, Writer};
use std::time::Duration;

const DEFAULT_RETRY: Duration = Duration::from_secs(5);

fn retry_after(h: Option<&str>) -> Duration {
    h.and_then(|v| v.trim().parse::<u64>().ok())
        .map(|s| Duration::from_secs(s.min(120)))
        .unwrap_or(DEFAULT_RETRY)
}

fn from_error(e: &reqwest::Error) -> SendResult {
    // A connection that never opened did not send anything; anything later may have landed.
    if e.is_connect() {
        SendResult::Other
    } else {
        SendResult::Timeout
    }
}

/// Steam `api/addtowishlist` and `api/removefromwishlist` answer JSON with `success`. Success only
/// with `success: true` (or 1); a 200 without it is a semantic error.
pub fn steam_result(status: u16, retry: Option<&str>, body: &str) -> SendResult {
    match status {
        200 => {
            let ok = serde_json::from_str::<serde_json::Value>(body)
                .ok()
                .and_then(|v| v.get("success").cloned())
                .is_some_and(|s| s == serde_json::json!(true) || s == serde_json::json!(1));
            if ok {
                SendResult::Accepted
            } else {
                SendResult::SemanticError
            }
        }
        401 | 403 => SendResult::Unauthorized,
        429 => SendResult::RateLimited(retry_after(retry)),
        _ => SendResult::Other,
    }
}

/// GOG `user/wishlist/{add|remove}/{id}` returns the wishlist. Success only when the returned list
/// already has the product in the desired state.
pub fn gog_result(
    status: u16,
    retry: Option<&str>,
    body: &str,
    product: &str,
    desired: Desired,
) -> SendResult {
    match status {
        200 => match crate::gog::library::parse_wishlist(body) {
            Ok(ids) => {
                let present = ids.iter().any(|i| i == product);
                if present == (desired == Desired::Present) {
                    SendResult::Accepted
                } else {
                    SendResult::SemanticError
                }
            }
            Err(_) => SendResult::SemanticError,
        },
        401 | 403 => SendResult::Unauthorized,
        429 => SendResult::RateLimited(retry_after(retry)),
        _ => SendResult::Other,
    }
}

/// `sessionid` cookie value from a stored cookie header. Hypothesis to validate: the store's
/// `g_sessionID` equals this cookie.
pub fn steam_sessionid(cookie_header: &str) -> Option<String> {
    cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|c| c.strip_prefix("sessionid="))
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

pub struct SteamWriter {
    pub http: reqwest::Client,
    pub session: crate::steam::api::Session,
}

impl Writer for SteamWriter {
    fn read_state(&mut self, op: &Operation) -> Result<Desired, ()> {
        let items = tauri::async_runtime::block_on(crate::steam::api::wishlist_raw(
            &self.http,
            &self.session,
        ))
        .map_err(|_| ())?;
        let present = items.iter().any(|i| {
            i.get("appid")
                .map(|a| a.to_string().trim_matches('"') == op.product_id)
                == Some(true)
        });
        Ok(if present {
            Desired::Present
        } else {
            Desired::Absent
        })
    }

    fn send(&mut self, op: &Operation) -> SendResult {
        let Some(sid) = steam_sessionid(&self.session.cookie_header) else {
            return SendResult::Unauthorized;
        };
        let path = match op.desired {
            Desired::Present => "addtowishlist",
            Desired::Absent => "removefromwishlist",
        };
        let req = self
            .http
            .post(format!("https://store.steampowered.com/api/{path}"))
            .header("Cookie", &self.session.cookie_header)
            .form(&[
                ("sessionid", sid.as_str()),
                ("appid", op.product_id.as_str()),
            ]);
        tauri::async_runtime::block_on(async move {
            match req.send().await {
                Ok(r) => {
                    let status = r.status().as_u16();
                    let retry = r
                        .headers()
                        .get("retry-after")
                        .and_then(|h| h.to_str().ok())
                        .map(str::to_owned);
                    let body = r.text().await.unwrap_or_default();
                    steam_result(status, retry.as_deref(), &body)
                }
                Err(e) => from_error(&e),
            }
        })
    }
}

pub struct GogWriter {
    pub http: reqwest::Client,
    pub bearer: String,
}

impl Writer for GogWriter {
    fn read_state(&mut self, op: &Operation) -> Result<Desired, ()> {
        let ids = tauri::async_runtime::block_on(crate::gog::library::fetch_wishlist_ids(
            &self.http,
            &self.bearer,
        ))
        .map_err(|_| ())?;
        Ok(if ids.iter().any(|i| i == &op.product_id) {
            Desired::Present
        } else {
            Desired::Absent
        })
    }

    /// A GET with a write effect: it is only ever requested here, never prefetched or followed.
    fn send(&mut self, op: &Operation) -> SendResult {
        if !op.product_id.chars().all(|c| c.is_ascii_digit()) {
            return SendResult::NotSupported;
        }
        let path = match op.desired {
            Desired::Present => "add",
            Desired::Absent => "remove",
        };
        let req = self
            .http
            .get(format!(
                "https://embed.gog.com/user/wishlist/{path}/{}",
                op.product_id
            ))
            .bearer_auth(&self.bearer);
        let product = op.product_id.clone();
        let desired = op.desired;
        tauri::async_runtime::block_on(async move {
            match req.send().await {
                Ok(r) => {
                    let status = r.status().as_u16();
                    let retry = r
                        .headers()
                        .get("retry-after")
                        .and_then(|h| h.to_str().ok())
                        .map(str::to_owned);
                    let body = r.text().await.unwrap_or_default();
                    gog_result(status, retry.as_deref(), &body, &product, desired)
                }
                Err(e) => from_error(&e),
            }
        })
    }
}

pub fn status_code(s: crate::ops::OpStatus) -> &'static str {
    use crate::ops::OpStatus::*;
    match s {
        Applied => "applied",
        AlreadyInDesiredState => "already_in_desired_state",
        Failed => "failed",
        NeedsReauth => "needs_reauth",
        NotSupported => "not_supported",
        Unverified => "unverified",
        CancelledNotSent => "cancelled_not_sent",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_success_needs_the_semantic_flag() {
        assert_eq!(
            steam_result(200, None, r#"{"success":true,"wishlistCount":3}"#),
            SendResult::Accepted
        );
        assert_eq!(
            steam_result(200, None, r#"{"success":1}"#),
            SendResult::Accepted
        );
        assert_eq!(
            steam_result(200, None, r#"{"success":false}"#),
            SendResult::SemanticError
        );
        assert_eq!(steam_result(200, None, "<html>"), SendResult::SemanticError);
        assert_eq!(steam_result(403, None, ""), SendResult::Unauthorized);
        assert_eq!(
            steam_result(429, Some("12"), ""),
            SendResult::RateLimited(Duration::from_secs(12))
        );
        assert_eq!(
            steam_result(429, Some("junk"), ""),
            SendResult::RateLimited(DEFAULT_RETRY)
        );
        assert_eq!(steam_result(500, None, ""), SendResult::Other);
    }

    #[test]
    fn gog_success_only_when_the_returned_list_agrees() {
        let without = r#"{"wishlist":{"2":true},"checksum":"x"}"#;
        let with = r#"{"wishlist":{"1":true,"2":true},"checksum":"x"}"#;
        assert_eq!(
            gog_result(200, None, without, "1", Desired::Absent),
            SendResult::Accepted
        );
        assert_eq!(
            gog_result(200, None, with, "1", Desired::Absent),
            SendResult::SemanticError
        );
        assert_eq!(
            gog_result(200, None, with, "1", Desired::Present),
            SendResult::Accepted
        );
        assert_eq!(
            gog_result(200, None, "{}", "1", Desired::Absent),
            SendResult::SemanticError
        );
        assert_eq!(
            gog_result(401, None, "", "1", Desired::Absent),
            SendResult::Unauthorized
        );
    }

    #[test]
    fn the_steam_sessionid_comes_from_its_cookie_only() {
        assert_eq!(
            steam_sessionid("a=1; sessionid=abc123; steamLoginSecure=x").as_deref(),
            Some("abc123")
        );
        assert_eq!(steam_sessionid("xsessionid=1"), None);
        assert_eq!(steam_sessionid("sessionid="), None);
    }

    #[test]
    fn retry_after_is_capped() {
        assert_eq!(retry_after(Some("100000")), Duration::from_secs(120));
    }
}
