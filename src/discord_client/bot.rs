// discord_client/bot.rs
//
// Core Discord bot implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;
use anyhow::Result;

/// Discord bot that connects Discord to SpacetimeDB
pub struct DiscordBot {
    /// SpacetimeDB base URL (e.g., http://localhost:3000)
    spacetime_url: String,
    
    /// HTTP client for SpacetimeDB API calls
    http_client: Client,
    
    /// Session cache: discord_user_id → session_id
    session_cache: Arc<RwLock<HashMap<String, u64>>>,
    
    /// Player name cache: discord_user_id → player name
    player_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl DiscordBot {
    /// Create a new Discord bot
    pub fn new(spacetime_url: impl Into<String>) -> Self {
        Self {
            spacetime_url: spacetime_url.into(),
            http_client: Client::new(),
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            player_cache: Arc::new(RwLock::new(HashMap::new())),
        }
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
        
        // Create new session
        log::info!("Creating new session for Discord user {}", user_id);
        
        let url = format!("{}/database/text-game/connect_session", self.spacetime_url);
        let payload = serde_json::json!({
            "interface_type": "discord",
            "connection_id": user_id
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            log::error!("Failed to create session: {}", error_text);
            return Err(anyhow::anyhow!("Failed to create session: {}", error_text));
        }
        
        // Parse session_id from response (adjust based on actual SpacetimeDB response format)
        let response_json: serde_json::Value = response.json().await?;
        let session_id = response_json["session_id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid session_id in response"))?;
        
        // Cache it
        {
            let mut cache = self.session_cache.write().await;
            cache.insert(user_id.to_string(), session_id);
        }
        
        log::info!("Created session {} for user {}", session_id, user_id);
        Ok(session_id)
    }
    
    /// Authenticate a user with a player name
    pub async fn authenticate_player(&self, user_id: &str, player_name: &str) -> Result<()> {
        let session_id = self.get_or_create_session(user_id).await?;
        
        log::info!("Authenticating user {} as player '{}'", user_id, player_name);
        
        let url = format!("{}/database/text-game/authenticate_player", self.spacetime_url);
        let payload = serde_json::json!({
            "session_id": session_id,
            "player_name": player_name
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            log::error!("Failed to authenticate player: {}", error_text);
            return Err(anyhow::anyhow!("Failed to authenticate player: {}", error_text));
        }
        
        // Cache player name
        {
            let mut cache = self.player_cache.write().await;
            cache.insert(user_id.to_string(), player_name.to_string());
        }
        
        log::info!("Successfully authenticated user {} as '{}'", user_id, player_name);
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
        
        let url = format!("{}/database/text-game/submit_command", self.spacetime_url);
        let payload = serde_json::json!({
            "session_id": session_id,
            "command_text": command
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            log::error!("Failed to submit command: {}", error_text);
            return Err(anyhow::anyhow!("Failed to submit command: {}", error_text));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        let result = response_json["result"]
            .as_str()
            .unwrap_or("Command submitted")
            .to_string();
        
        Ok(result)
    }
    
    /// Execute tick (process queued commands)
    pub async fn execute_tick(&self) -> Result<()> {
        log::debug!("Executing tick");
        
        let url = format!("{}/database/text-game/execute_tick", self.spacetime_url);
        
        let response = self.http_client
            .post(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            log::error!("Failed to execute tick: {}", error_text);
            return Err(anyhow::anyhow!("Failed to execute tick: {}", error_text));
        }
        
        Ok(())
    }
}
