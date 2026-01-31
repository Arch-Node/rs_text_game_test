// discord_client/message_handler.rs
//
// Message processing logic for Discord commands

use serenity::all::{Context, Message};
use crate::discord_client::bot::DiscordBot;
use crate::discord_client::formatter;
use anyhow::Result;

/// Handle incoming Discord messages
pub async fn handle_message(ctx: &Context, msg: &Message, bot: &DiscordBot) -> Result<()> {
    // Ignore bot's own messages
    if msg.author.bot {
        return Ok(());
    }
    
    // Get current bot user
    let bot_user = ctx.http.get_current_user().await?;
    
    // Check if this is a DM or guild message
    let is_dm = msg.guild_id.is_none();
    let mentions_bot = msg.mentions.iter().any(|u| u.id == bot_user.id);
    
    // In guilds, only respond to mentions; in DMs, respond to everything
    if !is_dm && !mentions_bot {
        return Ok(());
    }
    
    let user_id = msg.author.id.to_string();
    let content = msg.content.trim();
    
    // Remove bot mention from command if present
    let command = if mentions_bot {
        content
            .replace(&format!("<@{}>", bot_user.id), "")
            .trim()
            .to_string()
    } else {
        content.to_string()
    };
    
    // Check if user is authenticated
    let player_name = bot.get_player_name(&user_id).await;
    
    // Handle authentication flow
    if player_name.is_none() {
        if command.to_lowercase().starts_with("auth ") {
            let name = command[5..].trim();
            if name.is_empty() {
                msg.reply(&ctx, formatter::format_error("Please provide a player name: `auth YourName`")).await?;
                return Ok(());
            }
            
            match bot.authenticate_player(&user_id, name).await {
                Ok(_) => {
                    // If authenticated in a guild, tell them to DM the bot
                    if !is_dm {
                        let response = format!("✅ Authenticated as **{}**!\n\n📬 **Please send me a DM to start playing.** Game commands only work in direct messages to keep channels clean.", name);
                        msg.reply(&ctx, response).await?;
                        
                        // Try to send them a DM with instructions
                        match msg.author.create_dm_channel(&ctx.http).await {
                            Ok(dm_channel) => {
                                let dm_message = formatter::format_welcome(name);
                                match dm_channel.say(&ctx.http, dm_message).await {
                                    Ok(_) => log::info!("Sent welcome DM to user {}", msg.author.tag()),
                                    Err(e) => log::warn!("Failed to send DM to user {}: {}", msg.author.tag(), e),
                                }
                            }
                            Err(e) => log::warn!("Failed to create DM channel for user {}: {}", msg.author.tag(), e),
                        }
                    } else {
                        // Already in DM, send full welcome
                        let mut response = formatter::format_welcome(name);
                        
                        // Wait briefly for SpacetimeDB to sync player data
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        
                        // Try to get current room description
                        match bot.get_current_room(&user_id).await {
                            Ok(Some(room_desc)) => {
                                // We got the room! Add it to the message
                                response.push_str(&format!("\n\n{}", room_desc));
                            }
                            Ok(None) => {
                                // No room found (shouldn't happen but handle it)
                                response.push_str("\n\n📍 You are in an unknown location. Try using `look` to see your surroundings.");
                            }
                            Err(e) => {
                                // Error getting room (log it but don't fail)
                                log::warn!("Failed to get room on auth: {}", e);
                            }
                        }

                        msg.reply(&ctx, response).await?;
                    }
                }
                Err(e) => {
                    let response = formatter::format_error(&format!("Authentication failed: {}", e));
                    msg.reply(&ctx, response).await?;
                }
            }
            return Ok(());
        } else {
            let response = formatter::format_error("Please authenticate first: `auth YourName`");
            msg.reply(&ctx, response).await?;
            return Ok(());
        }
    }
    
    // Only allow game commands in DMs
    if !is_dm {
        let response = "📬 **Game commands only work in DMs!** Please send me a direct message to play.";
        msg.reply(&ctx, response).await?;
        return Ok(());
    }
    
    // Process game commands (only in DMs)
    handle_game_command(ctx, msg, bot, &user_id, &command).await?;
    Ok(())
}

