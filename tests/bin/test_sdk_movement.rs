// bin/test_sdk_movement.rs
//
// Test script for SDK client player movement functionality
//
// This test demonstrates:
// 1. Connection to SpacetimeDB via WebSocket
// 2. Using existing TestPlayer
// 3. Movement commands (N→E→S→W)
// 4. Position tracking

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
    
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║  SDK Client Movement Test                             ║");
    println!("║  Testing player movement: N → E → S → W               ║");
    println!("╚════════════════════════════════════════════════════════╝\n");
    
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
            println!("   📥 Received result: {}", result);
        }
    });
    
    println!("   ✅ Connected with subscriptions and background processor\n");
    
    // Give it a moment to establish connection
    sleep(Duration::from_secs(2)).await;
    
    // Step 2: Create session for TestPlayer
    println!("🔑 Step 2: Creating session for TestPlayer...");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let connection_id = format!("test-movement-{}", timestamp);
    
    println!("   Connection ID: {}", connection_id);
    
    // Get reducers handle
    let reducers = conn.reducers();
    
    // Call connect_session reducer
    reducers.connect_session("terminal".to_string(), connection_id.clone())?;
    sleep(Duration::from_secs(1)).await;
    
    println!("   ✅ Session created\n");
    
    // Step 3: Authenticate as TestPlayer
    println!("📊 Step 3: Authenticating as TestPlayer...");
    
    // For simplicity, use session_id 1 (in real app, query session table)
    let session_id = 1u64;
    let player_name = "TestPlayer".to_string();
    
    reducers.authenticate_player(session_id, player_name.clone())?;
    sleep(Duration::from_secs(1)).await;
    
    println!("   ✅ Authenticated as {}\n", player_name);
    
    // Step 4: Get initial position
    println!("📍 Step 4: Getting initial position...");
    println!("   (Would query Player table here - SDK subscriptions)");
    println!("   Assuming starting position: (100, 100, 100)\n");
    
    let start_x = 100;
    let start_y = 100;
    let start_z = 100;
    
    // Step 5: Test movement commands
    let movements = vec![
        ("north", "Y should increase by 1"),
        ("east", "X should increase by 1"),
        ("south", "Y should decrease by 1"),
        ("west", "X should decrease by 1"),
    ];
    
    println!("🎮 Step 5: Testing movement commands...\n");
    
    for (i, (cmd, expected)) in movements.iter().enumerate() {
        println!("─────────────────────────────────────────────────");
        println!("Movement {}/{}: {} ({})", i + 1, movements.len(), cmd, expected);
        println!("─────────────────────────────────────────────────");
        
        // Submit movement command
        println!("⏳ Submitting '{}'...", cmd);
        reducers.submit_command(cmd.to_string())?;
        println!("   ✅ Command submitted");
        
        // Wait for command to be queued
        sleep(Duration::from_millis(500)).await;
        
        // Execute tick to process movement
        println!("⏰ Executing tick...");
        reducers.execute_tick()?;
        println!("   ✅ Tick executed");
        
        // Wait for tick to complete and callback to fire
        sleep(Duration::from_secs(2)).await;
        
        println!("   📊 Movement should be processed");
        
        // Query all command_log entries to see if results are stored
        let all_logs: Vec<_> = conn.db().command_log().iter().collect();
        println!("   📋 Found {} command_log entries", all_logs.len());
        for log in all_logs.iter().rev().take(2) {
            println!("      Command: {:?}, Success: {}, Result: {:?}", 
                     log.command, log.success, log.result);
        }
        
        // Get result from SDK subscription
        let result = latest_result.lock().unwrap();
        if !result.is_empty() {
            println!("   💬 Result: {}", result);
        } else {
            println!("   ⚠️  No result received via callback");
        }
        println!("");
    }
    
    // Step 6: Final position check
    println!("═════════════════════════════════════════════════");
    println!("✅ Movement Test Complete!");
    println!("═════════════════════════════════════════════════\n");
    
    println!("📈 Summary:");
    println!("   - Connected to SpacetimeDB via WebSocket");
    println!("   - Created session: {}", connection_id);
    println!("   - Authenticated as: {}", player_name);
    println!("   - Executed 4 movement commands");
    println!("   - Each command was submitted and tick executed");
    println!("");
    println!("Expected final position: ({}, {}, {})", start_x, start_y, start_z);
    println!("  (N→E→S→W should return to start)");
    println!("");
    println!("To verify, run:");
    println!("  spacetime sql text-game --server local -- \"SELECT name, position_x, position_y, position_z FROM player WHERE name = 'TestPlayer'\"");
    println!("");
    
    Ok(())
}
