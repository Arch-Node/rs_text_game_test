# Current Single-Player Architecture Analysis
**Task 1.1 - Analysis for Multiplayer Transition**  
**Date:** January 23, 2026

## Overview
This document analyzes the current single-player architecture to identify what needs to change for multiplayer support.

---

## 1. Current GameState Structure

### Core State Container
```rust
pub struct GameState {
    pub world: World,      // The entire game world (static + dynamic)
    pub player: Player,    // Single player instance
    pub is_running: bool,  // Game loop control flag
}
```

**Key Observations:**
- ✅ Simple, monolithic state structure
- ⚠️ Single `Player` - needs to become collection of players
- ⚠️ `World` contains both static (rooms, layers) and potentially dynamic state
- ⚠️ `is_running` is global - needs per-session handling

---

## 2. World Structure (Static Content)

### World Container
```rust
pub struct World {
    pub meta: Meta,                           // World metadata (name, version)
    pub layers: HashMap<i32, Layer>,          // Layer definitions
    pub rooms_by_coord: HashMap<Coord, Room>, // All rooms indexed by coordinate
    pub anchors_by_id: HashMap<String, Anchor>, // Teleport points
    pub start: Coord,                         // Starting position
}
```

**Static vs Dynamic Analysis:**
- ✅ **Static (read-only, shareable):**
  - `meta` - world name, version
  - `layers` - layer definitions and scales
  - `rooms_by_coord` - room definitions, descriptions, exit definitions
  - `anchors_by_id` - teleport anchor points
  - `start` - starting coordinate
  
- ⚠️ **Potentially Dynamic (needs tracking):**
  - Room occupancy (which players are in which room)
  - Item locations (if items are added)
  - NPC positions and states (future)
  - Environmental states (doors, switches, etc. - future)

**Multiplayer Impact:**
- World can remain largely read-only
- Need separate structure for dynamic world state
- Room definitions are immutable, player positions are per-player

---

## 3. Player Model

### Current Player Structure
```rust
pub struct Player {
    pub class: PlayerClass,  // Teleporter or LayerWalker
    pub pos: Coord,          // Current position in world
}

pub enum PlayerClass {
    Teleporter,   // Can use teleport anchors
    LayerWalker,  // Can shift between layers
}
```

**Multiplayer Requirements:**
- ❌ Missing: Unique player identifier
- ❌ Missing: Player name/display name
- ❌ Missing: Inventory (future)
- ❌ Missing: Stats (health, mana, stamina - future)
- ❌ Missing: Skills (future)
- ❌ Missing: Experience/level (future)
- ❌ Missing: Equipped items (future)

**Changes Needed:**
```rust
// Proposed multiplayer player structure
pub struct Player {
    pub id: PlayerId,              // Unique identifier
    pub name: String,              // Display name
    pub account_id: AccountId,     // Link to account (for multiple characters)
    pub class: PlayerClass,        // Character class
    pub pos: Coord,                // Current position
    // Future additions:
    // pub inventory: Inventory,
    // pub stats: PlayerStats,
    // pub skills: SkillSet,
    // pub equipment: Equipment,
}
```

---

## 4. Command System

### Command Enumeration
```rust
pub enum Command {
    Look,                           // View current room
    Where,                          // Show coordinates
    Help,                           // Show available commands
    Quit,                           // Exit game
    Go { raw: String },             // Move through exit
    Teleport { anchor_id: String }, // Teleport to anchor (Teleporter only)
    Shift { dir: ShiftDir },        // Shift layers (LayerWalker only)
}
```

### Command Flow
1. **Input:** User types command string
2. **Parse:** `parse_command()` converts string → `Command` enum
3. **Execute:** `execute_command()` mutates `GameState` and writes to `MessageBuffer`
4. **Output:** Messages printed to terminal or buffered

