//! Phase 1 Epic diagnostics. Nothing here returns a token or a cookie.

use super::api::{self, EpicError, Tokens};
use crate::secrets::SecretStore;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const SECRET_NAME: &str = "epic";
const LOGIN_LABEL: &str = "login-epic";

pub struct EpicState {
    pub http: reqwest::Client,
    pub store: SecretStore,
    pub refresh_lock: tokio::sync::Mutex<()>,
}

impl EpicState {
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

    fn load(&self) -> Result<Option<Tokens>, DiagError> {
        match self.store.get(SECRET_NAME).map_err(DiagError::internal)? {
            Some(b) => serde_json::from_slice(&b)
                .map(Some)
                .map_err(DiagError::internal),
            None => Ok(None),
        }
    }

    /// Linked account and its display name, for the Connections view.
    pub fn linked(&self) -> Option<(String, Option<String>)> {
        self.load()
            .ok()
            .flatten()
            .map(|t| (t.account_id, t.display_name))
    }

    fn save(&self, t: &Tokens) -> Result<(), DiagError> {
        let b = serde_json::to_vec(t).map_err(DiagError::internal)?;
        self.store.put(SECRET_NAME, &b).map_err(DiagError::internal)
    }

    async fn fresh(&self) -> Result<Tokens, DiagError> {
        let _g = self.refresh_lock.lock().await;
        let t = self.load()?.ok_or(DiagError::not_connected())?;
        let now = crate::now_secs();
        if !t.needs_refresh(now) {
            return Ok(t);
        }
        if !t.refresh_usable(now) {
            return Err(DiagError::needs_reauth());
        }
        match api::refresh(&self.http, &t).await {
            Ok(n) => {
                self.save(&n)?;
                Ok(n)
            }
            Err(e) => Err(DiagError::from_epic(e)),
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
            message: "Epic is not connected".into(),
        }
    }
    fn needs_reauth() -> Self {
        Self {
            code: "needs_reauth",
            message: "Reconnect Epic".into(),
        }
    }
    fn from_epic(e: EpicError) -> Self {
        match e {
            EpicError::Network(_) => Self {
                code: "network",
                message: "No network or Epic is not answering".into(),
            },
            EpicError::Rejected => Self::needs_reauth(),
            other => Self {
                code: "provider",
                message: other.to_string(),
            },
        }
    }
}

#[derive(Serialize)]
pub struct EpicStatus {
    pub connected: bool,
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    /// Hosts (never paths or queries) visited during login, to diagnose unexpected flows.
    pub navigated_hosts: Vec<String>,
    /// A different account than the one whose data is stored: that data stays apart.
    pub account_changed: bool,
}

#[derive(Serialize)]
pub struct EpicReport {
    pub account_id: String,
    pub display_name: Option<String>,
    pub windows_assets: Result<usize, String>,
    pub entitlements_total: Result<usize, String>,
    pub entitlements_by_type: Option<BTreeMap<String, usize>>,
    pub access_token_was_expired: bool,
    pub token_refreshed: bool,
    pub library_service_items: Result<usize, String>,
    pub wishlist: &'static str,
}

#[tauri::command]
pub fn epic_status(state: State<'_, EpicState>) -> Result<EpicStatus, DiagError> {
    Ok(match state.load()? {
        Some(t) => EpicStatus {
            connected: true,
            account_id: Some(t.account_id),
            display_name: t.display_name,
            navigated_hosts: vec![],
            account_changed: false,
        },
        None => EpicStatus {
            connected: false,
            account_id: None,
            display_name: None,
            navigated_hosts: vec![],
            account_changed: false,
        },
    })
}

