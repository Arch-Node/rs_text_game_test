# Multiplayer System

> **⚠️ STATUS:** These modules are **architectural stubs** from the original design phase.
> 
> **ACTUAL BACKEND:** The live multiplayer system uses SpacetimeDB directly.
> - Tables: `player`, `room`, `session`, `queued_command`, `command_log`
> - Location: `/home/micah/software/repos/text_game_stdb/src/lib.rs`
> - SDK Bindings: `src/spacetimedb_client/` (19 auto-generated files)
>
> **PURPOSE OF THESE MODULES:**
> - Design documentation and learning reference
> - Potential bridge layer for client-side logic
> - May be refactored or deprecated as SDK integration completes

This directory contains the **original design** for a multiplayer game server with a tick-based command queue system. The actual implementation uses SpacetimeDB tables and reducers instead.

## 📖 Main Documentation

For the complete project overview, see the [main README](../../README.md).

For detailed architectural analysis and design decisions, see:
- [Architecture Analysis](../../docs/architecture-analysis.md) - Current single-player system analysis
- [Multiplayer State Design](../../docs/multiplayer-state-design.md) - Detailed tick-based system design
- [SpacetimeDB Integration](../../docs/spacetimedb-integration.md) - Backend architecture choices
- [SDK Client Guide](../../SDK_CLIENT_GUIDE.md) - SpacetimeDB SDK setup and usage
- [TODO Roadmap](../../TODO.md) - Full development roadmap

---

## 🎯 Design Philosophy

**Tick-Based Action Queue System:**
- Players submit actions → Validated → Queued → Execute on heartbeat → Broadcast results
- **Production tick rate:** 15 seconds (thoughtful, strategic gameplay)
- **Testing tick rate:** 3 seconds (fast iteration)
- Perfect for skills-heavy, less combat-heavy gameplay

**Multi-Dimensional World:**
- Separate 3D spatial planes (dimensions) that can shift and interact
- Each dimension maintains independent coordinate systems
- Players exist at (x, y, z) within a specific dimension
- Enables complex mechanics: dimension-shifting, parallel quests, ethereal travel

---

## 🗄️ Backend: SpacetimeDB

**Why SpacetimeDB:**
- Database + server combined into one
- Built-in real-time state synchronization (~100μs latency)
- ACID transactions prevent race conditions
- Purpose-built for multiplayer games (100k tx/s)
- No Docker/Kubernetes complexity

**Module Location:** `/home/micah/software/repos/text_game_stdb/`

### Database Schema

**Tables:**
- `player` - Character data with 4D position (x, y, z, dimension) and game state
- `room` - 3D rooms with coordinates (10 test rooms centered at 100,100,100)
- `session` - Active connections (Terminal/Signal/Discord) linked to players
- `queued_command` - Commands awaiting next tick execution
- `command_log` - Complete audit trail of all executed commands

**Reducers (Game Logic):**
- `connect_session(interface_type, connection_id)` - Create new connection
- `authenticate_player(session_id, player_name)` - Create/link player to session
- `submit_command(command_text)` - Queue command for next tick
- `execute_tick()` - Process all queued commands
- `create_room(...)` - Insert room into database
- `get_current_room()` - Query room at player's position
- `list_rooms_in_dimension(dimension)` - Query all rooms in dimension

**3D Movement:**
- Full 6-directional movement: north/south (±y), east/west (±x), up/down (±z)
- Coordinate center at (100,100,100) to avoid negative numbers in SQL
- Movement handled by `execute_tick` reducer after command validation

**Reducers (Game Logic):**
- `connect_session` - Create new connection
- `authenticate_player` - Link session to player account
- `submit_command` - Queue command (enforces one-per-player limit)
- `execute_tick` - Process all queued commands in batch
- `get_player_info` - Query player state

### Dimensions

Players exist in separate 3D spatial planes called **dimensions**:

**Coordinate System:**
- `position_x`, `position_y`, `position_z` - 3D coordinates within the dimension
- `dimension` - The spatial plane (e.g., "material", "ethereal", "shadow", "dream")

**Key Properties:**
- Dimensions can pass near each other and shift
- Coordinates don't directly map between dimensions
- A player at (10, 5, 0) in "material" ≠ (10, 5, 0) in "ethereal"
- Requires dimension-shifting abilities to traverse

**Example Use Cases:**
- Ethereal plane for spirit quests
- Shadow dimension for stealth mechanics
- Dream realm for visions/prophecies
- Material world as the default plane

---

## 🏗️ Architecture Overview

### Core Components

