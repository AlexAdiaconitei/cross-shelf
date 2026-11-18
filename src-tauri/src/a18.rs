//! Acceptance A18 across modules: sentinel secrets put into every session type and into the
//! shapes a log line, an error or an export can take must never come out in clear.

const ACCESS: &str = "SENTINEL_ACCESS_5c1e9d";
const REFRESH: &str = "SENTINEL_REFRESH_4b8a20";
const COOKIE: &str = "SENTINEL_COOKIE_77d0aa";
const WEBAPI: &str = "SENTINEL_WEBAPI_e3f101";

fn assert_clean(what: &str, text: &str) {
    for s in [ACCESS, REFRESH, COOKIE, WEBAPI] {
        assert!(!text.contains(s), "{what} leaked {s}: {text}");
    }
}

#[test]
fn session_types_never_print_their_secrets() {
    let gog = crate::gog::auth::Tokens {
        access_token: ACCESS.into(),
        refresh_token: REFRESH.into(),
        expires_in: 3600,
        user_id: "48000000000000001".into(),
        session_id: Some(COOKIE.into()),
        login_time: 1,
    };
    assert_clean("gog tokens", &format!("{gog:?}"));

    let epic = crate::epic::api::Tokens {
        access_token: ACCESS.into(),
        refresh_token: REFRESH.into(),
        expires_in: 3600,
        refresh_expires: 7200,
        account_id: "abc".into(),
        display_name: None,
        obtained_at: 1,
    };
    assert_clean("epic tokens", &format!("{epic:?}"));

    let steam = crate::steam::api::Session {
        cookie_header: format!(
            "sessionid={COOKIE}; steamLoginSecure=76561198000000001%7C%7C{COOKIE}"
        ),
        steamid: "76561198000000001".into(),
        webapi_token: WEBAPI.into(),
        obtained_at: 1,
    };
    assert_clean("steam session", &format!("{steam:?}"));
}

#[test]
fn log_lines_with_secrets_in_every_usual_shape_are_masked() {
    let lines = [
        format!("GET https://api.steampowered.com/x?access_token={WEBAPI}&steamid=1"),
        format!("Cookie: sessionid={COOKIE}; steamLoginSecure={COOKIE}"),
        format!(r#"token response {{"access_token":"{ACCESS}","refresh_token":"{REFRESH}"}}"#),
        format!("Authorization: bearer {ACCESS}"),
        format!("refresh failed for refresh_token={REFRESH}"),
    ];
    for l in &lines {
        assert_clean("log line", &crate::redact::redact(l));
    }
}

#[test]
fn provider_errors_carry_codes_not_request_data() {
    // Errors shown in the GUI and stored with a generation are fixed texts and plain codes.
    let e = crate::steam::api::SteamError::NeedsReauth;
    assert_clean("steam error", &e.to_string());
    assert_eq!(e.code(), "needs_reauth");
    let g = crate::gog::library::ReadError::Unauthorized(401);
    assert_clean("gog error", &g.to_string());
    assert_eq!(g.code(), "needs_reauth");
}
