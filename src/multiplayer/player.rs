// multiplayer/player.rs
//
// STATUS: LEGACY - This is a local Rust representation.
// ACTUAL BACKEND: SpacetimeDB table `player` in text_game_stdb/src/lib.rs
// SDK TYPE: src/spacetimedb_client/player_type.rs (auto-generated)
//
// DIFFERENCES:
// - SpacetimeDB uses position_x/y/z (i32) instead of Coord
// - SpacetimeDB uses String for dimension instead of enum
// - SpacetimeDB stores last_action as Option<u64> timestamp
// - This struct uses std::time::Instant which is not serializable
//
// RUST LEARNING: Structs and Enums
// - Structs group related data together (like a class without methods in the struct itself)
// - Enums represent "one of several variants" - perfect for status/state
// - We put methods in `impl` blocks separate from the struct definition
// - `Clone` is needed when we want to duplicate player data
// - Avoid Clone when possible (prefer references), but sometimes it's necessary

use std::time::Instant;
use crate::models::{PlayerClass, Coord};
use super::types::{PlayerId, AccountId};

/// Player state in the multiplayer game
/// 
/// RUST LEARNING: Struct fields can have various types:
/// - Owned types (String, Vec) - the struct owns this data
/// - Copy types (u32, bool, Coord if it derives Copy) - cheap to duplicate
/// - Complex types (Instant, Option) - standard library types
/// 
/// DESIGN NOTE: Connection details (Signal ID, phone, IP) are on Session, not Player
/// - A Player is a game character (persistent)
/// - A Session is a connection (temporary)
/// - One player can connect via multiple sessions simultaneously
///   (e.g., terminal on PC + Signal on phone)
#[derive(Debug, Clone)]
pub struct Player {
    /// Unique player identifier
    pub id: PlayerId,
    
    /// Display name (String is owned, heap-allocated)
    pub name: String,
    
    /// Account ID (for authentication)
    pub account_id: AccountId,
    
    /// Character class (determines abilities)
    pub class: PlayerClass,
    
    /// Current position in world
    pub pos: Coord,
    
    /// Connection status
    pub status: PlayerStatus,
    
    /// Last action timestamp
    /// RUST LEARNING: std::time::Instant is monotonic clock, doesn't go backwards
    pub last_action: Instant,
    
    /// Current action state (if executing multi-tick action)
    /// RUST LEARNING: Option<T> represents "maybe has a value"
    /// - Some(value) when there is a value
    /// - None when there isn't
    pub action_state: Option<ActionState>,
}

/// Player connection/activity status
/// 
/// RUST LEARNING: Enums in Rust are powerful!
/// - Each variant can optionally hold data (not shown here)
/// - Pattern matching ensures you handle all cases
/// - #[derive(Copy)] works because all variants have no data or only Copy data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
    Online,
    Idle,
    InCombat,
    Trading,
    Offline,
}

/// State for multi-tick actions (future expansion)
/// 
/// RUST LEARNING: This struct would hold data about actions that take
/// multiple ticks to complete (like crafting, casting spells, etc.)
#[derive(Debug, Clone)]
pub struct ActionState {
    pub action_type: String,  // TODO: Make this a proper enum
    pub ticks_remaining: u32,
    pub target: Option<String>,  // TODO: Proper targeting system
}

/// RUST LEARNING: Implementation Block
/// This is where we define methods on our types.
/// - `impl TypeName` starts an implementation block
/// - `&self` = immutable borrow of self (read-only method)
/// - `&mut self` = mutable borrow of self (can modify)
/// - `self` = takes ownership (consumes the value)
impl Player {
    /// Create a new player
    /// 
    /// RUST LEARNING: Associated function (no self parameter)
    /// Called like: Player::new(...)
    pub fn new(
        id: PlayerId,
        name: String,
        account_id: AccountId,
        class: PlayerClass,
        start_pos: Coord,
    ) -> Self {
        Self {
            id,
            name,
            account_id,
            class,
            pos: start_pos,
            status: PlayerStatus::Online,
            last_action: Instant::now(),
            action_state: None,
        }
    }
    
    /// Check if player can perform actions
    pub fn can_act(&self) -> bool {
        // RUST LEARNING: Pattern matching on enums
        // `matches!` is a macro that returns true if pattern matches
        matches!(self.status, PlayerStatus::Online | PlayerStatus::Idle)
    }
    
    /// Update last action timestamp
    /// 
    /// RUST LEARNING: `&mut self` means this method can modify the player
    pub fn mark_action(&mut self) {
        self.last_action = Instant::now();
        if self.status == PlayerStatus::Idle {
            self.status = PlayerStatus::Online;
        }
    }
    
    /// Check if player has been idle for given duration
    pub fn is_idle(&self, idle_threshold: std::time::Duration) -> bool {
        self.last_action.elapsed() > idle_threshold
    }
}

// TODO: Implement inventory, stats, skills, equipment
// These will be separate modules that Player will reference
