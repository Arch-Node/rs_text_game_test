[← Back to Main README](../../README.md)

---

# Terminal Client Testing Status

## ✅ Backend Ready

### SpacetimeDB Module
- ✅ Player table with 3D position (x, y, z) + dimension
- ✅ Room table with 10 rooms in material dimension (centered at 100,100,100)
- ✅ Session management
- ✅ Command queueing  
- ✅ Movement system (north/south/east/west/up/down)
- ✅ Reducers: connect_session, authenticate_player, submit_command, execute_tick
- ✅ Server running on localhost:3000

### Test Rooms
Coordinate system uses 100 as center (to avoid negative numbers):
```
Town Square       (100,100,100) - spawn point
Market Street     (100,101,100) - north
Craftsman         (101,100,100) - east  
Temple Gardens    (99,100,100)  - west
City Gate         (100,99,100)  - south
Clock Tower       (100,100,101) - up
Town Cellar       (100,100,99)  - down
Residential       (101,101,100) - northeast
Tavern            (100,102,100) - far north
Spirit Plaza      (100,100,100) - ethereal dimension
```

## ✅ Terminal Client Fixed

### What Was Fixed
The terminal client now correctly uses SpacetimeDB HTTP API:

1. **Reducer calls** - Send JSON arrays directly (e.g., `[session_id, "player_name"]`)
2. **No return data from reducers** - Reducers only return `Ok(())` or `Err(String)`
3. **Query tables separately** - After calling reducers, query tables with SQL to get data
4. **Correct SQL endpoint** - POST to `/database/sql/{database}` with SQL as body

### Implementation Pattern

```rust
// 1. Call reducer (creates data)
client.post("/database/call/text-game/authenticate_player")
    .json(&[session_id, "Hero"])
    .send()?;

// 2. Query table (retrieve data)
client.post("/database/sql/text-game")
    .body("SELECT * FROM player WHERE name = 'Hero'")
    .send()?;
```

## Quick Test Commands

```bash
# Build terminal client
cargo build --bin terminal_client --features spacetimedb-http

# Run terminal client
cargo run --bin terminal_client --features spacetimedb-http

# Create session
spacetime call text-game -s local connect_session '"terminal"' '"test-conn"'

# Authenticate  
spacetime call text-game -s local authenticate_player 1 '"Hero"'

# Check player
spacetime sql text-game -s local -- "SELECT * FROM player WHERE name = 'Hero'"

# Move north
spacetime call text-game -s local submit_command '"go north"'

# Execute tick
spacetime call text-game -s local execute_tick

# Check new position
spacetime sql text-game -s local -- "SELECT position_x, position_y, position_z FROM player WHERE name = 'Hero'"
```

## Next Steps

1. **Add tokio to default dependencies** so multiplayer module compiles
2. **Fix terminal client** to query tables after calling reducers  
3. **Or switch to SpacetimeDB SDK** for proper integration
4. **Test full gameplay loop** (connect → auth → look → move → look)

## Alternative: Simple CLI Tool

Instead of the async terminal client, we could make a simpler tool that just wraps spacetime commands:

```bash
#!/bin/bash
# game-cli.sh - Simple wrapper

case $1 in
  connect)
    spacetime call text-game -s local connect_session '"terminal"' "\"$2\""
    ;;
  auth)
    spacetime call text-game -s local authenticate_player 1 "\"$2\""
    ;;
  move)
    spacetime call text-game -s local submit_command "\"go $2\""
    ;;
  tick)
    spacetime call text-game -s local execute_tick
    ;;
  where)
    spacetime sql text-game -s local -- "SELECT * FROM player WHERE name = '$2'"
    ;;
esac
```

This would work immediately while we build the proper async client!

---

[← Back to Main README](../../README.md)
