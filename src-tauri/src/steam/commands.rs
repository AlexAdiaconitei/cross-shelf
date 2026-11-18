//! Phase 1 Steam diagnostics. Nothing here returns a token or a cookie.

use super::api::{self, Session, SteamError};
use crate::secrets::SecretStore;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use url::Url;

const SECRET_NAME: &str = "steam";
const LOGIN_LABEL: &str = "login-steam";

pub struct SteamState {
    pub http: reqwest::Client,
    pub store: SecretStore,
}

impl SteamState {
    pub fn new(store: SecretStore, user_agent: &str) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("http client");
        Self { http, store }
    }

    pub(crate) fn load(&self) -> Result<Option<Session>, DiagError> {
        match self.store.get(SECRET_NAME).map_err(DiagError::internal)? {
            Some(b) => serde_json::from_slice(&b)
                .map(Some)
                .map_err(DiagError::internal),
            None => Ok(None),
        }
    }

    /// Linked account (SteamID64), without its name, for the Connections view.
    pub fn linked(&self) -> Option<(String, Option<String>)> {
        self.load().ok().flatten().map(|s| (s.steamid, None))
    }

    fn save(&self, s: &Session) -> Result<(), DiagError> {
        let b = serde_json::to_vec(s).map_err(DiagError::internal)?;
        self.store.put(SECRET_NAME, &b).map_err(DiagError::internal)
    }
}

#[derive(Debug, Serialize)]
pub struct DiagError {
    pub code: &'static str,
    pub message: String,
}

impl DiagError {
    fn internal(m: impl ToString) -> Self {
        Self {
            code: "internal",
            message: m.to_string(),
        }
    }
    fn from_steam(e: SteamError) -> Self {
        match e {
            SteamError::Network(_) => Self {
                code: "network",
                message: "No network or Steam is not answering".into(),
            },
            SteamError::NeedsReauth => Self {
                code: "needs_reauth",
                message: "Reconnect Steam".into(),
            },
            SteamError::Private => Self {
                code: "private",
                message: "Private list or no permission".into(),
            },
            other => Self {
                code: "provider",
                message: other.to_string(),
            },
        }
    }
}

#[derive(Serialize)]
pub struct SteamStatus {
    pub connected: bool,
    pub steamid: Option<String>,
    /// A different account than the one whose data is stored: that data stays apart.
    pub account_changed: bool,
}

#[derive(Serialize)]
pub struct SteamReport {
    pub steamid: String,
    pub owned_games: Result<usize, String>,
    pub wishlist_items: Result<usize, String>,
    /// `None` inside Ok means the account is in no family group.
    pub family_group_found: Result<bool, String>,
    pub family_shared_apps: Option<usize>,
    pub family_multi_owner_apps: Option<usize>,
    pub session_renewed: bool,
}

