//! Epic adapter. Login, token refresh and the `library-service` read were tested against a real
//! account (docs/validation/phase-1.md); the launcher assets route is denied to this client and is
//! not used. The catalog mapping is in `sync`. Protocol read from derrod/legendary
//! 170964e75affc0c04be467ec2b12879376738ef5 (`legendary/api/egs.py`) and understood, not copied.
//! The Epic wishlist is not supported: the user dropped it on 2026-11-17 after the spike
//! could not read it reliably (see docs/validation/phase-1.md). The GUI says so plainly.

/// Play time is read on each sync only once `docs/validation/epic-playtime.md` records it working
/// with a real account (AGENTS.md: no capability without its record). Until then the app says
/// "Epic does not give it" and `epic_diag_playtime` is the way to validate it.
pub const PLAYTIME_VALIDATED: bool = false;

pub mod api;
pub mod commands;
pub mod sync;
#[cfg(windows)]
pub mod webview;
