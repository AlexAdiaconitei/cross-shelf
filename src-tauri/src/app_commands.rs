//! Commands of the app itself, not of one store: connections overview, settings, export and
//! local data (spec 9.1). None of them returns a token or a cookie.

use crate::connections::{self, ConnectionView};
use crate::{AppDb, with_db};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use ts_rs::TS;

/// Reconciles the stored secrets with the `connection` rows and returns every store's state.
#[tauri::command]
pub fn connections_list(
    steam: State<'_, crate::steam::commands::SteamState>,
    gog: State<'_, crate::gog::commands::GogState>,
    epic: State<'_, crate::epic::commands::EpicState>,
    db: State<'_, AppDb>,
) -> Result<Vec<ConnectionView>, String> {
    let linked = [
        ("steam", steam.linked()),
        ("gog", gog.linked()),
        ("epic", epic.linked()),
    ];
    with_db(&db, |c| {
        for (provider, link) in &linked {
            match link {
                Some((id, name)) => {
                    connections::record_connection(c, provider, id, name.as_deref())?;
                }
                None => connections::mark_disconnected(c, provider)?,
            }
        }
        connections::PROVIDERS
            .iter()
            .map(|p| connections::view(c, p))
            .collect()
    })
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct Settings {
    /// Count Steam family access as a separate reason in cleanup. Off by default (spec 6.2).
    pub include_family_in_cleanup: bool,
    /// Draw store covers from the store CDNs. On by default; off shows initials only.
    pub show_covers: bool,
    /// Interface language: `en` (default) or `es`.
    pub language: String,
    /// Last tab of Mis partidas: `board` (default), `list` or `diary`.
    pub plays_tab: String,
}

const KEY_FAMILY: &str = "include_family_in_cleanup";
const KEY_COVERS: &str = "show_covers";
const KEY_LANGUAGE: &str = "language";
const KEY_PLAYS_TAB: &str = "plays_tab";

/// Tabs of Mis partidas; anything else falls back to the board.
pub const PLAYS_TABS: &[&str] = &["board", "list", "diary"];

fn plays_tab_or_default(v: Option<String>) -> String {
    v.filter(|t| PLAYS_TABS.contains(&t.as_str()))
        .unwrap_or_else(|| "board".into())
}

fn get_text(c: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    c.query_row("SELECT value FROM setting WHERE key = ?1", [key], |r| {
        r.get::<_, String>(0)
    })
    .optional()
}

/// Languages the interface ships with; anything else falls back to English.
pub const LANGUAGES: &[&str] = &["en", "es"];

fn language_or_default(v: Option<String>) -> String {
    v.filter(|l| LANGUAGES.contains(&l.as_str()))
        .unwrap_or_else(|| "en".into())
}

fn get_bool(c: &Connection, key: &str, default: bool) -> Result<bool, rusqlite::Error> {
    Ok(
        c.query_row("SELECT value FROM setting WHERE key = ?1", [key], |r| {
            r.get::<_, String>(0)
        })
        .optional()?
        .map_or(default, |v| v == "true"),
    )
}

pub fn read_settings(c: &Connection) -> Result<Settings, rusqlite::Error> {
    Ok(Settings {
        include_family_in_cleanup: get_bool(c, KEY_FAMILY, false)?,
        show_covers: get_bool(c, KEY_COVERS, true)?,
        language: language_or_default(get_text(c, KEY_LANGUAGE)?),
        plays_tab: plays_tab_or_default(get_text(c, KEY_PLAYS_TAB)?),
    })
}

pub fn write_settings(c: &Connection, s: &Settings) -> Result<(), rusqlite::Error> {
    c.execute(
        "INSERT INTO setting(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![KEY_FAMILY, s.include_family_in_cleanup.to_string()],
    )?;
    c.execute(
        "INSERT INTO setting(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![KEY_COVERS, s.show_covers.to_string()],
    )?;
    c.execute(
        "INSERT INTO setting(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![KEY_LANGUAGE, language_or_default(Some(s.language.clone()))],
    )?;
    c.execute(
        "INSERT INTO setting(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![
            KEY_PLAYS_TAB,
            plays_tab_or_default(Some(s.plays_tab.clone()))
        ],
    )?;
    Ok(())
}

