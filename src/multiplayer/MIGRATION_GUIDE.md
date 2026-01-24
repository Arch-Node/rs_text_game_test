# Migration Guide: Local Types → SpacetimeDB SDK

## Overview

The `src/multiplayer/` modules were **architectural prototypes** created during the design phase. Now that SpacetimeDB is implemented, these local types are **superseded by auto-generated SDK types**.

## Type Mapping

### Players

| Local Type | SpacetimeDB Table | SDK Type | Notes |
|------------|-------------------|----------|-------|
| `multiplayer::Player` | `player` | `spacetimedb_client::Player` | SDK uses position_x/y/z instead of Coord |
| `multiplayer::PlayerId` | `player.id: u64` | `u64` | SpacetimeDB uses raw u64 |
| `multiplayer::PlayerStatus` | `player.status: String` | `String` | SDK stores as string, not enum |

**Key Differences:**
```rust
// OLD (local types)
use crate::multiplayer::{Player, PlayerId};
use crate::models::Coord;

let player = Player {
    id: PlayerId::new(123),
    pos: Coord { w: 0, x: 100, y: 100 },
    // ...
};

// NEW (SDK types)
use crate::spacetimedb_client::Player;

let player = Player {
    id: 123,
    position_x: 100,
    position_y: 100,
    position_z: 100,
    dimension: "material".to_string(),
    // ...
};
```

### Sessions

| Local Type | SpacetimeDB Table | SDK Type | Notes |
|------------|-------------------|----------|-------|
| `multiplayer::Session` | `session` | `spacetimedb_client::Session` | SDK includes Identity field |
| `multiplayer::SessionId` | `session.id: u64` | `u64` | SpacetimeDB uses raw u64 |
| `multiplayer::InterfaceType` | `session.interface_type: String` | `String` | SDK stores as string |

### Commands

| Local Type | SpacetimeDB Table | SDK Type | Notes |
|------------|-------------------|----------|-------|
| `multiplayer::QueuedCommand` | `queued_command` | `spacetimedb_client::QueuedCommand` | Direct mapping |
| `multiplayer::CommandQueue` | N/A - SpacetimeDB handles | Query table directly | No client-side queue needed |

### Rooms

| Local Type | SpacetimeDB Table | SDK Type | Notes |
|------------|-------------------|----------|-------|
| `models::Room` (old) | `room` | `spacetimedb_client::Room` | New 3D room system |
| `models::Coord` | `position_x/y/z + dimension` | Separate fields | No Coord struct in SDK |

## Using SDK Types

### 1. Querying Data

```rust
// Instead of local state management:
let state = MultiplayerGameState::new(...);
let player = state.players.get(&player_id);

// Use SDK queries (when full connection implemented):
use spacetimedb_client::Player;

// Via SQL query
conn.query("SELECT * FROM Player WHERE id = ?", player_id)?;

// Via subscription (real-time)
conn.db().player().on_insert(|player, _| {
    println!("New player: {}", player.name);
});
```

### 2. Calling Reducers

```rust
// Instead of local command execution:
execute_command(&mut state, command);

// Use SDK reducers:
use spacetimedb_client::*;

// Connect session
conn.reducers().connect_session("terminal".to_string(), "conn-id".to_string())?;

// Submit command
conn.reducers().submit_command(session_id, "north".to_string())?;

// Execute tick
conn.reducers().execute_tick()?;
```

### 3. State Management

```rust
// Instead of Arc<RwLock<MultiplayerGameState>>:
let state = Arc::new(RwLock::new(MultiplayerGameState::new(world)));

// SpacetimeDB handles all state:
// - ACID transactions (no manual locking)
// - Real-time synchronization (no manual sync)
// - Automatic persistence (no save/load)
// - Subscription updates (push-based, not polling)
```

## Client-Side Logic

The local types can still be useful for:

### 1. Client-Side Caching

