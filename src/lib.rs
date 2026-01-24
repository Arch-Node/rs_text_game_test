// lib.rs
//
// Library crate for the multiplayer text adventure game

// Terminal client module (requires reqwest/tokio features)
#[cfg(feature = "spacetimedb-http")]
pub mod terminal_client;

// SpacetimeDB SDK client bindings (generated code)
#[cfg(feature = "spacetimedb-sdk-client")]
pub mod spacetimedb_client;

// Signal messenger client
#[cfg(feature = "signal-client")]
pub mod signal_client;

// Discord bot client
#[cfg(feature = "discord-client")]
pub mod discord_client;
