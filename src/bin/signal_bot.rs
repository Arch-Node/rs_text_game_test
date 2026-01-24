// bin/signal_bot.rs
//
// Signal Messenger bot for Layered Realms text adventure game
//
// Usage:
//   cargo run --bin signal_bot --features signal-client

use env_logger::Env;
use rust_game_test::signal_client::SignalBot;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    // Get configuration from environment
    let signal_api_url = env::var("SIGNAL_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    let bot_number = env::var("BOT_NUMBER")
        .expect("BOT_NUMBER environment variable must be set");
    
    let spacetime_url = env::var("SPACETIME_URL")
        .unwrap_or_else(|_| "ws://localhost:3000".to_string());
    
    log::info!("=== Signal Bot Starting ===");
    log::info!("Signal API: {}", signal_api_url);
    log::info!("Bot Number: {}", bot_number);
    log::info!("SpacetimeDB (WebSocket): {}", spacetime_url);
    log::info!("Webhook: http://0.0.0.0:3001/signal/webhook");
    
    // Create and run bot (now async with WebSocket SDK)
    let bot = SignalBot::new(signal_api_url, spacetime_url, bot_number).await?;
    
    log::info!("Bot initialized, starting webhook server...");
    bot.run().await?;
    
    Ok(())
}
