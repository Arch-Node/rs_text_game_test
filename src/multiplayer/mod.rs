// multiplayer/mod.rs
//
// ⚠️ STATUS: LEGACY ARCHITECTURE - SpacetimeDB SDK provides actual backend
//
// ACTUAL BACKEND:
// - SpacetimeDB module: text_game_stdb/src/lib.rs
// - Generated SDK: src/spacetimedb_client/ (19 files, type-safe)
// - Tables: Player, Room, Session, QueuedCommand, CommandLog
// - Reducers: connect_session, authenticate_player, submit_command, execute_tick
//
// THESE MODULES:
// - Original design documentation and architecture
// - Local types that mirror (but don't replace) SpacetimeDB tables
// - Educational reference for Rust patterns
// - May be refactored as client-side bridge layer
//
// RUST LEARNING: Module System
// - `mod.rs` is the entry point for a directory-based module
// - It declares submodules and re-exports public types
// - Other files can access types via `use crate::multiplayer::TypeName`
// - `pub use` re-exports make types available at module level

// Declare submodules
pub mod types;
pub mod player;
pub mod session;
pub mod state;
pub mod command_queue;
pub mod events;
pub mod validation;
pub mod tick;

// Re-export commonly used types for convenience
// Instead of `use crate::multiplayer::types::PlayerId`, users can write:
// `use crate::multiplayer::PlayerId`
pub use types::{PlayerId, SessionId, AccountId, ConnectionIdentifiers};
pub use player::{Player, PlayerStatus};
pub use session::{Session, InterfaceType, AuthState};
pub use state::MultiplayerGameState;
pub use command_queue::{CommandQueue, QueuedCommand, CommandPriority};
pub use events::{GameEvent, EventBroadcaster};
pub use validation::{validate_command, ValidationResult};
pub use tick::{TickConfig, execute_tick};
