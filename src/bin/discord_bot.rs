// bin/discord_bot.rs
//
// Discord bot binary

use std::sync::Arc;
use serenity::all::{Client, Context, EventHandler, GatewayIntents, Message, Ready};
use tokio::time::{interval, Duration};
use log::{info, error};
use std::env;

use rust_game_test::discord_client::{DiscordBot, message_handler};

/// Discord bot event handler
struct Handler {
    bot: Arc<DiscordBot>,
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Err(e) = message_handler::handle_message(&ctx, &msg, &self.bot).await {
            log::error!("Error handling message: {}", e);
        }
    }
    
    async fn ready(&self, _: Context, ready: Ready) {
        // In Serenity 0.12, discriminator is now a NonZero<u16> and not optional
        info!("Discord bot connected as {}", ready.user.tag());
        info!("Bot is ready to receive commands!");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    // Get configuration from environment
    let discord_token = env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN environment variable must be set");
    
    let spacetime_url = env::var("SPACETIME_URL")
        .unwrap_or_else(|_| "ws://localhost:3000".to_string());
    
    info!("Starting Discord bot...");
    info!("SpacetimeDB (WebSocket): {}", spacetime_url);
    
    // Create bot instance with WebSocket SDK (now async)
    let bot = Arc::new(DiscordBot::new(spacetime_url).await?);
    
    // Start tick processor in background
    let tick_bot = bot.clone();
    tokio::spawn(async move {
        let mut tick_interval = interval(Duration::from_secs(3));
        loop {
            tick_interval.tick().await;
            if let Err(e) = tick_bot.execute_tick().await {
                error!("Tick execution failed: {}", e);
            }
        }
    });
    
    // Configure Discord client
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    
    let handler = Handler {
        bot: bot.clone(),
    };
    
    let mut client = Client::builder(&discord_token, intents)
        .event_handler(handler)
        .await?;
    
    info!("Discord bot starting...");
    
    // Start the client
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
    
    Ok(())
}
