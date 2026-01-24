// Terminal client using SpacetimeDB SDK
// Run with: cargo run --bin sdk_client --features spacetimedb-sdk-client

use std::io::{self, Write};

fn display_help() {
    println!("\n=== Available Commands ===");
    println!("Movement:");
    println!("  north, n    - Move north (+Y)");
    println!("  south, s    - Move south (-Y)");
    println!("  east, e     - Move east (+X)");
    println!("  west, w     - Move west (-X)");
    println!("  up, u       - Move up (+Z)");
    println!("  down, d     - Move down (-Z)");
    println!("\nInfo:");
    println!("  look        - Describe current location");
    println!("  status      - Show player info");
    println!("  rooms       - List rooms in dimension");
    println!("\nSystem:");
    println!("  tick        - Process queued commands");
    println!("  help, ?     - Show this help");
    println!("  quit, exit  - Exit game\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== SpacetimeDB Terminal Client ===\n");
    println!("Note: Using SDK-generated bindings for type-safe interaction\n");
    
    // For now, create a simple example showing the generated API
    println!("✓ Client generated with types:");
    println!("  - Player, Room, Session, QueuedCommand, CommandLog (tables)");
    println!("  - connect_session, authenticate_player, submit_command, execute_tick (reducers)");
    println!("\nTo use the full SDK client:");
    println!("1. Use spacetimedb_sdk::DbConnection::builder()");
    println!("2. Connect with .build(\"ws://localhost:3000/text-game\")?");
    println!("3. Subscribe to tables");
    println!("4. Register callbacks for table changes");
    println!("5. Call reducers through the generated API\n");
    
    // Simple interaction using command line spacetime tool
    print!("Enter player name: ");
    io::stdout().flush()?;
    let mut player_name = String::new();
    io::stdin().read_line(&mut player_name)?;
    let player_name = player_name.trim();
    
    if player_name.is_empty() {
        println!("No name entered, exiting.");
        return Ok(());
    }
    
    println!("\n=== Interactive Mode ===");
    println!("Using spacetime CLI for commands (SDK connection would be here)");
    
    // Main game loop - simplified without full SDK connection
    loop {
        print!("\n{}> ", player_name);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let command = input.trim();
        
        if command.is_empty() {
            continue;
        }
        
        match command.to_lowercase().as_str() {
            "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            "help" | "?" => {
                display_help();
                continue;
            }
            "status" => {
                println!("Querying player status...");
                let output = std::process::Command::new("spacetime")
                    .args(&["sql", "text-game", "-s", "local", "--",
                        &format!("SELECT name, position_x, position_y, position_z, dimension FROM player WHERE name = '{}'", player_name)])
                    .output()?;
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            "rooms" => {
                println!("Querying rooms...");
                let output = std::process::Command::new("spacetime")
                    .args(&["sql", "text-game", "-s", "local", "--",
                        "SELECT name, position_x, position_y, position_z, dimension FROM room LIMIT 10"])
                    .output()?;
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            _ => {
                println!("[Command: {}] - Would call: submit_command(session_id, \"{}\")", command, command);
                println!("                 Then call: execute_tick()");
                println!("\nNote: Full SDK integration requires DbConnection setup.");
                println!("      For now, use bash wrapper or HTTP client.");
            }
        }
    }
    
    Ok(())
}
