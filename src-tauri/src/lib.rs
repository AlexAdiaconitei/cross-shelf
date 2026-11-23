#[cfg(test)]
mod a18;
mod app_commands;
mod connections;
mod db;
mod epic;
mod export;
mod gog;
mod images;
mod matching;
mod ops;
mod plans;
mod plays;
mod playtime;
mod queries;
mod raw;
mod redact;
mod review;
mod secrets;
mod shape;
mod steam;
mod sync;
mod userdata;
mod writers;

use std::sync::Mutex;
use tauri::Manager;

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Shared database handle. One writer at a time; the app is single-instance.
pub struct AppDb(pub Mutex<rusqlite::Connection>);

#[derive(serde::Serialize)]
struct AppStatus {
    version: String,
    schema_version: u32,
    /// Signed builds carry the updater configuration; local and unsigned builds do not.
    updater: bool,
}

#[tauri::command]
fn app_status(app: tauri::AppHandle, db: tauri::State<AppDb>) -> Result<AppStatus, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let schema_version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    Ok(AppStatus {
        version: app.package_info().version.to_string(),
        schema_version,
        updater: app.config().plugins.0.contains_key("updater"),
    })
}

pub(crate) fn with_db<T>(
    db: &tauri::State<AppDb>,
    f: impl FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>,
) -> Result<T, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn library_list(db: tauri::State<AppDb>) -> Result<Vec<queries::LibraryEntry>, String> {
    with_db(&db, queries::list_library)
}

#[tauri::command]
fn wishlist_list(db: tauri::State<AppDb>) -> Result<Vec<queries::WishlistEntry>, String> {
    with_db(&db, queries::list_wishlist)
}

#[tauri::command]
fn sync_status_list(db: tauri::State<AppDb>) -> Result<Vec<queries::SyncStatus>, String> {
    with_db(&db, queries::sync_status)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be the first plugin: a second launch focuses the first window and exits.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}] {}",
                        record.level(),
                        record.target(),
                        redact::redact(&message.to_string())
                    ))
                })
                .build(),
        )
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // The updater needs a pubkey that only signed builds carry (scripts/updater-config.mjs
            // writes it). Without one the app runs, just without in-app updates.
            if app.config().plugins.0.contains_key("updater") {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            let dir = app.path().app_local_data_dir()?;
            let conn = db::open(&dir.join("crossshelf.db"))?;
            // A missed copy must not keep the app from starting; the next launch tries again.
            match db::daily_backup(&conn, &dir.join("backups"), now_secs() as i64) {
                Ok(Some(_)) => log::info!("daily backup of user data written"),
                Ok(None) => {}
                Err(e) => log::warn!("daily backup failed: {e}"),
            }
            app.manage(AppDb(Mutex::new(conn)));
            let ua = format!("CrossShelf/{}", app.package_info().version);
            app.manage(gog::commands::GogState::new(
                secrets::SecretStore::new(dir.join("secrets")),
                &ua,
            ));
            app.manage(epic::commands::EpicState::new(
                secrets::SecretStore::new(dir.join("secrets")),
                &ua,
            ));
            app.manage(steam::commands::SteamState::new(
                secrets::SecretStore::new(dir.join("secrets")),
                &ua,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_status,
            library_list,
            wishlist_list,
            sync_status_list,
            app_commands::connections_list,
            app_commands::settings_get,
            app_commands::settings_set,
            app_commands::export_data,
            app_commands::data_open_folder,
            app_commands::userdata_export,
            app_commands::userdata_preview,
            app_commands::userdata_import,
            app_commands::data_clear_cache,
            app_commands::review_list,
            app_commands::review_decide,
            app_commands::review_keep,
            app_commands::plans_list,
            app_commands::plan_create,
            app_commands::plan_confirm,
            app_commands::plan_reconcile,
            app_commands::plan_discard,
            app_commands::wishlist_write_test,
            app_commands::plays_list,
            app_commands::play_set,
            app_commands::play_note,
            app_commands::play_remove,
            app_commands::suggestions_list,
            app_commands::suggestion_dismiss,
            app_commands::diary_list,
            app_commands::playtime_of,
            app_commands::playtime_stores,
            gog::commands::gog_status,
            gog::commands::gog_connect,
            gog::commands::gog_diag_run,
            gog::commands::gog_diag_expire,
            gog::commands::gog_diag_shape,
            gog::commands::gog_sync,
            gog::commands::gog_disconnect,
            gog::commands::gog_playtime,
            gog::commands::gog_diag_playtime,
            epic::commands::epic_status,
            epic::commands::epic_connect,
            epic::commands::epic_diag_run,
            epic::commands::epic_diag_expire,
            epic::commands::epic_diag_shape,
            epic::commands::epic_diag_catalog,
            epic::commands::epic_diag_playtime,
            epic::commands::epic_sync,
            app_commands::plan_execute,
            app_commands::plan_cancel,
            app_commands::plan_retry,
            epic::commands::epic_disconnect,
            steam::commands::steam_status,
            steam::commands::steam_connect,
            steam::commands::steam_diag_run,
            steam::commands::steam_diag_shape,
            steam::commands::steam_diag_types,
            steam::commands::steam_sync,
            steam::commands::steam_disconnect
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event
                && let Some(db) = app.try_state::<AppDb>()
                && let Ok(conn) = db.0.lock()
            {
                db::optimize(&conn);
            }
        });
}
