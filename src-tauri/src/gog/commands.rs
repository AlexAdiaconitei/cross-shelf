//! Diagnostic commands for the phase 1 GOG spike. Nothing here returns a token or a cookie.

use super::auth::{self, AuthError, Tokens};
use super::library::{self, ReadError};
use crate::secrets::SecretStore;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const SECRET_NAME: &str = "gog";
const LOGIN_LABEL: &str = "login-gog";

pub struct GogState {
    pub http: reqwest::Client,
    pub store: SecretStore,
    /// Serialises token refreshes so two callers never use the same refresh token (spec 5.3).
    pub refresh_lock: tokio::sync::Mutex<()>,
}

impl GogState {
    pub fn new(store: SecretStore, user_agent: &str) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("http client");
        Self {
            http,
            store,
            refresh_lock: tokio::sync::Mutex::new(()),
        }
    }

    fn load(&self) -> Result<Option<Tokens>, String> {
        match self.store.get(SECRET_NAME).map_err(|e| e.to_string())? {
            Some(b) => serde_json::from_slice(&b)
                .map(Some)
                .map_err(|e| e.to_string()),
            None => Ok(None),
        }
    }

    /// Linked account (GOG user id), for the Connections view.
    pub fn linked(&self) -> Option<(String, Option<String>)> {
        self.load().ok().flatten().map(|t| (t.user_id, None))
    }

    fn save(&self, t: &Tokens) -> Result<(), String> {
        let b = serde_json::to_vec(t).map_err(|e| e.to_string())?;
        self.store.put(SECRET_NAME, &b).map_err(|e| e.to_string())
    }

    /// Valid tokens, refreshing and persisting them first when needed.
    pub(crate) async fn fresh(&self) -> Result<Tokens, DiagError> {
        let _g = self.refresh_lock.lock().await;
        let t = self
            .load()
            .map_err(DiagError::internal)?
            .ok_or(DiagError::not_connected())?;
        if !t.needs_refresh(auth::now_secs()) {
            return Ok(t);
        }
        match auth::refresh(&self.http, &t).await {
            Ok(n) => {
                self.save(&n).map_err(DiagError::internal)?;
                Ok(n)
            }
            Err(AuthError::Rejected(_)) => Err(DiagError::needs_reauth()),
            Err(e) => Err(DiagError::from_auth(e)),
        }
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
    fn not_connected() -> Self {
        Self {
            code: "not_connected",
            message: "GOG is not connected".into(),
        }
    }
    fn needs_reauth() -> Self {
        Self {
            code: "needs_reauth",
            message: "Reconnect GOG".into(),
        }
    }
    fn from_auth(e: AuthError) -> Self {
        match e {
            AuthError::Network(_) => Self {
                code: "network",
                message: "No network or GOG is not answering".into(),
            },
            AuthError::Rejected(_) => Self::needs_reauth(),
            other => Self {
                code: "provider",
                message: other.to_string(),
            },
        }
    }
    fn from_read(e: ReadError) -> Self {
        match e {
            ReadError::Network(_) => Self {
                code: "network",
                message: "No network or GOG is not answering".into(),
            },
            ReadError::Unauthorized(_) => Self::needs_reauth(),
            other => Self {
                code: "provider",
                message: other.to_string(),
            },
        }
    }
}

#[derive(Serialize)]
pub struct GogStatus {
    pub connected: bool,
    pub user_id: Option<String>,
    pub expires_at: Option<u64>,
    /// A different account than the one whose data is stored: that data stays apart.
    pub account_changed: bool,
}

#[derive(Serialize)]
pub struct GogReport {
    pub user_id: String,
    pub username: String,
    pub library_total: usize,
    pub library_pages: usize,
    pub library_by_platform: BTreeMap<String, usize>,
    pub wishlist_count: usize,
    pub wishlist_error: Option<String>,
    pub access_token_was_expired: bool,
    pub token_refreshed: bool,
    /// Product IDs only (no titles), to check what the wishlist endpoint returns.
    pub wishlist_ids: Vec<String>,
    /// Count of entries per JSON value in the wishlist map, e.g. {"true": 1}.
    pub wishlist_value_kinds: BTreeMap<String, usize>,
}

#[tauri::command]
pub fn gog_status(state: State<'_, GogState>) -> Result<GogStatus, DiagError> {
    Ok(match state.load().map_err(DiagError::internal)? {
        Some(t) => GogStatus {
            connected: true,
            user_id: Some(t.user_id.clone()),
            expires_at: Some(t.expires_at()),
            account_changed: false,
        },
        None => GogStatus {
            connected: false,
            user_id: None,
            expires_at: None,
            account_changed: false,
        },
    })
}