#[tauri::command]
pub fn settings_get(db: State<'_, AppDb>) -> Result<Settings, String> {
    with_db(&db, read_settings)
}

#[tauri::command]
pub fn settings_set(db: State<'_, AppDb>, settings: Settings) -> Result<Settings, String> {
    with_db(&db, |c| {
        write_settings(c, &settings)?;
        read_settings(c)
    })
}

#[derive(Debug, Deserialize, Clone, Copy, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ExportWhat {
    Library,
    Wishlist,
}

#[derive(Debug, Deserialize, Clone, Copy, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Csv,
    Json,
}

/// Writes the active accounts' library or wishlist to a path the user picked in the save dialog.
/// Returns the number of rows written.
#[tauri::command]
pub fn export_data(
    db: State<'_, AppDb>,
    what: ExportWhat,
    format: ExportFormat,
    path: String,
) -> Result<usize, String> {
    use crate::export;
    let path = std::path::PathBuf::from(path);
    let ext = match format {
        ExportFormat::Csv => "csv",
        ExportFormat::Json => "json",
    };
    export::check_destination(&path, ext).map_err(|e| e.to_string())?;
    let table = match what {
        ExportWhat::Library => export::library_table(&with_db(&db, crate::queries::list_library)?),
        ExportWhat::Wishlist => {
            export::wishlist_table(&with_db(&db, crate::queries::list_wishlist)?)
        }
    };
    let body = match format {
        ExportFormat::Csv => table.csv(),
        ExportFormat::Json => table.json(),
    };
    export::write_atomic(&path, &body).map_err(|e| e.to_string())?;
    log::info!("export written: {} rows", table.rows.len());
    Ok(table.rows.len())
}

