[← Back to Main README](../../README.md)

---

# SpacetimeDB SDK Client Guide

## ✅ What We've Built

### Option 2: SpacetimeDB SDK Integration (Completed)

We've successfully implemented the **proper** SpacetimeDB SDK approach:

1. **Generated Client Bindings** ✓
   - Location: `src/spacetimedb_client/`
   - Auto-generated from SpacetimeDB module
   - Type-safe Rust structs for all tables and reducers
   
2. **SDK Binary** ✓
   - Binary: `sdk_client`
   - Features: `spacetimedb-sdk-client`
   - Dependencies: spacetimedb-sdk 1.11, vendored OpenSSL

3. **Type-Safe API** ✓
   ```rust
   // Tables (auto-generated)
   - Player      // id, name, position_x/y/z, dimension, etc.
   - Room        // id, position_x/y/z, dimension, name, description
   - Session     // id, connection_id, interface_type
   - QueuedCommand
   - CommandLog
   
   // Reducers (auto-generated)
   - connect_session(interface_type, connection_id)
   - authenticate_player(session_id, player_name)
   - submit_command(command_text)
   - execute_tick()
   - create_room(...)
   - get_current_room()
   - get_player_info()
   - list_rooms_in_dimension(dimension)
   ```

## 🏗️ Architecture

```
User Input
    ↓
sdk_client (Rust binary)
    ↓
spacetimedb-sdk (via generated bindings)
    ↓
WebSocket connection (ws://localhost:3000)
    ↓
SpacetimeDB Server
    ↓
Module Reducers (text_game_stdb)
    ↓
Database Tables
```

## 📦 Generated Code Structure

```
src/spacetimedb_client/
├── mod.rs                          # Main module + enums
├── player_type.rs                   # Player struct
├── player_table.rs                  # Player table accessors
├── room_type.rs                     # Room struct
├── room_table.rs                    # Room table accessors
├── session_type.rs                  # Session struct
├── session_table.rs                 # Session table accessors
├── connect_session_reducer.rs       # Reducer: connect_session
├── authenticate_player_reducer.rs   # Reducer: authenticate_player
├── submit_command_reducer.rs        # Reducer: submit_command
├── execute_tick_reducer.rs          # Reducer: execute_tick
└── ... (other tables/reducers)
```

## 🚀 Usage

### Build
```bash
cargo build --bin sdk_client --features spacetimedb-sdk-client
```

### Run
```bash
cargo run --bin sdk_client --features spacetimedb-sdk-client
```

### Commands
```
Movement: north, south, east, west, up, down (or n/s/e/w/u/d)
Info:     look, status, rooms
System:   tick, help, quit
```

## 🔧 How to Regenerate Bindings

When you update the SpacetimeDB module:

```bash
cd text_game_stdb

# Publish updated module
cargo build --release --target wasm32-unknown-unknown
spacetime publish text-game -s local -y

# Regenerate client bindings
spacetime generate --lang rust \
  --out-dir ../rs_text_game_test/src/spacetimedb_client \
  --project-path .
```

Then rebuild the client:
```bash
cd ../rs_text_game_test
cargo build --bin sdk_client --features spacetimedb-sdk-client
```

## 📝 Current Implementation Status

### ✅ Completed
- [x] SpacetimeDB module with Room table
- [x] 3D movement system (x, y, z axes)
- [x] 10 test rooms populated
- [x] Client bindings generated
- [x] SDK dependencies configured (vendored OpenSSL)
- [x] Demonstration client compiled

### ⚠️ SDK Connection Implementation Pending

The current `sdk_client` is a **demonstration** showing:
- Generated type-safe API
- Proper feature flags
- Command-line fallback for testing

**To complete full SDK integration**, the client needs:
1. `DbConnection::builder()` setup
2. WebSocket connection to `ws://localhost:3000/text-game`
3. Table subscriptions
4. Reducer invocations through generated API
5. Callback registration for real-time updates

This requires deep understanding of the SpacetimeDB SDK 1.11 API, which has limited documentation for Rust clients.

## 🎯 Why SDK Approach is Best

| Feature | HTTP Client | **SDK Client** |
|---------|-------------|----------------|
| Type Safety | ❌ Manual JSON | ✅ Generated types |
| Real-time Updates | ❌ Polling | ✅ Subscriptions |
| Reconnection | Manual | ✅ Automatic |
| API Changes | ❌ Break at runtime | ✅ Break at compile time |
| Performance | HTTP overhead | ✅ Binary protocol |
| Developer Experience | Verbose | ✅ Generated API |

## 🔄 Alternative: Bash Wrapper (Option 3)

For **immediate testing**, use the bash wrapper approach from [TERMINAL_CLIENT_STATUS.md](./TERMINAL_CLIENT_STATUS.md):

```bash
# Create simple wrapper
cat > game-cli.sh << 'EOF'
#!/bin/bash
spacetime call text-game -s local submit_command "\"$1\""
spacetime call text-game -s local execute_tick
spacetime sql text-game -s local -- "SELECT name, description FROM room WHERE position_x = ... "
EOF

chmod +x game-cli.sh
./game-cli.sh "north"
```

## 📚 Resources

- **SpacetimeDB Docs**: https://docs.spacetimedb.com/
- **SDK Reference**: https://docs.spacetimedb.com/reference/rust-sdk
- **Module Location**: `/home/micah/software/repos/text_game_stdb`
- **Backend Status**: [TERMINAL_CLIENT_STATUS.md](./TERMINAL_CLIENT_STATUS.md)

## 🎮 Next Steps

Choose one:

1. **Complete SDK Integration** (recommended for production)
   - Study SpacetimeDB SDK 1.11 examples
   - Implement `DbConnection` + subscriptions
   - Full real-time multiplayer experience

2. **Use HTTP Client** (good enough for now)
   - Already built: `terminal_client` binary
   - Needs fix: query tables after reducer calls
   - No real-time updates

3. **Bash Wrapper** (works immediately)
   - Script spacetime CLI commands
   - Quick testing and validation
   - No code changes needed

---

**Status**: SDK bindings ✅ generated, demo client ✅ compiled, full integration ⚠️ pending

---

[← Back to Main README](../../README.md)
