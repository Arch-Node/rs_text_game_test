// terminal_client/client.rs
//
// SpacetimeDB WebSocket Client for Terminal Interface
//
// This client connects to SpacetimeDB via WebSocket SDK to interact with the game backend.

use crate::spacetimedb_client::{
    connect_session, authenticate_player, submit_command, execute_tick,
    CommandLog, CommandLogTableAccess, PlayerTableAccess, SessionTableAccess,
    DbConnection,
};
use crate::sdk_utils::create_connection_with_processor;
use crate::event_broadcaster::{setup_event_monitoring, format_event, GameEvent};
use spacetimedb_sdk::{DbContext, Table};
use std::sync::{Arc, Mutex};
use std::error::Error;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;

/// SpacetimeDB connection configuration
#[derive(Debug, Clone)]
pub struct SpacetimeConfig {
    /// SpacetimeDB server URL (e.g., ws://localhost:3000)
    pub server_url: String,
    
    /// Database/module name
    pub database_name: String,
}

impl Default for SpacetimeConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:3000".to_string(),
            database_name: "text-game".to_string(),
        }
    }
}

/// Terminal client for SpacetimeDB using WebSocket SDK
pub struct TerminalClient {
    config: SpacetimeConfig,
    conn: Arc<DbConnection>,
    
    /// Current session ID (assigned by SpacetimeDB)
    session_id: Option<u64>,
    
    /// Current player ID
    player_id: Option<u64>,
    
    /// Track latest command result
    latest_result: Arc<Mutex<Option<String>>>,
    
    /// Event receiver for multiplayer notifications
    event_rx: Option<mpsc::Receiver<GameEvent>>,
    
    /// Player name (needed for event filtering)
    player_name: Option<String>,
}

impl TerminalClient {
    /// Create a new terminal client with WebSocket connection
    pub async fn new(config: SpacetimeConfig) -> Result<Self, Box<dyn Error>> {
        println!("🔌 Connecting to SpacetimeDB via WebSocket...");
        
        // Track latest command result
        let latest_result = Arc::new(Mutex::new(None));
        let latest_result_clone = latest_result.clone();
        
        // Create connection with background processor
        let conn = create_connection_with_processor(&config.server_url, &config.database_name).await?;
        
        // Set up callback for command results
        conn.db().command_log().on_insert(move |_ctx, log: &CommandLog| {
            if let Some(ref result) = log.result {
                let mut latest = latest_result_clone.lock().unwrap();
                *latest = Some(result.clone());
            }
        });
        
        // Give connection time to establish
        sleep(Duration::from_millis(500)).await;
        
        println!("   ✅ Connected with real-time subscriptions");
        
        Ok(Self {
            config,
            conn,
            session_id: None,
            player_id: None,
            latest_result,
            event_rx: None,
            player_name: None,
        })
    }
    
    /// Connect to SpacetimeDB and create a session
    pub async fn connect(&mut self, connection_id: String) -> Result<u64, Box<dyn Error>> {
        println!("🔑 Creating session...");
        
        // Call connect_session reducer
        let reducers = self.conn.reducers();
        reducers.connect_session("terminal".to_string(), connection_id.clone())?;
        
        // Wait for session to be created
        sleep(Duration::from_secs(1)).await;
        
        // Query session table to find the session ID
        let sessions: Vec<_> = self.conn.db().session()
            .iter()
            .filter(|s| s.connection_id == connection_id)
            .collect();
        
        if let Some(session) = sessions.first() {
            self.session_id = Some(session.id);
            println!("   ✅ Session created: {}", session.id);
            Ok(session.id)
        } else {
            Err("Failed to find session in database".into())
        }
    }
    
    /// Authenticate and select/create character
    pub async fn authenticate(
        &mut self,
        session_id: u64,
        player_name: String,
    ) -> Result<u64, Box<dyn Error>> {
        println!("📝 Authenticating as '{}'...", player_name);
        
        // Call authenticate_player reducer
        let reducers = self.conn.reducers();
        reducers.authenticate_player(session_id, player_name.clone())?;
        
        // Wait for authentication to complete
        sleep(Duration::from_secs(1)).await;
        
        // Query player table to find the player ID
        let players: Vec<_> = self.conn.db().player()
            .iter()
            .filter(|p| p.name == player_name)
            .collect();
        
        if let Some(player) = players.first() {
            self.player_id = Some(player.id);
            self.player_name = Some(player_name.clone());
            
            // Set up event monitoring for multiplayer notifications
            let (event_tx, event_rx) = mpsc::channel(100);
            setup_event_monitoring(self.conn.clone(), player_name.clone(), event_tx);
            self.event_rx = Some(event_rx);
            
            println!("   ✅ Authenticated as '{}' (ID: {})", player_name, player.id);
            println!("   🌐 Multiplayer event monitoring enabled");
            Ok(player.id)
        } else {
            Err("Failed to find player in database".into())
        }
    }
    
