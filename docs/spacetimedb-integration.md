# SpacetimeDB Integration Plan

## Overview

SpacetimeDB is a relational database + server combined into one, designed specifically for multiplayer games. Instead of traditional architecture (client → web server → database), clients connect directly to SpacetimeDB and execute game logic as WASM modules inside the database.

## Why SpacetimeDB for This Project?

### Perfect Fit for Our Requirements:

1. **Built for Multiplayer Games**
   - Designed for real-time state synchronization (our tick-based system)
   - Used by BitCraft MMORPG - proven at scale
   - Built-in pub/sub for room-based event broadcasting

2. **Solves Key Challenges**
   - **State Synchronization:** Automatic real-time updates to all connected clients
   - **Race Conditions:** ACID transactions prevent "two players take same item" issues
   - **Persistence:** Built-in, no separate save/load logic needed
   - **Scaling:** Serverless, no Docker/K8s complexity

3. **Performance**
   - ~100μs transaction latency (perfect for 15-second ticks)
   - ~100,000 transactions/second capacity
   - Game logic runs in-database (WASM) - minimal network hops

4. **Developer Experience**
   - Write game logic in Rust (our current language)
   - Auto-generates client SDKs (could add web/mobile clients later)
   - No ORM complexity - direct table access
   - Built-in permissions system

## Architecture Changes with SpacetimeDB

### Current Stub Architecture:
```
Terminal/Signal/SMS → Rust Server → mpsc channels → Tick System → Broadcast Events
                                   ↓
                              (No persistence yet)
```

### SpacetimeDB Architecture (Option A - Full Integration):
```
Terminal/Signal/SMS → SpacetimeDB Module (WASM) → Tables + Subscriptions
                                 ↓
                           Rust Reducers:
                           - submit_command
                           - execute_tick
                           - broadcast_event
                                 ↓
                           Real-time sync to all clients
```

### Hybrid Architecture (Option B - Adapter Layer):
```
Terminal/Signal/SMS → Rust Adapter Layer → SpacetimeDB (persistence + pub/sub)
                              ↓
                      Our tick system (as designed)
                              ↓
                      SpacetimeDB tables for storage
                      SpacetimeDB subscriptions for events
```

## SpacetimeDB Concepts

### Tables (Schema)
Define data structures as Rust structs with `#[spacetimedb::table]`:

```rust
#[spacetimedb::table(name = player)]
pub struct Player {
    #[primarykey]
    pub id: u64,
    pub name: String,
    pub account_id: u64,
    pub class: String,
    pub position: Coord,
    pub status: PlayerStatus,
    pub last_action: Option<u64>,
}

#[spacetimedb::table(name = session)]
pub struct Session {
    #[primarykey]
    pub id: u64,
    pub player_id: Option<u64>,
    pub interface_type: InterfaceType,
    pub auth_state: AuthState,
    #[index(btree)]
    pub connection_timestamp: u64,
}

#[spacetimedb::table(name = queued_command)]
pub struct QueuedCommand {
    #[primarykey]
    #[autoinc]
    pub id: u64,
    pub player_id: u64,
    pub command: String,
    pub queued_at: u64,
    pub priority: u8,
}
```

### Reducers (Game Logic)
Functions that modify state, executed as transactions:

