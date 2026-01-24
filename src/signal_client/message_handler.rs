// signal_client/message_handler.rs
//
// Process incoming Signal messages and route to game commands

use super::bot::SignalBot;

impl SignalBot {
    /// Handle an incoming Signal message
    pub async fn handle_message(&self, sender: &str, text: &str) {
        log::info!("Received message from {}: {}", sender, text);
        
        let text = text.trim();
        
        // Check if user is authenticated
        if !self.is_authenticated(sender).await {
            self.handle_unauthenticated(sender, text).await;
            return;
        }
        
        // Handle authenticated commands
        if let Err(e) = self.handle_game_command(sender, text).await {
            log::error!("Error handling command: {}", e);
            // Error dropped here before await
        }
        let _ = self.send_message(sender, "Command received.").await;
    }
    
    /// Handle messages from unauthenticated users
    async fn handle_unauthenticated(&self, sender: &str, text: &str) {
        // First message should be their character name
        if text.is_empty() || text.len() > 20 {
            let _ = self.send_message(
                sender,
                "Welcome to Layered Realms! Please send your character name (1-20 characters):"
            ).await;
            return;
        }
        
        // Authenticate with the provided name
        match self.authenticate_player(sender, text).await {
            Ok(_) => {
                let welcome = format!(
                    "Welcome, {}! You are in Town Square.\n\n\
                    Commands: north, south, east, west, up, down, look, help\n\n\
                    Send a command to begin your adventure!",
                    text
                );
                let _ = self.send_message(sender, &welcome).await;
            }
            Err(e) => {
                log::error!("Authentication failed: {}", e);
                let _ = self.send_message(
                    sender,
                    "Sorry, couldn't create your character. Please try again with a different name."
                ).await;
            }
        }
    }
    
    /// Handle game commands from authenticated users
    async fn handle_game_command(&self, sender: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get session ID
        let session_id = self.get_or_create_session(sender).await?;
        
        // Handle instant commands locally
        if is_instant_command(command) {
            return self.handle_instant_command(sender, command).await;
        }
        
        // Submit queued command to SpacetimeDB
        self.submit_command(session_id, command).await?;
        
        // Send acknowledgment
        self.send_message(
            sender,
            "Command queued. Waiting for next tick..."
        ).await?;
        
        // Note: Actual response will come after tick execution
        // via a subscription/polling mechanism (not implemented yet)
        
        Ok(())
    }
    
    /// Handle instant commands that don't need tick processing
    async fn handle_instant_command(&self, sender: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = match command.to_lowercase().as_str() {
            "help" | "?" => {
                "Available commands:\n\
                • Movement: north/n, south/s, east/e, west/w, up/u, down/d\n\
                • Info: look, status, rooms\n\
                • System: help, quit"
            }
            "look" | "l" => {
                // TODO: Query current room from SpacetimeDB
                "You are in Town Square. A bustling central plaza."
            }
            "status" => {
                // TODO: Query player status from SpacetimeDB
                "Player status: Healthy, position (100,100,100)"
            }
            _ => {
                "Unknown instant command. Try 'help' for a list of commands."
            }
        };
        
        self.send_message(sender, response).await?;
        Ok(())
    }
}

/// Check if a command is instant (doesn't need tick processing)
fn is_instant_command(command: &str) -> bool {
    matches!(
        command.to_lowercase().as_str(),
        "help" | "?" | "look" | "l" | "status" | "inventory" | "i"
    )
}
