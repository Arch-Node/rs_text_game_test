// event_broadcaster.rs
//
// Cross-client event broadcasting for multiplayer interactions
//
// This module sets up callbacks to monitor other players' actions and broadcast
// events in real-time, enabling true multiplayer awareness.

use crate::spacetimedb_client::{
    DbConnection, Player, PlayerTableAccess, CommandLog, CommandLogTableAccess,
};
use spacetimedb_sdk::{DbContext, Table, TableWithPrimaryKey};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Events that can be broadcast to players
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// Another player moved to a new location
    PlayerMoved {
        player_name: String,
        from_x: i32,
        from_y: i32,
        from_z: i32,
        to_x: i32,
        to_y: i32,
        to_z: i32,
        dimension: String,
    },
    /// Another player joined the game
    PlayerJoined {
        player_name: String,
        position_x: i32,
        position_y: i32,
        position_z: i32,
        dimension: String,
    },
    /// Another player left/disconnected
    PlayerLeft {
        player_name: String,
    },
    /// Another player performed an action
    PlayerAction {
        player_name: String,
        action: String,
        position_x: i32,
        position_y: i32,
        position_z: i32,
        dimension: String,
    },
}

/// Event broadcaster that monitors game events and notifies clients
pub struct EventBroadcaster {
    /// Channel to receive events
    event_rx: tokio::sync::mpsc::Receiver<GameEvent>,
    /// Sender for event subscription
    event_tx: tokio::sync::mpsc::Sender<GameEvent>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel(100);
        Self { event_rx, event_tx }
    }

    /// Get a sender for broadcasting events
    pub fn get_sender(&self) -> mpsc::Sender<GameEvent> {
        self.event_tx.clone()
    }

    /// Start listening for events (blocks until shutdown)
    pub async fn run(mut self, mut on_event: impl FnMut(GameEvent) + Send + 'static) {
        while let Some(event) = self.event_rx.recv().await {
            on_event(event);
        }
    }
}

/// Set up event monitoring on a SpacetimeDB connection
/// 
/// This registers callbacks that watch for changes to player positions and actions,
/// broadcasting events to the provided channel so clients can display real-time
/// multiplayer notifications.
pub fn setup_event_monitoring(
    conn: Arc<DbConnection>,
    current_player_name: String,
    event_tx: mpsc::Sender<GameEvent>,
) {
    // Track player positions to detect movements
    let player_positions = Arc::new(tokio::sync::RwLock::new(
        std::collections::HashMap::<String, (i32, i32, i32, String)>::new()
    ));

    // Monitor player table updates for movements
    let positions_clone = player_positions.clone();
    let event_tx_clone = event_tx.clone();
    let current_player = current_player_name.clone();
    
    conn.db().player().on_update(move |_ctx, old_player, new_player| {
        // Ignore our own updates
        if new_player.name == current_player {
            return;
        }

        // Check if position changed
        let old_pos = (old_player.position_x, old_player.position_y, old_player.position_z);
        let new_pos = (new_player.position_x, new_player.position_y, new_player.position_z);
        
        if old_pos != new_pos {
            let event = GameEvent::PlayerMoved {
                player_name: new_player.name.clone(),
                from_x: old_player.position_x,
                from_y: old_player.position_y,
                from_z: old_player.position_z,
                to_x: new_player.position_x,
                to_y: new_player.position_y,
                to_z: new_player.position_z,
                dimension: new_player.dimension.clone(),
            };
            let _ = event_tx_clone.try_send(event);
        }
    });

    // Monitor new players joining
    let event_tx_clone = event_tx.clone();
    let current_player = current_player_name.clone();
    
    conn.db().player().on_insert(move |_ctx, player| {
        // Ignore our own insertion
        if player.name == current_player {
            return;
        }

        let event = GameEvent::PlayerJoined {
            player_name: player.name.clone(),
            position_x: player.position_x,
            position_y: player.position_y,
            position_z: player.position_z,
            dimension: player.dimension.clone(),
        };
        let _ = event_tx_clone.try_send(event);
    });

    // Monitor command log for other players' actions
    let event_tx_clone = event_tx.clone();
    let current_player = current_player_name.clone();
    
    conn.db().command_log().on_insert(move |ctx, log| {
        // Get the player who executed this command
        let players: Vec<_> = ctx.db.player()
            .iter()
            .filter(|p| p.id == log.player_id)
            .collect();
        
        if let Some(player) = players.first() {
            // Ignore our own commands
            if player.name == current_player {
                return;
            }

            // Broadcast action event
            let event = GameEvent::PlayerAction {
                player_name: player.name.clone(),
                action: log.command.clone(),
                position_x: player.position_x,
                position_y: player.position_y,
                position_z: player.position_z,
                dimension: player.dimension.clone(),
            };
            let _ = event_tx_clone.try_send(event);
        }
    });
}

/// Format a game event for display to the user
pub fn format_event(event: &GameEvent, current_position: Option<(i32, i32, i32, String)>) -> Option<String> {
    match event {
        GameEvent::PlayerMoved { player_name, to_x, to_y, to_z, dimension, .. } => {
            // Only show if in same dimension and nearby
            if let Some((my_x, my_y, my_z, my_dim)) = current_position {
                if dimension != &my_dim {
                    return None;
                }
                
                // Calculate distance
                let distance = ((to_x - my_x).pow(2) + (to_y - my_y).pow(2) + (to_z - my_z).pow(2)) as f32;
                let distance = distance.sqrt();
                
                // Only show nearby movements (within 5 units)
                if distance <= 5.0 {
                    let direction = if *to_x > my_x { "east" } 
                                  else if *to_x < my_x { "west" }
                                  else if *to_y > my_y { "north" }
                                  else if *to_y < my_y { "south" }
                                  else { "nearby" };
                    return Some(format!("👤 {} moved {}", player_name, direction));
                }
            }
            None
        }
        GameEvent::PlayerJoined { player_name, position_x, position_y, position_z, dimension } => {
            if let Some((my_x, my_y, my_z, my_dim)) = current_position {
                if dimension != &my_dim {
                    return None;
                }
                
                let distance = ((position_x - my_x).pow(2) + (position_y - my_y).pow(2) + (position_z - my_z).pow(2)) as f32;
                let distance = distance.sqrt();
                
                if distance <= 5.0 {
                    return Some(format!("✨ {} joined the area", player_name));
                }
            }
            None
        }
        GameEvent::PlayerLeft { player_name } => {
            Some(format!("👋 {} left the game", player_name))
        }
        GameEvent::PlayerAction { player_name, action, position_x, position_y, position_z, dimension } => {
            if let Some((my_x, my_y, my_z, my_dim)) = current_position {
                if dimension != &my_dim {
                    return None;
                }
                
                let distance = ((position_x - my_x).pow(2) + (position_y - my_y).pow(2) + (position_z - my_z).pow(2)) as f32;
                let distance = distance.sqrt();
                
                // Only show nearby actions
                if distance <= 3.0 {
                    return Some(format!("🎮 {} used: {}", player_name, action));
                }
            }
            None
        }
    }
}
