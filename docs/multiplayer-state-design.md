[← Back to Main README](../README.md)

---

# Multiplayer State Model Design
**Task 1.2 - Tick-Based Action Queue System**  
**Date:** January 23, 2026

## Core Design Philosophy

**Tick-Based Execution Model:**
Players submit actions → Actions are validated → Queued for execution → Execute on next heartbeat/tick

This provides:
- ✅ Consistent game state (all actions execute at same time)
- ✅ Fair turn ordering
- ✅ Easy to implement skill checks and combat
- ✅ Natural synchronization point
- ✅ Reduced race conditions
- ✅ Predictable behavior for multiplayer

---

## 1. Architecture Overview

```
Player Input → Validation → Command Queue → [TICK/HEARTBEAT] → Execution → State Update → Broadcast Events
```

### Tick Cycle
```rust
loop {
    // 1. Receive commands from all connected players
    receive_player_commands();
    
    // 2. Validate each command
    validate_and_queue_commands();
    
    // 3. Wait for tick (heartbeat)
    wait_for_tick();  // e.g., every 2-5 seconds
    
    // 4. Execute all queued commands in order
    execute_command_batch();
    
    // 5. Update world state
    update_world_state();
    
    // 6. Broadcast results to affected players
    broadcast_results();
}
```

---

## 2. State Model Structure

### 2.1 Top-Level State Container
```rust
pub struct MultiplayerGameState {
    /// Static world data (rooms, layers, anchors) - rarely changes
    pub world: Arc<World>,
    
    /// Active players indexed by ID
    pub players: HashMap<PlayerId, Player>,
    
    /// Active sessions indexed by ID
    pub sessions: HashMap<SessionId, Session>,
    
    /// Command queue for next tick
    pub command_queue: CommandQueue,
    
    /// Room occupancy tracking (which players are where)
    pub room_occupancy: HashMap<Coord, HashSet<PlayerId>>,
    
    /// World tick counter
    pub tick_count: u64,
    
    /// Dynamic world state (items, NPCs, environment)
    pub world_state: DynamicWorldState,
    
    /// Server running flag
    pub is_running: bool,
}
```

### 2.2 Static World (Read-Only, Shared)
```rust
/// Static world data - loaded once, shared by all players
pub struct World {
    pub meta: Meta,
    pub layers: HashMap<i32, Layer>,
    pub rooms_by_coord: HashMap<Coord, Room>,
    pub anchors_by_id: HashMap<String, Anchor>,
    pub start: Coord,
}

// Wrapped in Arc for cheap cloning across async tasks
pub type SharedWorld = Arc<World>;
```

**Characteristics:**
- Loaded from `rooms.json` at server startup
- Never modified during gameplay (or very rarely)
- Can be shared across threads without locking
- Cloned cheaply via `Arc`

### 2.3 Player State
```rust
pub struct Player {
    /// Unique player identifier
    pub id: PlayerId,
    
    /// Display name
    pub name: String,
    
    /// Account ID (one account can have multiple characters)
    pub account_id: AccountId,
    
    /// Character class
    pub class: PlayerClass,
    
    /// Current position in world
    pub pos: Coord,
    
    /// Connection status
    pub status: PlayerStatus,
    
    /// Last action timestamp
    pub last_action: Instant,
    
    /// Current action state (if executing multi-tick action)
    pub action_state: Option<ActionState>,
    
    // Future additions:
    // pub inventory: Inventory,
    // pub stats: PlayerStats,
    // pub skills: SkillSet,
    // pub equipment: Equipment,
    // pub quests: QuestLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
    Online,
    Idle,
    InCombat,
    Trading,
    Offline,
}

/// For actions that take multiple ticks (future)
pub struct ActionState {
    pub action_type: ActionType,
    pub ticks_remaining: u32,
    pub target: Option<TargetId>,
}
```

### 2.4 Session Management
```rust
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,
    
    /// Associated player ID (if authenticated)
    pub player_id: Option<PlayerId>,
    
    /// Account ID (for authentication)
    pub account_id: Option<AccountId>,
    
    /// Interface type
    pub interface: InterfaceType,
    
    /// Connection channel for sending messages
    pub tx: MessageSender,
    
    /// Session metadata
    pub created_at: Instant,
    pub last_active: Instant,
    
    /// Authentication state
    pub auth_state: AuthState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceType {
    Terminal,
    Signal,
    SMS,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthState {
    Unauthenticated,
    Authenticated,
    CharacterSelected,
}

/// Message sender for async communication
pub type MessageSender = mpsc::Sender<ServerMessage>;
```