**State Mutations by Command:**
- `Look` - No mutation, reads current room
- `Where` - No mutation, reads player position
- `Help` - No mutation, reads player class for available commands
- `Quit` - Sets `is_running = false`
- `Go` - **Mutates:** Changes `player.pos`
- `Teleport` - **Mutates:** Changes `player.pos` (with validation)
- `Shift` - **Future mutation:** Changes `player.pos` across layers

**Multiplayer Implications:**
- ✅ Commands are well-structured as enums
- ⚠️ Need to add `player_id` context to know which player is acting
- ⚠️ Position changes need to broadcast to other players in same room
- ⚠️ Need new commands: `say`, `tell`, `examine`, `trade`, etc.
- ⚠️ Need to validate that player is in correct state for command
- ⚠️ Need concurrency control for state mutations

---

## 5. Command Parsing

### Current Implementation
```rust
pub fn parse_command(line: &str) -> Option<Command>
```

**Characteristics:**
- Simple string tokenization
- Case-insensitive matching
- Supports aliases (n/s/e/w for go north/south/east/west)
- Returns `Option<Command>` - None for empty input, Some(Help) for unknown

**Multiplayer Changes Needed:**
- ✅ Parsing logic can remain largely unchanged
- ⚠️ Need to add player/session context for routing
- ⚠️ Need to add new social commands (chat, trade, etc.)
- ⚠️ Consider command rate limiting per player
- ⚠️ Add command validation against player state

---

## 6. Output System

### MessageBuffer Structure
```rust
pub struct MessageBuffer {
    messages: Vec<String>,
    pub mode: OutputMode,
}

pub enum OutputMode {
    BufferOnly,         // Accumulate only (GUI/testing)
    TerminalAndBuffer,  // Print + accumulate (terminal)
    TerminalOnly,       // Print only (no buffer)
}
```

**Current Flow:**
- Commands write output to `MessageBuffer`
- Depending on mode, messages are printed immediately and/or buffered
- Terminal mode: immediate println!()
- Buffer mode: accumulated for later retrieval

**Multiplayer Requirements:**
- ✅ Buffer system is good foundation
- ⚠️ Need per-session output buffering
- ⚠️ Need message routing (who sees what)
- ⚠️ Need message formatting per interface type (Terminal/Signal/SMS)
- ⚠️ Need event broadcasting (room-based, proximity-based, global)
- ⚠️ Need message types/categories (narrative, system, chat, combat, error)

**Proposed Changes:**
```rust
pub enum MessageType {
    Narrative,  // Room descriptions, actions
    System,     // Server messages, notifications
    Chat,       // Player communication
    Combat,     // Combat events
    Error,      // Error messages
}

pub struct Message {
    pub msg_type: MessageType,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

pub enum MessageTarget {
    Player(PlayerId),           // Single player
    Room(Coord),                // All in room
    Proximity(Coord, u32),      // Within distance
    Global,                     // Everyone
}
```

---

## 7. Side Effects and State Changes

### Current Side Effects
All side effects occur synchronously within `execute_command()`:

1. **Position Changes** (`Go`, `Teleport`, `Shift`)
   - Direct mutation: `state.player.pos = new_coord`
   - Immediate
   - No validation of conflicts

2. **Game Loop Control** (`Quit`)
   - Sets `state.is_running = false`
   - Terminates main loop

3. **Output Generation** (All commands)
   - Writes to `MessageBuffer`
   - Synchronous, immediate

**Multiplayer Considerations:**
- ⚠️ Need atomic position updates
- ⚠️ Need to check if destination is valid/available
- ⚠️ Need to broadcast position changes to relevant players
- ⚠️ Need transaction-like behavior for complex actions
- ⚠️ Need event log for persistence/replay

---

## 8. Game Loop (main.rs)

### Current Loop Structure
```rust
while state.is_running {
    // 1. Check heartbeat (world tick events)
    heartbeat.check_and_beat(&mut state, &mut output);
    
    // 2. Wait for user input
    let line = prompt_line()?;
    
    // 3. Parse command
    if let Some(cmd) = parse_command(&line) {
        // 4. Clear output buffer
        output.clear();
        
        // 5. Execute command
        execute_command(&mut state, cmd, &exit_alias_map, &mut output);
    }
}
```

