[← Back to Main README](../README.md)

---

# SpacetimeDB Quick Start Guide

## Installation Complete! ✅

SpacetimeDB is now installed and running on `localhost:3000`

## What We Have

1. **SpacetimeDB Server:** Running locally for development
2. **CLI Tool:** `spacetime` command available in terminal
3. **Rust SDK:** Added to Cargo.toml (version 1.11)

## Next Steps

### 1. Create a SpacetimeDB Module

We have two options for integrating SpacetimeDB:

#### Option A: Separate Module Project (Recommended for prototyping)
Create a standalone SpacetimeDB module:

```bash
# Create new SpacetimeDB module project
cd ~/software/repos
spacetime generate --lang rust rs_text_game_module
cd rs_text_game_module
```

This creates a fresh project specifically for SpacetimeDB with proper structure.

#### Option B: Integrate into Existing Project
Add SpacetimeDB module capabilities to current project:

```bash
# Create src/spacetimedb_module/ directory
mkdir -p src/spacetimedb_module

# Add lib.rs with SpacetimeDB tables and reducers
```

Requires additional Cargo.toml configuration for WASM compilation.

### 2. Define Initial Tables

Start with core game state tables:

```rust
// src/spacetimedb_module/lib.rs
use spacetimedb::{table, reducer, ReducerContext};

#[table(name = player)]
pub struct Player {
    #[primarykey]
    #[autoinc]
    pub id: u64,
    pub name: String,
    pub position_x: i32,
    pub position_y: i32,
}

#[table(name = session)]
pub struct Session {
    #[primarykey]
    #[autoinc]
    pub id: u64,
    pub player_id: Option<u64>,
    pub connected_at: u64,
}
```

### 3. Create First Reducer

Add a simple command handler:

```rust
#[reducer]
pub fn create_player(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let player = Player {
        id: 0, // autoinc
        name,
        position_x: 0,
        position_y: 0,
    };
    
    ctx.db.player().insert(player)?;
    Ok(())
}
```

### 4. Compile and Publish Module

```bash
# Compile to WASM
cargo build --release --target wasm32-unknown-unknown

# Publish to local SpacetimeDB
spacetime publish -s localhost:3000 target/wasm32-unknown-unknown/release/module_name.wasm -c test_game
```

### 5. Test with CLI

```bash
# Call reducer from CLI
spacetime call -s localhost:3000 test_game create_player "\"PlayerOne\""

# Query table
spacetime sql -s localhost:3000 test_game "SELECT * FROM player"
```

## Current Architecture Decision Point

We need to decide: **Hybrid vs Full Integration**

### Hybrid Approach (Easier to start)
- Keep existing Rust game code (main.rs, multiplayer modules)
- Use SpacetimeDB only for persistence
- Our tick system runs in Rust process
- SpacetimeDB stores state + broadcasts events

**Pros:** Easier migration, keep current design
**Cons:** Don't get full SpacetimeDB performance

### Full Integration (Better long-term)
- Move game logic into SpacetimeDB reducers
- Tick system becomes scheduled reducer
- All state in SpacetimeDB tables
- Maximum performance, auto-scaling

**Pros:** Full SpacetimeDB benefits, simpler deployment
**Cons:** Bigger architectural shift

## Recommendation for Today

**Start with Hybrid:**
1. Create separate SpacetimeDB module (Option A above)
2. Define Player, Session, World tables
3. Test basic persistence (create player, query state)
4. Keep existing multiplayer stub code
5. Build adapter layer to connect them

This lets us:
- Learn SpacetimeDB incrementally
- Validate it works for our game
- Keep optionality for full migration later

## Useful Commands

```bash
# Start server
spacetime start

# Stop server (Ctrl+C in server terminal)

# List running modules
spacetime list -s localhost:3000

# View logs
spacetime logs -s localhost:3000 test_game

# Delete module
spacetime delete -s localhost:3000 test_game
```

## Resources

- [SpacetimeDB Rust SDK Docs](https://docs.rs/spacetimedb/latest/spacetimedb/)
- [Rust Quickstart](https://spacetimedb.com/docs/quickstarts/rust)
- [Table Reference](https://spacetimedb.com/docs/modules/rust/tables)
- [Reducer Reference](https://spacetimedb.com/docs/modules/rust/reducers)

## Next Task

Would you like to:
1. **Create prototype module** - Build simple SpacetimeDB module with Player/Session tables
2. **Continue architecture design** - Finish Tasks 1.3-1.7 before coding
3. **Explore SpacetimeDB features** - Test queries, subscriptions, real-time updates

Choose your path!

---

[← Back to Main README](../README.md)
