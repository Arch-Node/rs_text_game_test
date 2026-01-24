// lib.rs
//
// Library crate for the multiplayer text adventure game

pub mod models;
pub mod file_format;
pub mod exits;
pub mod world;
pub mod commands;
pub mod output;
pub mod heartbeat;

// Multiplayer modules (LEGACY - requires tokio)
// Actual backend uses SpacetimeDB - see spacetimedb_client/ instead
#[cfg(feature = "spacetimedb-http")]
pub mod multiplayer;

// Terminal client module (requires reqwest/tokio features)
#[cfg(feature = "spacetimedb-http")]
pub mod terminal_client;

// SpacetimeDB SDK client bindings (generated code)
#[cfg(feature = "spacetimedb-sdk-client")]
pub mod spacetimedb_client;

// Signal messenger client
#[cfg(feature = "signal-client")]
pub mod signal_client;
