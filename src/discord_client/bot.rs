// discord_client/bot.rs
//
// Core Discord bot implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;
use anyhow::Result;

/// Discord bot that connects Discord to SpacetimeDB
#[derive(Clone)]
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
    
    /// Get recent command results for a player
    /// Returns (success, command, error_message_if_any)
    pub async fn get_command_results(&self, user_id: &str, limit: usize) -> Result<Vec<(bool, String, Option<String>)>> {
        // First, get the player ID by querying the session
        let session_id = self.get_or_create_session(user_id).await?;
        
        // Query command_log for recent commands
        let url = format!("{}/database/sql/text-game", self.spacetime_url);
        let query = format!(
            "SELECT success, command, error_message FROM command_log WHERE player_id IN (SELECT player_id FROM session WHERE id = {}) ORDER BY executed_at DESC LIMIT {}",
            session_id, limit
        );
        
        let response = self.http_client
            .post(&url)
            .body(query)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to query command log"));
        }
        
        let results: serde_json::Value = response.json().await?;
        let mut command_results = Vec::new();
        
        if let Some(rows) = results.as_array() {
            for row in rows {
                let success = row["success"].as_bool().unwrap_or(false);
                let command = row["command"].as_str().unwrap_or("").to_string();
                let error_message = row["error_message"].as_str().map(|s| s.to_string());
                command_results.push((success, command, error_message));
            }
        }
        
        Ok(command_results)
    }
    
    /// Get player's current room description
    pub async fn get_current_room(&self, user_id: &str) -> Result<Option<String>> {
        let session_id = self.get_or_create_session(user_id).await?;
        
        // Query player position
        let url = format!("{}/database/sql/text-game", self.spacetime_url);
        let query = format!(
            "SELECT p.position_x, p.position_y, p.position_z, p.dimension FROM player p JOIN session s ON s.player_id = p.id WHERE s.id = {}",
            session_id
        );
        
        let response = self.http_client
            .post(&url)
            .body(query)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Ok(None);
        }
        
        let results: serde_json::Value = response.json().await?;
        
        if let Some(rows) = results.as_array() {
            if let Some(row) = rows.first() {
                let x = row["position_x"].as_i64().unwrap_or(100);
                let y = row["position_y"].as_i64().unwrap_or(100);
                let z = row["position_z"].as_i64().unwrap_or(100);
                let dimension = row["dimension"].as_str().unwrap_or("material");
                
                // Query room at this position
                let room_query = format!(
                    "SELECT name, description FROM room WHERE position_x = {} AND position_y = {} AND position_z = {} AND dimension = '{}'",
                    x, y, z, dimension
                );
                
                let room_response = self.http_client
                    .post(&url)
                    .body(room_query)
                    .send()
                    .await?;
                
                if room_response.status().is_success() {
                    let room_results: serde_json::Value = room_response.json().await?;
                    if let Some(room_rows) = room_results.as_array() {
                        if let Some(room) = room_rows.first() {
                            let name = room["name"].as_str().unwrap_or("Unknown");
                            let description = room["description"].as_str().unwrap_or("");
                            return Ok(Some(format!("**{}**\n{}", name, description)));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
}
