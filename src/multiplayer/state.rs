// multiplayer/state.rs
//
// STATUS: LEGACY ARCHITECTURE - SpacetimeDB handles all state management.
//
// ACTUAL STATE STORAGE:
// - SpacetimeDB tables (player, room, session, queued_command, command_log)
// - ACID transactions handle concurrency (no manual locking needed)
// - Real-time synchronization via WebSocket subscriptions
// - State persists automatically to disk
//
// THIS MODULE:
// - Original design for in-memory state management
// - Could be repurposed as client-side cache
// - May be deprecated as SDK integration completes
//
// RUST LEARNING: Smart Pointers and Concurrency
// - `Arc<T>` = Atomic Reference Counted pointer (thread-safe shared ownership)
// - `RwLock<T>` = Read-Write lock (many readers OR one writer)
// - `Mutex<T>` = Mutual exclusion lock (only one accessor at a time)
// - Use Arc when multiple parts of code need to own the same data
// - Use RwLock when reads are common, writes are rare
// - Use Mutex when simpler locking is sufficient
// - NOTE: SpacetimeDB handles all this for us!

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::models::{World, Coord};
use super::types::{PlayerId, SessionId};
use super::player::Player;
use super::session::Session;
use super::command_queue::CommandQueue;

/// The main multiplayer game state
/// 
/// RUST LEARNING: This struct holds all mutable game state.
/// It will be wrapped in Arc<RwLock<...>> for thread-safe access.
/// 
/// Design principle:
/// - Static data (World) is in Arc (cheap to clone)
/// - Dynamic data is mutable and needs locking
#[derive(Debug)]
pub struct MultiplayerGameState {
    /// Static world data (rooms, layers, anchors) - rarely changes
    /// RUST LEARNING: Arc allows shared ownership without copying
    pub world: Arc<World>,
    
    /// Active players indexed by ID
    /// RUST LEARNING: HashMap provides O(1) lookup by key
    pub players: HashMap<PlayerId, Player>,
    
    /// Active sessions indexed by ID
    pub sessions: HashMap<SessionId, Session>,
    
    /// Command queue for next tick
    pub command_queue: CommandQueue,
    
    /// Room occupancy tracking (which players are where)
    /// RUST LEARNING: HashSet = set of unique values, no duplicates
    pub room_occupancy: HashMap<Coord, HashSet<PlayerId>>,
    
    /// World tick counter
    pub tick_count: u64,
    
    /// Server running flag
    pub is_running: bool,
}

impl MultiplayerGameState {
    /// Create new game state
    pub fn new(world: Arc<World>) -> Self {
        Self {
            world,
            players: HashMap::new(),
            sessions: HashMap::new(),
            command_queue: CommandQueue::new(),
            room_occupancy: HashMap::new(),
            tick_count: 0,
            is_running: true,
        }
    }
    
    /// Add a player to the game
    pub fn add_player(&mut self, player: Player) {
        let player_id = player.id;
        let pos = player.pos;
        
        // Add to players map
        self.players.insert(player_id, player);
        
        // Add to room occupancy
        self.room_occupancy
            .entry(pos)
            .or_insert_with(HashSet::new)
            .insert(player_id);
    }
    
    /// Remove a player from the game
    pub fn remove_player(&mut self, player_id: PlayerId) -> Option<Player> {
        if let Some(player) = self.players.remove(&player_id) {
            // Remove from room occupancy
            if let Some(room_players) = self.room_occupancy.get_mut(&player.pos) {
                room_players.remove(&player_id);
            }
            Some(player)
        } else {
            None
        }
    }
    
    /// Get all players in a specific room
    /// 
    /// RUST LEARNING: Returning Vec<&Player> (references, not owned)
    /// - Avoids cloning player data
    /// - Caller can read but not modify
    /// - Lifetime is tied to &self (references valid as long as state exists)
    pub fn players_in_room(&self, coord: Coord) -> Vec<&Player> {
        self.room_occupancy
            .get(&coord)
            .map(|player_ids| {
                player_ids
                    .iter()
                    .filter_map(|id| self.players.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Move a player to a new position
    /// 
    /// RUST LEARNING: Error Handling with Result
    /// - Result<T, E> is either Ok(value) or Err(error)
    /// - Forces caller to handle errors explicitly
    /// - ? operator propagates errors up the call stack
    pub fn move_player(&mut self, player_id: PlayerId, new_pos: Coord) -> Result<(), String> {
        // Get player (mutable reference)
        let player = self.players
            .get_mut(&player_id)
            .ok_or_else(|| format!("Player {} not found", player_id))?;
        
        let old_pos = player.pos;
        
        // Update player position
        player.pos = new_pos;
        
        // Update room occupancy
        if let Some(old_room) = self.room_occupancy.get_mut(&old_pos) {
            old_room.remove(&player_id);
        }
        
        self.room_occupancy
            .entry(new_pos)
            .or_insert_with(HashSet::new)
            .insert(player_id);
        
        Ok(())
    }
    
    /// Find session for a given player
    pub fn find_session_for_player(&self, player_id: PlayerId) -> Option<&Session> {
        self.sessions
            .values()
            .find(|session| session.player_id == Some(player_id))
    }
}

/// Thread-safe wrapper for game state
/// 
/// RUST LEARNING: Smart Pointer Composition
/// - Arc<RwLock<T>> is a common pattern
/// - Arc provides shared ownership across threads
/// - RwLock provides synchronized access (read/write locking)
/// - Multiple threads can read simultaneously
/// - Only one thread can write at a time
pub struct SafeGameState {
    state: Arc<RwLock<MultiplayerGameState>>,
}

impl SafeGameState {
    /// Create new safe game state
    pub fn new(state: MultiplayerGameState) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }
    
    /// Get read-only access to state
    /// 
    /// RUST LEARNING: Async Locks
    /// - `.read().await` acquires read lock asynchronously
    /// - Multiple readers can hold lock simultaneously
    /// - Returns RwLockReadGuard that auto-unlocks when dropped
    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, MultiplayerGameState> {
        self.state.read().await
    }
    
    /// Get mutable access to state
    /// 
    /// RUST LEARNING: Write Locks
    /// - `.write().await` acquires exclusive write lock
    /// - Blocks all other readers and writers
    /// - Returns RwLockWriteGuard that auto-unlocks when dropped
    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, MultiplayerGameState> {
        self.state.write().await
    }
    
    /// Clone the Arc (cheap - just increments reference count)
    /// 
    /// RUST LEARNING: Arc::clone
    /// - Doesn't clone the data, just the pointer
    /// - Increments atomic reference count
    /// - Very cheap operation (just a pointer copy + atomic increment)
    pub fn clone_arc(&self) -> Arc<RwLock<MultiplayerGameState>> {
        Arc::clone(&self.state)
    }
}

// TODO: Add persistence methods (save/load state)
// TODO: Add state snapshots for rollback
// TODO: Add metrics and statistics
