//! Raw responses of each sync, kept locally to diagnose schema changes (spec 4.5).
//!
//! Only response bodies of read endpoints go here: they carry product data, never the request's
//! token or cookies. Files stay on this machine, in `raw/<store>/`, and only the newest
//! `KEEP` per resource survive.

use std::path::Path;

const KEEP: usize = 3;

/// Writes `raw/<store>/<resource>-<now>.json` and deletes older files of that resource beyond
/// `KEEP`. Failures are logged and ignored: diagnostics never break a sync.
pub fn save(base: &Path, store: &str, resource: &str, now: i64, items: &[serde_json::Value]) {
    if let Err(e) = try_save(base, store, resource, now, items) {
        log::warn!("raw response not kept for {store}/{resource}: {e}");
    }
}

fn try_save(
    base: &Path,
    store: &str,
    resource: &str,
    now: i64,
    items: &[serde_json::Value],
) -> std::io::Result<()> {
    let dir = base.join("raw").join(store);
    std::fs::create_dir_all(&dir)?;
    let body = serde_json::to_vec(items).map_err(std::io::Error::other)?;
    std::fs::write(dir.join(format!("{resource}-{now:012}.json")), body)?;
    let prefix = format!("{resource}-");
    let mut files: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(&prefix) && n.ends_with(".json"))
        .collect();
    // Zero-padded timestamps sort as text.
    files.sort();
    let excess = files.len().saturating_sub(KEEP);
    for old in &files[..excess] {
        std::fs::remove_file(dir.join(old))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_newest_files_per_resource() {
        let dir = tempfile::tempdir().unwrap();
        for t in 1..=5 {
            save(
                dir.path(),
                "steam",
                "library",
                t,
                &[serde_json::json!({"appid": t})],
            );
        }
        save(dir.path(), "steam", "wishlist", 9, &[]);
        let mut names: Vec<String> = std::fs::read_dir(dir.path().join("raw").join("steam"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(
            names,
            [
                "library-000000000003.json",
                "library-000000000004.json",
                "library-000000000005.json",
                "wishlist-000000000009.json"
            ]
        );
    }
}
