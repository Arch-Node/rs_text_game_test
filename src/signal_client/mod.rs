// signal_client/mod.rs
//
// Signal Messenger interface for the text adventure game
// 
// Architecture:
// Signal Messages → Webhook → SignalBot → SpacetimeDB → Responses via Signal
//
// This uses signal-cli-rest-api for Signal protocol integration.

pub mod bot;
pub mod webhook;
pub mod message_handler;
pub mod formatter;

pub use bot::SignalBot;
pub use webhook::start_webhook_server;
