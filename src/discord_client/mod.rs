// discord_client/mod.rs
//
// Discord bot interface for the text adventure game
// 
// Architecture:
// Discord Messages → Bot → MessageHandler → SpacetimeDB → Responses via Discord
//
// This uses the serenity library for Discord API integration.

pub mod bot;
pub mod message_handler;
pub mod formatter;

pub use bot::DiscordBot;
