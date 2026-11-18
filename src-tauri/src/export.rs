//! CSV export (spec 4.4): UTF-8 with BOM, RFC 4180 quoting, CRLF line ends. Cells that a
//! spreadsheet would read as a formula are neutralised so an exported title can never run code
//! when the file is opened in Excel. Exports never contain secrets: callers pass plain rows.

const BOM: &str = "\u{feff}";

/// Quotes a cell when RFC 4180 requires it, and defuses spreadsheet formulas.
pub fn cell(raw: &str) -> String {
    let defused;
    let text = if raw.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        defused = format!("'{raw}");
        defused.as_str()
    } else {
        raw
    };
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_owned()
    }
}

pub fn to_csv(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::from(BOM);
    let line = |cells: Vec<String>| cells.join(",") + "\r\n";
    out.push_str(&line(header.iter().map(|h| cell(h)).collect()));
    for r in rows {
        out.push_str(&line(r.iter().map(|c| cell(c)).collect()));
    }
    out
}

/// UTC date and time, ISO 8601, from Unix seconds. Plain arithmetic, no time zone database.
pub fn iso_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// A table ready for CSV or JSON. Column names are the JSON keys too.
pub struct Table {
    pub header: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn csv(&self) -> String {
        to_csv(&self.header, &self.rows)
    }

    /// An array of objects, one per row, with every value as a string (64-bit IDs stay exact).
    pub fn json(&self) -> String {
        let items: Vec<serde_json::Map<String, serde_json::Value>> = self
            .rows
            .iter()
            .map(|r| {
                self.header
                    .iter()
                    .zip(r)
                    .map(|(h, v)| ((*h).to_owned(), serde_json::Value::String(v.clone())))
                    .collect()
            })
            .collect();
        serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".into())
    }
}

fn opt<T: ToString>(v: &Option<T>) -> String {
    v.as_ref().map(ToString::to_string).unwrap_or_default()
}

/// Library rows. No account IDs: family owners are exported as a count, never their SteamIDs.
pub fn library_table(entries: &[crate::queries::LibraryEntry]) -> Table {
    let rows = entries
        .iter()
        .map(|e| {
            let classes: Vec<&str> = e.access.iter().map(|a| a.class.as_str()).collect();
            let owners: usize = e.access.iter().map(|a| a.owners.len()).sum();
            let reasons: Vec<&str> = e
                .access
                .iter()
                .filter_map(|a| a.exclude_reason.as_deref())
                .collect();
            let observed = e.access.iter().map(|a| a.observed_at).max();
            vec![
                e.store.clone(),
                e.native_id.clone(),
                opt(&e.title),
                e.kind.clone(),
                e.windows.clone(),
                classes.join("|"),
                owners.to_string(),
                reasons.join("|"),
                opt(&e.url),
                observed.map(iso_utc).unwrap_or_default(),
            ]
        })
        .collect();
    Table {
        header: vec![
            "store",
            "product_id",
            "title",
            "kind",
            "windows",
            "access",
            "family_owners",
            "exclusion_reason",
            "url",
            "observed_at",
        ],
        rows,
    }
}

pub fn wishlist_table(entries: &[crate::queries::WishlistEntry]) -> Table {
    let rows = entries
        .iter()
        .map(|w| {
            vec![
                w.store.clone(),
                w.native_id.clone(),
                opt(&w.title),
                opt(&w.priority),
                w.added_at.map(iso_utc).unwrap_or_default(),
                if w.confirmed { "yes" } else { "unconfirmed" }.to_owned(),
                if w.already_held_here { "yes" } else { "no" }.to_owned(),
                opt(&w.url),
            ]
        })
        .collect();
    Table {
        header: vec![
            "store",
            "product_id",
            "title",
            "priority",
            "added_at",
            "confirmed",
            "held_in_same_store",
            "url",
        ],
        rows,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("bad_path")]
    BadPath,
    #[error("bad_extension")]
    BadExtension,
    #[error("io")]
    Io(#[from] std::io::Error),
}

/// Checks a destination chosen in the save dialog: absolute, existing folder, right extension.
pub fn check_destination(path: &std::path::Path, ext: &str) -> Result<(), ExportError> {
    if !path.is_absolute() || path.file_name().is_none() {
        return Err(ExportError::BadPath);
    }
    if !path.parent().is_some_and(std::path::Path::is_dir) {
        return Err(ExportError::BadPath);
    }
    let ok = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(ext));
    if !ok {
        return Err(ExportError::BadExtension);
    }
    Ok(())
}

