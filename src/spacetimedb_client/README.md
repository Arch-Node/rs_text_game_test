# SpacetimeDB SDK Client

This is a proper SpacetimeDB client using the official SDK.

## Setup

First, generate client bindings from the SpacetimeDB module:

```bash
cd text_game_stdb
spacetime generate --lang rust --out-dir ../rs_text_game_test/src/spacetimedb_client
```

This creates Rust code that matches your SpacetimeDB tables and reducers.

## Usage

```bash
cargo run --bin sdk_client --features spacetimedb-sdk-client
```

## How It Works

1. **Generated Code**: SpacetimeDB generates Rust structs/types from your module
2. **Auto-Subscriptions**: SDK automatically subscribes to table changes
3. **Real-Time Updates**: Get callbacks when data changes
4. **Type Safety**: All interactions are type-checked at compile time

## Advantages Over HTTP

- ✅ Real-time updates via WebSocket subscriptions
- ✅ Automatic reconnection
- ✅ Type-safe API (generated from your module)
- ✅ Efficient binary protocol
- ✅ Built-in state management
