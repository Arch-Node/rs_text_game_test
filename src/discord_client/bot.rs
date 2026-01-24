// discord_client/bot.rs
//
// Core Discord bot implementation using WebSocket SDK for SpacetimeDB

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
            if let Some(ref result) = log.result {
                // Use player_id to track results
                let player_id = log.player_id;
                // Use try_write to avoid blocking in callback
                if let Ok(mut results) = command_results_clone.try_write() {
                    results.insert(player_id, result.clone());
                    log::debug!("📥 Command result for player {}: {}", player_id, result);
                }
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
        
        let reducers = self.conn.reducers();
        reducers.authenticate_player(session_id, player_name.to_string())?;
        
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
        
        log::info!("User {} submitting command: {}", user_id, command);
        
        // Clear previous result for this session
        {
            let mut results = self.command_results.write().await;
            results.remove(&session_id);
        }
        
        let reducers = self.conn.reducers();
        reducers.submit_command(command.to_string())?;
        
        log::debug!("✅ Command submitted");
        Ok("Command queued for processing...".to_string())
    }
    
    /// Get command result for a user/session
    pub async fn get_command_result(&self, user_id: &str) -> Result<Option<String>> {
        let session_id = self.get_or_create_session(user_id).await?;
        
        // Check if we have a result for this session
        let results = self.command_results.read().await;
        Ok(results.get(&session_id).cloned())
    }
    
    /// Execute tick (process queued commands)
    pub async fn execute_tick(&self) -> Result<()> {
        log::debug!("Executing tick");
        
        let reducers = self.conn.reducers();
        reducers.execute_tick()?;
        
        Ok(())
    }
    
    /// Get player's current room description from local cache
    pub async fn get_current_room(&self, user_id: &str) -> Result<Option<String>> {
        let session_id = self.get_or_create_session(user_id).await?;
        
        // Find player through session
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.id == session_id)
            .collect();
        
        let player_id = sessions.first()
            .and_then(|s| s.player_id)
            .ok_or_else(|| anyhow::anyhow!("No player found for session"))?;
        
        // Get player position
        let players: Vec<_> = self.conn.db().player()
            .iter()
            .filter(|p| p.id == player_id)
            .collect();
        
        if let Some(player) = players.first() {
            // Find room at player's position
            let rooms: Vec<_> = self.conn.db().room()
                .iter()
                .filter(|r| {
                    r.position_x == player.position_x
                        && r.position_y == player.position_y
                        && r.position_z == player.position_z
                        && r.dimension == player.dimension
                })
                .collect();
            
            if let Some(room) = rooms.first() {
                return Ok(Some(format!("**{}**\n{}", room.name, room.description)));
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
}
