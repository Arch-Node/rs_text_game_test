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
                        if let Ok(dm_channel) = msg.author.create_dm_channel(&ctx.http).await {
                            let dm_message = formatter::format_welcome(name);
                            let _ = dm_channel.say(&ctx.http, dm_message).await;
                        }
                    } else {
                        // Already in DM, send full welcome
                        let response = formatter::format_welcome(name);
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
        _ => {}
    }
    
    // Submit command to SpacetimeDB
    match bot.submit_command(user_id, command).await {
        Ok(result) => {
            let response = formatter::format_command_result(&result);
            msg.reply(&ctx, response).await?;
        }
        Err(e) => {
            let response = formatter::format_error(&format!("Command failed: {}", e));
            msg.reply(&ctx, response).await?;
        }
    }
    Ok(())
}