```
src/multiplayer/
├── mod.rs              # Module exports and public API
├── types.rs            # Core type definitions (PlayerId, SessionId, AccountId)
├── player.rs           # Player state and management
├── session.rs          # Client session handling (Terminal, Signal, Discord)
├── state.rs            # Game state container with thread-safe access
├── command_queue.rs    # Command queuing and prioritization
├── validation.rs       # Pre-execution command validation
├── events.rs           # Event broadcasting to players
└── tick.rs             # Tick execution and world updates
```

### Data Flow

```
┌─────────────┐
│   Client    │ (Terminal/Signal/Discord)
└──────┬──────┘
       │ 1. Submit Command
       ▼
┌─────────────────────┐
│  Command Parser     │
└──────┬──────────────┘
       │ 2. Parse
       ▼
┌─────────────────────┐
│  Validator          │ → Immediate feedback if invalid
└──────┬──────────────┘
       │ 3. Queue (if valid)
       ▼
┌─────────────────────┐
│  Command Queue      │
└──────┬──────────────┘
       │
       │ [WAIT FOR TICK - 15 seconds]
       │
       ▼
┌─────────────────────┐
│  Tick Executor      │ → Execute all queued commands
└──────┬──────────────┘
       │ 4. Execute batch
       ▼
┌─────────────────────┐
│  Game State Update  │ → Update positions, stats, world
└──────┬──────────────┘
       │ 5. Generate events
       ▼
┌─────────────────────┐
│  Event Broadcaster  │ → Send to affected players
└──────┬──────────────┘
       │ 6. Deliver messages
       ▼
┌─────────────────────┐
│   Clients           │ → Receive updates
└─────────────────────┘
```

---

## 🔑 Key Concepts

### State Separation

- **Static World (Arc<World>):** Rooms, layers, exits, anchors
  - Loaded once from `rooms.json`
  - Read-only, shared across all tasks
  - No locking needed

- **Dynamic Game State (RwLock<GameState>):**
  - Players, sessions, command queue
  - Room occupancy tracking
  - Write-protected with read-write lock

- **Per-Session Communication (Channels):**
  - Each session has `mpsc::Sender<ServerMessage>`
  - Messages sent asynchronously
  - Non-blocking delivery

### Command Queue System

```rust
pub struct CommandQueue {
    pending: Vec<QueuedCommand>,    // Normal priority
    priority: Vec<QueuedCommand>,   // High priority (system commands)
    failed: Vec<FailedCommand>,     // Validation failures
}
```

**Command Lifecycle:**
1. **Submission:** Player sends command text
2. **Validation:** Check if action is valid (exit exists, class can perform, etc.)
3. **Queueing:** Add to pending/priority queue
4. **Execution:** Run on next tick (sorted by priority)
5. **Broadcasting:** Notify affected players

### Event System

Events are generated during command execution and broadcast to relevant players:

- **Room-based:** Events seen by players in same room (enter/leave)
- **Player-specific:** Events only for the acting player (look, where)
- **Global:** Events seen by all players (connect/disconnect)

```rust
pub enum GameEvent {
    PlayerMoved { player_id, from, to },
    PlayerEntered { player_id, player_name, to_room },
    PlayerLeft { player_id, player_name, from_room, exit_used },
    RoomDescription { player_id, room_coord },
    // ... more events
}
```

---

## 🔐 Concurrency & Thread Safety

### Smart Pointer Strategy

```rust
// Static world - cheap cloning via Arc
Arc<World>

// Mutable game state - read-write lock
Arc<RwLock<MultiplayerGameState>>

// Per-session messaging - async channels
mpsc::Sender<ServerMessage>
```

### Lock Granularity

- **No Lock:** Reading static world data
- **Read Lock:** Querying player positions, room occupancy
- **Write Lock:** Executing tick, updating positions, modifying queues

### Async Design

- All I/O operations are async (network, channels)
- Tick execution is async
- Event broadcasting is async
- Uses Tokio runtime for task scheduling

---

## 📊 Performance Characteristics

### Tick-Based Benefits

✅ **Predictable:** All state changes at known intervals  
✅ **Fair:** Commands execute in priority order  
✅ **Simple:** No complex locking during execution  
✅ **Scalable:** One batch update vs continuous processing  
✅ **Multi-Interface Friendly:** Accounts for varying latency across interfaces  

### Resource Usage

- **Memory:** O(players) for state, O(queued_commands) for queue
- **CPU:** Burst during tick execution, idle between ticks
- **Network:** Batched messages reduce overhead

---

## 🎮 Gameplay Implications

### 15-Second Ticks Enable