### 2.5 Command Queue System
```rust
pub struct CommandQueue {
    /// Commands waiting to execute on next tick
    pub pending: Vec<QueuedCommand>,
    
    /// Priority commands (execute first)
    pub priority: Vec<QueuedCommand>,
    
    /// Failed validations (for feedback)
    pub failed: Vec<FailedCommand>,
}

pub struct QueuedCommand {
    /// When this was queued
    pub queued_at: Instant,
    
    /// Which player issued this
    pub player_id: PlayerId,
    
    /// Session that sent it
    pub session_id: SessionId,
    
    /// The actual command
    pub command: Command,
    
    /// Execution priority
    pub priority: CommandPriority,
    
    /// Validation result
    pub validation: ValidationResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    System = 3,
}

pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct FailedCommand {
    pub player_id: PlayerId,
    pub session_id: SessionId,
    pub command: String,
    pub reason: String,
    pub timestamp: Instant,
}
```

---

## 3. Command Processing Flow

### 3.1 Command Submission
```rust
/// Player submits a command from their interface
pub async fn submit_command(
    state: &MultiplayerGameState,
    session_id: SessionId,
    command_text: String,
) -> Result<()> {
    // 1. Get session and player info
    let session = state.sessions.get(&session_id)?;
    let player_id = session.player_id.ok_or("Not authenticated")?;
    let player = state.players.get(&player_id)?;
    
    // 2. Parse command text
    let command = parse_command(&command_text)?;
    
    // 3. Validate command
    let validation = validate_command(state, player, &command);
    
    // 4. INSTANT COMMANDS: Execute immediately (no queueing)
    // Observation/info commands like Look, Inventory, Status, Help, Say
    if validation.is_instant {
        if validation.is_valid {
            execute_command_immediately(state, player_id, session_id, &command).await?;
            return Ok(());
        } else {
            send_error_to_session(session_id, &validation.errors).await;
            return Err("Instant command validation failed");
        }
    }
    
    // 5. QUEUED COMMANDS: Check if player already has a command queued
    if state.command_queue.player_has_queued_command(player_id) {
        send_error_to_session(
            session_id,
            "You already have a command queued. Wait for the next tick to submit another action."
        ).await;
        return Err("Command already queued for this player");
    }
    
    // 6. Validate queued command
    
    if !validation.is_valid {
        // Send immediate feedback
        send_error_to_session(session_id, &validation.errors).await;
        
        // Record failed command
        state.command_queue.failed.push(FailedCommand {
            player_id,
            session_id,
            command: command_text,
            reason: validation.errors.join("; "),
            timestamp: Instant::now(),
        });
        
        return Err("Command validation failed");
    }
    
    // 7. Queue for execution on next tick
    let queued = QueuedCommand {
        queued_at: Instant::now(),
        player_id,
        session_id,
        command,
        priority: CommandPriority::Normal,
        validation,
    };
    
    state.command_queue.pending.push(queued);
    
    // 8. Send acknowledgment
    send_to_session(session_id, "Command queued for next tick.").await;
    
    Ok(())
}
```

### 3.2 Command Validation
```rust
/// Validate command before queueing
pub fn validate_command(
    state: &MultiplayerGameState,
    player: &Player,
    command: &Command,
) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    match command {
        Command::Go { raw } => {
            // Check if exit exists in current room
            let room = state.world.rooms_by_coord.get(&player.pos).unwrap();
            let exit_id = normalize_exit_name(raw);
            
            if !room.exits.contains_key(&exit_id) {
                errors.push(format!("No exit '{}' in current room", raw));
            }
            
            // Check if player can move (not in combat, not trading, etc.)
            if player.status == PlayerStatus::InCombat {
                errors.push("Cannot move while in combat".to_string());
            }
        }
        
        Command::Teleport { anchor_id } => {
            // Check class ability
            if !player.class.can_teleport() {
                errors.push("Your class cannot teleport".to_string());
            }
            
            // Check if anchor exists
            if !state.world.anchors_by_id.contains_key(anchor_id) {
                errors.push(format!("Unknown anchor '{}'", anchor_id));
            }
            
            // Check same layer restriction
            if let Some(anchor) = state.world.anchors_by_id.get(anchor_id) {
                if anchor.pos.layer != player.pos.layer {
                    errors.push("Cannot teleport between layers".to_string());
                }
            }
        }
        
        Command::Shift { dir } => {
            if !player.class.can_shift_layers() {
                errors.push("Your class cannot shift layers".to_string());
            }
            // Additional validation for layer shifting...
        }
        
        Command::Look | Command::Where | Command::Help => {
            // These are always valid
        }
        
        Command::Quit => {
            warnings.push("You will disconnect on next tick".to_string());
        }
    }
    
    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
    }
}
```

