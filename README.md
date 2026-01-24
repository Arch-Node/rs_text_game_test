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
  - ⚠️ Terminal HTTP client (needs fix)
  - ⚠️ Terminal SDK client (demo complete, needs full integration)
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

---

## 📁 Project Structure

## 📁 Project Structure

```
rs_text_game_test/          # Main game project
├── Cargo.toml              # Project configuration
├── src/
│   ├── lib.rs             # Library exports
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
- [SpacetimeDB Integration](docs/spacetimedb-integration.md) - Backend architecture choices
- [Architecture Analysis](docs/architecture-analysis.md) - Design and implementation decisions
- [State Design](docs/multiplayer-state-design.md) - Tick-based command queue design
- [TODO Roadmap](TODO.md) - Complete development roadmap

---

## 🛠️ Development

### Running Tests

```bash
# Backend validation
./tests/test_backend.sh

# Signal bot validation
./tests/test_signal_bot.sh

# Rust unit tests
cargo test
```

### Contributing

See the individual module READMEs for implementation details:
- [SpacetimeDB Client](src/spacetimedb_client/SDK_CLIENT_GUIDE.md)
- [Terminal Client](src/terminal_client/TERMINAL_CLIENT_STATUS.md)
- [Signal Bot](src/signal_client/SIGNAL_STATUS.md)

---

## 📜 License

This is a learning project - feel free to use and modify as needed.
