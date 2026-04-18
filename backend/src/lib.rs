// AGPL-3.0 — La Ruche

// Temporary: many modules are scaffolded but not yet wired in routes/handlers.
// Keep strict clippy for active code paths while allowing planned code to exist.
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod errors;
pub mod jobs;
pub mod storage;
