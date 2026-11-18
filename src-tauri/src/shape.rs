//! Describes the shape of a list of JSON records without revealing what they contain: field
//! paths, JSON types, and, only for fields with very few distinct values over a large enough list
//! (kinds, platforms, flags), how many records have each value. Titles and IDs have too many
//! distinct values to ever show up. Used by the diagnostics to learn real response formats.

use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// A field may expose value counts only if it takes at most this many distinct values...
const MAX_DISTINCT: usize = 8;
/// ...across at least this many records, so a short list can never be read back value by value.
const MIN_RECORDS: usize = 30;
const MAX_DEPTH: usize = 3;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Field {
    pub path: String,
    pub types: Vec<&'static str>,
    pub present_in: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_counts: Option<BTreeMap<String, usize>>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ShapeReport {
    pub records: usize,
    pub fields: Vec<Field>,
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn walk(
    prefix: &str,
    v: &Value,
    depth: usize,
    out: &mut Vec<(String, &'static str, Option<String>)>,
) {
    match v {
        Value::Object(m) if depth < MAX_DEPTH => {
            for (k, inner) in m {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                walk(&path, inner, depth + 1, out);
            }
            if !prefix.is_empty() {
                out.push((prefix.to_owned(), "object", None));
            }
        }
        Value::Array(a) => {
            out.push((
                format!("{prefix}[]"),
                "array",
                Some(format!("len={}", a.len().min(9))),
            ));
            // Elements too (first few per record): lists of objects get their own paths, lists
            // of scalars get one `[*]` path whose values are counted like any other field.
            for el in a.iter().take(10) {
                match el {
                    Value::Object(_) => walk(&format!("{prefix}[]"), el, depth + 1, out),
                    scalar => walk(&format!("{prefix}[*]"), scalar, depth + 1, out),
                }
            }
        }
        scalar => {
            let value = match scalar {
                Value::Bool(b) => Some(b.to_string()),
                Value::String(s) if s.len() <= 40 => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            };
            out.push((prefix.to_owned(), type_name(scalar), value));
        }
    }
}

/// Per path: JSON types seen, records that have it, and the (capped) distinct values with counts.
type Acc = BTreeMap<String, (BTreeSet<&'static str>, usize, BTreeMap<String, usize>)>;

pub fn shape_report(records: &[Value]) -> ShapeReport {
    // path -> (types, present_in, distinct values seen, capped)
    let mut acc: Acc = BTreeMap::new();
    for r in records {
        let mut flat = Vec::new();
        walk("", r, 0, &mut flat);
        for (path, ty, val) in flat {
            let e = acc.entry(path).or_default();
            e.0.insert(ty);
            e.1 += 1;
            if let Some(v) = val {
                // Stop collecting once a field proves to be high-cardinality.
                if e.2.len() <= MAX_DISTINCT {
                    *e.2.entry(v).or_insert(0) += 1;
                }
            }
        }
    }
    let show_values = records.len() >= MIN_RECORDS;
    let fields = acc
        .into_iter()
        .map(|(path, (types, present_in, values))| Field {
            path,
            types: types.into_iter().collect(),
            present_in,
            value_counts: (show_values && !values.is_empty() && values.len() <= MAX_DISTINCT)
                .then_some(values),
        })
        .collect();
    ShapeReport {
        records: records.len(),
        fields,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn many(n: usize, f: impl Fn(usize) -> Value) -> Vec<Value> {
        (0..n).map(f).collect()
    }

    #[test]
    fn low_cardinality_fields_show_counts_and_titles_and_ids_do_not() {
        let recs = many(40, |i| {
            json!({ "id": format!("id{i}"), "title": format!("Secret Title {i}"),
                    "kind": if i % 4 == 0 { "dlc" } else { "game" }, "free": i % 2 == 0 })
        });
        let r = shape_report(&recs);
        let by = |p: &str| r.fields.iter().find(|f| f.path == p).unwrap();
        assert_eq!(by("kind").value_counts.as_ref().unwrap()["game"], 30);
        assert_eq!(by("kind").value_counts.as_ref().unwrap()["dlc"], 10);
        assert!(by("free").value_counts.is_some());
        assert!(by("title").value_counts.is_none());
        assert!(by("id").value_counts.is_none());
        assert!(!serde_json::to_string(&r).unwrap().contains("Secret Title"));
    }

    #[test]
    fn short_lists_never_reveal_values() {
        let recs = many(
            5,
            |i| json!({ "name": format!("Personal {i}"), "kind": "game" }),
        );
        let r = shape_report(&recs);
        assert!(r.fields.iter().all(|f| f.value_counts.is_none()));
        assert!(!serde_json::to_string(&r).unwrap().contains("Personal"));
    }

    #[test]
    fn nested_paths_types_and_presence_are_reported() {
        let recs = many(30, |i| {
            if i == 0 {
                json!({ "meta": { "platform": "windows", "tags": ["a","b"] } })
            } else {
                json!({ "meta": { "platform": "windows" } })
            }
        });
        let r = shape_report(&recs);
        let tags = r.fields.iter().find(|f| f.path == "meta.tags[]").unwrap();
        assert_eq!(
            (tags.present_in, tags.types.as_slice()),
            (1, ["array"].as_slice())
        );
        let platform = r.fields.iter().find(|f| f.path == "meta.platform").unwrap();
        assert_eq!(platform.present_in, 30);
    }

    #[test]
    fn list_elements_are_reported_and_counted() {
        let recs = many(40, |i| {
            json!({ "platform": if i % 2 == 0 { json!(["Windows", "Mac"]) } else { json!(["Windows"]) },
                    "categories": [{ "path": "games" }], "titles": [format!("Secret {i}")] })
        });
        let r = shape_report(&recs);
        let by = |p: &str| r.fields.iter().find(|f| f.path == p).unwrap();
        let platform = by("platform[*]").value_counts.as_ref().unwrap();
        assert_eq!((platform["Windows"], platform["Mac"]), (40, 20));
        assert_eq!(
            by("categories[].path").value_counts.as_ref().unwrap()["games"],
            40
        );
        assert!(by("titles[*]").value_counts.is_none());
        assert!(!serde_json::to_string(&r).unwrap().contains("Secret"));
    }

    #[test]
    fn empty_input_is_an_empty_report() {
        let r = shape_report(&[]);
        assert_eq!((r.records, r.fields.len()), (0, 0));
    }
}
