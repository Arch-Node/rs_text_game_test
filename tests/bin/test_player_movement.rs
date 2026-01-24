// bin/test_player_movement.rs
//
// Test script for player movement and map navigation
//
// This test demonstrates:
// 1. Logging in as "TestPlayer"
// 2. Moving around the map
// 3. Viewing room descriptions
// 4. Tracking position changes

use rust_game_test::terminal_client::{
    TerminalClient,
    auth,
    client::SpacetimeConfig,
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  TestPlayer Movement Test                            ║");
    println!("║  Testing map navigation and room exploration          ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");
    
    // Create SpacetimeDB client
    let config = SpacetimeConfig::default();
    let mut client = TerminalClient::new(config);
    
    // Step 1: Connect
    println!("📡 Step 1: Connecting to SpacetimeDB server...");
    let connection_id = auth::generate_connection_id();
    
    let session_id = match client.connect(connection_id.clone()).await {
        Ok(id) => {
            println!("   ✅ Connected! Session ID: {}\n", id);
            id
        }
        Err(e) => {
            eprintln!("   ❌ Connection failed: {}\n", e);
            return Err(e);
        }
    };
    
    // Step 2: Authenticate as TestPlayer
    println!("🔑 Step 2: Authenticating as TestPlayer...");
    let player_name = "TestPlayer".to_string();
    
    let player_id = match client.authenticate(session_id, player_name.clone()).await {
        Ok(id) => {
            println!("   ✅ Authenticated! Player ID: {}\n", id);
            id
        }
        Err(e) => {
            eprintln!("   ❌ Authentication failed: {}\n", e);
            eprintln!("   Note: Player 'TestPlayer' might not exist yet.");
            eprintln!("   Create the player first using the terminal_client.\n");
            return Err(e);
        }
    };
    
    // Step 3: Get initial position
    println!("📊 Step 3: Getting initial player position...");
    let initial_info = match client.get_player_info().await {
        Ok(info) => {
            println!("   Player: {}", info.name);
            println!("   Starting Position: ({}, {}, {})", info.position_x, info.position_y, info.position_z);
            println!("   Dimension: {}", info.dimension);
            println!("   Status: {}\n", info.status);
            info
        }
        Err(e) => {
            eprintln!("   ⚠️ Warning: Could not get player info: {}\n", e);
            return Err(e);
        }
    };
    
    // Step 4: Look at current room
    println!("👁️  Step 4: Looking at starting room...");
    match submit_and_wait(&mut client, "look", player_id).await {
        Ok(()) => println!(""),
        Err(e) => eprintln!("   ⚠️ Look command failed: {}\n", e),
    }
    
    // Step 5: Movement test sequence
    println!("🗺️  Step 5: Testing movement commands...\n");
    
    let movement_sequence = vec![
        ("north", "Moving north..."),
        ("look", "Looking around..."),
        ("east", "Moving east..."),
        ("look", "Looking around..."),
        ("south", "Moving south..."),
        ("look", "Looking around..."),
        ("west", "Moving west..."),
        ("look", "Looking around..."),
    ];
    
    let mut current_pos = (initial_info.position_x, initial_info.position_y, initial_info.position_z);
    
    for (i, (cmd, description)) in movement_sequence.iter().enumerate() {
        println!("─────────────────────────────────────────────────");
        println!("Move {}/{}: {}", i + 1, movement_sequence.len(), description);
        println!("─────────────────────────────────────────────────");
        
        // Submit command and wait for tick
        match submit_and_wait(&mut client, cmd, player_id).await {
            Ok(()) => {},
            Err(e) => {
                eprintln!("   ⚠️ Command failed: {}\n", e);
                continue;
            }
        }
        
        // Get updated position for movement commands
        if cmd.contains("north") || cmd.contains("south") || 
           cmd.contains("east") || cmd.contains("west") || 
           cmd.contains("up") || cmd.contains("down") {
            match client.get_player_info().await {
                Ok(info) => {
                    let new_pos = (info.position_x, info.position_y, info.position_z);
                    if new_pos != current_pos {
                        println!("\n   📍 Position changed: {:?} → {:?}", current_pos, new_pos);
                        current_pos = new_pos;
                    } else {
                        println!("\n   ⚠️ Position unchanged (movement blocked or invalid)");
                    }
                }
                Err(e) => {
                    eprintln!("   ⚠️ Could not get position: {}", e);
                }
            }
        }
        
        println!("");
        
        // Small delay between commands
        if i < movement_sequence.len() - 1 {
            sleep(Duration::from_millis(800)).await;
        }
    }
    
    // Step 6: Final position report
    println!("═════════════════════════════════════════════════");
    println!("📊 Final Position Report");
    println!("═════════════════════════════════════════════════");
    
    match client.get_player_info().await {
        Ok(info) => {
            println!("   Player: {}", info.name);
            println!("   Starting: ({}, {}, {})", 
                     initial_info.position_x, initial_info.position_y, initial_info.position_z);
            println!("   Final:    ({}, {}, {})", 
                     info.position_x, info.position_y, info.position_z);
            
            let distance = ((info.position_x - initial_info.position_x).abs() + 
                          (info.position_y - initial_info.position_y).abs() + 
                          (info.position_z - initial_info.position_z).abs());
            
            if distance == 0 {
                println!("\n   ✅ Returned to starting position!");
            } else {
                println!("\n   📏 Distance from start: {} tiles", distance);
            }
        }
        Err(e) => {
            eprintln!("   ⚠️ Could not get final position: {}", e);
        }
    }
    
    println!("\n═════════════════════════════════════════════════");
    println!("✅ Movement Test Complete!");
    println!("═════════════════════════════════════════════════\n");
    
    Ok(())
}

/// Submit a command and wait for tick execution, then check results
async fn submit_and_wait(
    client: &mut TerminalClient,
    command: &str,
    player_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Submit command
    print!("   ⏳ Submitting '{}'... ", command);
    match client.submit_command(command.to_string()).await {
        Ok(_result) => {
            println!("✓");
        }
        Err(e) => {
            println!("✗");
            return Err(e);
        }
    }
    
    // Wait for tick execution
    print!("   ⏰ Waiting for tick... ");
    sleep(Duration::from_secs(3)).await;
    sleep(Duration::from_millis(500)).await;
    println!("✓");
    
    // Query command results
    print!("   🔍 Checking results... ");
    let query = format!(
        "SELECT success, error_message FROM command_log WHERE player_id = {} ORDER BY timestamp DESC LIMIT 1",
        player_id
    );
    
    match client.query(query).await {
        Ok(results) => {
            if let Some(array) = results.as_array() {
                if !array.is_empty() {
                    if let Some(result_obj) = array[0].as_object() {
                        let success = result_obj.get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        
                        if success {
                            println!("✓");
                            println!("   ✅ Command succeeded");
                        } else {
                            println!("✗");
                            let error_msg = result_obj.get("error_message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown error");
                            println!("   ❌ Command failed: {}", error_msg);
                        }
                    } else {
                        println!("?");
                        println!("   ⚠️ Unexpected result format");
                    }
                } else {
                    println!("?");
                    println!("   ⚠️ No results in command_log");
                }
            } else {
                println!("?");
                println!("   ⚠️ Unexpected result type");
            }
        }
        Err(e) => {
            println!("✗");
            eprintln!("   ❌ Query failed: {}", e);
        }
    }
    
    Ok(())
}
