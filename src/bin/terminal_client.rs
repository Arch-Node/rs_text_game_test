// bin/terminal_client.rs
//
// Terminal Client for Multiplayer Text Adventure
//
// This is a standalone terminal application that connects to the SpacetimeDB
// backend and allows players to interact with the game.

use rust_game_test::terminal_client::{
    TerminalClient, DisplayFormatter, InputHandler,
    auth::{self, AuthState},
    client::{SpacetimeConfig, CommandResult},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Display welcome message
    auth::display_welcome();
    
    // Create SpacetimeDB client
    let config = SpacetimeConfig::default();
    let mut client = TerminalClient::new(config);
    
    // Generate connection ID
    let connection_id = auth::generate_connection_id();
    println!("Connection ID: {}\n", connection_id);
    
    // Connect to SpacetimeDB
    println!("Connecting to server...");
    let session_id = match client.connect(connection_id).await {
        Ok(id) => {
            auth::display_connected(id);
            id
        }
        Err(e) => {
            eprintln!("{}", DisplayFormatter::format_error(&format!(
                "Failed to connect: {}", e
            )));
            return Err(e);
        }
    };
    
    // Authenticate player
    let player_name = auth::prompt_player_name()?;
    let player_id = match client.authenticate(session_id, player_name.clone()).await {
        Ok(id) => {
            auth::display_authenticated(&player_name, id);
            id
        }
        Err(e) => {
            eprintln!("{}", DisplayFormatter::format_error(&format!(
                "Failed to authenticate: {}", e
            )));
            return Err(e);
        }
    };
    
    // Get initial player status
    match client.get_player_info().await {
        Ok(info) => {
            println!("\n{}\n", DisplayFormatter::format_player_status(&info));
        }
        Err(e) => {
            eprintln!("{}", DisplayFormatter::format_error(&format!(
                "Failed to get player info: {}", e
            )));
        }
    }
    
    // Show help
    DisplayFormatter::display_help();
    
    // Main game loop
    println!("\n═══ Game Started ═══\n");
    
    loop {
        // Get input
        let input = match InputHandler::prompt() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Input error: {}", e);
                continue;
            }
        };
        
        // Skip empty input
        if input.is_empty() {
            continue;
        }
        
        // Handle quit
        if InputHandler::is_quit_command(&input) {
            println!("\nDisconnecting... Goodbye!");
            break;
        }
        
        // Handle help
        if InputHandler::is_help_command(&input) {
            DisplayFormatter::display_help();
            continue;
        }
        
        // Handle status
        if InputHandler::is_status_command(&input) {
            match client.get_player_info().await {
                Ok(info) => {
                    println!("\n{}\n", DisplayFormatter::format_player_status(&info));
                }
                Err(e) => {
                    eprintln!("{}", DisplayFormatter::format_error(&format!(
                        "Failed to get player info: {}", e
                    )));
                }
            }
            continue;
        }
        
        // Submit command to SpacetimeDB
        match client.submit_command(input.clone()).await {
            Ok(result) => {
                println!("{}\n", DisplayFormatter::format_command_result(
                    result.success,
                    &result.message,
                    result.is_queued,
                ));
            }
            Err(e) => {
                eprintln!("{}\n", DisplayFormatter::format_error(&format!(
                    "Command failed: {}", e
                )));
            }
        }
    }
    
    Ok(())
}