### 3.3 Tick Execution
```rust
/// Execute all queued commands on tick
pub async fn execute_tick(state: &mut MultiplayerGameState) -> Result<()> {
    state.tick_count += 1;
    
    // 1. Sort commands by priority
    let mut commands = std::mem::take(&mut state.command_queue.pending);
    commands.sort_by_key(|c| c.priority);
    
    // 2. Execute each command
    let mut events = Vec::new();
    
    for queued in commands {
        match execute_queued_command(state, &queued).await {
            Ok(mut cmd_events) => {
                events.append(&mut cmd_events);
            }
            Err(e) => {
                // Send error to player
                send_error_to_player(queued.session_id, &e).await;
            }
        }
    }
    
    // 3. Process world updates
    process_world_updates(state).await;
    
    // 4. Broadcast events
    broadcast_events(state, events).await;
    
    // 5. Clean up
    cleanup_disconnected_sessions(state).await;
    
    Ok(())
}

async fn execute_queued_command(
    state: &mut MultiplayerGameState,
    queued: &QueuedCommand,
) -> Result<Vec<GameEvent>> {
    let player = state.players.get_mut(&queued.player_id)?;
    let mut events = Vec::new();
    
    match &queued.command {
        Command::Go { raw } => {
            let old_pos = player.pos;
            let room = state.world.rooms_by_coord.get(&old_pos).unwrap();
            let exit_id = normalize_exit_name(raw);
            let exit_spec = room.exits.get(&exit_id)?;
            
            // Calculate destination
            let new_pos = resolve_exit_target(old_pos, exit_spec)?;
            
            // Update player position
            player.pos = new_pos;
            
            // Update room occupancy
            state.room_occupancy
                .entry(old_pos)
                .or_default()
                .remove(&player.id);
            state.room_occupancy
                .entry(new_pos)
                .or_default()
                .insert(player.id);
            
            // Create events
            events.push(GameEvent::PlayerLeft {
                player_id: player.id,
                player_name: player.name.clone(),
                from_room: old_pos,
                exit_used: exit_id.clone(),
            });
            
            events.push(GameEvent::PlayerEntered {
                player_id: player.id,
                player_name: player.name.clone(),
                to_room: new_pos,
            });
            
            events.push(GameEvent::PlayerMoved {
                player_id: player.id,
                from: old_pos,
                to: new_pos,
            });
        }
        
        Command::Look => {
            // Read-only, just send room description
            events.push(GameEvent::RoomDescription {
                player_id: player.id,
                room_coord: player.pos,
            });
        }
        
        Command::Quit => {
            events.push(GameEvent::PlayerDisconnected {
                player_id: player.id,
                player_name: player.name.clone(),
            });
        }
        
        // ... other commands
        _ => {}
    }
    
    Ok(events)
}
```

---

## 4. Room Occupancy Tracking

### 4.1 Data Structure
```rust
/// Maps room coordinates to set of players in that room
pub type RoomOccupancy = HashMap<Coord, HashSet<PlayerId>>;

impl MultiplayerGameState {
    /// Get all players in a specific room
    pub fn players_in_room(&self, coord: Coord) -> Vec<&Player> {
        self.room_occupancy
            .get(&coord)
            .map(|player_ids| {
                player_ids
                    .iter()
                    .filter_map(|id| self.players.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get all players within proximity of a coordinate
    pub fn players_in_proximity(&self, center: Coord, radius: u32) -> Vec<&Player> {
        self.room_occupancy
            .iter()
            .filter(|(coord, _)| manhattan_distance(center, **coord) <= radius)
            .flat_map(|(_, player_ids)| {
                player_ids.iter().filter_map(|id| self.players.get(id))
            })
            .collect()
    }
}

fn manhattan_distance(a: Coord, b: Coord) -> u32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()) as u32
}
```

