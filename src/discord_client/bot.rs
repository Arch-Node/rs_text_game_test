// discord_client/bot.rs
//
// Core Discord bot implementation using WebSocket SDK for SpacetimeDB
//
// EVENT OPTIMIZATION CHANGES NEEDED (See docs/evnet_structure.md):
// ------------------------------------------------------------------
// PHASE 2: Binary Encoding
//   - Update event handling to support binary/JSON formats
//   - Add format negotiation during player authentication
//   - Deserialize events based on configured format
//   - Keep JSON mode for debugging Discord commands
//
// PHASE 3: Dictionary Compression  
//   - Maintain per-player string tables (HashMap<discord_user_id, StringTable>)
//   - Handle StringTableAdd events in background task
//   - Resolve string IDs before formatting for Discord
//   - Store in event_channels metadata
//
// PHASE 4: Delta Compression
//   - Track previous state per Discord user
//   - Reconstruct full events from deltas before sending to Discord
//   - Beneficial for high-traffic channels with many players

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use spacetimedb_sdk::{DbContext, Table};
use tokio::time::{sleep, Duration};

use crate::spacetimedb_client::{
    connect_session, authenticate_player, submit_command, execute_tick,
    CommandLog, CommandLogTableAccess, PlayerTableAccess, 
    SessionTableAccess, RoomTableAccess, DbConnection,
};
use crate::sdk_utils::create_connection_with_processor;
use crate::event_broadcaster::{setup_event_monitoring, format_event, GameEvent};
use tokio::sync::mpsc;

/// Discord bot that connects Discord to SpacetimeDB via WebSocket SDK
#[derive(Clone)]
pub struct DiscordBot {
    /// SpacetimeDB WebSocket connection
    conn: Arc<DbConnection>,
    
    /// Session cache: discord_user_id → session_id
    session_cache: Arc<RwLock<HashMap<String, u64>>>,
    
    /// Player name cache: discord_user_id → player name
    player_cache: Arc<RwLock<HashMap<String, String>>>,
    
    /// Track latest command results per session
    command_results: Arc<RwLock<HashMap<u64, String>>>,
    
    /// Event receiver for multiplayer notifications (per player)
    event_channels: Arc<RwLock<HashMap<String, mpsc::Sender<GameEvent>>>>,
}

