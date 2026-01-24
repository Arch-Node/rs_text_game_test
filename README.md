# Rust Text Adventure Game Engine

**Multiplayer Skills-Heavy Text Adventure** accessible via Terminal, Signal, and SMS.

Built with Rust and SpacetimeDB, featuring a tick-based command queue system and multi-dimensional world.

> **🚧 Development Status:** 
> - ✅ SpacetimeDB backend complete with Room table and 3D movement
> - ✅ SDK client bindings generated (type-safe API)
> - ✅ **Signal Messenger bot** (Phase 1 complete - see [SIGNAL_STATUS.md](src/signal_client/SIGNAL_STATUS.md))
> - ⚠️ Terminal/SMS interfaces need completion
> - See [TODO.md](TODO.md) for full roadmap, [SDK_CLIENT_GUIDE.md](src/spacetimedb_client/SDK_CLIENT_GUIDE.md) for SDK setup

---

## 🎮 Key Features

- **Skills-Heavy Gameplay:** Crafting, persuasion, stealth, exploration-focused
- **Light Combat:** Avoidable, skill-based when it occurs
- **Multi-Dimensional World:** Material, ethereal, shadow, and dream planes
- **Multi-Interface Support:** 
  - ✅ Terminal (local single-player)
  - ⚠️ Terminal HTTP client (needs fix)
  - ✅ **Signal Messenger** (webhook-based, Phase 1 complete)
  - 🔲 SMS (planned)
- **Tick-Based System:** 15-second strategic turns (3s for testing)
- **Real-Time Sync:** SpacetimeDB provides ACID transactions and instant updates
- **3D Movement:** Full x/y/z navigation within each dimension

---

## 🗄️ Backend: SpacetimeDB

**Why SpacetimeDB:**
- Database + server combined into one
- Built-in real-time state synchronization (~100μs latency)
- ACID transactions prevent race conditions
- Purpose-built for multiplayer games (100k tx/s capacity)
- Automatic persistence and scaling

**Module Status:** ✅ Published and running on `localhost:3000`

**Database Schema:**
- `player` - Characters with 4D position (x, y, z, dimension)
- `room` - 10 test rooms with 3D coordinates (100,100,100 center)
- `session` - Active connections (Terminal/Signal/SMS)
- `queued_command` - Commands awaiting tick execution
- `command_log` - Complete audit trail

**Dimensions:** Separate 3D spatial planes (material, ethereal, shadow, dream) that can shift and interact but maintain independent coordinate systems.

> **📝 Terminology Note:** The SpacetimeDB backend and documentation use the term "dimension" to refer to separate spatial planes (material, ethereal, etc.). The legacy single-player code in `src/models.rs` uses the term "layer" for the same concept. Both terms refer to the same architectural feature.

---

## 📁 Project Structure

## 📁 Project Structure

```
rs_text_game_test/          # Main game project
├── Cargo.toml              # Project configuration
├── rooms.json              # World data (legacy - now in SpacetimeDB)
├── src/
│   ├── main.rs            # Single-player terminal game (original)
│   ├── bin/
│   │   ├── terminal_client.rs  # HTTP-based client (needs fix)
│   │   ├── sdk_client.rs       # SDK-based client (demo)
│   │   └── signal_bot.rs       # Signal Messenger bot (✅ Phase 1 complete)
│   ├── terminal_client/   # HTTP client modules
│   │   ├── client.rs      # TerminalClient (needs query pattern fix)
│   │   ├── auth.rs        # Authentication flow
│   │   ├── display.rs     # Display formatting
│   │   ├── input.rs       # Input handling
│   │   └── TERMINAL_CLIENT_STATUS.md  # Testing status and known issues
│   ├── spacetimedb_client/ # Generated SDK bindings (19 files)
│   │   ├── mod.rs         # Main module
│   │   ├── player_type.rs, room_type.rs, session_type.rs
│   │   ├── *_reducer.rs   # Type-safe reducer APIs
│   │   └── SDK_CLIENT_GUIDE.md  # SDK setup and usage guide
│   ├── signal_client/     # Signal bot modules (✅ complete)
│   │   ├── bot.rs         # SignalBot core implementation
│   │   ├── webhook.rs     # HTTP webhook server
│   │   ├── message_handler.rs  # Message processing
│   │   ├── formatter.rs   # Signal message formatting
│   │   ├── mod.rs         # Module exports
│   │   ├── SIGNAL_STATUS.md  # Integration status
│   │   ├── SIGNAL_BOT_SETUP.md  # Setup guide
│   │   ├── SIGNAL_IMPLEMENTATION_SUMMARY.md
│   │   ├── SIGNAL_INTEGRATION.md
│   │   └── CONFIGURATION_GUIDE.md
│   ├── models.rs          # Core data structures
│   ├── commands.rs        # Command parsing
│   ├── world.rs           # World loading
│   ├── output.rs          # Message output
│   └── multiplayer/       # Multiplayer architecture (legacy/design reference)
│       ├── types.rs, player.rs, session.rs, state.rs
│       ├── command_queue.rs, validation.rs, events.rs
│       └── tick.rs
├── docs/                  # Architecture documentation
│   ├── architecture-analysis.md
│   ├── multiplayer-state-design.md
│   ├── spacetimedb-integration.md
│   └── spacetimedb-quickstart.md
├── tests/                 # Test scripts
│   ├── test_backend.sh    # Backend validation script
│   └── test_signal_bot.sh # Signal bot validation script
├── bot_config.toml.example  # Bot configuration example
└── TODO.md                # Complete development roadmap

text_game_stdb/            # SpacetimeDB module (separate repo/dir)
├── src/lib.rs             # Tables and reducers (WASM module)
├── Cargo.toml
└── target/wasm32-unknown-unknown/release/
    └── text_game_stdb.wasm # Compiled module
```