fn msg<T>(r: Result<T, SteamError>) -> Result<T, String> {
    r.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn steam_status(state: State<'_, SteamState>) -> Result<SteamStatus, DiagError> {
    Ok(match state.load()? {
        Some(s) => SteamStatus {
            connected: true,
            steamid: Some(s.steamid),
            account_changed: false,
        },
        None => SteamStatus {
            connected: false,
            steamid: None,
            account_changed: false,
        },
    })
}

/// Names only, never values: tells whether the login cookies reached us.
fn cookie_names(win: &tauri::WebviewWindow) -> Vec<String> {
    let url = Url::parse("https://store.steampowered.com/").expect("static url");
    win.cookies_for_url(url)
        .map(|c| c.iter().map(|c| c.name().to_owned()).collect())
        .unwrap_or_default()
}

fn cookie_header(win: &tauri::WebviewWindow) -> Result<String, DiagError> {
    let url = Url::parse("https://store.steampowered.com/").expect("static url");
    let cookies = win.cookies_for_url(url).map_err(DiagError::internal)?;
    Ok(cookies
        .iter()
        .map(|c| format!("{}={}", c.name(), c.value()))
        .collect::<Vec<_>>()
        .join("; "))
}

#[tauri::command]
pub async fn steam_connect(
    app: AppHandle,
    state: State<'_, SteamState>,
    db: State<'_, crate::AppDb>,
) -> Result<SteamStatus, DiagError> {
    if let Some(w) = app.get_webview_window(LOGIN_LABEL) {
        let _ = w.set_focus();
        return Err(DiagError {
            code: "in_progress",
            message: "A Steam sign-in window is already open".into(),
        });
    }
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(DiagError::internal)?
        .join("webview")
        .join("steam");

    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let nav_tx = tx.clone();

    let win = WebviewWindowBuilder::new(&app, LOGIN_LABEL, WebviewUrl::External(api::login_url()))
        .title("CrossShelf · Steam")
        .inner_size(520.0, 780.0)
        .data_directory(dir)
        .on_navigation(move |url| {
            // Let the page load: the session cookies are final once it has.
            if api::is_return_url(url)
                && let Some(s) = nav_tx.lock().ok().and_then(|mut g| g.take())
            {
                let _ = s.send(true);
            }
            true
        })
        .build()
        .map_err(DiagError::internal)?;

    let close_tx = tx.clone();
    win.on_window_event(move |e| {
        if !matches!(
            e,
            WindowEvent::Destroyed | WindowEvent::CloseRequested { .. }
        ) {
            return;
        }
        if let Some(s) = close_tx.lock().ok().and_then(|mut g| g.take()) {
            let _ = s.send(false);
        }
    });

    let finished = rx.await.unwrap_or(false);
    if !finished {
        let _ = win.destroy();
        return Err(DiagError {
            code: "cancelled",
            message: "Sign-in cancelled".into(),
        });
    }
    // Give the redirect target a moment to set its final cookies.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let header = cookie_header(&win);
    let names = cookie_names(&win);
    let _ = win.destroy();
    let header = header?;
    let session = match api::fetch_session(&state.http, &header).await {
        Ok(s) => s,
        Err(e) => {
            let detail = api::probe_config(&state.http, &header).await;
            let mut err = DiagError::from_steam(e);
            err.message = format!(
                "{}. Cookies presentes (nombres): [{}]. {}",
                err.message,
                names.join(","),
                detail
            );
            return Err(err);
        }
    };
    state.save(&session)?;
    let name = api::persona_name(&state.http, &session).await;
    let change = {
        let conn =
            db.0.lock()
                .map_err(|_| DiagError::internal("database lock poisoned"))?;
        crate::connections::record_connection(&conn, "steam", &session.steamid, name.as_deref())
            .map_err(DiagError::internal)?
    };
    Ok(SteamStatus {
        connected: true,
        steamid: Some(session.steamid),
        account_changed: change == crate::connections::AccountChange::Changed,
    })
}

#[tauri::command]
pub async fn steam_diag_run(state: State<'_, SteamState>) -> Result<SteamReport, DiagError> {
    let stored = state.load()?.ok_or(DiagError {
        code: "not_connected",
        message: "Steam is not connected".into(),
    })?;
    // The web API token is short-lived: re-obtain it from the stored cookies on every run.
    let session = api::fetch_session(&state.http, &stored.cookie_header)
        .await
        .map_err(DiagError::from_steam)?;
    let renewed = session.webapi_token != stored.webapi_token;
    state.save(&session)?;

    let owned = msg(api::owned_games_count(&state.http, &session).await);
    let wish = msg(api::wishlist_count(&state.http, &session).await);
    let group = api::family_group(&state.http, &session).await;
    let (found, shared, multi) = match &group {
        Ok(Some(g)) => match api::shared_library_counts(&state.http, &session, g).await {
            Ok((n, m)) => (Ok(true), Some(n), Some(m)),
            Err(e) => (Err(e.to_string()), None, None),
        },
        Ok(None) => (Ok(false), None, None),
        Err(e) => (Err(e.to_string()), None, None),
    };
    Ok(SteamReport {
        steamid: session.steamid.clone(),
        owned_games: owned,
        wishlist_items: wish,
        family_group_found: found,
        family_shared_apps: shared,
        family_multi_owner_apps: multi,
        session_renewed: renewed,
    })
}

#[tauri::command]
pub fn steam_disconnect(
    app: AppHandle,
    state: State<'_, SteamState>,
    db: State<'_, crate::AppDb>,
) -> Result<(), DiagError> {
    state
        .store
        .delete(SECRET_NAME)
        .map_err(DiagError::internal)?;
    if let Ok(conn) = db.0.lock() {
        crate::connections::mark_disconnected(&conn, "steam").map_err(DiagError::internal)?;
    }
    if let Ok(dir) = app.path().app_local_data_dir() {
        let _ = std::fs::remove_dir_all(dir.join("webview").join("steam"));
    }
    Ok(())
}

#[derive(Serialize)]
pub struct SteamShapes {
    pub owned_games: Result<crate::shape::ShapeReport, String>,
    pub wishlist: Result<crate::shape::ShapeReport, String>,
    pub family_apps: Result<crate::shape::ShapeReport, String>,
}

/// Field names, types and low-cardinality counts of the raw responses. No titles, no IDs.
#[tauri::command]
pub async fn steam_diag_shape(state: State<'_, SteamState>) -> Result<SteamShapes, DiagError> {
    let stored = state.load()?.ok_or(DiagError {
        code: "not_connected",
        message: "Steam is not connected".into(),
    })?;
    let session = api::fetch_session(&state.http, &stored.cookie_header)
        .await
        .map_err(DiagError::from_steam)?;
    state.save(&session)?;
    let rep = |r: Result<Vec<serde_json::Value>, SteamError>| {
        r.map(|v| crate::shape::shape_report(&v))
            .map_err(|e| e.to_string())
    };
    let owned = rep(api::owned_games_raw(&state.http, &session).await);
    let wish = rep(api::wishlist_raw(&state.http, &session).await);
    let family = match api::family_group(&state.http, &session).await {
        Ok(Some(g)) => rep(api::shared_apps_raw(&state.http, &session, &g).await),
        Ok(None) => Err("no family group".into()),
        Err(e) => Err(e.to_string()),
    };
    Ok(SteamShapes {
        owned_games: owned,
        wishlist: wish,
        family_apps: family,
    })
}

#[derive(Serialize)]
pub struct SteamTypeShapes {
    pub requested: usize,
    pub owned_sample: Result<crate::shape::ShapeReport, String>,
    pub wishlist_sample: Result<crate::shape::ShapeReport, String>,
}

/// Asks the store for type and name of a sample of owned and wished apps and reports the shape.
#[tauri::command]
pub async fn steam_diag_types(state: State<'_, SteamState>) -> Result<SteamTypeShapes, DiagError> {
    let stored = state.load()?.ok_or(DiagError {
        code: "not_connected",
        message: "Steam is not connected".into(),
    })?;
    let session = api::fetch_session(&state.http, &stored.cookie_header)
        .await
        .map_err(DiagError::from_steam)?;
    state.save(&session)?;
    let ids = |v: Result<Vec<serde_json::Value>, SteamError>| -> Vec<u64> {
        v.unwrap_or_default()
            .iter()
            .filter_map(|g| g.get("appid").and_then(|a| a.as_u64()))
            .take(100)
            .collect()
    };
    let owned_ids = ids(api::owned_games_raw(&state.http, &session).await);
    let wish_ids = ids(api::wishlist_raw(&state.http, &session).await);
    let rep = |r: Result<Vec<serde_json::Value>, SteamError>| {
        r.map(|v| crate::shape::shape_report(&v))
            .map_err(|e| e.to_string())
    };
    Ok(SteamTypeShapes {
        requested: owned_ids.len() + wish_ids.len(),
        owned_sample: rep(api::store_items_raw(&state.http, &owned_ids).await),
        wishlist_sample: rep(api::store_items_raw(&state.http, &wish_ids).await),
    })
}

#[derive(Serialize)]
pub struct SteamSyncReport {
    pub library: &'static str,
    pub library_items: usize,
    pub family: &'static str,
    pub family_shared_apps: usize,
    pub wishlist: &'static str,
    pub wishlist_items: usize,
    pub account_changed: bool,
    pub errors: Vec<String>,
}

fn label(s: crate::sync::Stored) -> &'static str {
    use crate::sync::Stored::*;
    match s {
        Complete => "complete",
        Partial => "partial",
        Failed => "failed",
        FailedEmptyUnconfirmed => "failed_empty_unconfirmed",
    }
}