```rust
// Cache recent queries locally
struct ClientCache {
    player: Option<spacetimedb_client::Player>,
    current_room: Option<spacetimedb_client::Room>,
    nearby_players: Vec<spacetimedb_client::Player>,
}
```

### 2. Input Validation

```rust
// Validate before sending to server
use spacetimedb_client::*;

fn validate_command(text: &str) -> Result<(), String> {
    // Check command format before calling reducer
    if text.trim().is_empty() {
        return Err("Empty command".to_string());
    }
    Ok(())
}
```

### 3. Display Logic

```rust
// Format SDK types for display
use spacetimedb_client::Player;

fn format_player_status(player: &Player) -> String {
    format!(
        "{} at ({},{},{}) in {}",
        player.name,
        player.position_x,
        player.position_y,
        player.position_z,
        player.dimension
    )
}
```

## Migration Steps

### Phase 1: Use Both (Current)
- Keep local types for reference and learning
- Use SDK types for actual backend interaction
- Mark local types as `LEGACY` in comments

### Phase 2: Bridge Layer (Optional)
- Create conversion functions: `Player → spacetimedb_client::Player`
- Adapt local command handlers to call reducers
- Use local types for business logic, SDK for I/O

### Phase 3: Full SDK (Recommended)
- Remove local type definitions
- Use SDK types everywhere
- Keep only display/validation logic client-side

## Example: Full Migration

### Before (Local Types)
```rust
use crate::multiplayer::{Player, MultiplayerGameState};
use std::sync::Arc;
use tokio::sync::RwLock;

let state = Arc::new(RwLock::new(MultiplayerGameState::new(world)));

// Add player
let player = Player::new(PlayerId::new(1), "Hero".to_string(), ...);
state.write().await.add_player(player);

// Move player
if let Some(player) = state.write().await.players.get_mut(&player_id) {
    player.pos = new_pos;
}
```

### After (SDK Types)
```rust
use spacetimedb_sdk::DbConnection;
use crate::spacetimedb_client::*;

// Connect to SpacetimeDB
let conn = DbConnection::builder()
    .build("ws://localhost:3000/text-game")
    .await?;

// Subscribe to updates
conn.db().player().on_update(|player, _| {
    println!("Player moved: {} to ({},{},{})",
        player.name, player.position_x, player.position_y, player.position_z);
});

// Authenticate (creates player if needed)
conn.reducers().authenticate_player(0, "Hero".to_string())?;

// Submit movement command
conn.reducers().submit_command(0, "north".to_string())?;

// Execute tick (server processes all commands)
conn.reducers().execute_tick()?;
```

## Benefits of SDK Approach

| Feature | Local Types | SDK Types |
|---------|-------------|-----------|
| Type Safety | ✅ Compile-time | ✅ Generated from schema |
| Concurrency | Manual Arc/RwLock | ✅ Automatic (ACID) |
| Persistence | Manual save/load | ✅ Automatic |
| Real-time | Manual polling | ✅ Push subscriptions |
| Network | DIY protocol | ✅ Binary protocol |
| API Changes | Runtime errors | ✅ Compile-time errors |

## Conclusion

**Current Approach:** Keep `src/multiplayer/` as reference documentation while using SDK types for actual implementation.

**Future Options:**
1. **Pure SDK**: Remove local types entirely, use SDK everywhere
2. **Bridge Layer**: Convert between local types (business logic) and SDK types (I/O)
3. **Hybrid**: Keep useful abstractions, delegate storage to SDK

**Recommendation:** Start with Pure SDK for terminal client, then add abstractions only if needed.

---

**See Also:**
- [SDK Client Guide](../../SDK_CLIENT_GUIDE.md) - How to use the SDK
- [SpacetimeDB Module](../../../text_game_stdb/src/lib.rs) - Backend tables and reducers
- [Generated SDK](../spacetimedb_client/) - Auto-generated client types