**Characteristics:**
- Blocking I/O on stdin
- Single-threaded
- Synchronous execution
- Heartbeat for periodic world updates

**Multiplayer Server Requirements:**
- ❌ Cannot block on single player input
- ✅ Heartbeat system is good foundation for world ticks
- ⚠️ Need async I/O for multiple connections
- ⚠️ Need command queue per player
- ⚠️ Need worker pool or async task spawning
- ⚠️ Need to handle player connections/disconnections
- ⚠️ Need session timeout handling

---

## 9. Room and Exit System

### Room Definition
```rust
pub struct Room {
    pub id: String,                       // Unique room identifier
    pub pos: Coord,                       // Position in world
    pub name: String,                     // Display name
    pub desc: String,                     // Description text
    pub exits: HashMap<String, ExitSpec>, // Available exits
    pub tags: Vec<String>,                // Room tags/flags
}

pub struct ExitSpec {
    pub to: ExitTo,           // Destination
    pub desc: Option<String>, // Exit description
}

pub enum ExitTo {
    Relative { dx: i32, dy: i32, dz: i32 }, // Relative offset
    Absolute(Coord),                         // Absolute coordinate
}
```

**Multiplayer Implications:**
- ✅ Room structure is solid for read-only world
- ⚠️ Need to track which players are in each room
- ⚠️ Need to handle capacity limits (if any)
- ⚠️ Need to broadcast enter/leave events
- ⚠️ Future: Door states, locked exits, etc.

---

## 10. Concurrency Considerations

### Current State Access Patterns
- **Single mutable reference:** `&mut GameState`
- **No locking:** Not needed for single-threaded
- **No transactions:** Immediate state updates
- **No versioning:** No conflict resolution needed

### Multiplayer State Access Needs
- **Shared world state:** Multiple readers, occasional writers
- **Per-player state:** Independent mutations
- **Room occupancy:** Frequent updates, needs consistency
- **Command execution:** Atomic operations preferred

**Proposed Approach:**
```rust
// Shared world (read-mostly)
Arc<World>

// Mutable game state (write-protected)
Arc<RwLock<GameState>>

// Per-player state (independent locks)
Arc<Mutex<Player>>

// Room occupancy tracking
Arc<RwLock<HashMap<Coord, HashSet<PlayerId>>>>
```

---

## Summary of Required Changes

### Critical Changes (Must Do)
1. ✅ Convert `GameState.player` from single `Player` to `HashMap<PlayerId, Player>`
2. ✅ Add player/session identification to all commands
3. ✅ Implement async I/O and connection handling
4. ✅ Add message routing and broadcasting system
5. ✅ Separate static world data from dynamic game state
6. ✅ Add concurrency control (locks, atomics)
7. ✅ Implement per-session output buffers

### Important Changes (Should Do)
8. Add player visibility/awareness system
9. Add event system for state change notifications
10. Expand command set for multiplayer interactions
11. Add authentication and session management
12. Implement persistence for player state

### Future Enhancements (Nice to Have)
13. Add inventory system
14. Add stats and skills
15. Add NPC system
16. Add combat system
17. Add quest tracking
18. Add trading between players

---

## Next Steps for Task 1

With current architecture analysis complete, proceed to:

- ✅ **1.1 Analyze Current Architecture** - COMPLETE
- ⏭️ **1.2 Define Multiplayer State Model** - Next
- ⏭️ **1.3 Design Command/Message Protocol**
- ⏭️ **1.4 Architecture Pattern Selection**
- ⏭️ **1.5 Session Abstraction Layer**
- ⏭️ **1.6 State Synchronization Strategy**
- ⏭️ **1.7 Authentication and Identity**
- ⏭️ **1.8 Error Handling and Resilience**
- ⏭️ **1.9 Logging and Observability**
- ⏭️ **1.10 Create Architecture Documentation**