impl DiscordBot {
    /// Create a new Discord bot with WebSocket SDK connection
    pub async fn new(spacetime_url: impl Into<String>) -> Result<Self> {
        log::info!("🔌 Connecting to SpacetimeDB via WebSocket...");
        
        // Track command results
        let command_results = Arc::new(RwLock::new(HashMap::new()));
        let command_results_clone = command_results.clone();
        
        // Create connection with background processor
        let conn: Arc<crate::spacetimedb_client::DbConnection> = create_connection_with_processor(&spacetime_url.into(), "text-game")
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        
        // Set up callback for command results - note: these fire in background processor
        conn.db().command_log().on_insert(move |_ctx, log| {
            log::info!("📥 command_log callback fired: player_id={}, result={:?}", log.player_id, log.result);
            if let Some(ref result) = log.result {
                // Use player_id to track results
                let player_id = log.player_id;
                let results_clone = command_results_clone.clone();
                let result_text = result.clone();
                
                // Spawn async task to store result without blocking callback
                tokio::spawn(async move {
                    let mut results = results_clone.write().await;
                    results.insert(player_id, result_text.clone());
                    log::info!("✅ Stored command result for player {}: {}", player_id, result_text);
                });
            } else {
                log::warn!("⚠️ command_log entry has no result");
            }
        });
        
        log::info!("   ✅ Connected with real-time subscriptions");
        
        Ok(Self {
            conn,
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            player_cache: Arc::new(RwLock::new(HashMap::new())),
            command_results,
            event_channels: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Get or create a SpacetimeDB session for this Discord user
    pub async fn get_or_create_session(&self, user_id: &str) -> Result<u64> {
        // Check cache first
        {
            let cache = self.session_cache.read().await;
            if let Some(&session_id) = cache.get(user_id) {
                log::debug!("Using cached session {} for user {}", session_id, user_id);
                return Ok(session_id);
            }
        }
        
        // Create new session via SpacetimeDB SDK
        log::info!("Creating new session for Discord user {}", user_id);
        
        let reducers = self.conn.reducers();
        reducers.connect_session("discord".to_string(), user_id.to_string())?;
        
        // Wait for session to be created
        sleep(Duration::from_secs(1)).await;
        
        // Query session table to find the session ID
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.connection_id == user_id)
            .collect();
        
        if let Some(session) = sessions.first() {
            let session_id = session.id;
            
            // Cache it
            {
                let mut cache = self.session_cache.write().await;
                cache.insert(user_id.to_string(), session_id);
            }
            
            log::info!("✅ Session {} created for Discord user {}", session_id, user_id);
            Ok(session_id)
        } else {
            Err(anyhow::anyhow!("Failed to create session"))
        }
    }
    
    /// Authenticate a user with a player name
    pub async fn authenticate_player(&self, user_id: &str, player_name: &str) -> Result<()> {
        let session_id = self.get_or_create_session(user_id).await?;
        
        log::info!("Authenticating user {} as player '{}'", user_id, player_name);
        
        // Parse Discord user ID as u64 for account_id
        let account_id = user_id.parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid user ID: {}", e))?;
        
        let reducers = self.conn.reducers();
        reducers.authenticate_player(account_id, player_name.to_string())?;
        
        // Wait for authentication
        sleep(Duration::from_secs(1)).await;
        
        // Cache player name
        {
            let mut cache = self.player_cache.write().await;
            cache.insert(user_id.to_string(), player_name.to_string());
        }
        
        // Set up multiplayer event monitoring for this player
        let (event_tx, mut event_rx) = mpsc::channel(100);
        setup_event_monitoring(self.conn.clone(), player_name.to_string(), event_tx.clone());
        
        // Store the sender channel
        {
            let mut channels = self.event_channels.write().await;
            channels.insert(player_name.to_string(), event_tx);
        }
        
        // Spawn task to forward events (in real implementation, this would send to Discord)
        let bot_clone = self.clone();
        let player_name_clone = player_name.to_string();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                // Get current player position
                let current_pos = bot_clone.get_player_position(&player_name_clone).await;
                
                if let Some(message) = format_event(&event, current_pos) {
                    log::info!("Discord event for {}: {}", player_name_clone, message);
                    // In full implementation: send message to Discord channel
                }
            }
        });
        
