// signal_client/bot.rs
//
// Core Signal bot implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;
use serde_json::json;

/// Signal bot that connects Signal Messenger to SpacetimeDB
#[derive(Clone)]
pub struct SignalBot {
    /// signal-cli-rest-api base URL (e.g., http://localhost:8080)
    signal_api_url: String,
    
    /// Bot's registered Signal phone number
    bot_number: String,
    
    /// SpacetimeDB base URL (e.g., http://localhost:3000)
    spacetime_url: String,
    
    /// HTTP client for API calls
    http_client: Client,
    
    /// Session cache: phone number → session_id
    session_cache: Arc<RwLock<HashMap<String, u64>>>,
    
    /// Player name cache: phone number → player name
    player_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl SignalBot {
    /// Create a new Signal bot
    pub fn new(
        signal_api_url: impl Into<String>,
        spacetime_url: impl Into<String>,
        bot_number: impl Into<String>,
    ) -> Self {
        Self {
            signal_api_url: signal_api_url.into(),
            bot_number: bot_number.into(),
            spacetime_url: spacetime_url.into(),
            http_client: Client::new(),
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            player_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Send a message via Signal
    pub async fn send_message(&self, recipient: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
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
            return Err("Failed to send Signal message".into());
        }
        
        Ok(())
    }
    
    /// Get or create a SpacetimeDB session for this phone number
    pub async fn get_or_create_session(&self, phone: &str) -> Result<u64, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.session_cache.read().await;
            if let Some(&session_id) = cache.get(phone) {
                log::debug!("Using cached session {} for {}", session_id, phone);
                return Ok(session_id);
            }
        }
        
        // Create new session via SpacetimeDB
        log::info!("Creating new session for {}", phone);
        
        let url = format!("{}/database/text-game/call", self.spacetime_url);
        let payload = json!({
            "fn": "connect_session",
            "args": [
                {"type": "String", "value": "signal"},
                {"type": "String", "value": phone},
            ]
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            log::error!("Failed to create session: {:?}", response.text().await?);
            return Err("Failed to create session".into());
        }
        
        // Query to get the session ID (would need to query the session table)
        // For now, return a placeholder - this needs proper implementation
        let session_id = 0; // TODO: Query session table by connection_id
        
        // Cache it
        {
            let mut cache = self.session_cache.write().await;
            cache.insert(phone.to_string(), session_id);
        }
        
        Ok(session_id)
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let session_id = self.get_or_create_session(phone).await?;
        
        log::info!("Authenticating player '{}' for {}", player_name, phone);
        
        let url = format!("{}/database/text-game/call", self.spacetime_url);
        let payload = json!({
            "fn": "authenticate_player",
            "args": [
                {"type": "U64", "value": session_id},
                {"type": "String", "value": player_name},
            ]
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            log::error!("Failed to authenticate player: {:?}", response.text().await?);
            return Err("Failed to authenticate player".into());
        }
        
        // Cache player name
        {
            let mut cache = self.player_cache.write().await;
            cache.insert(phone.to_string(), player_name.to_string());
        }
        
        Ok(())
    }
    
    /// Submit a command to SpacetimeDB
    pub async fn submit_command(
        &self,
        session_id: u64,
        command: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Submitting command for session {}: {}", session_id, command);
        
        let url = format!("{}/database/text-game/call", self.spacetime_url);
        let payload = json!({
            "fn": "submit_command",
            "args": [
                {"type": "String", "value": command},
            ]
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            log::error!("Failed to submit command: {:?}", response.text().await?);
            return Err("Failed to submit command".into());
        }
        
        Ok(())
    }
    
    /// Execute a tick to process queued commands
    pub async fn execute_tick(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Executing tick");
        
        let url = format!("{}/database/text-game/call", self.spacetime_url);
        let payload = json!({
            "fn": "execute_tick",
            "args": []
        });
        
        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            log::error!("Failed to execute tick: {:?}", response.text().await?);
            return Err("Failed to execute tick".into());
        }
        
        Ok(())
    }
    
    /// Run the bot (webhook server + background tick processor)
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Starting Signal bot on {} → SpacetimeDB at {}",
            self.bot_number, self.spacetime_url);
        
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
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
