// lib.rs
//
// Library crate for the multiplayer text adventure game

// Terminal client module - now uses WebSocket SDK
#[cfg(feature = "spacetimedb-sdk-client")]
pub mod terminal_client;

// SpacetimeDB SDK client bindings (generated code)
#[cfg(any(feature = "spacetimedb-sdk-client", feature = "signal-client", feature = "discord-client"))]
pub mod spacetimedb_client;

// SpacetimeDB SDK utilities (connection setup with subscriptions)
#[cfg(any(feature = "spacetimedb-sdk-client", feature = "signal-client", feature = "discord-client"))]
pub mod sdk_utils;

// Event broadcaster for real-time multiplayer notifications
#[cfg(any(feature = "spacetimedb-sdk-client", feature = "signal-client", feature = "discord-client"))]
pub mod event_broadcaster;

// Signal messenger client - now uses WebSocket SDK for SpacetimeDB
#[cfg(feature = "signal-client")]
pub mod signal_client;

// Discord bot client - now uses WebSocket SDK for SpacetimeDB
#[cfg(feature = "discord-client")]
pub mod discord_client;