/// Writes through a temporary file in the same folder and renames it, so a failure never leaves
/// half a file where the user expects the export.
pub fn write_atomic(path: &std::path::Path, contents: &str) -> Result<(), ExportError> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".crossshelf-tmp");
    let tmp = std::path::PathBuf::from(tmp);
    std::fs::write(&tmp, contents)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_dates_are_utc_and_exact() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso_utc(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    fn lib_entry() -> crate::queries::LibraryEntry {
        crate::queries::LibraryEntry {
            store: "steam".into(),
            account_id: "76561198000000001".into(),
            native_id: "10".into(),
            title: Some("=cmd".into()),
            kind: "game".into(),
            windows: "unknown".into(),
            url: None,
            image_url: None,
            access: vec![crate::queries::Access {
                class: "family_shared".into(),
                owners: vec!["76561198000000002".into()],
                owner_names: vec!["Marta".into()],
                exclude_reason: Some("3".into()),
                observed_at: 0,
            }],
        }
    }

    #[test]
    fn library_export_has_no_account_ids() {
        let t = library_table(&[lib_entry()]);
        for out in [t.csv(), t.json()] {
            assert!(!out.contains("76561198000000001"), "own account id leaked");
            assert!(!out.contains("76561198000000002"), "family owner id leaked");
        }
        assert!(t.csv().contains("'=cmd"), "formula defused in CSV");
        let v: serde_json::Value = serde_json::from_str(&t.json()).unwrap();
        assert_eq!(v[0]["family_owners"], "1");
        assert_eq!(v[0]["access"], "family_shared");
    }

    #[test]
    fn wishlist_export_marks_unconfirmed_entries() {
        let w = crate::queries::WishlistEntry {
            store: "gog".into(),
            account_id: "48000000000000001".into(),
            native_id: "1".into(),
            title: None,
            url: None,
            image_url: None,
            priority: None,
            added_at: Some(0),
            confirmed: false,
            already_held_here: true,
        };
        let t = wishlist_table(&[w]);
        let csv = t.csv();
        assert!(!csv.contains("48000000000000001"));
        assert!(csv.contains("unconfirmed"));
        assert!(csv.contains("1970-01-01T00:00:00Z"));
    }

    #[test]
    fn destinations_are_checked_and_written_whole() {
        let dir = tempfile::tempdir().unwrap();
        let ok = dir.path().join("lista.CSV");
        check_destination(&ok, "csv").unwrap();
        assert!(matches!(
            check_destination(&dir.path().join("x.json"), "csv"),
            Err(ExportError::BadExtension)
        ));
        assert!(matches!(
            check_destination(std::path::Path::new("relativo.csv"), "csv"),
            Err(ExportError::BadPath)
        ));
        assert!(matches!(
            check_destination(&dir.path().join("no").join("x.csv"), "csv"),
            Err(ExportError::BadPath)
        ));
        write_atomic(&ok, "a").unwrap();
        write_atomic(&ok, "b").unwrap();
        assert_eq!(std::fs::read_to_string(&ok).unwrap(), "b");
        let leftovers = std::fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(leftovers, 1, "no temporary file left behind");
    }

    #[test]
    fn starts_with_a_bom_and_uses_crlf() {
        let csv = to_csv(&["a", "b"], &[vec!["1".into(), "2".into()]]);
        assert!(csv.starts_with('\u{feff}'));
        assert_eq!(&csv[3..], "a,b\r\n1,2\r\n");
    }

    #[test]
    fn quotes_commas_quotes_and_newlines() {
        assert_eq!(cell("a,b"), "\"a,b\"");
        assert_eq!(cell("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(cell("two\nlines"), "\"two\nlines\"");
        assert_eq!(cell("plain"), "plain");
    }

    #[test]
    fn keeps_accents_and_symbols_intact() {
        assert_eq!(cell("Pokémon™ Épico"), "Pokémon™ Épico");
    }

    #[test]
    fn formulas_are_defused() {
        for evil in ["=HYPERLINK(\"x\")", "+1+1", "-2+3", "@SUM(A1)"] {
            assert!(cell(evil).trim_matches('"').starts_with('\''), "{evil}");
        }
        // A negative-looking title is still readable, just prefixed.
        assert_eq!(cell("-Title"), "'-Title");
    }

    #[test]
    fn ids_above_js_safe_integer_stay_text() {
        let csv = to_csv(&["steamid"], &[vec!["76561198000000001".into()]]);
        assert!(csv.contains("76561198000000001"));
    }
}
