// signal_client/message_handler.rs
//
// Process incoming Signal messages and route to game commands

use super::bot::SignalBot;

impl SignalBot {
    /// Handle an incoming Signal message
    pub async fn handle_message(&self, sender: &str, text: &str, is_group: bool) {
        log::info!("Received message from {} (group={}): {}", sender, is_group, text);
        
        let text = text.trim();
        
        // Check if user is authenticated
        if !self.is_authenticated(sender).await {
            self.handle_unauthenticated(sender, text, is_group).await;
            return;
        }
        
        // If authenticated but in a group, reject game commands
        if is_group {
            let _ = self.send_message(
                sender,
                "📧 Game commands only work in direct messages! Please message me directly to play."
            ).await;
            return;
        }
        
        // Handle authenticated commands (only in DMs)
        if let Err(e) = self.handle_game_command(sender, text).await {
            log::error!("Error handling command: {}", e);
            // Error dropped here before await
        }
        let _ = self.send_message(sender, "Command received.").await;
    }
    
    /// Handle messages from unauthenticated users
    async fn handle_unauthenticated(&self, sender: &str, text: &str, is_group: bool) {
        // Only allow authentication command
        if !text.to_lowercase().starts_with("auth ") && text.to_lowercase() != "auth" {
            let msg = if is_group {
                "Please authenticate with: auth YourName"
            } else {
                "Welcome to Layered Realms! Please authenticate with: auth YourName"
            };
            let _ = self.send_message(sender, msg).await;
            return;
        }
        
        // Extract player name
        let player_name = if text.to_lowercase() == "auth" {
            ""
        } else {
            text[4..].trim()
        };
        
        if player_name.is_empty() || player_name.len() > 20 {
            let _ = self.send_message(
                sender,
                "Please provide a character name (1-20 characters): auth YourName"
            ).await;
            return;
        }
        
        // Authenticate with the provided name
        match self.authenticate_player(sender, player_name).await {
            Ok(_) => {
                if is_group {
                    // Authenticated in group - send DM instruction
                    let _ = self.send_message(
                        sender,
                        &format!(
                            "✅ Authenticated as {}!\n\n\
                            📧 Please send me a direct message to start playing. \
                            Game commands only work in DMs to keep groups clean.",
                            player_name
                        )
                    ).await;
                    
                    // Try to send a DM with welcome
                    let welcome = format!(
                        "Welcome, {}! 🎮\n\n\
                        You are in Town Square.\n\n\
                        Commands:\n\
                        • Movement: north, south, east, west, up, down\n\
                        • Info: look, help, status\n\n\
                        Send a command to begin your adventure!",
                        player_name
                    );
                    let _ = self.send_message(sender, &welcome).await;
                } else {
                    // Authenticated in DM - send full welcome
                    let welcome = format!(
                        "Welcome, {}! 🎮\n\n\
                        You are in Town Square.\n\n\
                        Commands:\n\
                        • Movement: north, south, east, west, up, down\n\
                        • Info: look, help, status\n\n\
                        Send a command to begin your adventure!",
                        player_name
                    );
                    let _ = self.send_message(sender, &welcome).await;
                }
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