    /// Submit a command to the game
    pub async fn submit_command(&self, command: String) -> Result<CommandResult, Box<dyn Error>> {
        self.session_id.ok_or("Not connected - call connect() first")?;
        
        // Clear previous result
        {
            let mut result = self.latest_result.lock().unwrap();
            *result = None;
        }
        
        // Call submit_command reducer
        let reducers = self.conn.reducers();
        reducers.submit_command(command.clone())?;
        
        Ok(CommandResult {
            success: true,
            message: "Command queued for next tick".to_string(),
            is_queued: true,
        })
    }
    
    /// Execute a tick and wait for result
    pub async fn execute_tick_and_wait(&self) -> Result<Option<String>, Box<dyn Error>> {
        // Execute tick
        let reducers = self.conn.reducers();
        reducers.execute_tick()?;
        
        // Wait for tick to process
        sleep(Duration::from_secs(1)).await;
        
        // Get result from callback
        let result = self.latest_result.lock().unwrap();
        Ok(result.clone())
    }
    
    /// Get player information from local cache
    pub async fn get_player_info(&self) -> Result<PlayerInfo, Box<dyn Error>> {
        let player_id = self.player_id.ok_or("Not authenticated - call authenticate() first")?;
        
        // Query player table from local cache
        let players: Vec<_> = self.conn.db().player()
            .iter()
            .filter(|p| p.id == player_id)
            .collect();
        
        if let Some(player) = players.first() {
            Ok(PlayerInfo {
                id: player.id,
                name: player.name.clone(),
                class: player.class.clone(),
                position_x: player.position_x,
                position_y: player.position_y,
                position_z: player.position_z,
                dimension: player.dimension.clone(),
                status: player.status.clone(),
            })
        } else {
            Err("Player not found in local cache".into())
        }
    }
    
    /// Get all players in the dimension (for multiplayer visibility)
    pub fn get_players_in_dimension(&self, dimension: &str) -> Vec<PlayerInfo> {
        self.conn.db().player()
            .iter()
            .filter(|p| p.dimension == dimension)
            .map(|p| PlayerInfo {
                id: p.id,
                name: p.name.clone(),
                class: p.class.clone(),
                position_x: p.position_x,
                position_y: p.position_y,
                position_z: p.position_z,
                dimension: p.dimension.clone(),
                status: p.status.clone(),
            })
            .collect()
    }
    
    /// Check for and return pending multiplayer events
    /// Returns None if no events are available
    ///
    /// EVENT OPTIMIZATION CHANGES NEEDED (See docs/evnet_structure.md):
    /// ----------------------------------------------------------------
    /// PHASE 2: Binary Encoding
    ///   - Accept both JSON and binary event formats
    ///   - Add format parameter to connection config
    ///   - Deserialize events based on negotiated format:
    ///     match format {
    ///         EventFormat::Json => serde_json::from_slice(&data)?,
    ///         EventFormat::MessagePack => rmp_serde::from_slice(&data)?,
    ///     }
    ///
    /// PHASE 3: Dictionary Compression
    ///   - Maintain local string table for interned strings
    ///   - Handle StringTableAdd events to populate table
    ///   - Resolve string IDs back to strings for display
    ///   - Example: player_name_id: 42 -> "Alice" (from local table)
    ///
    /// PHASE 4: Delta Compression
    ///   - Maintain previous state for this player
    ///   - Reconstruct full state from delta events
    ///   - Example: PlayerMovedDelta { to_x: Some(5) } + previous state
    pub fn poll_event(&mut self) -> Option<String> {
        if let Some(ref mut rx) = self.event_rx {
            // Try to receive without blocking
            match rx.try_recv() {
                Ok(event) => {
                    // Get current player position for filtering
                    let current_pos = self.player_id.and_then(|id| {
                        let players: Vec<_> = self.conn.db().player()
                            .iter()
                            .filter(|p| p.id == id)
                            .collect();
                        players.first().map(|p| {
                            (p.position_x, p.position_y, p.position_z, p.dimension.clone())
                        })
                    });
                    
                    format_event(&event, current_pos)
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

/// Result from submitting a command
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub is_queued: bool,
}

/// Player information from database
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub id: u64,
    pub name: String,
    pub class: String,
    pub position_x: i32,
    pub position_y: i32,
    pub position_z: i32,
    pub dimension: String,
    pub status: String,
}
