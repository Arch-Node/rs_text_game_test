// terminal_client/auth.rs
//
// Authentication and session management for terminal client

use std::io::{self, Write};

/// Authentication state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthState {
    NotConnected,
    Connected,
    Authenticated,
}

/// Get player name from user input
pub fn prompt_player_name() -> io::Result<String> {
    print!("Enter your character name: ");
    io::stdout().flush()?;
    
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    
    Ok(name.trim().to_string())
}

/// Generate a unique connection ID for this terminal session
pub fn generate_connection_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    format!("terminal-{}-{}", timestamp, std::process::id())
}

/// Display welcome message
pub fn display_welcome() {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║     Welcome to Multiplayer Text Adventure Game       ║");
    println!("║              Terminal Client v0.1.0                   ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");
    println!("Connecting to SpacetimeDB server...\n");
}

/// Display connection success
pub fn display_connected(session_id: u64) {
    println!("✓ Connected! Session ID: {}\n", session_id);
}

/// Display authentication success
pub fn display_authenticated(player_name: &str, player_id: u64) {
    println!("✓ Authenticated as '{}' (ID: {})\n", player_name, player_id);
    println!("Type 'help' for available commands, 'quit' to exit.\n");
}
