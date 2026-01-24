# SpacetimeDB Setup Guide

## ⚠️ Current Status: HTTP API Not Available Locally

**The HTTP-based terminal client cannot work with SpacetimeDB's local development server.**

SpacetimeDB's `spacetime start` command provides a local development server that **only supports**:
- ✅ WebSocket connections (via SDK)
- ✅ CLI commands (`spacetime call`, `spacetime sql`)

The local server **does not provide HTTP REST API endpoints** for reducers and SQL queries.

### Working Clients

1. **SpacetimeDB SDK Client** (WebSocket-based) ✅
   ```bash
   cargo run --features spacetimedb-sdk-client --bin sdk_client
   ```
   See [SDK_CLIENT_GUIDE.md](src/spacetimedb_client/SDK_CLIENT_GUIDE.md)

2. **SpacetimeDB CLI** ✅
   ```bash
   spacetime call text-game connect_session '"terminal"' '"conn-123"' --server local
   spacetime sql text-game "SELECT * FROM player" --server local
   ```

3. **Discord Bot** (if pointing to cloud) ✅
4. **Signal Bot** (if pointing to cloud) ✅

### Why the HTTP Client Fails

The terminal client attempts to use HTTP REST endpoints like:
- `/v1/database/{identity}/call` - **404 Not Found** on local server
- `/v1/database/{identity}/sql` - **404 Not Found** on local server

These endpoints are only available on SpacetimeDB cloud servers, not in local development mode.

### Solution Options

**Option 1: Use the SDK Client** (Recommended)
The SDK client uses WebSockets and works with the local server:
```bash
cargo run --features spacetimedb-sdk-client --bin sdk_client
```

**Option 2: Deploy to SpacetimeDB Cloud**
The HTTP API is available on cloud servers. You'd need to:
1. Create a SpacetimeDB cloud account
2. Publish to cloud: `spacetime publish text-game`
3. Update client config to point to your cloud instance

**Option 3: Use CLI for Testing**
Test the backend logic directly with CLI commands:
```bash
# Connect
spacetime call text-game connect_session '"terminal"' '"test-123"' --server local

# Auth
spacetime call text-game authenticate_player '1' '"PlayerName"' --server local

# Submit command
spacetime call text-game submit_command '1' '"look"' --server local

# Execute tick
spacetime call text-game execute_tick --server local

# Query
spacetime sql text-game "SELECT * FROM player" --server local
```

---

## Setup for SDK Client (Works Locally)

If you're seeing a **404 Not Found** error when running the terminal client tests, it means the SpacetimeDB module hasn't been published yet.

## Solution: Publish the SpacetimeDB Module

The game backend needs to be published to SpacetimeDB before the terminal client can connect to it.

### Prerequisites

1. **SpacetimeDB must be running:**
   ```bash
   spacetime start
   ```

2. **The SpacetimeDB module must exist:**
   - Check if `../text_game_stdb` directory exists
   - This directory should contain the Rust module with reducers and tables

### Publishing the Module

```bash
# Navigate to the SpacetimeDB module directory
cd ../text_game_stdb

# Publish the module to SpacetimeDB with the name "text-game"
spacetime publish text-game

# Verify it was published
cd ../rs_text_game_test
spacetime list
```

You should see output like:
```
Databases:
  text-game  ...
```

### Alternative: Create a Mock Module

If the `text_game_stdb` directory doesn't exist yet, you can create a minimal SpacetimeDB module:

```bash
# Create a new SpacetimeDB module
cd ..
spacetime init text_game_stdb

cd text_game_stdb

# Edit src/lib.rs to add the required reducers and tables
# (See the multiplayer documentation for details)

# Publish it
spacetime publish text-game
```

### Required Tables

The module must include these tables:
- `session` - Player connections
- `player` - Player data
- `queued_command` - Command queue
- `command_log` - Command execution results
- `room` - Room definitions

### Required Reducers

The module must include these reducers:
- `connect_session(interface_type, connection_id)` - Create session
- `authenticate_player(session_id, player_name)` - Authenticate player
- `submit_command(session_id, command)` - Submit command to queue
- `execute_tick()` - Process command queue

## Verifying Setup

Once published, verify the setup:

```bash
# Check that the database exists
spacetime list

# Test a simple query
spacetime call text-game connect_session '"terminal"' '"test-123"'

# Run the automated test
./test_terminal_tick.sh
```

## Common Issues

### "No databases found"
- The module hasn't been published yet
- Run `spacetime publish text-game` in the module directory

### "Reducer not found"
- The published module is missing required reducers
- Check that all reducers are implemented in the module
- Republish the module after adding missing reducers

### "Table does not exist"
- The published module is missing required tables
- Check that all tables are defined in the module
- Republish the module after adding missing tables

### Wrong database name
If your database has a different name than "text-game", update the config:

Edit [src/terminal_client/client.rs](src/terminal_client/client.rs):
```rust
impl Default for SpacetimeConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            database_name: "your-database-name-here".to_string(),  // Change this
        }
    }
}
```

## Next Steps

After publishing the module successfully:

1. Create TestPlayer:
   ```bash
   ./create_test_player.sh
   ```

2. Run movement test:
   ```bash
   ./test_player_movement.sh
   ```

3. Run automated tick test:
   ```bash
   ./test_terminal_tick.sh
   ```