---

## 🚀 Getting Started

### Prerequisites

- **Rust 1.76+** (SpacetimeDB requires resolver = "3")
- **SpacetimeDB CLI**

### Install SpacetimeDB

```bash
curl https://install.spacetimedb.com | bash
export PATH="$HOME/.local/bin:$PATH"
```

### Start SpacetimeDB Server

```bash
spacetime start  # Runs on localhost:3000
```

### Build and Play

**Single-player terminal game (original):**
```bash
cd rs_text_game_test
cargo run
```

**SpacetimeDB module:**
```bash
cd text_game_stdb

# Build WASM module
cargo build --release --target wasm32-unknown-unknown

# Publish to local server
spacetime publish text-game -s local -y
```

**SDK Terminal Client (Option 2 - Recommended):**
```bash
cd rs_text_game_test

# Generate client bindings (when module changes)
cd ../text_game_stdb
spacetime generate --lang rust --out-dir ../rs_text_game_test/src/spacetimedb_client --project-path .
cd ../rs_text_game_test

# Build and run SDK client
cargo run --bin sdk_client --features spacetimedb-sdk-client
```

**HTTP Terminal Client (Option 1 - Needs API fix):**
```bash
cargo run --bin terminal_client --features spacetimedb-http
```

**Signal Messenger Bot (✅ Phase 1 Complete):**
```bash
# Set environment variables
export BOT_NUMBER="+15551234567"          # Your Signal number
export SIGNAL_API_URL="http://localhost:8080"
export SPACETIME_URL="http://localhost:3000"

# Run the bot
cargo run --features signal-client --bin signal_bot
```

See [SDK_CLIENT_GUIDE.md](src/spacetimedb_client/SDK_CLIENT_GUIDE.md) for SDK details and [SIGNAL_BOT_SETUP.md](src/signal_client/SIGNAL_BOT_SETUP.md) for Signal bot setup.

### Test Multiplayer Backend

```bash
# Create session
spacetime call text-game -s local connect_session '"terminal"' '"test-session-1"'

# Authenticate player (creates new character if needed)
spacetime call text-game -s local authenticate_player 123 '"Hero"'

# Submit queued command (waits for next tick)
spacetime call text-game -s local submit_command '"go north"'

# Submit instant command (executes immediately)
spacetime call text-game -s local submit_command '"look"'

# Execute tick (processes all queued commands)
spacetime call text-game -s local execute_tick

# View results
spacetime sql text-game -s local "SELECT * FROM player"
spacetime sql text-game -s local "SELECT * FROM command_log"
```

---

## 🎯 Architecture Highlights

**Tick System:** Commands are validated → queued → executed in batches every 15 seconds (3s for testing)

