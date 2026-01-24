// terminal_client/client.rs
//
// SpacetimeDB HTTP Client for Terminal Interface
//
// This client connects to SpacetimeDB's HTTP API to interact with the game backend.

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// SpacetimeDB connection configuration
#[derive(Debug, Clone)]
pub struct SpacetimeConfig {
    /// SpacetimeDB server URL (e.g., http://localhost:3000)
    pub server_url: String,
    
    /// Database/module name
    pub database_name: String,
}

impl Default for SpacetimeConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            database_name: "text-game".to_string(),
        }
    }
}

/// Terminal client for SpacetimeDB
pub struct TerminalClient {
    config: SpacetimeConfig,
    http_client: Client,
    
    /// Current session ID (assigned by SpacetimeDB)
    session_id: Option<u64>,
    
    /// Current player ID
    player_id: Option<u64>,
    
    /// SpacetimeDB identity token
    identity: Option<String>,
}

impl TerminalClient {
    /// Create a new terminal client
    pub fn new(config: SpacetimeConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            session_id: None,
            player_id: None,
            identity: None,
        }
    }
    
    /// Connect to SpacetimeDB and create a session
    pub async fn connect(&mut self, connection_id: String) -> Result<u64, Box<dyn Error>> {
        let url = format!(
            "{}/database/call/{}/connect_session",
            self.config.server_url, self.config.database_name
        );
        
        // Call connect_session reducer with interface type and connection ID
        let response = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "args": ["terminal", connection_id]
            }))
            .send()
            .await?;
        
        let body: serde_json::Value = response.json().await?;
        
        // Extract session ID from response
        // Note: SpacetimeDB reducer responses vary; adjust parsing as needed
        let session_id = body["session_id"]
            .as_u64()
            .ok_or("Failed to parse session_id")?;
        
        self.session_id = Some(session_id);
        Ok(session_id)
    }
    
    /// Authenticate and select/create character
    pub async fn authenticate(
        &mut self,
        session_id: u64,
        player_name: String,
    ) -> Result<u64, Box<dyn Error>> {
        let url = format!(
            "{}/database/call/{}/authenticate_player",
            self.config.server_url, self.config.database_name
        );
        
        let response = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "args": [session_id, player_name]
            }))
            .send()
            .await?;
        
        let body: serde_json::Value = response.json().await?;
        
        let player_id = body["player_id"]
            .as_u64()
            .ok_or("Failed to parse player_id")?;
        
        self.player_id = Some(player_id);
        Ok(player_id)
    }
    
    /// Submit a command to the game
    pub async fn submit_command(&self, command: String) -> Result<CommandResult, Box<dyn Error>> {
        let url = format!(
            "{}/database/call/{}/submit_command",
            self.config.server_url, self.config.database_name
        );
        
        let response = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "args": [command]
            }))
            .send()
            .await?;
        
        let body: serde_json::Value = response.json().await?;
        
        // Parse command result
        let result = CommandResult {
            success: body["success"].as_bool().unwrap_or(false),
            message: body["message"].as_str().unwrap_or("").to_string(),
            is_queued: body["is_queued"].as_bool().unwrap_or(false),
        };
        
        Ok(result)
    }
    
    /// Get player information
    pub async fn get_player_info(&self) -> Result<PlayerInfo, Box<dyn Error>> {
        let url = format!(
            "{}/database/call/{}/get_player_info",
            self.config.server_url, self.config.database_name
        );
        
        let response = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "args": []
            }))
            .send()
            .await?;
        
        let body: serde_json::Value = response.json().await?;
        
        let player_info = PlayerInfo {
            id: body["id"].as_u64().unwrap_or(0),
            name: body["name"].as_str().unwrap_or("").to_string(),
            class: body["class"].as_str().unwrap_or("").to_string(),
            position_x: body["position_x"].as_i64().unwrap_or(0) as i32,
            position_y: body["position_y"].as_i64().unwrap_or(0) as i32,
            position_z: body["position_z"].as_i64().unwrap_or(0) as i32,
            dimension: body["dimension"].as_str().unwrap_or("material").to_string(),
            status: body["status"].as_str().unwrap_or("Online").to_string(),
        };
        
        Ok(player_info)
    }
    
    /// Query the database using SQL
    pub async fn query(&self, sql: String) -> Result<serde_json::Value, Box<dyn Error>> {
        let url = format!(
            "{}/database/sql/{}", 
            self.config.server_url, self.config.database_name
        );
        
        let response = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "query": sql
            }))
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}

/// Result from submitting a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub is_queued: bool,
}

/// Player information from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: u64,
    pub name: String,
    pub class: String,
    pub position_x: i32,
    pub position_y: i32,
    pub position_z: i32,
    pub dimension: String,
    pub status: String,
}