/// Opens the real GOG login page in its own window and waits for the redirect.
#[tauri::command]
pub async fn gog_connect(
    app: AppHandle,
    state: State<'_, GogState>,
    db: State<'_, crate::AppDb>,
) -> Result<GogStatus, DiagError> {
    if let Some(w) = app.get_webview_window(LOGIN_LABEL) {
        let _ = w.set_focus();
        return Err(DiagError {
            code: "in_progress",
            message: "A GOG sign-in window is already open".into(),
        });
    }
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(DiagError::internal)?
        .join("webview")
        .join("gog");

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let nav_tx = tx.clone();

    // No capability targets this window: the provider page cannot call app commands.
    let win = WebviewWindowBuilder::new(&app, LOGIN_LABEL, WebviewUrl::External(auth::login_url()))
        .title("CrossShelf · GOG")
        .inner_size(520.0, 760.0)
        .data_directory(dir)
        .on_navigation(move |url| {
            if let Some(code) = auth::parse_return_url(url) {
                if let Some(s) = nav_tx.lock().ok().and_then(|mut g| g.take()) {
                    let _ = s.send(Some(code));
                }
                return false;
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
            let _ = s.send(None);
        }
    });

    let outcome = rx.await;
    let _ = win.destroy();
    let code = match outcome {
        Ok(Some(c)) => c,
        _ => {
            return Err(DiagError {
                code: "cancelled",
                message: "Sign-in cancelled".into(),
            });
        }
    };
    let tokens = auth::exchange_code(&state.http, &code)
        .await
        .map_err(DiagError::from_auth)?;
    state.save(&tokens).map_err(DiagError::internal)?;
    let name = library::fetch_username(&state.http, &tokens.access_token)
        .await
        .ok();
    let change = {
        let conn =
            db.0.lock()
                .map_err(|_| DiagError::internal("database lock poisoned"))?;
        crate::connections::record_connection(&conn, "gog", &tokens.user_id, name.as_deref())
            .map_err(DiagError::internal)?
    };
    Ok(GogStatus {
        connected: true,
        user_id: Some(tokens.user_id.clone()),
        expires_at: Some(tokens.expires_at()),
        account_changed: change == crate::connections::AccountChange::Changed,
    })
}

/// Test aid: pretend the access token expired so the next call must refresh.
#[tauri::command]
pub fn gog_diag_expire(state: State<'_, GogState>) -> Result<(), DiagError> {
    let mut t = state
        .load()
        .map_err(DiagError::internal)?
        .ok_or(DiagError::not_connected())?;
    t.login_time = 0;
    state.save(&t).map_err(DiagError::internal)
}

/// Reads identity, the whole library and the wishlist, and returns counts only.
#[tauri::command]
pub async fn gog_diag_run(state: State<'_, GogState>) -> Result<GogReport, DiagError> {
    let before = state
        .load()
        .map_err(DiagError::internal)?
        .map(|t| t.login_time);
    let was_expired = state
        .load()
        .map_err(DiagError::internal)?
        .is_some_and(|t| t.needs_refresh(auth::now_secs()));
    let t = state.fresh().await?;
    let token_refreshed = before.is_some_and(|b| b != t.login_time);

    let username = library::fetch_username(&state.http, &t.access_token)
        .await
        .map_err(DiagError::from_read)?;
    let (items, pages) = library::fetch_library(&state.http, &t.user_id, &t.access_token)
        .await
        .map_err(DiagError::from_read)?;
    let (wishlist_ids, wishlist_error) =
        match library::fetch_wishlist_ids(&state.http, &t.access_token).await {
            Ok(ids) => (ids, None),
            Err(e) => (vec![], Some(e.to_string())),
        };
    let wishlist_count = wishlist_ids.len();
    let wishlist_value_kinds = library::fetch_wishlist_body(&state.http, &t.access_token)
        .await
        .map(|b| library::wishlist_value_kinds(&b))
        .unwrap_or_default();
    Ok(GogReport {
        user_id: t.user_id.clone(),
        username,
        library_total: items.len(),
        library_pages: pages,
        library_by_platform: library::count_by_platform(&items),
        wishlist_count,
        wishlist_error,
        access_token_was_expired: was_expired,
        token_refreshed,
        wishlist_ids: wishlist_ids.into_iter().take(20).collect(),
        wishlist_value_kinds,
    })
}

/// Deletes the secret and the provider web profile. Does not claim remote revocation.
#[tauri::command]
pub fn gog_disconnect(
    app: AppHandle,
    state: State<'_, GogState>,
    db: State<'_, crate::AppDb>,
) -> Result<(), DiagError> {
    state
        .store
        .delete(SECRET_NAME)
        .map_err(DiagError::internal)?;
    if let Ok(conn) = db.0.lock() {
        crate::connections::mark_disconnected(&conn, "gog").map_err(DiagError::internal)?;
    }
    if let Ok(dir) = app.path().app_local_data_dir() {
        let _ = std::fs::remove_dir_all(dir.join("webview").join("gog"));
    }
    Ok(())
}