        log::info!("✅ User {} authenticated as '{}' with multiplayer events", user_id, player_name);
        Ok(())
    }
    
    /// Get cached player name for a user
    pub async fn get_player_name(&self, user_id: &str) -> Option<String> {
        let cache = self.player_cache.read().await;
        cache.get(user_id).cloned()
    }
    
    /// Submit a command to SpacetimeDB
    pub async fn submit_command(&self, user_id: &str, command: &str) -> Result<String> {
        let session_id = self.get_or_create_session(user_id).await?;
        
        log::info!("📤 User {} (session {}) submitting command: '{}'", user_id, session_id, command);
        
        // Find player_id through session to clear previous result
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.id == session_id)
            .collect();
        
        if let Some(session) = sessions.first() {
            if let Some(player_id) = session.player_id {
                // Clear previous result for this player
                let mut results = self.command_results.write().await;
                results.remove(&player_id);
            }
        }
        
        let reducers = self.conn.reducers();
        reducers.submit_command(command.to_string())?;
        
        log::info!("✅ Command '{}' submitted to backend", command);
        Ok("Command queued for processing...".to_string())
    }
    
    /// Get command result for a user/session
    pub async fn get_command_result(&self, user_id: &str) -> Result<Option<String>> {
        log::info!("🔎 get_command_result called for user {}", user_id);
        
        let session_id = self.get_or_create_session(user_id).await?;
        log::info!("🔎 Session ID: {}", session_id);
        
        // Find player_id through session
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.id == session_id)
            .collect();
        
        log::info!("🔎 Found {} matching sessions", sessions.len());
        
        let player_id = sessions.first()
            .and_then(|s| {
                log::info!("🔎 Session has player_id: {:?}", s.player_id);
                s.player_id
            })
            .ok_or_else(|| anyhow::anyhow!("No player found for session"))?;
        
        log::info!("🔎 Looking up command result for player_id {}", player_id);
        
        // Check if we have a result for this player
        let results = self.command_results.read().await;
        log::info!("🔎 Results map contains {} entries", results.len());
        for (pid, res) in results.iter() {
            log::info!("🔎   - player {} => {}", pid, res);
        }
        
        let result = results.get(&player_id).cloned();
        
        if result.is_some() {
            log::info!("✅ Found command result for player {}", player_id);
        } else {
            log::warn!("❌ No command result found for player {}", player_id);
        }
        
        Ok(result)
    }
    
    /// Execute tick (process queued commands)
    pub async fn execute_tick(&self) -> Result<()> {
        log::debug!("⏰ Executing tick...");
        
        let reducers = self.conn.reducers();
        reducers.execute_tick()?;
        
        log::debug!("⏰ Tick executed successfully");
        Ok(())
    }
    
    /// Get player's current room description from local cache
    pub async fn get_current_room(&self, user_id: &str) -> Result<Option<String>> {
        let session_id = self.get_or_create_session(user_id).await?;
        log::info!("🔍 Looking up room for user {} with session {}", user_id, session_id);
        
        // Find player through session
        let all_sessions: Vec<_> = self.conn.db().session().iter().collect();
        log::info!("📊 Total sessions in cache: {}", all_sessions.len());
        for s in &all_sessions {
            log::info!("  Session: id={}, player_id={:?}, auth_state={}", s.id, s.player_id, s.auth_state);
        }
        
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.id == session_id)
            .collect();
        
        log::info!("📊 Matching sessions for session_id {}: {}", session_id, sessions.len());
        
        let player_id = sessions.first()
            .and_then(|s| s.player_id)
            .ok_or_else(|| anyhow::anyhow!("No player found for session"))?;
        
        log::info!("🎮 Found player_id: {}", player_id);
        
        // Get player position
        let all_players: Vec<_> = self.conn.db().player().iter().collect();
        log::info!("📊 Total players in cache: {}", all_players.len());
        for p in &all_players {
            log::info!("  Player: id={}, name={}, account_id={}, pos=({},{},{}) dim={}", 
                p.id, p.name, p.account_id, p.position_x, p.position_y, p.position_z, p.dimension);
        }
        
        let players: Vec<_> = self.conn.db().player()
            .iter()
            .filter(|p| p.id == player_id)
            .collect();
        
        log::info!("📊 Matching players for player_id {}: {}", player_id, players.len());
        log::info!("📊 Matching players for player_id {}: {}", player_id, players.len());
        
        if let Some(player) = players.first() {
            log::info!("🏠 Player position: ({}, {}, {}) in dimension '{}'", 
                player.position_x, player.position_y, player.position_z, player.dimension);
            
            // Find room at player's position
            let all_rooms: Vec<_> = self.conn.db().room().iter().collect();
            log::info!("📊 Total rooms in cache: {}", all_rooms.len());
            for r in &all_rooms {
                log::info!("  Room: name='{}', pos=({},{},{}) dim={}", 
                    r.name, r.position_x, r.position_y, r.position_z, r.dimension);
            }
            
            let rooms: Vec<_> = self.conn.db().room()
                .iter()
                .filter(|r| {
                    r.position_x == player.position_x
                        && r.position_y == player.position_y
                        && r.position_z == player.position_z
                        && r.dimension == player.dimension
                })
                .collect();
            
            log::info!("📊 Matching rooms at player position: {}", rooms.len());
            
            if let Some(room) = rooms.first() {
                // Build exit list
                let mut exits = Vec::new();
                let all_rooms: Vec<_> = self.conn.db().room().iter().collect();
                
                for other_room in all_rooms {
                    if other_room.dimension != room.dimension {
                        continue;
                    }
                    
                    let dx = other_room.position_x - room.position_x;
                    let dy = other_room.position_y - room.position_y;
                    let dz = other_room.position_z - room.position_z;
                    
                    match (dx, dy, dz) {
                        (0, 1, 0) => exits.push("north".to_string()),
                        (0, -1, 0) => exits.push("south".to_string()),
                        (1, 0, 0) => exits.push("east".to_string()),
                        (-1, 0, 0) => exits.push("west".to_string()),
                        (0, 0, 1) => exits.push("up".to_string()),
                        (0, 0, -1) => exits.push("down".to_string()),
                        _ => {}
                    }
                }
                
                let exit_text = if exits.is_empty() {
                    "No obvious exits".to_string()
                } else {
                    format!("Exits: {}", exits.join(", "))
                };
                
                return Ok(Some(format!("📍 **{}**\n\n{}\n\n🚪 {}", room.name, room.description, exit_text)));
            }
        }
        
        Ok(None)
    }
    
    /// Get player info from local cache
    pub async fn get_player_info(&self, user_id: &str) -> Result<Option<(String, i32, i32, i32, String)>> {
        let session_id = self.get_or_create_session(user_id).await?;
        
        // Find player through session
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.id == session_id)
            .collect();
        
        let player_id = sessions.first()
            .and_then(|s| s.player_id)
            .ok_or_else(|| anyhow::anyhow!("No player found for session"))?;
        
        // Get player info
        let players: Vec<_> = self.conn.db().player()
            .iter()
            .filter(|p| p.id == player_id)
            .collect();
        
        if let Some(player) = players.first() {
            Ok(Some((
                player.name.clone(),
                player.position_x,
                player.position_y,
                player.position_z,
                player.dimension.clone(),
            )))
        } else {
            Ok(None)
        }
    }
    
    /// Get player position by player name (for event filtering)
    async fn get_player_position(&self, player_name: &str) -> Option<(i32, i32, i32, String)> {
        let players: Vec<_> = self.conn.db().player()
            .iter()
            .filter(|p| p.name == player_name)
            .collect();
        
        players.first().map(|p| {
            (p.position_x, p.position_y, p.position_z, p.dimension.clone())
        })
    }
    
    /// Check if a movement direction is valid for the player
    pub async fn is_valid_direction(&self, user_id: &str, direction: &str) -> Result<bool> {
        let session_id = self.get_or_create_session(user_id).await?;
        
        log::info!("🧭 Validating direction '{}' for user {}", direction, user_id);
        
        // Find player through session
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.id == session_id)
            .collect();
        
        let player_id = sessions.first()
            .and_then(|s| s.player_id)
            .ok_or_else(|| anyhow::anyhow!("No player found for session"))?;
        
        log::info!("🧭 Player ID: {}", player_id);
        
        // Get player position
        let players: Vec<_> = self.conn.db().player()
            .iter()
            .filter(|p| p.id == player_id)
            .collect();
        
        if let Some(player) = players.first() {
            log::info!("🧭 Player at ({}, {}, {}) in '{}'", player.position_x, player.position_y, player.position_z, player.dimension);
            
            // Calculate target position based on direction
            let (dx, dy, dz) = match direction.to_lowercase().as_str() {
                "north" | "n" => (0, 1, 0),
                "south" | "s" => (0, -1, 0),
                "east" | "e" => (1, 0, 0),
                "west" | "w" => (-1, 0, 0),
                "up" | "u" => (0, 0, 1),
                "down" | "d" => (0, 0, -1),
                _ => return Ok(false),
            };
            
            let target_x = player.position_x + dx;
            let target_y = player.position_y + dy;
            let target_z = player.position_z + dz;
            
            log::info!("🧭 Target position: ({}, {}, {})", target_x, target_y, target_z);
            
            // Check if a room exists at the target position
            let room_exists = self.conn.db().room()
                .iter()
                .any(|r| {
                    r.position_x == target_x
                        && r.position_y == target_y
                        && r.position_z == target_z
                        && r.dimension == player.dimension
                });
            
            log::info!("🧭 Room exists at target: {}", room_exists);
            
            return Ok(room_exists);
        }
        
        Ok(false)
    }
}
