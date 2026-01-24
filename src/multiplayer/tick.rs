// multiplayer/tick.rs
//
// RUST LEARNING: Async/Await and Error Handling
// - Async functions return impl Future (computed later)
// - .await yields control until operation completes
// - ? operator propagates errors up the call stack
// - Use async when doing I/O or waiting for events

use std::time::Duration;
use crate::commands::Command;
use crate::exits::resolve_exit_target;
use super::state::MultiplayerGameState;
use super::command_queue::QueuedCommand;
use super::events::{GameEvent, EventBroadcaster};

/// Configuration for tick timing
#[derive(Debug, Clone)]
pub struct TickConfig {
    /// How long between ticks
    pub tick_interval: Duration,
    
    /// Maximum commands per player per tick
    pub max_commands_per_player: usize,
    
    /// Command timeout (discard if queued too long)
    pub command_timeout: Duration,
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            tick_interval: Duration::from_secs(15),   // 15 seconds for production
            max_commands_per_player: 1,
            command_timeout: Duration::from_secs(30),
        }
    }
}

impl TickConfig {
    /// Fast config for testing/development
    pub fn fast_testing() -> Self {
        Self {
            tick_interval: Duration::from_secs(3),
            max_commands_per_player: 1,
            command_timeout: Duration::from_secs(10),
        }
    }
}

/// Execute a game tick - process all queued commands
/// 
/// RUST LEARNING: Comprehensive async function
/// - Takes &mut for state mutation
/// - Returns Result for error handling
/// - Uses .await for async operations
/// - Combines multiple subsystems
pub async fn execute_tick(state: &mut MultiplayerGameState) -> Result<(), String> {
    state.tick_count += 1;
    
    // Step 1: Remove timed-out commands
    let config = TickConfig::default();
    state.command_queue.remove_old_commands(config.command_timeout);
    
    // Step 2: Get all commands sorted by priority
    let commands = state.command_queue.drain_commands();
    
    // Step 3: Execute each command and collect events
    let mut events = Vec::new();
    
    for queued_cmd in commands {
        // RUST LEARNING: Pattern matching on Result
        // match result { Ok(value) => ..., Err(error) => ... }
        match execute_queued_command(state, &queued_cmd).await {
            Ok(mut cmd_events) => {
                events.append(&mut cmd_events);
            }
            Err(e) => {
                // Log error and send to player
                eprintln!("Command execution error: {}", e);
                // TODO: Send error to player's session
            }
        }
    }
    
    // Step 4: Process world updates (heartbeat effects)
    // TODO: NPC movement, environmental effects, etc.
    
    // Step 5: Broadcast events to affected players
    EventBroadcaster::broadcast(state, events).await?;
    
    // Step 6: Cleanup disconnected sessions
    cleanup_disconnected_sessions(state);
    
    Ok(())
}

/// Execute a single queued command
/// 
/// RUST LEARNING: Detailed async function with multiple patterns
async fn execute_queued_command(
    state: &mut MultiplayerGameState,
    queued: &QueuedCommand,
) -> Result<Vec<GameEvent>, String> {
    // Get player (mutable)
    // RUST LEARNING: ok_or_else creates an error if None
    let player = state.players.get_mut(&queued.player_id)
        .ok_or_else(|| format!("Player {} not found", queued.player_id))?;
    
    let mut events = Vec::new();
    
    // Mark player as active
    player.mark_action();
    
    // RUST LEARNING: Pattern matching on Command enum
    match &queued.command {
        Command::Go { raw } => {
            let old_pos = player.pos;
            
            // Get current room
            let room = state.world.rooms_by_coord.get(&old_pos)
                .ok_or("Current room not found")?;
            
            // Find exit
            let exit_name = raw.trim().to_lowercase();
            let exit_spec = room.exits.get(&exit_name)
                .ok_or_else(|| format!("Exit '{}' not found", raw))?;
            
            // Calculate destination
            let new_pos = resolve_exit_target(old_pos, exit_spec)
                .map_err(|e| format!("Exit resolution failed: {}", e))?;
            
            // Verify destination exists
            if !state.world.rooms_by_coord.contains_key(&new_pos) {
                return Err("Destination room not found".to_string());
            }
            
            // Update player position
            player.pos = new_pos;
            
            // Update room occupancy
            if let Some(old_room_players) = state.room_occupancy.get_mut(&old_pos) {
                old_room_players.remove(&player.id);
            }
            state.room_occupancy
                .entry(new_pos)
                .or_default()
                .insert(player.id);
            
            // Generate events
            events.push(GameEvent::PlayerLeft {
                player_id: player.id,
                player_name: player.name.clone(),
                from_room: old_pos,
                exit_used: exit_name.clone(),
            });
            
            events.push(GameEvent::PlayerEntered {
                player_id: player.id,
                player_name: player.name.clone(),
                to_room: new_pos,
            });
            
            events.push(GameEvent::RoomDescription {
                player_id: player.id,
                room_coord: new_pos,
            });
        }
        
        Command::Look => {
            events.push(GameEvent::RoomDescription {
                player_id: player.id,
                room_coord: player.pos,
            });
        }
        
        Command::Where => {
            // TODO: Send position info to player
        }
        
        Command::Help => {
            // TODO: Send help text to player
        }
        
        Command::Quit => {
            events.push(GameEvent::PlayerDisconnected {
                player_id: player.id,
                player_name: player.name.clone(),
            });
            // TODO: Actually disconnect the player
        }
        
        Command::Teleport { anchor_id } => {
            // TODO: Implement teleport
            return Err("Teleport not yet implemented".to_string());
        }
        
        Command::Shift { dir: _ } => {
            // TODO: Implement layer shift
            return Err("Layer shift not yet implemented".to_string());
        }
    }
    
    Ok(events)
}

/// Clean up disconnected or timed-out sessions
fn cleanup_disconnected_sessions(state: &mut MultiplayerGameState) {
    let timeout = Duration::from_secs(300); // 5 minutes
    
    // RUST LEARNING: retain keeps only elements matching predicate
    state.sessions.retain(|_id, session| {
        !session.is_timed_out(timeout)
    });
    
    // TODO: Remove players associated with removed sessions
}

// TODO: Implement rate limiting (max actions per player per tick)
// TODO: Add tick statistics and monitoring
// TODO: Implement action rollback for conflicts
// TODO: Add support for multi-tick actions (crafting, casting, etc.)
