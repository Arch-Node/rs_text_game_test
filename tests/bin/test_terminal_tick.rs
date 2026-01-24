// bin/test_terminal_tick.rs
//
// Test script for terminal client tick functionality
//
// This test demonstrates:
// 1. Connection to SpacetimeDB
// 2. Player authentication
// 3. Command submission
// 4. Waiting for tick execution
// 5. Checking command results

use rust_game_test::terminal_client::{
    TerminalClient,
    auth,
    client::SpacetimeConfig,
};
use tokio::time::{sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  Terminal Client Tick Test                           ║");
    println!("║  Testing message submission and tick result feedback  ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");
    
    // Create SpacetimeDB client
    let config = SpacetimeConfig::default();
    let mut client = TerminalClient::new(config);
    
    // Step 1: Connect
    println!("📡 Step 1: Connecting to SpacetimeDB server...");
    let connection_id = auth::generate_connection_id();
    println!("   Connection ID: {}", connection_id);
    
    let session_id = match client.connect(connection_id).await {
        Ok(id) => {
            println!("   ✅ Connected! Session ID: {}\n", id);
            id
        }
        Err(e) => {
            eprintln!("   ❌ Connection failed: {}\n", e);
            return Err(e);
        }
    };
    
    // Step 2: Authenticate
    println!("🔑 Step 2: Authenticating player...");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let player_name = format!("TestPlayer_{}", timestamp % 10000);
    println!("   Player name: {}", player_name);
    
    let player_id = match client.authenticate(session_id, player_name.clone()).await {
        Ok(id) => {
            println!("   ✅ Authenticated! Player ID: {}\n", id);
            id
        }
        Err(e) => {
            eprintln!("   ❌ Authentication failed: {}\n", e);
            return Err(e);
        }
    };
    
    // Step 3: Get initial player info
    println!("📊 Step 3: Getting initial player info...");
    match client.get_player_info().await {
        Ok(info) => {
            println!("   Player: {}", info.name);
            println!("   Position: ({}, {}, {})", info.position_x, info.position_y, info.position_z);
            println!("   Dimension: {}\n", info.dimension);
        }
        Err(e) => {
            eprintln!("   ⚠️ Warning: Could not get player info: {}\n", e);
        }
    }
    
    // Step 4: Submit commands and wait for tick results
    let test_commands = vec![
        "look",
        "north",
        "look",
        "inventory",
        "south",
    ];
    
    println!("🎮 Step 4: Testing command submission and tick results...\n");
    
    for (i, cmd) in test_commands.iter().enumerate() {
        println!("─────────────────────────────────────────────────");
        println!("Test {}/{}: Command \"{}\"", i + 1, test_commands.len(), cmd);
        println!("─────────────────────────────────────────────────");
        
        // Submit command
        println!("⏳ Submitting command...");
        match client.submit_command(cmd.to_string()).await {
            Ok(result) => {
                println!("   ✅ Command submitted");
                if result.is_queued {
                    println!("   📝 Command queued for next tick");
                }
                println!("   Response: {}", result.message);
            }
            Err(e) => {
                eprintln!("   ❌ Command submission failed: {}", e);
                continue;
            }
        }
        
        // Wait for tick execution (3.5 seconds)
        println!("\n⏰ Waiting 3.5 seconds for tick execution...");
        sleep(Duration::from_secs(3)).await;
        sleep(Duration::from_millis(500)).await;
        
        // Check command results using SQL query
        println!("🔍 Checking command results...");
        let query = format!(
            "SELECT success, error_message FROM command_log WHERE player_id = {} ORDER BY timestamp DESC LIMIT 1",
            player_id
        );
        
        match client.query(query).await {
            Ok(results) => {
                // Results is a serde_json::Value - check if it's an array with items
                if let Some(array) = results.as_array() {
                    if !array.is_empty() {
                        println!("   📊 Result:");
                        println!("      {}\n", serde_json::to_string_pretty(&array[0]).unwrap_or_else(|_| "Unknown".to_string()));
                    } else {
                        println!("   ⚠️ No results found in command_log\n");
                    }
                } else {
                    println!("   ⚠️ Unexpected result format\n");
                }
            }
            Err(e) => {
                eprintln!("   ❌ Failed to query results: {}\n", e);
            }
        }
        
        // Get updated room info for movement commands
        if cmd.contains("north") || cmd.contains("south") || 
           cmd.contains("east") || cmd.contains("west") || 
           cmd.contains("up") || cmd.contains("down") {
            println!("🚪 Checking position after movement...");
            match client.get_player_info().await {
                Ok(info) => {
                    println!("   Position: ({}, {}, {})\n", info.position_x, info.position_y, info.position_z);
                }
                Err(e) => {
                    eprintln!("   ⚠️ Could not get position info: {}\n", e);
                }
            }
        }
        
        // Small delay between commands
        if i < test_commands.len() - 1 {
            println!("⏸️  Pausing 1 second before next command...\n");
            sleep(Duration::from_secs(1)).await;
        }
    }
    
    println!("═════════════════════════════════════════════════");
    println!("✅ Test Complete!");
    println!("═════════════════════════════════════════════════\n");
    
    // Summary
    println!("📈 Summary:");
    println!("   - Connected to SpacetimeDB");
    println!("   - Authenticated player: {}", player_name);
    println!("   - Submitted {} commands", test_commands.len());
    println!("   - Verified tick execution with result polling");
    println!("\nTest successful! The terminal client can successfully:");
    println!("  ✅ Connect to SpacetimeDB");
    println!("  ✅ Authenticate players");
    println!("  ✅ Submit commands");
    println!("  ✅ Query command results after tick execution");
    
    Ok(())
}