type Rows = (
    Vec<crate::sync::ProductRow>,
    Vec<crate::sync::EntitlementRow>,
);

/// Stores one entitlement read (or its failure with a plain code) and returns (label, count).
fn store_rights(
    db: &crate::AppDb,
    me: &str,
    resource: crate::sync::Resource,
    read: Result<Rows, SteamError>,
    errors: &mut Vec<String>,
    now: i64,
) -> Result<(&'static str, usize), DiagError> {
    use crate::sync::{self, ReadOutcome};
    let mut conn =
        db.0.lock()
            .map_err(|_| DiagError::internal("database lock poisoned"))?;
    let c = match read {
        // Every Steam read here answers in one response: a valid one is complete and confirmed.
        Ok((p, r)) => sync::commit_entitlements(
            &mut conn,
            super::sync::STORE,
            me,
            resource,
            &p,
            &r,
            &ReadOutcome::Complete,
            true,
            now,
        )
        .map_err(DiagError::internal)?,
        Err(e) => {
            errors.push(format!("{resource:?}: {e}"));
            let c = sync::commit_entitlements(
                &mut conn,
                super::sync::STORE,
                me,
                resource,
                &[],
                &[],
                &ReadOutcome::Failed,
                false,
                now,
            )
            .map_err(DiagError::internal)?;
            sync::set_generation_error(&conn, c.generation_id, e.code())
                .map_err(DiagError::internal)?;
            c
        }
    };
    Ok((label(c.stored), c.item_count))
}

