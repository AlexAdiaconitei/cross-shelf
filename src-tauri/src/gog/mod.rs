//! GOG adapter (spec 5.3 / 6.1 / 6.3). Public Galaxy client credentials come from
//! heroic-gogdl `gogdl/auth.py` at commit 6c6ef53fd4965d8d9b943c73d9400fe55eadc6ba. They are
//! public client values, not secrets, and are not presented as such.

pub mod auth;
pub mod commands;
pub mod library;
pub mod sync;
