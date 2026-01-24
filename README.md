# Rust Text Adventure Game Engine

**Multiplayer Skills-Heavy Text Adventure** accessible via Terminal, Signal, and Discord.

Built with Rust and SpacetimeDB, featuring a tick-based command queue system and multi-dimensional world.

> **🚧 Development Status:** 
> - ✅ SpacetimeDB backend complete with Room table and 3D movement
> - ✅ SDK client bindings generated (type-safe API)
> - ✅ **Signal Messenger bot** (Phase 1 complete - see [Signal README](src/signal_client/README.md))
> - ✅ **Discord bot** (Implementation complete - see [DISCORD_STATUS.md](src/discord_client/DISCORD_STATUS.md))
> - ⚠️ Terminal client needs completion
> - See [TODO.md](TODO.md) for full roadmap, [SDK_CLIENT_GUIDE.md](src/spacetimedb_client/SDK_CLIENT_GUIDE.md) for SDK setup

---

## 🚀 Quick Start

### ⚠️ Note on HTTP Client

The HTTP-based terminal client **does not work with SpacetimeDB's local development server** because the local server doesn't provide HTTP REST API endpoints - it only supports WebSocket (SDK) and CLI access.

**Working interfaces:**
- ✅ **SDK Client** (WebSocket) - [SDK_CLIENT_GUIDE.md](src/spacetimedb_client/SDK_CLIENT_GUIDE.md)
- ✅ **SpacetimeDB CLI** (`spacetime call`/`spacetime sql`)
- ❌ **HTTP Terminal Client** - Needs SpacetimeDB cloud deployment

See [SPACETIMEDB_SETUP.md](SPACETIMEDB_SETUP.md) for full explanation.

### Test the Backend with SDK Client

The SDK client works with the local server:

```bash
# 1. Start SpacetimeDB (in a separate terminal)
spacetime start

# 2. Publish the module
cd text_game_stdb
spacetime publish text-game --server local

# 3. Run the SDK client (WebSocket-based)
cd ../rs_text_game_test
cargo run --features spacetimedb-sdk-client --bin sdk_client
```

### Test with CLI Commands

```bash
# Connect session
spacetime call text-game connect_session '"terminal"' '"test-123"' --server local

# Authenticate player  
spacetime call text-game authenticate_player '1' '"TestPlayer"' --server local

# Submit command
spacetime call text-game submit_command '1' '"look"' --server local

# Execute tick
spacetime call text-game execute_tick --server local

# Query data
spacetime sql text-game "SELECT * FROM player" --server local
```

### End-to-End Test (Shows HTTP Limitations)

```bash
# This publishes the module successfully
# but shows expected 404 errors for HTTP client tests
./end_to_end_test.sh
```

The test will show:
- ✅ Module publishing works
- ✅ CLI commands work  
- ❌ HTTP client tests fail (expected - no HTTP API on local server)

---

## 🎮 Key Features

- **Skills-Heavy Gameplay:** Crafting, persuasion, stealth, exploration-focused
- **Light Combat:** Avoidable, skill-based when it occurs
- **Multi-Dimensional World:** Material, ethereal, shadow, and dream planes
- **Multi-Interface Support:** 
  - ⚠️ Terminal HTTP client (needs fix)
  - ⚠️ Terminal SDK client (demo complete, needs full integration)
  - ✅ **Signal Messenger** (webhook-based, Phase 1 complete)
  - ✅ **Discord bot** (gateway-based, implementation complete)
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
- `session` - Active connections (Terminal/Signal/Discord)
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
│   │   ├── signal_bot.rs       # Signal Messenger bot (✅ Phase 1 complete)
│   │   └── discord_bot.rs      # Discord bot (✅ implementation complete)
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
│   │   ├── README.md      # Overview and status
│   │   └── SIGNAL_BOT_SETUP.md  # Setup guide
│   ├── discord_client/    # Discord bot modules (✅ complete)
│   │   ├── bot.rs         # DiscordBot core implementation
│   │   ├── message_handler.rs  # Message processing
│   │   ├── formatter.rs   # Discord message formatting
│   │   ├── mod.rs         # Module exports
│   │   ├── DISCORD_STATUS.md  # Implementation status
│   │   └── DISCORD_BOT_SETUP.md  # Setup guide
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

**Usage:** Authenticate with `auth YourName` in any chat, then the bot will DM you. All gameplay happens in DMs to keep group chats clean.

**Discord Bot (✅ Implementation Complete):**
```bash
# Set environment variables
export DISCORD_TOKEN="your-bot-token-here"  # From Discord Developer Portal
export SPACETIME_URL="http://localhost:3000"

# Run the bot
cargo run --features discord-client --bin discord_bot
```

**Usage:** Authenticate with `@BotName auth YourName` in any channel, then the bot will DM you. All gameplay happens in DMs to keep channels clean.

See [SDK_CLIENT_GUIDE.md](src/spacetimedb_client/SDK_CLIENT_GUIDE.md) for SDK details, [SIGNAL_BOT_SETUP.md](src/signal_client/SIGNAL_BOT_SETUP.md) for Signal bot setup, and [DISCORD_BOT_SETUP.md](src/discord_client/DISCORD_BOT_SETUP.md) for Discord bot setup.

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

**Multi-Interface:** Same backend supports Terminal (rich text), Signal (notifications), and Discord (bot commands)

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
- ⬜ Discord bot integration

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

This project is licensed under GPL v3 - feel free to use and modify as needed for learning purposes.

### Third-Party Licenses

**SpacetimeDB**: This project uses [SpacetimeDB](https://github.com/clockworklabs/SpacetimeDB) as its backend database. SpacetimeDB is licensed under the BSL 1.1 (Business Source License 1.1), which converts to AGPL v3.0 with a linking exception after a certain period. The linking exception means you can use SpacetimeDB without having to open source your own code. See the [SpacetimeDB LICENSE](https://github.com/clockworklabs/SpacetimeDB/blob/master/LICENSE.txt) for details.