/// Handle game commands
async fn handle_game_command(ctx: &Context, msg: &Message, bot: &DiscordBot, user_id: &str, command: &str) -> Result<()> {
    let cmd_lower = command.to_lowercase();
    
    // Handle instant commands (don't need tick processing)
    match cmd_lower.as_str() {
        "help" | "?" => {
            let response = formatter::format_help();
            msg.reply(&ctx, response).await?;
            return Ok(());
        }
        "status" => {
            if let Some(name) = bot.get_player_name(user_id).await {
                let response = formatter::format_status(&name);
                msg.reply(&ctx, response).await?;
            }
            return Ok(());
        }
        "look" | "l" => {
            // Get current room description
            match bot.get_current_room(user_id).await {
                Ok(Some(room_desc)) => {
                    msg.reply(&ctx, room_desc).await?;
                }
                Ok(None) => {
                    msg.reply(&ctx, "You are in an unknown location. Try using the command: `teleport 100 100 100 material` to go to the starting room.").await?;
                }
                Err(e) => {
                    log::error!("Failed to get room: {}", e);
                    msg.reply(&ctx, "❌ Failed to look around.").await?;
                }
            }
            return Ok(());
        }
        _ => {}
    }
    
    // Check if this is a movement command and validate it
    if is_movement_command(&cmd_lower) {
        match bot.is_valid_direction(user_id, &cmd_lower).await {
            Ok(false) => {
                msg.reply(&ctx, "❌ You can't go that way! Use `look` to see available exits.").await?;
                return Ok(());
            }
            Err(e) => {
                log::error!("Failed to validate direction: {}", e);
                msg.reply(&ctx, "❌ Failed to validate movement.").await?;
                return Ok(());
            }
            Ok(true) => {
                // Direction is valid, continue
            }
        }
    }
    
    // Submit command to SpacetimeDB
    match bot.submit_command(user_id, command).await {
        Ok(_) => {
            // Send "processing" message
            let processing_msg = msg.reply(&ctx, "⏳ Processing command...").await?;
            
            // Clone context and IDs for async task
            let ctx_clone = ctx.clone();
            let bot_clone = bot.clone();
            let user_id_clone = user_id.to_string();
            let channel_id = msg.channel_id;
            
            // Spawn task to wait for result
            tokio::spawn(async move {
                // Wait for tick to execute - poll multiple times with delays
                let mut attempts = 0;
                let max_attempts = 8;  // Try for up to 4 seconds (8 * 500ms)
                
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    attempts += 1;
                    
                    log::info!("🔄 Polling for result, attempt {}/{}", attempts, max_attempts);
                    
                    // Check for result
                    match bot_clone.get_command_result(&user_id_clone).await {
                        Ok(Some(result)) => {
                            // Found result! Edit the processing message
                            log::info!("✅ Found result on attempt {}: {}", attempts, result);
                            let _ = channel_id.edit_message(&ctx_clone.http, processing_msg.id, serenity::builder::EditMessage::new().content(result)).await;
                            break;
                        }
                        Ok(None) => {
                            // No result yet
                            if attempts >= max_attempts {
                                log::warn!("❌ No result after {} attempts", attempts);
                                let _ = channel_id.edit_message(&ctx_clone.http, processing_msg.id, serenity::builder::EditMessage::new().content("⚠️ Command queued but no result yet. Try 'look' to see your location.")).await;
                                break;
                            }
                            // Continue polling
                        }
                        Err(e) => {
                            log::error!("Failed to get command result: {}", e);
                            let _ = channel_id.edit_message(&ctx_clone.http, processing_msg.id, serenity::builder::EditMessage::new().content("⚠️ Command submitted but couldn't fetch result.")).await;
                            break;
                        }
                    }
                }
            });
        }
        Err(e) => {
            let response = formatter::format_error(&format!("Command failed: {}", e));
            msg.reply(&ctx, response).await?;
        }
    }
    Ok(())
}

/// Check if a command is a movement command
fn is_movement_command(cmd: &str) -> bool {
    matches!(
        cmd.to_lowercase().as_str(),
        "north" | "south" | "east" | "west" | "up" | "down" | "n" | "s" | "e" | "w" | "u" | "d"
    )
}