#[derive(Serialize)]
pub struct GogShapes {
    pub galaxy_library: Result<crate::shape::ShapeReport, String>,
    pub filtered_products: Result<crate::shape::ShapeReport, String>,
}

/// Field names, types and low-cardinality counts of the raw responses. No titles, no IDs.
#[tauri::command]
pub async fn gog_diag_shape(state: State<'_, GogState>) -> Result<GogShapes, DiagError> {
    let t = state.fresh().await?;
    let rep = |r: Result<Vec<serde_json::Value>, ReadError>| {
        r.map(|v| crate::shape::shape_report(&v))
            .map_err(|e| e.to_string())
    };
    Ok(GogShapes {
        galaxy_library: rep(
            library::fetch_library_raw(&state.http, &t.user_id, &t.access_token).await,
        ),
        filtered_products: rep(
            library::fetch_filtered_products_raw(&state.http, &t.access_token).await,
        ),
    })
}

#[derive(Serialize)]
pub struct GogSyncReport {
    pub library: &'static str,
    pub library_items: usize,
    pub games: usize,
    pub movies: usize,
    pub wishlist: &'static str,
    pub wishlist_items: usize,
    pub account_changed: bool,
    pub error: Option<String>,
}

fn stored_label(s: crate::sync::Stored) -> &'static str {
    use crate::sync::Stored::*;
    match s {
        Complete => "complete",
        Partial => "partial",
        Failed => "failed",
        FailedEmptyUnconfirmed => "failed_empty_unconfirmed",
    }
}

