//! Central secret redaction. Every log line goes through `redact` before it is written.
//! Spec sections 4.3 and 12: no token, cookie or authorization code may reach logs.

use regex::Regex;
use std::sync::OnceLock;

const MASK: &str = "[REDACTED]";

fn patterns() -> &'static [Regex] {
    static P: OnceLock<Vec<Regex>> = OnceLock::new();
    P.get_or_init(|| {
        [
            // Authorization headers and bearer tokens.
            r"(?i)(bearer|basic)\s+[A-Za-z0-9\-._~+/=]{8,}",
            // key=value / key: value / "key":"value" for sensitive keys.
            r#"(?i)((?:access_?token|refresh_?token|webapi_?token|id_?token|token|authorization|auth_?code|code|password|passwd|secret|client_?secret|sessionid|session_?id|cookie|set-cookie|steamLoginSecure|EPIC_[A-Z_]*|api_?key|key)["']?\s*[:=]\s*["']?)[^\s"'&;,}]+"#,
            // JWT-like three-part tokens.
            r"eyJ[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}",
        ]
        .iter()
        .map(|p| Regex::new(p).expect("valid redaction pattern"))
        .collect()
    })
}

/// Returns `input` with anything that looks like a secret replaced by a mask.
pub fn redact(input: &str) -> String {
    let p = patterns();
    let mut out = p[0].replace_all(input, format!("$1 {MASK}")).into_owned();
    out = p[1].replace_all(&out, format!("${{1}}{MASK}")).into_owned();
    out = p[2].replace_all(&out, MASK).into_owned();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sentinel values: if any of these survives, redaction is broken (acceptance A18).
    const SENTINELS: &[&str] = &[
        "SENTINEL_ACCESS_9f3a1c",
        "SENTINEL_REFRESH_77be02",
        "SENTINEL_COOKIE_c0ffee1",
        "SENTINEL_PASSWORD_hunter2x",
        "SENTINEL_CODE_a1b2c3d4",
    ];

    #[test]
    fn masks_key_value_forms() {
        let line = format!(
            "access_token={} refresh_token: {} password=\"{}\" code={} sessionid={}",
            SENTINELS[0], SENTINELS[1], SENTINELS[3], SENTINELS[4], SENTINELS[2]
        );
        let out = redact(&line);
        for s in SENTINELS {
            assert!(!out.contains(s), "leaked {s} in {out}");
        }
        assert!(out.contains(MASK));
    }

    #[test]
    fn masks_json_and_headers() {
        let line = format!(
            r#"{{"access_token":"{}","refresh_token":"{}"}} Authorization: Bearer {}"#,
            SENTINELS[0], SENTINELS[1], SENTINELS[2]
        );
        let out = redact(&line);
        for s in &SENTINELS[..3] {
            assert!(!out.contains(s), "leaked {s} in {out}");
        }
    }

    #[test]
    fn masks_jwt() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abcdefghijklmnop";
        assert!(!redact(&format!("got {jwt} ok")).contains("eyJhbGci"));
    }

    #[test]
    fn leaves_plain_text_alone() {
        let s = "Sincronización de Steam completada: 120 productos";
        assert_eq!(redact(s), s);
    }
}