fn appids(items: &[serde_json::Value]) -> Vec<u64> {
    items
        .iter()
        .filter_map(|g| g.get("appid").and_then(|a| a.as_u64()))
        .collect()
}

/// Store records for every id, a chunk at a time. Stops at the first failed chunk and returns
/// what arrived before it with the error.
async fn store_details(
    http: &reqwest::Client,
    ids: &[u64],
) -> (Vec<crate::sync::ProductRow>, Option<SteamError>) {
    let mut rows = Vec::new();
    for chunk in ids.chunks(api::STORE_ITEMS_CHUNK) {
        match api::store_items_raw(http, chunk).await {
            Ok(items) => rows.extend(super::sync::map_store_items(&items)),
            Err(e) => return (rows, Some(e)),
        }
    }
    (rows, None)
}

/// Reads library, family and wishlist and stores each by generation. A failure in one keeps the
/// last copy of that one and does not stop the others.
#[tauri::command]
pub async fn steam_sync(
    app: AppHandle,
    state: State<'_, SteamState>,
    db: State<'_, crate::AppDb>,
) -> Result<SteamSyncReport, DiagError> {
    use crate::sync::{self, ReadOutcome, Resource};
    let stored = state.load()?.ok_or(DiagError {
        code: "not_connected",
        message: "Steam is not connected".into(),
    })?;
    let session = api::fetch_session(&state.http, &stored.cookie_header)
        .await
        .map_err(DiagError::from_steam)?;
    state.save(&session)?;
    let now = crate::now_secs() as i64;
    let me = session.steamid.clone();
    let name = api::persona_name(&state.http, &session).await;
    let account_changed = {
        let conn =
            db.0.lock()
                .map_err(|_| DiagError::internal("database lock poisoned"))?;
        crate::connections::record_connection(&conn, "steam", &me, name.as_deref())
            .map_err(DiagError::internal)?
            == crate::connections::AccountChange::Changed
    };
    let mut errors = Vec::new();
    let data_dir = app.path().app_local_data_dir().ok();
    let keep_raw = |resource: &str, items: &[serde_json::Value]| {
        if let Some(d) = &data_dir {
            crate::raw::save(d, "steam", resource, now, items);
        }
    };

    let owned = api::owned_games_raw(&state.http, &session).await;
    if let Ok(g) = &owned {
        keep_raw("library", g);
    }
    let mut detail_ids = appids(owned.as_deref().unwrap_or_default());
    let owned = owned.map(|g| super::sync::map_owned(&g));
    let (library, library_items) =
        store_rights(&db, &me, Resource::Library, owned, &mut errors, now)?;

    // Not belonging to any group is a normal, confirmed state (spec 6.2): it clears family rights.
    let family_read = match api::family_group(&state.http, &session).await {
        Ok(None) => Ok((vec![], vec![])),
        Ok(Some(g)) => api::shared_apps_raw(&state.http, &session, &g)
            .await
            .inspect(|apps| {
                keep_raw("family", apps);
                detail_ids.extend(appids(apps));
            })
            .map(|apps| super::sync::map_family(&apps, &me)),
        Err(e) => Err(e),
    };
    let owner_ids = family_read
        .as_ref()
        .map(|(_, rows)| sync::family_owner_ids(rows))
        .unwrap_or_default();
    let (family, family_shared_apps) =
        store_rights(&db, &me, Resource::Family, family_read, &mut errors, now)?;

    // Who shares each game, by public profile name. Best effort: a private profile stays unnamed.
    let mut names = Vec::new();
    for id in &owner_ids {
        if let Some(n) = api::public_name(&state.http, id).await {
            names.push((id.clone(), n));
        }
    }
    if !names.is_empty() {
        let conn =
            db.0.lock()
                .map_err(|_| DiagError::internal("database lock poisoned"))?;
        sync::remember_family_names(&conn, &names, now).map_err(DiagError::internal)?;
    }

    let wish = api::wishlist_raw(&state.http, &session).await;
    if let Ok(items) = &wish {
        keep_raw("wishlist", items);
        detail_ids.extend(appids(items));
    }
    let (wishlist, wishlist_items) = {
        let mut conn =
            db.0.lock()
                .map_err(|_| DiagError::internal("database lock poisoned"))?;
        let c = match wish {
            Ok(items) => sync::commit_wishlist(
                &mut conn,
                super::sync::STORE,
                &me,
                &[],
                &super::sync::map_wishlist(&items),
                &ReadOutcome::Complete,
                true,
                now,
            )
            .map_err(DiagError::internal)?,
            Err(e) => {
                errors.push(format!("wishlist: {e}"));
                let c = sync::commit_wishlist(
                    &mut conn,
                    super::sync::STORE,
                    &me,
                    &[],
                    &[],
                    &ReadOutcome::Failed,
                    false,
                    now,
                )
                .map_err(DiagError::internal)?;
                sync::set_generation_error(&conn, c.generation_id, e.code())
                    .map_err(DiagError::internal)?;
                c
            }
        };
        (label(c.stored), c.item_count)
    };

    // Names, types and Windows support from the public catalog. Wishlist entries come without a
    // name and owned games without a type. A failure here only leaves those unknown.
    detail_ids.sort_unstable();
    detail_ids.dedup();
    let (details, detail_error) = store_details(&state.http, &detail_ids).await;
    if let Some(e) = detail_error {
        errors.push(format!("catalog: {e}"));
    }
    {
        let mut conn =
            db.0.lock()
                .map_err(|_| DiagError::internal("database lock poisoned"))?;
        sync::update_products(&mut conn, super::sync::STORE, &details, now)
            .map_err(DiagError::internal)?;
        crate::connections::touch_sync(&conn, "steam", now).map_err(DiagError::internal)?;
    }
    Ok(SteamSyncReport {
        library,
        library_items,
        family,
        family_shared_apps,
        wishlist,
        wishlist_items,
        account_changed,
        errors,
    })
}