---

## 5. Event Broadcasting

### 5.1 Event Types
```rust
pub enum GameEvent {
    /// Player moved between rooms
    PlayerMoved {
        player_id: PlayerId,
        from: Coord,
        to: Coord,
    },
    
    /// Player entered a room (visible to others in room)
    PlayerEntered {
        player_id: PlayerId,
        player_name: String,
        to_room: Coord,
    },
    
    /// Player left a room (visible to others in room)
    PlayerLeft {
        player_id: PlayerId,
        player_name: String,
        from_room: Coord,
        exit_used: String,
    },
    
    /// Player joined the game
    PlayerConnected {
        player_id: PlayerId,
        player_name: String,
    },
    
    /// Player disconnected
    PlayerDisconnected {
        player_id: PlayerId,
        player_name: String,
    },
    
    /// Room description (only to requesting player)
    RoomDescription {
        player_id: PlayerId,
        room_coord: Coord,
    },
    
    /// World tick completed
    TickCompleted {
        tick_count: u64,
    },
    
    // Future events:
    // Chat message
    // Combat action
    // Item taken/dropped
    // NPC interaction
    // etc.
}
```

### 5.2 Event Broadcasting Logic
```rust
pub async fn broadcast_events(
    state: &MultiplayerGameState,
    events: Vec<GameEvent>,
) -> Result<()> {
    for event in events {
        match event {
            GameEvent::PlayerEntered { to_room, player_name, .. } => {
                // Send to all players in the room
                let players = state.players_in_room(to_room);
                for player in players {
                    if let Some(session) = find_session_for_player(state, player.id) {
                        send_message(session, format!("{} enters the room.", player_name)).await;
                    }
                }
            }
            
            GameEvent::PlayerLeft { from_room, player_name, exit_used, .. } => {
                // Send to all players in the room
                let players = state.players_in_room(from_room);
                for player in players {
                    if let Some(session) = find_session_for_player(state, player.id) {
                        send_message(session, format!("{} leaves {} .", player_name, exit_used)).await;
                    }
                }
            }
            
            GameEvent::RoomDescription { player_id, room_coord } => {
                // Send only to the requesting player
                if let Some(session) = find_session_for_player(state, player_id) {
                    let room = state.world.rooms_by_coord.get(&room_coord).unwrap();
                    let description = format_room_description(room, state);
                    send_message(session, description).await;
                }
            }
            
            GameEvent::PlayerConnected { player_name, .. } => {
                // Global announcement
                broadcast_to_all(state, format!("{} has joined the game!", player_name)).await;
            }
            
            GameEvent::PlayerDisconnected { player_name, .. } => {
                // Global announcement
                broadcast_to_all(state, format!("{} has left the game.", player_name)).await;
            }
            
            _ => {}
        }
    }
    
    Ok(())
}
```

---

## 6. Concurrency Strategy

### 6.1 Locking Approach
```rust
use tokio::sync::{RwLock, Mutex};

/// Thread-safe game state wrapper
pub struct SafeGameState {
    /// Static world (no lock needed)
    pub world: Arc<World>,
    
    /// Mutable game state (write-protected)
    pub state: Arc<RwLock<MultiplayerGameState>>,
}

impl SafeGameState {
    /// Read-only access to state
    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, MultiplayerGameState> {
        self.state.read().await
    }
    
    /// Mutable access to state (blocks other readers/writers)
    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, MultiplayerGameState> {
        self.state.write().await
    }
}
```

### 6.2 Lock-Free Operations
- Reading static world data (no lock)
- Session message sending (channel-based)
- Individual player state queries (fine-grained locks possible)

### 6.3 Locked Operations
- Command queue modifications
- Player position updates
- Room occupancy changes
- Tick execution (exclusive lock for entire tick)

---

## 7. Tick Timing Configuration