```rust
#[spacetimedb::reducer]
pub fn submit_command(ctx: &ReducerContext, command_text: String) -> Result<(), String> {
    // 1. Get session from context
    let session_id = ctx.sender; // SpacetimeDB provides authenticated sender
    let session = ctx.db.session().find_by_id(&session_id)
        .ok_or("Session not found")?;
    
    // 2. Check if player already has command queued
    let existing = ctx.db.queued_command()
        .filter(|cmd| cmd.player_id == session.player_id)
        .collect::<Vec<_>>();
    
    if !existing.is_empty() {
        return Err("You already have a command queued".to_string());
    }
    
    // 3. Parse and validate command
    let command = parse_command(&command_text)?;
    let player = ctx.db.player().find_by_id(&session.player_id.unwrap())?;
    let validation = validate_command(ctx, &player, &command);
    
    // 4. If instant command, execute immediately
    if validation.is_instant {
        execute_instant_command(ctx, &player, &command)?;
        return Ok(());
    }
    
    // 5. Otherwise, queue for next tick
    if !validation.is_valid {
        return Err(validation.errors.join("; "));
    }
    
    ctx.db.queued_command().insert(QueuedCommand {
        id: 0, // autoinc
        player_id: session.player_id.unwrap(),
        command: command_text,
        queued_at: ctx.timestamp,
        priority: 0,
    })?;
    
    Ok(())
}

#[spacetimedb::reducer]
pub fn execute_tick(ctx: &ReducerContext) -> Result<(), String> {
    // 1. Get all queued commands
    let commands = ctx.db.queued_command().iter().collect::<Vec<_>>();
    
    // 2. Execute each command
    for cmd in commands {
        let player = ctx.db.player().find_by_id(&cmd.player_id)?;
        execute_queued_command(ctx, &player, &cmd)?;
        
        // 3. Remove from queue
        ctx.db.queued_command().delete_by_id(&cmd.id);
    }
    
    // 4. Broadcast events (automatic via subscriptions)
    Ok(())
}
```

### Subscriptions (Real-time Updates)
Clients subscribe to queries and get automatic updates:

```rust
// Clients can subscribe to:
// "SELECT * FROM player WHERE position.room_id = ?"
// Automatically notified when any player in room changes
```

## Implementation Approaches

### Option A: Full SpacetimeDB Integration (Recommended for Long-term)

**Pros:**
- Maximum performance (logic runs in-database)
- Automatic persistence and sync
- Scalability built-in
- Simpler deployment (just publish module)

**Cons:**
- Bigger architectural change from current stubs
- Need to learn SpacetimeDB patterns
- Some current designs (mpsc channels) don't map directly
- Less control over tick scheduling

**Best for:** Production deployment, scaling to many players

### Option B: Hybrid (SpacetimeDB as Backend)

**Pros:**
- Keep our current tick-based architecture
- Use SpacetimeDB for persistence + pub/sub only
- Easier migration from current stubs
- More control over game loop

**Cons:**
- Don't get full SpacetimeDB performance benefits
- Still need to manage server process
- More complex than Option A

**Best for:** Faster initial development, preserve current design

## Recommended Path Forward

### Phase 1: Prototype with Hybrid Approach
1. Keep current stub architecture (tick system, command queue)
2. Add SpacetimeDB as persistence layer
3. Use SpacetimeDB subscriptions for event broadcasting
4. Validate tick-based gameplay works with real backend

### Phase 2: Evaluate Full Migration
After gameplay is working:
1. Measure performance bottlenecks
2. Decide if moving logic into SpacetimeDB reducers is beneficial
3. Could incrementally move hot paths into database

## Next Steps

1. **Install SpacetimeDB locally:**
   ```bash
   curl https://install.spacetimedb.com | bash
   spacetime start
   ```

2. **Add dependency to Cargo.toml:**
   ```toml
   [dependencies]
   spacetimedb = "0.11"  # Check latest version
   ```

3. **Create initial table definitions** (map our existing types)

4. **Build simple reducer** (test round-trip)

5. **Integrate with existing terminal interface** (prove concept)

6. **Add Signal/SMS adapters** once persistence works

## Key Questions to Answer

- [ ] How do we schedule tick execution in SpacetimeDB? (Reducer cron? External trigger?)
- [ ] Can we do "instant" commands that don't create transactions?
- [ ] How do we handle different interfaces (Terminal vs Signal vs SMS) connecting?
- [ ] What's the best way to represent ConnectionIdentifiers in SpacetimeDB?
- [ ] Do we need client SDK or can we use HTTP API for Signal/SMS bots?

## Resources

- [SpacetimeDB Docs](https://spacetimedb.com/docs)
- [Rust SDK Reference](https://docs.rs/spacetimedb/latest/spacetimedb/)
- [BitCraft Case Study](https://spacetimedb.com/blog/bitcraft-spacetimedb) (MMORPG using SpacetimeDB)
- [GitHub Examples](https://github.com/ClockworkLabs/SpacetimeDB/tree/master/modules)