- **Thoughtful Strategy:** Time to plan next move
- **Coordination:** Players can discuss actions
- **Skill Checks:** Complex resolution during tick
- **Async Gameplay:** Check game periodically, not constant attention
- **Multi-Interface Compatibility:** Works despite varying message delays

### Command Types

**Instant Commands** (Execute Immediately):
- `look` - Observe room
- `examine` - Inspect item/player
- `inventory` - Check your items
- `status` - View character status
- `help` - Get help text
- `say` / `tell` / `emote` - Communication
- `who` / `score` / `time` - Info queries

These execute instantly with no delay. You can look while crafting, check inventory while traveling, etc.

**Queued Commands** (Execute on Tick):
- `go` / `move` - Movement
- `take` / `drop` - Item manipulation
- `attack` / `cast` - Combat actions
- `craft` / `gather` - Skill actions
- Any world-changing action

### Command Limits

- **ONE QUEUED COMMAND PER PLAYER AT A TIME**
- Instant commands (look, inventory, etc.) never count toward this limit
- If player tries to submit a second queued command before tick executes:
  - **Immediate rejection** with error message
  - "You already have a command queued. Wait for the next tick."
- Prevents spam and ensures fair play
- Everyone gets exactly one action per tick
- Clear feedback on command status

---

## 🚀 Getting Started (Future)

### Basic Server Loop (Planned)

```rust
async fn main() {
    // Load world
    let world = load_world("rooms.json")?;
    
    // Create game state
    let state = SafeGameState::new(
        MultiplayerGameState::new(Arc::new(world))
    );
    
    // Start tick loop
    tokio::spawn(tick_loop(state.clone()));
    
    // Start network listeners
    tokio::spawn(terminal_listener(state.clone()));
    tokio::spawn(signal_listener(state.clone()));
    tokio::spawn(discord_listener(state.clone()));
    
    // Run forever
    tokio::signal::ctrl_c().await?;
}

async fn tick_loop(state: SafeGameState) {
    let config = TickConfig::default();
    let mut interval = tokio::time::interval(config.tick_interval);
    
    loop {
        interval.tick().await;
        
        let mut game_state = state.write().await;
        execute_tick(&mut game_state).await?;
    }
}
```

---

## 📝 Learning Rust Patterns

Each module file contains extensive comments explaining Rust concepts:

- **[types.rs](types.rs):** Newtype pattern, Display trait
- **[player.rs](player.rs):** Structs, enums, impl blocks
- **[session.rs](session.rs):** Async channels, message passing
- **[state.rs](state.rs):** Arc, RwLock, thread safety
- **[command_queue.rs](command_queue.rs):** Collections, sorting, iterators
- **[validation.rs](validation.rs):** Pattern matching, Option/Result
- **[events.rs](events.rs):** Async/await, generic functions
- **[tick.rs](tick.rs):** Comprehensive async patterns

---

## 🛠️ Current Status

**Phase:** Architecture Design & Stub Implementation  
**Completed:**
- ✅ Architecture analysis
- ✅ State model design
- ✅ Module structure with educational stubs

**Next Steps:**
- Implement command parsing with player context
- Add async network listeners (Terminal, Signal, Discord)
- Implement authentication system
- Add persistence layer
- Write integration tests

---

## 🔗 Related Documentation

- [Main README](../README.md) - Project overview
- [TODO Roadmap](../TODO.md) - Full development plan
- [Architecture Analysis](../docs/architecture-analysis.md) - Current system analysis
- [Multiplayer State Design](../docs/multiplayer-state-design.md) - Detailed design specs

---

## 💡 Design Rationale

### Why Tick-Based?

1. **Skills-Heavy Gameplay:** Time to evaluate options, not twitch reflexes
2. **Multi-Interface Support:** Different interfaces have varying latency, ticks normalize this
3. **State Consistency:** All updates at once prevents race conditions
4. **Simplicity:** Easier to reason about than fully async
5. **Fairness:** Priority system ensures equitable execution

### Why 15 Seconds?

- Balances thoughtfulness with engagement
- Accommodates typical network round-trip times across interfaces
- Allows complex skill checks to resolve
- Encourages player coordination
- Reduces server load vs 1-second ticks

### Alternative Approaches Considered

❌ **Fully Async (Immediate Execution):**
- Complex concurrency issues
- Unfair for high-latency interfaces
- Hard to coordinate multiplayer actions

❌ **Pure Turn-Based (Wait for All Players):**
- Deadlocks if player disconnects
- Unfair for idle players
- Doesn't scale well

✅ **Hybrid Tick-Based (Chosen):**
- Best of both worlds
- Continuous for exploration
- Synchronized for resolution
- Scales to many players
