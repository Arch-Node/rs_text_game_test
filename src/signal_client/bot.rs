// signal_client/bot.rs
//
// Core Signal bot implementation using WebSocket SDK for SpacetimeDB

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;
use serde_json::json;
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

/// Signal bot that connects Signal Messenger to SpacetimeDB
#[derive(Clone)]
pub struct SignalBot {
    /// signal-cli-rest-api base URL (e.g., http://localhost:8080)
    signal_api_url: String,
    
    /// Bot's registered Signal phone number
    bot_number: String,
    
    /// SpacetimeDB WebSocket connection
    conn: Arc<DbConnection>,
    
    /// HTTP client for Signal API calls only
    http_client: Client,
    
    /// Session cache: phone number → session_id
    session_cache: Arc<RwLock<HashMap<String, u64>>>,
    
    /// Player name cache: phone number → player name
    player_cache: Arc<RwLock<HashMap<String, String>>>,
    
    /// Track latest command results per session
    command_results: Arc<RwLock<HashMap<u64, String>>>,
    
    /// Event receiver for multiplayer notifications (per player)
    event_channels: Arc<RwLock<HashMap<String, mpsc::Sender<GameEvent>>>>,
}

impl SignalBot {
    /// Create a new Signal bot with WebSocket SDK connection
    pub async fn new(
        signal_api_url: impl Into<String>,
        spacetime_url: impl Into<String>,
        bot_number: impl Into<String>,
    ) -> Result<Self> {
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
            signal_api_url: signal_api_url.into(),
            bot_number: bot_number.into(),
            conn,
            http_client: Client::new(),
            event_channels: Arc::new(RwLock::new(HashMap::new())),
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            player_cache: Arc::new(RwLock::new(HashMap::new())),
            command_results,
        })
    }
    
    /// Send a message via Signal
    pub async fn send_message(&self, recipient: &str, text: &str) -> Result<()> {
        let url = format!("{}/v2/send", self.signal_api_url);
        
        let payload = json!({
            "number": self.bot_number,
            "recipients": [recipient],
            "message": text,
        });
        
        log::info!("Sending Signal message to {}: {}", recipient, text);
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            log::error!("Failed to send Signal message: {:?}", response.text().await?);
            return Err(anyhow::anyhow!("Failed to send Signal message"));
        }
        
        Ok(())
    }
    
    /// Get or create a SpacetimeDB session for this phone number
    pub async fn get_or_create_session(&self, phone: &str) -> Result<u64> {
        // Check cache first
        {
            let cache = self.session_cache.read().await;
            if let Some(&session_id) = cache.get(phone) {
                log::debug!("Using cached session {} for {}", session_id, phone);
                return Ok(session_id);
            }
        }
        
        // Create new session via SpacetimeDB SDK
        log::info!("Creating new session for {}", phone);
        
        let reducers = self.conn.reducers();
        reducers.connect_session("signal".to_string(), phone.to_string())?;
        
        // Wait for session to be created
        sleep(Duration::from_secs(1)).await;
        
        // Query session table to find the session ID
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.connection_id == phone)
            .collect();
        
        if let Some(session) = sessions.first() {
            let session_id = session.id;
            
            // Cache it
            {
                let mut cache = self.session_cache.write().await;
                cache.insert(phone.to_string(), session_id);
            }
            
            log::info!("✅ Session {} created for {}", session_id, phone);
            Ok(session_id)
        } else {
            Err(anyhow::anyhow!("Failed to create session"))
        }
    }
    
    /// Check if player is authenticated
    pub async fn is_authenticated(&self, phone: &str) -> bool {
        self.player_cache.read().await.contains_key(phone)
    }
    
    /// Authenticate player (link session to player)
    pub async fn authenticate_player(
        &self,
        phone: &str,
        player_name: &str,
    ) -> Result<()> {
        let session_id = self.get_or_create_session(phone).await?;
        
        log::info!("Authenticating player '{}' for {}", player_name, phone);
        
        let reducers = self.conn.reducers();
        reducers.authenticate_player(session_id, player_name.to_string())?;
        
        // Wait for authentication
        sleep(Duration::from_secs(1)).await;
        
        // Cache player name
        {
            let mut cache = self.player_cache.write().await;
            cache.insert(phone.to_string(), player_name.to_string());
        }
        
        // Set up multiplayer event monitoring for this player
        let (event_tx, mut event_rx) = mpsc::channel(100);
        setup_event_monitoring(self.conn.clone(), player_name.to_string(), event_tx.clone());
        
        // Store the sender channel
        {
            let mut channels = self.event_channels.write().await;
            channels.insert(player_name.to_string(), event_tx);
        }
        
        // Spawn task to forward events to Signal
        let bot_clone = self.clone();
        let phone_clone = phone.to_string();
        let player_name_clone = player_name.to_string();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                // Get current player position
                let current_pos = bot_clone.get_player_position(&player_name_clone).await;
                
                if let Some(message) = format_event(&event, current_pos) {
                    let _ = bot_clone.send_message(&phone_clone, &message).await;
                }
            }
        });
        
        log::info!("✅ Player '{}' authenticated for {} with multiplayer events", player_name, phone);
        Ok(())
    }
    
    /// Submit a command to SpacetimeDB
    pub async fn submit_command(
        &self,
        phone: &str,
        command: &str,
    ) -> Result<()> {
        let session_id = self.get_or_create_session(phone).await?;
        
        log::info!("Submitting command for {}: {}", phone, command);
        
        // Clear previous result for this session
        {
            let mut results = self.command_results.write().await;
            results.remove(&session_id);
        }
        
        let reducers = self.conn.reducers();
        reducers.submit_command(command.to_string())?;
        
        log::debug!("✅ Command submitted");
        Ok(())
    }
    
    /// Execute a tick to process queued commands
    pub async fn execute_tick(&self) -> Result<()> {
        log::debug!("Executing tick");
        
        let reducers = self.conn.reducers();
        reducers.execute_tick()?;
        
        Ok(())
    }
    
    /// Get command result for a phone number/session
    pub async fn get_command_result(&self, phone: &str) -> Result<Option<String>> {
        let session_id = self.get_or_create_session(phone).await?;
        
        // Check if we have a result for this session
        let results = self.command_results.read().await;
        Ok(results.get(&session_id).cloned())
    }
    
    /// Get player's current room description from local cache
    pub async fn get_current_room(&self, phone: &str) -> Result<Option<String>> {
        let session_id = self.get_or_create_session(phone).await?;
        
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
                return Ok(Some(format!("📍 {}\n\n{}", room.name, room.description)));
            }
        }
        
        Ok(None)
    }
    
    /// Get player info from local cache
    pub async fn get_player_info(&self, phone: &str) -> Result<Option<(String, i32, i32, i32, String)>> {
        let session_id = self.get_or_create_session(phone).await?;
        
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
    
    /// Run the bot (webhook server + background tick processor)
    pub async fn run(self) -> Result<()> {
        log::info!("Starting Signal bot on {} → SpacetimeDB (WebSocket)",
            self.bot_number);
        
        // Clone for background task
        let bot_clone = self.clone();
        
        // Start background tick processor
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                if let Err(e) = bot_clone.execute_tick().await {
                    log::error!("Tick execution failed: {}", e);
                }
            }
        });
        
        // Start webhook server
        crate::signal_client::start_webhook_server(self)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
    }
}