#[tauri::command]
pub async fn epic_connect(
    app: AppHandle,
    state: State<'_, EpicState>,
    db: State<'_, crate::AppDb>,
) -> Result<EpicStatus, DiagError> {
    if let Some(w) = app.get_webview_window(LOGIN_LABEL) {
        let _ = w.set_focus();
        return Err(DiagError {
            code: "in_progress",
            message: "A Epic sign-in window is already open".into(),
        });
    }
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(DiagError::internal)?
        .join("webview")
        .join("epic");

    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let nav_tx = tx.clone();
    let hosts = Arc::new(Mutex::new(Vec::<String>::new()));
    let nav_hosts = hosts.clone();

    // No capability targets this window: the provider page cannot call app commands.
    let win = WebviewWindowBuilder::new(&app, LOGIN_LABEL, WebviewUrl::External(api::login_url()))
        .title("CrossShelf · Epic")
        .inner_size(520.0, 780.0)
        .data_directory(dir)
        .on_navigation(move |url| {
            if let (Some(h), Ok(mut v)) = (url.host_str(), nav_hosts.lock())
                && v.last().map(String::as_str) != Some(h)
                && v.len() < 40
            {
                v.push(h.to_owned());
            }
            // The landing page renders the code: let it load, then read it natively below.
            if api::is_redirect_page(url)
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

    let landed = rx.await.unwrap_or(false);
    let seen = hosts.lock().map(|h| h.clone()).unwrap_or_default();
    let mut code = None;
    if landed {
        // The page may need a moment to render its JSON.
        for _ in 0..12 {
            tokio::time::sleep(std::time::Duration::from_millis(700)).await;
            if let Ok(text) = super::webview::body_text(&win).await
                && let Some(c) = api::parse_redirect_body(&text)
            {
                code = Some(c);
                break;
            }
        }
    }
    let _ = win.destroy();
    let Some(code) = code else {
        return Err(DiagError {
            code: if landed { "no_code" } else { "cancelled" },
            message: format!(
                "{}. Hosts visitados: {}",
                if landed {
                    "Epic did not return a readable code"
                } else {
                    "Sign-in cancelled"
                },
                seen.join(" > ")
            ),
        });
    };
    let tokens = api::exchange_code(&state.http, &code)
        .await
        .map_err(DiagError::from_epic)?;
    state.save(&tokens)?;
    let change = {
        let conn =
            db.0.lock()
                .map_err(|_| DiagError::internal("database lock poisoned"))?;
        crate::connections::record_connection(
            &conn,
            "epic",
            &tokens.account_id,
            tokens.display_name.as_deref(),
        )
        .map_err(DiagError::internal)?
    };
    Ok(EpicStatus {
        account_changed: change == crate::connections::AccountChange::Changed,
        connected: true,
        account_id: Some(tokens.account_id),
        display_name: tokens.display_name,
        navigated_hosts: seen,
    })
}

/// Test aid: pretend the access token expired so the next call must refresh.
#[tauri::command]
pub fn epic_diag_expire(state: State<'_, EpicState>) -> Result<(), DiagError> {
    let mut t = state.load()?.ok_or(DiagError::not_connected())?;
    t.obtained_at = t.obtained_at.saturating_sub(t.expires_in + 1);
    state.save(&t)
}

#[tauri::command]
pub async fn epic_diag_run(state: State<'_, EpicState>) -> Result<EpicReport, DiagError> {
    let before = state.load()?.map(|t| t.obtained_at);
    let was_expired = state
        .load()?
        .is_some_and(|t| t.needs_refresh(crate::now_secs()));
    let t = state.fresh().await?;
    let token_refreshed = before.is_some_and(|b| b != t.obtained_at);

    let assets = api::fetch_assets(&state.http, &t.access_token).await;
    let ents = api::fetch_entitlements(&state.http, &t.account_id, &t.access_token).await;
    let lib_service = api::library_service_count(&state.http, &t.access_token).await;
    Ok(EpicReport {
        account_id: t.account_id.clone(),
        display_name: t.display_name.clone(),
        windows_assets: assets.map(|a| a.len()).map_err(|e| e.to_string()),
        entitlements_by_type: ents.as_ref().ok().map(|e| api::count_by_type(e)),
        entitlements_total: ents.map(|e| e.len()).map_err(|e| e.to_string()),
        access_token_was_expired: was_expired,
        token_refreshed,
        library_service_items: lib_service.map_err(|e| e.to_string()),
        wishlist: "not implemented: needs the store web session (a separate capability)",
    })
}

#[tauri::command]
pub fn epic_disconnect(
    app: AppHandle,
    state: State<'_, EpicState>,
    db: State<'_, crate::AppDb>,
) -> Result<(), DiagError> {
    state
        .store
        .delete(SECRET_NAME)
        .map_err(DiagError::internal)?;
    if let Ok(conn) = db.0.lock() {
        crate::connections::mark_disconnected(&conn, "epic").map_err(DiagError::internal)?;
    }
    if let Ok(dir) = app.path().app_local_data_dir() {
        let _ = std::fs::remove_dir_all(dir.join("webview").join("epic"));
    }
    Ok(())
}

#[derive(Serialize)]
pub struct EpicSyncReport {
    pub library: &'static str,
    pub library_items: usize,
    pub games: usize,
    /// Items whose catalog entry did not arrive: kept with unknown title and kind.
    pub without_catalog: usize,
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

/// Reads the library (`library-service`, every page) and its catalog entries, and stores them by
/// generation. A failed or unfinished library read keeps the last copy. A catalog chunk that
/// fails only leaves those titles unknown: the rights themselves were read in full.
#[tauri::command]
pub async fn epic_sync(
    app: AppHandle,
    state: State<'_, EpicState>,
    db: State<'_, crate::AppDb>,
) -> Result<EpicSyncReport, DiagError> {
    use crate::sync::{self, ReadOutcome, Resource};
    let t = state.fresh().await?;
    let now = crate::now_secs() as i64;
    let lock = || {
        db.0.lock()
            .map_err(|_| DiagError::internal("database lock poisoned"))
    };
    let account_changed = crate::connections::record_connection(
        &*lock()?,
        "epic",
        &t.account_id,
        t.display_name.as_deref(),
    )
    .map_err(DiagError::internal)?
        == crate::connections::AccountChange::Changed;
    let data_dir = app.path().app_local_data_dir().ok();
    let keep_raw = |resource: &str, items: &[serde_json::Value]| {
        if let Some(d) = &data_dir {
            crate::raw::save(d, "epic", resource, now, items);
        }
    };

    let records = api::library_service_raw(&state.http, &t.access_token).await;
    let mut error = None;
    let mut catalog = Vec::new();
    if let Ok(recs) = &records {
        keep_raw("library", recs);
        let requests = super::sync::catalog_requests(recs);
        let (items, _, first_error) = fetch_catalog(&state.http, &t.access_token, requests).await;
        catalog = items;
        error = first_error.map(|e| format!("catalog: {e}"));
        keep_raw("catalog", &catalog);
    }

    let mut conn = lock()?;
    let report = match records {
        Ok(recs) => {
            let (products, rights) = super::sync::map_library(&recs, &catalog);
            let games = products.iter().filter(|p| p.kind == "game").count();
            let without_catalog = products.iter().filter(|p| p.title.is_none()).count();
            // `library_service_raw` only returns after the last page: a valid answer is complete,
            // and an empty one is Epic saying the library is empty.
            let c = sync::commit_entitlements(
                &mut conn,
                super::sync::STORE,
                &t.account_id,
                Resource::Library,
                &products,
                &rights,
                &ReadOutcome::Complete,
                true,
                now,
            )
            .map_err(DiagError::internal)?;
            EpicSyncReport {
                library: stored_label(c.stored),
                library_items: c.item_count,
                games,
                without_catalog,
                account_changed,
                error,
            }
        }
        Err(e) => {
            let c = sync::commit_entitlements(
                &mut conn,
                super::sync::STORE,
                &t.account_id,
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
            EpicSyncReport {
                library: stored_label(c.stored),
                library_items: 0,
                games: 0,
                without_catalog: 0,
                account_changed,
                error: Some(e.to_string()),
            }
        }
    };
    crate::connections::touch_sync(&conn, "epic", now).map_err(DiagError::internal)?;
    Ok(report)
}

#[derive(Serialize)]
pub struct EpicShapes {
    pub library_service: Result<crate::shape::ShapeReport, String>,
}

/// Field names, types and low-cardinality counts of the raw responses. No titles, no IDs.
#[tauri::command]
pub async fn epic_diag_shape(state: State<'_, EpicState>) -> Result<EpicShapes, DiagError> {
    let t = state.fresh().await?;
    Ok(EpicShapes {
        library_service: api::library_service_raw(&state.http, &t.access_token)
            .await
            .map(|v| crate::shape::shape_report(&v))
            .map_err(|e| e.to_string()),
    })
}

#[derive(Serialize)]
pub struct EpicCatalogShape {
    pub sampled: usize,
    pub returned: usize,
    pub failed_namespaces: usize,
    pub catalog: Result<crate::shape::ShapeReport, String>,
}

/// Catalog requests in flight at once. Epic answers one namespace per request and a library has
/// hundreds of them: in series the whole lookup took two minutes on a real 571-item library.
const CATALOG_PARALLEL: usize = 6;

/// Every catalog entry for the given namespaces, a few requests at a time. A failed chunk only
/// leaves its items without catalog data; the count of failed chunks and the first error come back.
async fn fetch_catalog(
    http: &reqwest::Client,
    bearer: &str,
    requests: std::collections::BTreeMap<String, Vec<String>>,
) -> (Vec<serde_json::Value>, usize, Option<String>) {
    let mut jobs = requests.into_iter().flat_map(|(ns, ids)| {
        ids.chunks(api::CATALOG_CHUNK)
            .map(|c| (ns.clone(), c.to_vec()))
            .collect::<Vec<_>>()
    });
    let mut running = tokio::task::JoinSet::new();
    let (mut items, mut failed, mut first_error) = (Vec::new(), 0, None);
    loop {
        while running.len() < CATALOG_PARALLEL {
            let Some((ns, ids)) = jobs.next() else { break };
            let (http, bearer) = (http.clone(), bearer.to_owned());
            running.spawn(async move {
                let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
                api::catalog_bulk_raw(&http, &bearer, &ns, &refs).await
            });
        }
        let Some(done) = running.join_next().await else {
            break;
        };
        match done {
            Ok(Ok(mut v)) => items.append(&mut v),
            Ok(Err(e)) => {
                failed += 1;
                first_error.get_or_insert_with(|| e.to_string());
            }
            Err(e) => {
                failed += 1;
                first_error.get_or_insert_with(|| e.to_string());
            }
        }
    }
    (items, failed, first_error)
}

/// Looks up catalog data for the whole library and reports its shape. Both answers are kept in
/// the local raw folder (like every sync) so the mapping can be built from what Epic really sends.
#[tauri::command]
pub async fn epic_diag_catalog(
    app: AppHandle,
    state: State<'_, EpicState>,
) -> Result<EpicCatalogShape, DiagError> {
    let t = state.fresh().await?;
    let records = api::library_service_raw(&state.http, &t.access_token)
        .await
        .map_err(DiagError::from_epic)?;
    let now = crate::now_secs() as i64;
    let data_dir = app.path().app_local_data_dir().ok();
    if let Some(d) = &data_dir {
        crate::raw::save(d, "epic", "library", now, &records);
    }
    let by_ns = super::sync::catalog_requests(&records);
    let sampled = by_ns.values().map(Vec::len).sum();
    let (items, failed, _) = fetch_catalog(&state.http, &t.access_token, by_ns).await;
    if let Some(d) = &data_dir {
        crate::raw::save(d, "epic", "catalog", now, &items);
    }
    Ok(EpicCatalogShape {
        sampled,
        returned: items.len(),
        failed_namespaces: failed,
        catalog: Ok(crate::shape::shape_report(&items)),
    })
}
