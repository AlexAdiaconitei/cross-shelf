//! Steam adapter, phase 1 spike (spec 5.3, 6.1 to 6.3). Every mechanism here is
//! PENDIENTE DE PRUEBA until run against a real account: see docs/validation/phase-1.md.
//!
//! Login happens on store.steampowered.com in its own window. Afterwards the cookies are read
//! with the native webview API (never from page JavaScript) and exchanged for a short-lived
//! `webapi_token` via the store's own config endpoint (precedent: Augmented Steam, Playnite).

pub mod api;
pub mod commands;
pub mod sync;