/// Reads library and wishlist and stores them by generation. A failed read keeps what was there.
#[tauri::command]
pub async fn gog_sync(
    app: AppHandle,
    state: State<'_, GogState>,
    db: State<'_, crate::AppDb>,
) -> Result<GogSyncReport, DiagError> {
    use crate::sync::{self, ReadOutcome, Resource};
    let t = state.fresh().await?;
    let now = crate::now_secs() as i64;
    let account = t.user_id.clone();
    let name = library::fetch_username(&state.http, &t.access_token)
        .await
        .ok();
    let lock = || {
        db.0.lock()
            .map_err(|_| DiagError::internal("database lock poisoned"))
    };
    let account_changed =
        crate::connections::record_connection(&*lock()?, "gog", &account, name.as_deref())
            .map_err(DiagError::internal)?
            == crate::connections::AccountChange::Changed;

    let data_dir = app.path().app_local_data_dir().ok();
    let keep_raw = |resource: &str, items: &[serde_json::Value]| {
        if let Some(d) = &data_dir {
            crate::raw::save(d, "gog", resource, now, items);
        }
    };
    let lib = library::fetch_filtered_products_raw(&state.http, &t.access_token).await;
    if let Ok(records) = &lib {
        keep_raw("library", records);
    }
    let mut error = None;
    let (lib_label, lib_items, games, movies) = {
        let mut conn = lock()?;
        match lib {
            Ok(records) => {
                let (products, rights) = super::sync::map_products(&records);
                let games = products.iter().filter(|p| p.kind == "game").count();
                let movies = products.iter().filter(|p| p.kind == "movie").count();
                let c = sync::commit_entitlements(
                    &mut conn,
                    super::sync::store(),
                    &account,
                    Resource::Library,
                    &products,
                    &rights,
                    &ReadOutcome::Complete,
                    false,
                    now,
                )
                .map_err(DiagError::internal)?;
                (stored_label(c.stored), c.item_count, games, movies)
            }
            Err(e) => {
                error = Some(e.to_string());
                let c = sync::commit_entitlements(
                    &mut conn,
                    super::sync::store(),
                    &account,
                    Resource::Library,
                    &[],
                    &[],
                    &ReadOutcome::Failed,
                    false,
                    now,
                )
                .map_err(DiagError::internal)?;
                sync::set_generation_error(&conn, c.generation_id, e.code())
                    .map_err(DiagError::internal)?;
                (stored_label(c.stored), 0, 0, 0)
            }
        }
    };

    let wish = library::fetch_wishlist_body(&state.http, &t.access_token)
        .await
        .and_then(|body| {
            keep_raw("wishlist", &library::wishlist_raw_entries(&body));
            library::parse_wishlist(&body)
        });
    // Wishlist entries come without a title: ask the public catalog. A failure only leaves the
    // title unknown.
    let mut catalog = Vec::new();
    if let Ok(ids) = &wish {
        for chunk in ids.chunks(library::CATALOG_CHUNK) {
            match library::fetch_catalog(&state.http, chunk).await {
                Ok(records) => catalog.extend(super::sync::map_catalog(&records)),
                Err(e) => {
                    error.get_or_insert_with(|| format!("catalog: {e}"));
                    break;
                }
            }
        }
    }
    let (wish_label, wish_items) = {
        let mut conn = lock()?;
        sync::update_products(&mut conn, super::sync::store(), &catalog, now)
            .map_err(DiagError::internal)?;
        let c = match wish {
            // `fetch_wishlist_ids` only succeeds on a valid answer, so an empty list is confirmed.
            Ok(ids) => sync::commit_wishlist(
                &mut conn,
                super::sync::store(),
                &account,
                &[],
                &super::sync::map_wishlist(&ids),
                &ReadOutcome::Complete,
                true,
                now,
            )
            .map_err(DiagError::internal)?,
            Err(e) => {
                error.get_or_insert_with(|| e.to_string());
                let c = sync::commit_wishlist(
                    &mut conn,
                    super::sync::store(),
                    &account,
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
        crate::connections::touch_sync(&conn, "gog", now).map_err(DiagError::internal)?;
        (stored_label(c.stored), c.item_count)
    };
    Ok(GogSyncReport {
        library: lib_label,
        library_items: lib_items,
        games,
        movies,
        wishlist: wish_label,
        wishlist_items: wish_items,
        account_changed,
        error,
    })
}

/// A GOG play time read is reused for this long before asking again (plan 0.3.0, 2.4).
const PLAYTIME_REUSE_SECS: i64 = 3600;

/// Minutes played of one GOG product, asked for when its sheet opens. `None` means GOG does not
/// give it (not validated yet, or no Galaxy record). Only the product id crosses from the UI; the
/// session stays here.
#[tauri::command]
pub async fn gog_playtime(
    state: State<'_, GogState>,
    db: State<'_, crate::AppDb>,
    native_id: String,
) -> Result<Option<i64>, DiagError> {
    if !super::PLAYTIME_VALIDATED {
        return Ok(None);
    }
    if native_id.is_empty() || !native_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(DiagError::internal("bad id"));
    }
    let Some((account, _)) = state.linked() else {
        return Err(DiagError::not_connected());
    };
    let now = crate::now_secs() as i64;
    let lock = || {
        db.0.lock()
            .map_err(|_| DiagError::internal("database lock poisoned"))
    };
    let cached = crate::playtime::recent(
        &*lock()?,
        "gog",
        &account,
        &native_id,
        now,
        PLAYTIME_REUSE_SECS,
    )
    .map_err(DiagError::internal)?;
    if let Some(minutes) = cached {
        return Ok(minutes);
    }
    let t = state.fresh().await?;
    let minutes = library::fetch_playtime(&state.http, &t.user_id, &t.access_token, &native_id)
        .await
        .map_err(|e| {
            log::warn!("gog playtime: {}", e.code());
            DiagError::from_read(e)
        })?;
    let row = crate::playtime::PlaytimeRow {
        native_id,
        minutes,
        last_played: None,
    };
    crate::playtime::record(&mut *lock()?, "gog", &t.user_id, &[row], false, now)
        .map_err(DiagError::internal)?;
    Ok(minutes)
}

#[derive(Serialize)]
pub struct GogPlaytimeCheck {
    /// Title and hours (or `None`: no Galaxy record) of a few owned games, to compare with Galaxy.
    pub sample: Vec<(Option<String>, Option<f64>)>,
}

/// Validation tool for GOG play time (`docs/validation/gog-playtime.md`): asks for the time of the
/// first five owned games, whatever `PLAYTIME_VALIDATED` says, and stores nothing.
#[tauri::command]
pub async fn gog_diag_playtime(
    state: State<'_, GogState>,
    db: State<'_, crate::AppDb>,
) -> Result<GogPlaytimeCheck, DiagError> {
    let t = state.fresh().await?;
    let games: Vec<(String, Option<String>)> = {
        let conn =
            db.0.lock()
                .map_err(|_| DiagError::internal("database lock poisoned"))?;
        let mut st = conn
            .prepare(
                "SELECT e.native_id, p.title FROM entitlement e
                 LEFT JOIN product p ON p.store = e.store AND p.native_id = e.native_id
                 WHERE e.store = 'gog' AND e.account_id = ?1 AND e.class = 'owned'
                 ORDER BY p.title LIMIT 5",
            )
            .map_err(DiagError::internal)?;
        st.query_map([&t.user_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .and_then(Iterator::collect)
            .map_err(DiagError::internal)?
    };
    let mut sample = Vec::new();
    for (id, title) in games {
        let minutes = library::fetch_playtime(&state.http, &t.user_id, &t.access_token, &id)
            .await
            .map_err(DiagError::from_read)?;
        sample.push((title, minutes.map(|m| m as f64 / 60.0)));
    }
    Ok(GogPlaytimeCheck { sample })
}
