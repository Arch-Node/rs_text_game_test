// terminal_client/client.rs
//
// SpacetimeDB HTTP Client for Terminal Interface
//
// This client connects to SpacetimeDB's HTTP API to interact with the game backend.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// SpacetimeDB connection configuration
#[derive(Debug, Clone)]
pub struct SpacetimeConfig {
    /// SpacetimeDB server URL (e.g., http://localhost:3000)
    pub server_url: String,
    
    /// Database/module name
    pub database_name: String,
    
    /// Database identity (hex string) - required for v1 API
    /// If None, will try to use database_name (may not work with v1 API)
    pub database_identity: Option<String>,
}

impl Default for SpacetimeConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            database_name: "text-game".to_string(),
            // Default database identity for local "text-game" database
            // This can be found by running: spacetime list --server local
            database_identity: Some("c200c8c814d241dfb0798838e7b3cfe858ea94c3677be065b7eddae1ffc72d4a".to_string()),
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
        let db_id = self.config.database_identity.as_ref()
            .unwrap_or(&self.config.database_name);
        
        let url = format!(
            "{}/v1/database/{}/call",
            self.config.server_url, db_id
        );
        
        // Call connect_session reducer with v1 API format
        let payload = serde_json::json!({
            "fn": "connect_session",
            "args": ["terminal", connection_id.clone()]
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        // Check if reducer call succeeded
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to connect: {} - {}", status, error_text).into());
        }
        
        // Query the session table to get the session ID using v1 API
        let query_url = format!(
            "{}/v1/database/{}/sql",
            self.config.server_url, db_id
        );
        
        let query = format!("SELECT id FROM session WHERE connection_id = '{}'", connection_id);
        let query_response = self.http_client
            .post(&query_url)
            .header("Content-Type", "text/plain")
            .body(query)
            .send()
            .await?;
        
        let result: serde_json::Value = query_response.json().await?;
        
        // Extract session ID from query result
        let session_id = result[0]["id"]
            .as_u64()
            .ok_or("Failed to find session in database")?;
        
        self.session_id = Some(session_id);
        Ok(session_id)
    }
    
    /// Authenticate and select/create character
    pub async fn authenticate(
        &mut self,
        session_id: u64,
        player_name: String,
    ) -> Result<u64, Box<dyn Error>> {
        let db_id = self.config.database_identity.as_ref()
            .unwrap_or(&self.config.database_name);
        
        let url = format!(
            "{}/v1/database/{}/call",
            self.config.server_url, db_id
        );
        
        let payload = serde_json::json!({
            "fn": "authenticate_player",
            "args": [session_id, player_name.clone()]
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        // Check if reducer call succeeded
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to authenticate: {} - {}", status, error_text).into());
        }
        
        // Query the player table to get the player ID
        let query_url = format!(
            "{}/v1/database/{}/sql",
            self.config.server_url, db_id
        );
        
        let query = format!("SELECT id FROM player WHERE name = '{}'", player_name);
        let query_response = self.http_client
            .post(&query_url)
            .header("Content-Type", "text/plain")
            .body(query)
            .send()
            .await?;
        
        let result: serde_json::Value = query_response.json().await?;
        
        let player_id = result[0]["id"]
            .as_u64()
            .ok_or("Failed to find player in database")?;
        
        self.player_id = Some(player_id);
        Ok(player_id)
    }
    
    /// Submit a command to the game
    pub async fn submit_command(&self, command: String) -> Result<CommandResult, Box<dyn Error>> {
        let session_id = self.session_id.ok_or("Not connected - call connect() first")?;
        
        let db_id = self.config.database_identity.as_ref()
            .unwrap_or(&self.config.database_name);
        
        let url = format!(
            "{}/v1/database/{}/call",
            self.config.server_url, db_id
        );
        
        let payload = serde_json::json!({
            "fn": "submit_command",
            "args": [session_id, command.clone()]
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        // Check response status
        let success = response.status().is_success();
        let message = if success {
            "Command queued for next tick".to_string()
        } else {
            format!("Failed to submit command: {}", response.status())
        };
        
        Ok(CommandResult {
            success,
            message,
            is_queued: success,
        })
    }
    
    /// Get player information
    pub async fn get_player_info(&self) -> Result<PlayerInfo, Box<dyn Error>> {
        let player_id = self.player_id.ok_or("Not authenticated - call authenticate() first")?;
        
        let db_id = self.config.database_identity.as_ref()
            .unwrap_or(&self.config.database_name);
        
        let url = format!(
            "{}/v1/database/{}/sql",
            self.config.server_url, db_id
        );
        
        let query = format!("SELECT * FROM player WHERE id = {}", player_id);
        let response = self.http_client
            .post(&url)
            .header("Content-Type", "text/plain")
            .body(query)
            .send()
            .await?;
        
        let result: serde_json::Value = response.json().await?;
        
        let row = &result[0];
        let player_info = PlayerInfo {
            id: row["id"].as_u64().unwrap_or(0),
            name: row["name"].as_str().unwrap_or("").to_string(),
            class: row["class"].as_str().unwrap_or("").to_string(),
            position_x: row["position_x"].as_i64().unwrap_or(100) as i32,
            position_y: row["position_y"].as_i64().unwrap_or(100) as i32,
            position_z: row["position_z"].as_i64().unwrap_or(100) as i32,
            dimension: row["dimension"].as_str().unwrap_or("material").to_string(),
            status: row["status"].as_str().unwrap_or("Online").to_string(),
        };
        
        Ok(player_info)
    }
    
    /// Query the database using SQL
    pub async fn query(&self, sql: String) -> Result<serde_json::Value, Box<dyn Error>> {
        let db_id = self.config.database_identity.as_ref()
            .unwrap_or(&self.config.database_name);
        
        let url = format!(
            "{}/v1/database/{}/sql", 
            self.config.server_url, db_id
        );
        
        let response = self.http_client
            .post(&url)
            .header("Content-Type", "text/plain")
            .body(sql)
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
