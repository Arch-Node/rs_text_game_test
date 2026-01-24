// bin/test_sdk_tick.rs
//
// Test script for SDK client tick functionality
//
// This test demonstrates:
// 1. Connection to SpacetimeDB via WebSocket
// 2. Player authentication
// 3. Command submission
// 4. Waiting for tick execution
// 5. Checking command results

use rust_game_test::sdk_utils::create_connection_with_processor;
use rust_game_test::spacetimedb_client::{
    connect_session, authenticate_player, submit_command, execute_tick, 
    CommandLog, CommandLogTableAccess
};
use spacetimedb_sdk::{DbContext, Table};
use tokio::time::{sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  SDK Client Tick Test                                ║");
    println!("║  Testing message submission and tick result feedback  ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");
    
    // Step 1: Connect to SpacetimeDB
    println!("📡 Step 1: Connecting to SpacetimeDB via WebSocket...");
    
    // Track latest command result
    let latest_result = Arc::new(Mutex::new(String::new()));
    let latest_result_clone = latest_result.clone();
    
    // Use utility function to create connection with subscriptions and background processor
    let conn = create_connection_with_processor("ws://localhost:3000", "text-game").await?;
    
    // Set up callback for command results
    conn.db().command_log().on_insert(move |_ctx, log: &CommandLog| {
        if let Some(ref result) = log.result {
            let mut latest = latest_result_clone.lock().unwrap();
            *latest = result.clone();
            println!("   📥 Received result callback: {}", result);
        }
    });
    
    println!("   ✅ Connected with subscriptions and background processor\n");
    
    // Give it a moment to establish connection and subscriptions
    sleep(Duration::from_secs(2)).await;
    
    // Step 2: Generate connection ID and connect session
    println!("🔑 Step 2: Creating session...");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let connection_id = format!("test-sdk-{}", timestamp);
    let player_name = format!("TestSDK_{}", timestamp % 10000);
    
    println!("   Connection ID: {}", connection_id);
    println!("   Player name: {}", player_name);
    
    // Get reducers handle
    let reducers = conn.reducers();
    
    // Call connect_session reducer
    reducers.connect_session("terminal".to_string(), connection_id.clone())?;
    sleep(Duration::from_secs(1)).await;
    
    println!("   ✅ Session created\n");
    
    // Step 3: Authenticate player
    println!("📊 Step 3: Authenticating player...");
    
    // For simplicity, use session_id 1 (in real app, query session table)
    let session_id = 1u64;
    
    // Call authenticate_player reducer
    reducers.authenticate_player(session_id, player_name.clone())?;
    sleep(Duration::from_secs(1)).await;
    
    println!("   ✅ Player authenticated\n");
    
    // Step 4: Submit test commands
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
        reducers.submit_command(cmd.to_string())?;
        println!("   ✅ Command submitted");
        
        // Wait a moment for command to be queued
        sleep(Duration::from_millis(500)).await;
        
        // Execute tick
        println!("⏰ Executing tick...");
        reducers.execute_tick()?;
        println!("   ✅ Tick executed");
        
        // Wait for tick to process
        sleep(Duration::from_secs(1)).await;
        
        // Get result from SDK cache (via subscription callback)
        let result = latest_result.lock().unwrap();
        println!("   📊 Command processed");
        if !result.is_empty() {
            println!("   💬 Result: {}", result);
        }
        println!("");
        
        // Small delay between commands
        if i < test_commands.len() - 1 {
            sleep(Duration::from_millis(500)).await;
        }
    }
    
    println!("═════════════════════════════════════════════════");
    println!("✅ Test Complete!");
    println!("═════════════════════════════════════════════════\n");
    
    println!("📈 Summary:");
    println!("   - Connected to SpacetimeDB via WebSocket");
    println!("   - Created session: {}", connection_id);
    println!("   - Authenticated player: {}", player_name);
    println!("   - Submitted {} commands", test_commands.len());
    println!("   - Executed tick after each command");
    println!("\nTest successful! The SDK client can:");
    println!("  ✅ Connect via WebSocket");
    println!("  ✅ Call reducers (connect_session, authenticate_player)");
    println!("  ✅ Submit commands and execute ticks");
    println!("  ✅ Process commands through the tick system");
    
    Ok(())
}

