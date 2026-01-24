// multiplayer/events.rs
//
// RUST LEARNING: Async Programming and Broadcasting
// - `async fn` can be awaited (non-blocking)
// - Used for I/O operations (network, file, channels)
// - Must be called with `.await` from async context
// - Returns Future that can be polled by async runtime (tokio)

use crate::models::Coord;
use super::types::{PlayerId, SessionId};
use super::state::MultiplayerGameState;
use super::session::ServerMessage;

/// Game events that are broadcast to players
/// 
/// RUST LEARNING: Enum with associated data
/// - Each variant can hold different types of data
/// - Named fields (struct-like) or unnamed fields (tuple-like)
/// - Pattern matching extracts this data
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// Player moved between rooms
    PlayerMoved {
        player_id: PlayerId,
        from: Coord,
        to: Coord,
    },
    
    /// Player entered a room (visible to others)
    PlayerEntered {
        player_id: PlayerId,
        player_name: String,
        to_room: Coord,
    },
    
    /// Player left a room (visible to others)
    PlayerLeft {
        player_id: PlayerId,
        player_name: String,
        from_room: Coord,
        exit_used: String,
    },
    
    /// Player connected to game
    PlayerConnected {
        player_id: PlayerId,
        player_name: String,
    },
    
    /// Player disconnected
    PlayerDisconnected {
        player_id: PlayerId,
        player_name: String,
    },
    
    /// Room description requested
    RoomDescription {
        player_id: PlayerId,
        room_coord: Coord,
    },
    
    /// World tick completed
    TickCompleted {
        tick_count: u64,
    },
    
    // TODO: Add more event types:
    // - Chat messages
    // - Combat events
    // - Item pickups/drops
    // - NPC interactions
    // - Skill checks
}

/// Event broadcaster - sends events to appropriate players
/// 
/// RUST LEARNING: This is a utility struct for event distribution logic
/// Could be implemented as standalone functions or as methods on GameState
pub struct EventBroadcaster;

impl EventBroadcaster {
    /// Broadcast events to affected players
    /// 
    /// RUST LEARNING: async fn and Result
    /// - async means this returns Future<Output = Result<(), String>>
    /// - Must be .awaited when called
    /// - Result forces error handling
    pub async fn broadcast(
        state: &MultiplayerGameState,
        events: Vec<GameEvent>,
    ) -> Result<(), String> {
        for event in events {
            match event {
                GameEvent::PlayerEntered { to_room, player_name, player_id } => {
                    // Send to all players in the room except the one who entered
                    Self::broadcast_to_room(state, to_room, |pid| pid != player_id, 
                        ServerMessage::Text(format!("{} enters the room.", player_name))
                    ).await?;
                }
                
                GameEvent::PlayerLeft { from_room, player_name, exit_used, player_id } => {
                    // Send to all players still in the room
                    Self::broadcast_to_room(state, from_room, |pid| pid != player_id,
                        ServerMessage::Text(format!("{} leaves {}.", player_name, exit_used))
                    ).await?;
                }
                
                GameEvent::RoomDescription { player_id, room_coord } => {
                    // Send only to requesting player
                    Self::send_to_player(state, player_id, 
                        Self::format_room_description(state, room_coord)?
                    ).await?;
                }
                
                GameEvent::PlayerConnected { player_name, .. } => {
                    // Global announcement
                    Self::broadcast_to_all(state,
                        ServerMessage::SystemNotice(format!("{} has joined the game!", player_name))
                    ).await?;
                }
                
                GameEvent::PlayerDisconnected { player_name, .. } => {
                    // Global announcement
                    Self::broadcast_to_all(state,
                        ServerMessage::SystemNotice(format!("{} has left the game.", player_name))
                    ).await?;
                }
                
                GameEvent::TickCompleted { tick_count } => {
                    // Optional: send tick notification to all players
                    // (probably don't want this in production)
                    if cfg!(debug_assertions) {
                        Self::broadcast_to_all(state,
                            ServerMessage::Text(format!("--- Tick {} ---", tick_count))
                        ).await?;
                    }
                }
                
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Send message to all players in a specific room
    /// 
    /// RUST LEARNING: Generic function with trait bounds
    /// - `F: Fn(PlayerId) -> bool` means F is a function type
    /// - Takes PlayerId, returns bool (filter function)
    /// - This allows flexible filtering
    async fn broadcast_to_room<F>(
        state: &MultiplayerGameState,
        room: Coord,
        filter: F,
        message: ServerMessage,
    ) -> Result<(), String>
    where
        F: Fn(PlayerId) -> bool,
    {
        let players = state.players_in_room(room);
        
        for player in players {
            if filter(player.id) {
                Self::send_to_player(state, player.id, message.clone()).await?;
            }
        }
        
        Ok(())
    }
    
    /// Send message to all connected players
    async fn broadcast_to_all(
        state: &MultiplayerGameState,
        message: ServerMessage,
    ) -> Result<(), String> {
        // RUST LEARNING: Iterator over HashMap values
        for session in state.sessions.values() {
            if session.is_ready() {
                // RUST LEARNING: Ignore send errors (player might have disconnected)
                let _ = session.send(message.clone()).await;
            }
        }
        
        Ok(())
    }
    
    /// Send message to a specific player
    async fn send_to_player(
        state: &MultiplayerGameState,
        player_id: PlayerId,
        message: ServerMessage,
    ) -> Result<(), String> {
        // Find session for this player
        let session = state
            .find_session_for_player(player_id)
            .ok_or_else(|| format!("No session for player {}", player_id))?;
        
        // Send message
        session.send(message).await
            .map_err(|e| format!("Failed to send message: {}", e))?;
        
        Ok(())
    }
    
    /// Format a room description
    fn format_room_description(
        state: &MultiplayerGameState,
        coord: Coord,
    ) -> Result<ServerMessage, String> {
        let room = state.world.rooms_by_coord.get(&coord)
            .ok_or_else(|| format!("Room not found at {:?}", coord))?;
        
        // Get players in room
        let other_players: Vec<String> = state
            .players_in_room(coord)
            .iter()
            .map(|p| p.name.clone())
            .collect();
        
        // Get exits
        let exits: Vec<String> = room.exits.keys().cloned().collect();
        
        Ok(ServerMessage::RoomInfo {
            name: room.name.clone(),
            description: room.desc.clone(),
            exits,
            players: other_players,
        })
    }
}

// TODO: Implement event filtering based on visibility
// TODO: Add event priorities (urgent vs normal)
// TODO: Add event batching for efficiency
// TODO: Add event history/logging
