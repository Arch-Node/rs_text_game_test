// lib.rs
//
// Library crate for the multiplayer text adventure game

// Terminal client module (requires reqwest/tokio features)
#[cfg(feature = "spacetimedb-http")]
pub mod terminal_client;

// SpacetimeDB SDK client bindings (generated code)
#[cfg(feature = "spacetimedb-sdk-client")]
pub mod spacetimedb_client;

// SpacetimeDB SDK utilities (connection setup with subscriptions)
#[cfg(feature = "spacetimedb-sdk-client")]
pub mod sdk_utils;

// Signal messenger client
#[cfg(feature = "signal-client")]
pub mod signal_client;

// Discord bot client
#[cfg(feature = "discord-client")]
pub mod discord_client;