**Command Throttling:** Players can submit ONE queued command at a time (instant commands don't count)

**Instant Commands:** Observation/info commands execute immediately without queueing:
- `look`, `examine`, `inventory`, `status`, `help`
- `say`, `tell`, `emote` (communication)
- `who`, `score`, `time` (info queries)

**Dimensions:** Separate 3D spatial planes that can shift/interact but maintain independent coordinate systems:
- **Material** - The physical world
- **Ethereal** - Spirit plane, ghost-like
- **Shadow** - Stealth and darkness
- **Dream** - Visions and prophecy

**ACID Guarantees:** All state changes are transactional (no race conditions when two players take the same item)

**Multi-Interface:** Same backend supports Terminal (rich text), Signal (notifications), and SMS (concise)

---

## 📊 Development Status

**Phase 1: Core Infrastructure** (In Progress)
- ✅ SpacetimeDB module created and published
- ✅ Tick-based command queue implemented
- ✅ One-command-per-player throttling
- ✅ Instant vs queued command distinction
- ✅ Multi-dimensional coordinate system (x, y, z, dimension)
- ✅ Room table with 3D coordinates (10 test rooms)
- ✅ Full 6-directional movement (n/s/e/w/up/down)
- ✅ SDK client bindings generated (type-safe API)
- ✅ **Signal messenger integration** (Phase 1 complete, webhook-based)
- ⚠️ Terminal interface integration (SDK demo complete, full connection pending)
- ⬜ SMS gateway integration

See [TODO.md](TODO.md) for complete roadmap, [SDK_CLIENT_GUIDE.md](src/spacetimedb_client/SDK_CLIENT_GUIDE.md) for SDK details.

---

## 📚 Documentation

- **[SDK Client Guide](src/spacetimedb_client/SDK_CLIENT_GUIDE.md)** - SpacetimeDB SDK setup and usage
- **[Signal Status](src/signal_client/SIGNAL_STATUS.md)** - Signal Messenger integration status (✅ Phase 1 complete)
- **[Signal Bot Setup](src/signal_client/SIGNAL_BOT_SETUP.md)** - Signal bot setup and configuration guide
- **[Terminal Client Status](src/terminal_client/TERMINAL_CLIENT_STATUS.md)** - Testing status and known issues
- **[Configuration Guide](src/signal_client/CONFIGURATION_GUIDE.md)** - Bot configuration methods
- [Multiplayer Architecture](src/multiplayer/README.md) - Detailed system design (legacy/reference)
- [SpacetimeDB Integration](docs/spacetimedb-integration.md) - Backend architecture choices
- [Architecture Analysis](docs/architecture-analysis.md) - Single-player → multiplayer transition
- [State Design](docs/multiplayer-state-design.md) - Tick-based command queue design
- [TODO Roadmap](TODO.md) - Complete development roadmap

---

## 🛠️ Module Overview (Original Single-Player Code)

### `main.rs` - Entry Point
- Initializes the game engine
- Sets up the terminal game loop (single-player)
- Handles user input prompts
- Minimal glue code to tie modules together
- **Note:** Will be refactored to client mode for multiplayer

### `models.rs` - Core Data Structures
**Runtime game state:**
- `Coord` - 3D coordinates (w=layer, x,y=position)
- `Player` - Player state and position
- `PlayerClass` - Teleporter or LayerWalker abilities
- `GameState` - Complete game state container
- `World` - Validated world data with lookup maps
- `Room` - Individual room with exits and description
- `ExitSpec` - Exit destination specification
- `Anchor` - Teleportation anchor points
- `Layer` - Layer metadata (scale, name)
- `Meta` - World metadata

### `file_format.rs` - JSON Deserialization
**Serde structs for loading `rooms.json`:**
- `RoomsFile` - Root JSON structure
- `FileRoom`, `FileLayer`, `FileAnchor` - JSON representations
- `FileExitSpec` - Supports relative (`to_rel`) or absolute (`to_abs`) exits
- Keeps file format separate from runtime models

### `exits.rs` - Exit System
**Exit definitions and handling:**
- `ExitKind` - Directional, Teleport, LayerShift, Special
- `ExitDef` - Exit definitions with aliases
- `EXIT_DEFS` - Configurable exit types (north, south, window, mousehole, etc.)
- `build_exit_alias_map()` - Creates command alias lookup
- `resolve_exit_target()` - Converts exit specs to coordinates

### `world.rs` - World Loading & Validation
**World initialization:**
- `load_world_from_rooms_json()` - Loads and parses JSON
- `validate_world()` - Ensures world consistency:
  - Start position exists
  - All exits point to valid rooms
  - Anchors reference existing rooms
  - Layer scales are valid
- Converts file format structs to runtime models

### `commands.rs` - Command System
**Command parsing and execution:**
- `Command` enum - All available commands
- `parse_command()` - Parses user input to commands
- `execute_command()` - Routes commands to handlers
- Action handlers:
  - `do_look()` - Display current room
  - `do_go()` - Move through exits
  - `do_teleport()` - Teleport to anchors (Teleporter class)
  - `do_shift()` - Shift between layers (LayerWalker class, TODO)
  - `do_where()` - Show current position
  - `do_help()` - Display available commands

### `output.rs` - Flexible Output System
**Message accumulation for portability:**
- `MessageBuffer` - Accumulates game output
- `OutputMode`:
  - `BufferOnly` - Store messages (GUI/web/testing)
  - `TerminalAndBuffer` - Print AND store (terminal play)
  - `TerminalOnly` - Print without buffering
- Enables non-terminal interfaces (GUI, web, automated testing)

### `examples.rs` - Usage Examples
Demonstrates how to use the engine in different contexts:
- Silent execution for testing
- GUI integration patterns
- Web API response generation

## Building and Running

### Prerequisites
```bash
# Install Rust and Cargo
sudo apt install cargo
# or use rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build
```bash
cd rust_game_test
cargo build          # Debug build
cargo build --release # Optimized release build
```

### Run
```bash
cargo run
```

### Test
```bash
cargo test
```

## Game Features

### Layered World System
- Multiple z-layers (w coordinate)
- Different scales per layer (macrocell concept)
- Layer-specific room networks

### Player Classes
- **Teleporter**: Can teleport to anchors within same layer
- **LayerWalker**: Can shift between layers (planned feature)

### Exits
- Explicit exit system (only listed exits are valid)
- Relative exits (dx, dy within same layer)
- Absolute exits (full w, x, y coordinates)
- Configurable aliases (n/north, e/east, etc.)
- Special exits (window, mousehole, custom)

### Commands
```
look | l              View current room
where                 Show coordinates
go <exit>             Move through an exit
n, s, e, w           Directional shortcuts
teleport <anchor>     Teleport to anchor (Teleporter only)
shift up|down         Change layers (LayerWalker only, TODO)
help | ?              Show commands
quit                  Exit game
```

## Extending the Game

### Adding New Rooms
Edit `rooms.json`:
```json
{
  "id": "new_room",
  "pos": { "w": 0, "x": 2, "y": 0 },
  "name": "New Room",
  "desc": "A freshly created room.",
  "exits": {
    "west": { "to_rel": { "dx": -1, "dy": 0 } }
  }
}
```

### Adding New Exit Types
Edit `EXIT_DEFS` in `src/exits.rs`:
```rust
ExitDef { 
    id: "portal", 
    kind: ExitKind::Special, 
    aliases: &["portal", "enter portal"] 
}
```

### Adding New Commands
1. Add variant to `Command` enum in `commands.rs`
2. Handle in `parse_command()`
3. Implement handler function (e.g., `do_my_command()`)
4. Route in `execute_command()`

### Creating a GUI Frontend
```rust
use output::{MessageBuffer, OutputMode};

let mut output = MessageBuffer::new(OutputMode::BufferOnly);
execute_command(&mut state, cmd, &exit_alias_map, &mut output);

// Display in GUI
for message in output.get_messages() {
    gui_text_area.append(message);
}
```

## Design Principles

1. **Separation of Concerns**: Each module has a single, clear responsibility
2. **Format Independence**: File format separate from runtime models
3. **Output Flexibility**: Works in terminal, GUI, web, or headless contexts
4. **Type Safety**: Rust's type system prevents invalid game states
5. **Explicit Over Implicit**: Rooms must explicitly define available exits
6. **Validation First**: World is validated at load time, not runtime

## Future Enhancements

- [ ] Implement layer shifting with macrocell snapping
- [ ] Add inventory system
- [ ] Add NPC and conversation system
- [ ] Add items and object interaction
- [ ] Add save/load game state
- [ ] Create GUI frontend (egui, iced, or web)
- [ ] Add scripting for room behaviors
- [ ] Multiplayer support

## Learning Rust Notes

This project demonstrates several Rust concepts:
- Module organization and visibility
- Ownership and borrowing (passing `&mut GameState`)
- Error handling with `Result<T, E>`
- Enums and pattern matching
- Trait implementations
- JSON deserialization with Serde
- Hash maps for fast lookups
- String handling and formatting

## License

This is a learning project - feel free to use and modify as needed.