```rust
pub struct TickConfig {
    /// How long between ticks
    pub tick_interval: Duration,
    
    /// Maximum commands per player per tick (ALWAYS 1)
    /// Players can only have ONE command queued at any time
    pub max_commands_per_player: usize,
    
    /// Command timeout (discard if queued too long)
    pub command_timeout: Duration,
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            tick_interval: Duration::from_secs(15),   // 15 second ticks (production)
            max_commands_per_player: 1,               // Strictly ONE action per player
            command_timeout: Duration::from_secs(30), // Discard after 30s
        }
    }
}

// For testing/development
impl TickConfig {
    pub fn fast_testing() -> Self {
        Self {
            tick_interval: Duration::from_secs(3),    // 3 second ticks (testing)
            max_commands_per_player: 1,               // Still only ONE
            command_timeout: Duration::from_secs(10),
        }
    }
}
```

**Command Types:**

- **Instant Commands:** Execute immediately, never queued
  - Observation: look, examine, inventory, status
  - Information: help, who, score, time
  - Communication: say, tell, emote
  - These bypass the tick system entirely
  - You can look while waiting for your queued action
  
- **Queued Commands:** Execute on next tick
  - Movement: go, move, enter, leave
  - Actions: take, drop, attack, cast, craft, gather
  - Anything that changes the world state

**Command Throttling Policy:**

- **ONE QUEUED COMMAND AT A TIME:** Each player/character can only have one queued command
- **Instant commands don't count:** You can look/talk/check status anytime
- **Immediate Rejection:** If player tries to submit another queued command, they get:
  - Error message: "You already have a command queued. Wait for the next tick to submit another action."
  - Command is NOT queued
  - Player must wait for tick to execute before submitting next queued command
- **Prevents Spam:** Players can't flood the action queue
- **Fair Play:** Everyone gets exactly one action per tick
- **Clear Feedback:** Players know immediately if their command is queued or rejected

**Example Flow:**
```
Player: "go north"
Server: "Command queued for next tick."

[Before tick executes - instant commands work]
Player: "look"
Server: [Immediately shows room description]

Player: "inventory"
Server: [Immediately shows inventory]

Player: "take sword"  ← Tries second queued command
Server: "You already have a command queued. Wait for the next tick to submit another action."

[Tick executes after 15 seconds]
Player moves north

Player: "look"  ← Instant
Server: [Shows new room]

Player: "take sword"  ← Now allowed
Server: "Command queued for next tick."
```

**Tick Rate Philosophy:**

- **Production (15 seconds):** 
  - Gives players time to think and plan
  - Reduces server load
  - Better for SMS interface (accounts for message latency)
  - Encourages strategic, thoughtful gameplay
  - Perfect for skills-based challenges
  - Allows time for coordination between players

- **Testing/Development (3 seconds):**
  - Fast feedback during development
  - Quick iteration on gameplay mechanics
  - Easier to test multiplayer interactions
  - Good for debugging command queue

**Configurable per Environment:**
```rust
// Load from environment or config file
let tick_config = if cfg!(debug_assertions) {
    TickConfig::fast_testing()  // 3s for dev/test
} else {
    TickConfig::default()       // 15s for production
};
```

**Advantages of 15-Second Ticks:**
- ✅ Players can queue multiple actions mentally
- ✅ Works great for text/SMS with network delays
- ✅ Encourages planning over twitch reflexes
- ✅ Fits skills-heavy, not combat-heavy design
- ✅ Allows for complex skill checks and resolution
- ✅ Better for async gameplay (check game periodically)

---

## Summary

### Key Architectural Decisions

✅ **Tick-based execution:** Commands queued and executed in batches  
✅ **Validation before queueing:** Immediate feedback on invalid commands  
✅ **Event broadcasting:** State changes generate events sent to affected players  
✅ **Room-based visibility:** Players see events in their room  
✅ **Priority queue:** System commands execute before player commands  
✅ **Async I/O:** Non-blocking command submission and event delivery  

### State Separation

- **Static (Arc):** World, rooms, layers, anchors
- **Dynamic (RwLock):** Players, sessions, room occupancy, command queue
- **Per-Session (Channel):** Message delivery

### Next Steps

- ✅ 1.2 Define Multiplayer State Model - COMPLETE
- ⏭️ 1.3 Design Command/Message Protocol
- ⏭️ 1.4 Architecture Pattern Selection (partially done - tick-based chosen)
- ⏭️ 1.5 Session Abstraction Layer

---

[← Back to Main README](../README.md)
