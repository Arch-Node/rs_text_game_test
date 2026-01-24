// terminal_client/mod.rs
//
// Terminal Interface for SpacetimeDB Multiplayer Backend
//
// This module provides a terminal-based client that connects to the SpacetimeDB
// backend and allows players to interact with the multiplayer game.

pub mod client;
pub mod auth;
pub mod display;
pub mod input;

pub use client::TerminalClient;
pub use auth::AuthState;
pub use display::DisplayFormatter;
pub use input::InputHandler;