/// Opens the app's local data folder in the file explorer.
#[tauri::command]
pub fn data_open_folder(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// Empties the cache file (libraries, wishlists, products, play time, sync history). Connections,
/// their secrets and everything the user decided stay: the next sync reads everything again.
pub fn clear_synced(c: &Connection) -> Result<(), rusqlite::Error> {
    crate::db::clear_cache(c)
}

#[tauri::command]
pub fn data_clear_cache(db: State<'_, AppDb>) -> Result<(), String> {
    with_db(&db, clear_synced)
}

/// Every wishlist entry of the active accounts with what it might be covered by (spec 7, 8.1).
#[tauri::command]
pub fn review_list(db: State<'_, AppDb>) -> Result<Vec<crate::review::ReviewItem>, String> {
    with_db(&db, |c| {
        let s = read_settings(c)?;
        crate::review::build(c, s.include_family_in_cleanup)
    })
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Same,
    Different,
    /// Forget the user's decision on this pair.
    Clear,
}

#[derive(Debug, Deserialize, Clone, TS)]
#[ts(export)]
pub struct ProductId {
    pub store: String,
    pub native_id: String,
}

fn known_store(s: &str) -> Result<(), String> {
    if connections::PROVIDERS.contains(&s) {
        Ok(())
    } else {
        Err("tienda desconocida".into())
    }
}

/// "Es el mismo" / "Son diferentes" on a pair of products. Persistent across syncs.
#[tauri::command]
pub fn review_decide(
    db: State<'_, AppDb>,
    wanted: ProductId,
    held: ProductId,
    decision: Decision,
) -> Result<(), String> {
    known_store(&wanted.store)?;
    known_store(&held.store)?;
    let d = match decision {
        Decision::Same => Some("same"),
        Decision::Different => Some("different"),
        Decision::Clear => None,
    };
    with_db(&db, |c| {
        crate::review::set_manual(
            c,
            (&wanted.store, &wanted.native_id),
            (&held.store, &held.native_id),
            d,
            crate::now_secs() as i64,
        )
    })
}

/// "Mantener en esta wishlist" on one entry of one account.
#[tauri::command]
pub fn review_keep(
    db: State<'_, AppDb>,
    account_id: String,
    entry: ProductId,
    keep: bool,
    reason: Option<String>,
) -> Result<(), String> {
    known_store(&entry.store)?;
    with_db(&db, |c| {
        crate::review::set_keep(
            c,
            &account_id,
            &entry.store,
            &entry.native_id,
            keep,
            reason.as_deref(),
            crate::now_secs() as i64,
        )
    })
}

fn plan_call<T>(
    db: &State<'_, AppDb>,
    f: impl FnOnce(&mut Connection) -> Result<T, crate::plans::PlanError>,
) -> Result<T, String> {
    let mut conn = db.0.lock().map_err(|_| "internal".to_string())?;
    f(&mut conn).map_err(|e| {
        log::warn!("plan refused: {}", e.code());
        e.code().to_string()
    })
}

#[tauri::command]
pub fn plans_list(db: State<'_, AppDb>) -> Result<Vec<crate::plans::BatchView>, String> {
    with_db(&db, crate::plans::list)
}

/// A draft batch for one store and account. Errors are plain codes the UI words.
#[tauri::command]
pub fn plan_create(
    db: State<'_, AppDb>,
    store: String,
    account_id: String,
    native_ids: Vec<String>,
) -> Result<crate::plans::BatchView, String> {
    plan_call(&db, |c| {
        let family = read_settings(c)?.include_family_in_cleanup;
        crate::plans::require_covered(c, &store, &account_id, &native_ids, family)?;
        crate::plans::create(
            c,
            &store,
            &account_id,
            &native_ids,
            crate::now_secs() as i64,
        )
    })
}

#[tauri::command]
pub fn plan_confirm(
    db: State<'_, AppDb>,
    batch_id: String,
    revision: String,
) -> Result<crate::plans::BatchView, String> {
    plan_call(&db, |c| {
        // A decision undone between the review and the confirmation takes the entry out again.
        let b = crate::plans::get(c, &batch_id)?.ok_or(crate::plans::PlanError::BadState)?;
        let ids: Vec<String> = b.ops.iter().map(|o| o.native_id.clone()).collect();
        let family = read_settings(c)?.include_family_in_cleanup;
        crate::plans::require_covered(c, &b.store, &b.account_id, &ids, family)?;
        crate::plans::confirm(c, &batch_id, &revision, crate::now_secs() as i64)
    })
}

/// Cancels what a confirmed batch has not sent yet.
#[tauri::command]
pub fn plan_cancel(
    db: State<'_, AppDb>,
    batch_id: String,
) -> Result<crate::plans::BatchView, String> {
    plan_call(&db, |c| {
        crate::plans::cancel(c, &batch_id, crate::now_secs() as i64)
    })
}

/// Puts the failed operations of a finished batch back to pending; `plan_execute` sends them.
#[tauri::command]
pub fn plan_retry(
    db: State<'_, AppDb>,
    batch_id: String,
) -> Result<crate::plans::BatchView, String> {
    plan_call(&db, |c| crate::plans::retry_failed(c, &batch_id))
}

#[tauri::command]
pub fn plan_reconcile(
    db: State<'_, AppDb>,
    batch_id: String,
) -> Result<crate::plans::BatchView, String> {
    plan_call(&db, |c| {
        crate::plans::reconcile(c, &batch_id, crate::now_secs() as i64)
    })
}

/// Sends a confirmed batch of a store whose write is validated (only GOG today), one operation
/// at a time. Each operation reads the wishlist, sends only if needed and reads it back
/// (`ops::run`); its result is stored before the next one starts, so an interrupted batch never
/// sends twice what already finished.
#[tauri::command]
pub async fn plan_execute(
    db: State<'_, AppDb>,
    gog: State<'_, crate::gog::commands::GogState>,
    steam: State<'_, crate::steam::commands::SteamState>,
    batch_id: String,
) -> Result<crate::plans::BatchView, String> {
    use crate::ops::{Desired, Ledger, Operation, Writer};
    let (store, account, pending) = plan_call(&db, |c| crate::plans::executable(c, &batch_id))?;
    // A writer factory for the batch's store. The session must belong to the account the batch
    // was reviewed for.
    let make: Box<dyn Fn() -> Box<dyn Writer + Send> + Send + Sync> = match store.as_str() {
        "gog" => {
            let t = gog.fresh().await.map_err(|e| e.code.to_string())?;
            if t.user_id != account {
                return Err("account_not_active".into());
            }
            let http = gog.http.clone();
            Box::new(move || {
                Box::new(crate::writers::GogWriter {
                    http: http.clone(),
                    bearer: t.access_token.clone(),
                })
            })
        }
        "steam" => {
            let stored = steam
                .load()
                .map_err(|e| e.code.to_string())?
                .ok_or("not_connected")?;
            let session = crate::steam::api::fetch_session(&steam.http, &stored.cookie_header)
                .await
                .map_err(|e| e.code().to_string())?;
            if session.steamid != account {
                return Err("account_not_active".into());
            }
            let http = steam.http.clone();
            Box::new(move || {
                Box::new(crate::writers::SteamWriter {
                    http: http.clone(),
                    session: session.clone(),
                })
            })
        }
        _ => return Err("store_unsupported".into()),
    };
    let mut ledger = Ledger::default();
    for (op_id, native_id) in pending {
        let mut w = make();
        let op = Operation {
            id: op_id.clone(),
            product_id: native_id,
            desired: Desired::Absent,
        };
        let (status, back) = tauri::async_runtime::spawn_blocking(move || {
            let s = crate::ops::run(w.as_mut(), &op, &mut ledger, &|| false, &mut |d| {
                std::thread::sleep(d)
            });
            (s, ledger)
        })
        .await
        .map_err(|_| "internal".to_string())?;
        ledger = back;
        let code = crate::writers::status_code(status);
        log::info!("batch op on {store}: {code}");
        plan_call(&db, |c| {
            crate::plans::record_result(c, &batch_id, &op_id, code, crate::now_secs() as i64)
        })?;
        if code == "needs_reauth" {
            break;
        }
    }
    plan_call(&db, |c| {
        crate::plans::get(c, &batch_id)?.ok_or(crate::plans::PlanError::BadState)
    })
}

#[tauri::command]
pub fn plan_discard(db: State<'_, AppDb>, batch_id: String) -> Result<(), String> {
    plan_call(&db, |c| crate::plans::discard(c, &batch_id))
}

fn play_call<T>(
    db: &State<'_, AppDb>,
    f: impl FnOnce(&mut Connection) -> Result<T, crate::plays::PlayError>,
) -> Result<T, String> {
    let mut conn = db.0.lock().map_err(|_| "internal".to_string())?;
    f(&mut conn).map_err(|e| {
        log::warn!("play refused: {}", e.code());
        e.code().to_string()
    })
}

/// Every play of Mis partidas, by state and in the user's order.
#[tauri::command]
pub fn plays_list(db: State<'_, AppDb>) -> Result<Vec<crate::plays::PlayView>, String> {
    with_db(&db, crate::plays::list)
}

/// Creates or moves a play: to the top of `state`, or ahead of `before` in that state. Returns
/// the whole list so the board redraws from one answer.
#[tauri::command]
pub fn play_set(
    db: State<'_, AppDb>,
    store: String,
    native_id: String,
    state: String,
    before: Option<ProductId>,
) -> Result<Vec<crate::plays::PlayView>, String> {
    play_call(&db, |c| {
        crate::plays::set(
            c,
            &store,
            &native_id,
            &state,
            before
                .as_ref()
                .map(|b| (b.store.as_str(), b.native_id.as_str())),
            crate::now_secs() as i64,
        )?;
        Ok(crate::plays::list(c)?)
    })
}

#[tauri::command]
pub fn play_note(
    db: State<'_, AppDb>,
    store: String,
    native_id: String,
    note: String,
) -> Result<(), String> {
    play_call(&db, |c| {
        crate::plays::note(c, &store, &native_id, &note, crate::now_secs() as i64)
    })
}

/// State changes the play time hints at, for the tray above the board. Nothing changes until the
/// user accepts one (`play_set`) or dismisses it.
#[tauri::command]
pub fn suggestions_list(
    db: State<'_, AppDb>,
) -> Result<Vec<crate::plays::suggestions::Suggestion>, String> {
    with_db(&db, |c| {
        crate::plays::suggestions::suggestions(c, crate::now_secs() as i64)
    })
}

/// The diary tab: games with a play or hours, by last activity, with the last week's hours.
#[tauri::command]
pub fn diary_list(db: State<'_, AppDb>) -> Result<Vec<crate::plays::diary::DiaryRow>, String> {
    with_db(&db, |c| {
        crate::plays::diary::diary(c, crate::now_secs() as i64)
    })
}

/// Play time of the products of one game sheet, from the cache (Steam, and Epic once validated).
/// GOG is asked for apart, by `gog_playtime`, when the sheet opens.
#[tauri::command]
pub fn playtime_of(
    db: State<'_, AppDb>,
    products: Vec<ProductId>,
) -> Result<Vec<crate::playtime::ProductTime>, String> {
    let keys: Vec<(&str, &str)> = products
        .iter()
        .map(|p| (p.store.as_str(), p.native_id.as_str()))
        .collect();
    with_db(&db, |c| crate::playtime::of_products(c, &keys))
}

/// Which stores give play time in this build. Epic and GOG only once validated with a real
/// account (`docs/validation/`); the sheet says "<store> does not give it" otherwise.
#[tauri::command]
pub fn playtime_stores() -> Vec<String> {
    let mut v = vec!["steam".to_owned()];
    if crate::epic::PLAYTIME_VALIDATED {
        v.push("epic".into());
    }
    if crate::gog::PLAYTIME_VALIDATED {
        v.push("gog".into());
    }
    v
}

#[tauri::command]
pub fn suggestion_dismiss(
    db: State<'_, AppDb>,
    store: String,
    native_id: String,
    target: String,
) -> Result<(), String> {
    play_call(&db, |c| {
        crate::plays::suggestions::dismiss(c, &store, &native_id, &target, crate::now_secs() as i64)
    })
}

#[tauri::command]
pub fn play_remove(db: State<'_, AppDb>, store: String, native_id: String) -> Result<(), String> {
    play_call(&db, |c| crate::plays::remove(c, &store, &native_id))
}

#[derive(Debug, Serialize, Clone, TS)]
#[ts(export)]
pub struct WriteTestResult {
    /// `applied`, `already_in_desired_state`, `failed`, `needs_reauth`, `not_supported`,
    /// `unverified` or `cancelled_not_sent`.
    pub status: String,
}

/// Validation tool (spec 6.4, 8.3): one add or one remove of one product the user chose, through
/// the same state machine a batch would use, then a reread. Only reachable from diagnostics, and
/// only the user runs it, with their own explicit decision for every mutation.
#[tauri::command]
pub async fn wishlist_write_test(
    steam: State<'_, crate::steam::commands::SteamState>,
    gog: State<'_, crate::gog::commands::GogState>,
    store: String,
    native_id: String,
    present: bool,
) -> Result<WriteTestResult, String> {
    use crate::ops::{Desired, Ledger, Operation};
    if native_id.is_empty() || !native_id.chars().all(|c| c.is_ascii_digit()) {
        return Err("bad_id".into());
    }
    let desired = if present {
        Desired::Present
    } else {
        Desired::Absent
    };
    let op = Operation {
        id: format!("test-{store}-{native_id}-{}", crate::now_secs()),
        product_id: native_id,
        desired,
    };
    let status = match store.as_str() {
        "steam" => {
            let stored = steam
                .load()
                .map_err(|e| e.code.to_string())?
                .ok_or("not_connected")?;
            let session = crate::steam::api::fetch_session(&steam.http, &stored.cookie_header)
                .await
                .map_err(|e| e.code().to_string())?;
            let mut w = crate::writers::SteamWriter {
                http: steam.http.clone(),
                session,
            };
            tauri::async_runtime::spawn_blocking(move || {
                crate::ops::run(&mut w, &op, &mut Ledger::default(), &|| false, &mut |d| {
                    std::thread::sleep(d)
                })
            })
            .await
            .map_err(|_| "internal".to_string())?
        }
        "gog" => {
            let t = gog.fresh().await.map_err(|e| e.code.to_string())?;
            let mut w = crate::writers::GogWriter {
                http: gog.http.clone(),
                bearer: t.access_token,
            };
            tauri::async_runtime::spawn_blocking(move || {
                crate::ops::run(&mut w, &op, &mut Ledger::default(), &|| false, &mut |d| {
                    std::thread::sleep(d)
                })
            })
            .await
            .map_err(|_| "internal".to_string())?
        }
        _ => return Err("store_unsupported".into()),
    };
    let status = crate::writers::status_code(status).to_string();
    log::info!("wishlist write test on {store}: {status}");
    Ok(WriteTestResult { status })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::{EntitlementRow, ReadOutcome, Resource, commit_entitlements};

    fn db() -> Connection {
        let dir = tempfile::tempdir().unwrap();
        let conn = crate::db::open(&dir.path().join("t.db")).unwrap();
        std::mem::forget(dir);
        conn
    }

    #[test]
    fn family_in_cleanup_is_off_until_the_user_turns_it_on() {
        let c = db();
        assert!(!read_settings(&c).unwrap().include_family_in_cleanup);
        write_settings(
            &c,
            &Settings {
                include_family_in_cleanup: true,
                show_covers: true,
                language: "en".into(),
                plays_tab: "board".into(),
            },
        )
        .unwrap();
        assert!(read_settings(&c).unwrap().include_family_in_cleanup);
    }

    #[test]
    fn covers_are_on_until_the_user_turns_them_off() {
        let c = db();
        assert!(read_settings(&c).unwrap().show_covers);
        write_settings(
            &c,
            &Settings {
                include_family_in_cleanup: false,
                show_covers: false,
                language: "es".into(),
                plays_tab: "board".into(),
            },
        )
        .unwrap();
        let s = read_settings(&c).unwrap();
        assert!(!s.show_covers);
        assert_eq!(s.language, "es");
        assert!(!s.include_family_in_cleanup);
    }

    #[test]
    fn the_plays_tab_is_remembered_and_unknown_values_fall_back_to_the_board() {
        let c = db();
        assert_eq!(read_settings(&c).unwrap().plays_tab, "board");
        let mut s = read_settings(&c).unwrap();
        s.plays_tab = "diary".into();
        write_settings(&c, &s).unwrap();
        assert_eq!(read_settings(&c).unwrap().plays_tab, "diary");
        s.plays_tab = "kanban".into();
        write_settings(&c, &s).unwrap();
        assert_eq!(read_settings(&c).unwrap().plays_tab, "board");
    }

    #[test]
    fn language_is_english_by_default_and_rejects_unknown_values() {
        let c = db();
        assert_eq!(read_settings(&c).unwrap().language, "en");
        write_settings(
            &c,
            &Settings {
                include_family_in_cleanup: false,
                show_covers: true,
                language: "fr".into(),
                plays_tab: "board".into(),
            },
        )
        .unwrap();
        assert_eq!(read_settings(&c).unwrap().language, "en");
    }

    #[test]
    fn clearing_the_cache_keeps_the_connection() {
        let mut c = db();
        connections::record_connection(&c, "gog", "a", Some("Ana")).unwrap();
        commit_entitlements(
            &mut c,
            "gog",
            "a",
            Resource::Library,
            &[],
            &[EntitlementRow {
                native_id: "1".into(),
                class: "owned".into(),
                owners: vec![],
                exclude_reason: None,
            }],
            &ReadOutcome::Complete,
            false,
            1,
        )
        .unwrap();
        clear_synced(&c).unwrap();
        assert!(crate::queries::list_library(&c).unwrap().is_empty());
        let v = connections::view(&c, "gog").unwrap();
        assert_eq!(v.account_id.as_deref(), Some("a"));
        assert_eq!(v.capabilities[0].state, "not_synced");
    }
}
