// multiplayer/session.rs
//
// RUST LEARNING: Async Communication with Channels
// - Channels allow sending messages between async tasks
// - `mpsc` = Multi-Producer, Single-Consumer
// - `Sender<T>` can be cloned and shared across tasks
// - `Receiver<T>` is unique to one task
// - Perfect for sending messages to connected clients

use std::time::Instant;
use tokio::sync::mpsc;
use super::types::{SessionId, PlayerId, AccountId, ConnectionIdentifiers};

/// Messages sent from server to client
/// 
/// RUST LEARNING: Enums with Data
/// Each variant can carry different types of data.
/// This is more powerful than enums in C/Java!
#[derive(Debug, Clone)]
pub enum ServerMessage {
    /// Plain text message
    Text(String),
    
    /// Error message
    Error(String),
    
    /// Room description with structured data
    RoomInfo {
        name: String,
        description: String,
        exits: Vec<String>,
        players: Vec<String>,
    },
    
    /// System notification
    SystemNotice(String),
    
    /// Connection status update
    StatusUpdate(String),
}

/// Type alias for the sender side of message channel
/// 
/// RUST LEARNING: Type Aliases
/// Makes complex types easier to read and refactor.
/// This is the "write end" of a channel.
pub type MessageSender = mpsc::Sender<ServerMessage>;

/// Type alias for the receiver side of message channel
/// This is the "read end" of a channel.
pub type MessageReceiver = mpsc::Receiver<ServerMessage>;

/// A client session (connection to game server)
/// 
/// RUST LEARNING: This struct represents an active connection.
/// - It holds a channel sender to communicate with the client
/// - The actual network I/O happens elsewhere
/// - This design separates "session logic" from "network protocol"
/// - Connection identifiers allow same player to connect via multiple interfaces
#[derive(Debug)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,
    
    /// Associated player ID (None if not authenticated)
    /// RUST LEARNING: Option is used for "maybe" values
    pub player_id: Option<PlayerId>,
    
    /// Account ID (for authentication)
    pub account_id: Option<AccountId>,
    
    /// Interface type (Terminal, Signal, SMS)
    pub interface: InterfaceType,
    
    /// Connection identifiers (Signal ID, phone number, IP, MAC, etc.)
    /// RUST LEARNING: This allows us to identify and authenticate users
    /// across different connection methods
    pub connection_ids: ConnectionIdentifiers,
    
    /// Channel for sending messages to this session
    /// RUST LEARNING: The Sender can be cloned to share across tasks
    pub tx: MessageSender,
    
    /// Session metadata
    pub created_at: Instant,
    pub last_active: Instant,
    
    /// Authentication state
    pub auth_state: AuthState,
}

/// Type of client interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceType {
    Terminal,
    Signal,
    SMS,
}

/// Authentication state for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthState {
    /// Not yet authenticated
    Unauthenticated,
    
    /// Authenticated but no character selected
    Authenticated,
    
    /// Fully connected with character
    CharacterSelected,
}

impl Session {
    /// Create a new session
    pub fn new(
        id: SessionId,
        interface: InterfaceType,
        connection_ids: ConnectionIdentifiers,
        buffer_size: usize,
    ) -> (Self, MessageReceiver) {
        // RUST LEARNING: Channel creation
        // Returns a tuple: (sender, receiver)
        let (tx, rx) = mpsc::channel(buffer_size);
        
        let session = Self {
            id,
            player_id: None,
            account_id: None,
            interface,
            connection_ids,
            tx,
            created_at: Instant::now(),
            last_active: Instant::now(),
            auth_state: AuthState::Unauthenticated,
        };
        
        (session, rx)
    }
    
    /// Send a message to this session
    /// 
    /// RUST LEARNING: Async Functions
    /// - `async fn` can be awaited
    /// - Must be called with `.await` from another async context
    /// - Returns Result<(), SendError> from the channel
    pub async fn send(&self, msg: ServerMessage) -> Result<(), mpsc::error::SendError<ServerMessage>> {
        // RUST LEARNING: `.send().await` is async operation
        // It will wait until the message is queued
        self.tx.send(msg).await
    }
    
    /// Mark session as active
    pub fn mark_active(&mut self) {
        self.last_active = Instant::now();
    }
    
    /// Check if session has timed out
    pub fn is_timed_out(&self, timeout: std::time::Duration) -> bool {
        self.last_active.elapsed() > timeout
    }
    
    /// Authenticate the session
    pub fn authenticate(&mut self, account_id: AccountId) {
        self.account_id = Some(account_id);
        self.auth_state = AuthState::Authenticated;
    }
    
    /// Associate with a player/character
    pub fn select_character(&mut self, player_id: PlayerId) {
        self.player_id = Some(player_id);
        self.auth_state = AuthState::CharacterSelected;
    }
    
    /// Get primary connection identifier for logging/display
    pub fn connection_identifier(&self) -> String {
        self.connection_ids.primary_id()
    }
    
    /// Check if session is ready to play (authenticated and character selected)
    pub fn is_ready(&self) -> bool {
        self.auth_state == AuthState::CharacterSelected && self.player_id.is_some()
    }
}

// TODO: Implement actual authentication logic
// TODO: Add session persistence for reconnection
// TODO: Add rate limiting per session
// TODO: Link connection identifiers to accounts for authentication
// TODO: Handle multiple simultaneous sessions for same account (terminal + phone)
