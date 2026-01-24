# Terminal Client

Interactive terminal client for the multiplayer text adventure game, connecting to SpacetimeDB backend.

## Features

- Direct connection to SpacetimeDB HTTP API
- Character creation and authentication
- Instant commands (look, status, help)
- Queued commands (movement, actions)
- Real-time display of command results

## Building

```bash
cargo build --bin terminal_client --features spacetimedb-http --release
```

## Running

Make sure SpacetimeDB server is running:
```bash
spacetime start
```

Then run the terminal client:
```bash
cargo run --bin terminal_client --features spacetimedb-http
```

## Architecture

The terminal client connects to SpacetimeDB via HTTP:

1. **Connect** - Creates a session with `connect_session` reducer
2. **Authenticate** - Links to player account with `authenticate_player`
3. **Play** - Submits commands via `submit_command` reducer
4. **Query** - Gets player status via `get_player_info`

## Commands

- Movement: `go north`, `go south`, `go east`, `go west`, `go up`, `go down`
- Observation: `look`, `examine`, `inventory`, `status`
- Communication: `say <message>`, `tell <player> <message>`
- System: `help`, `quit`

## Implementation Status

✅ Client architecture  
✅ HTTP connection to SpacetimeDB  
✅ Authentication flow  
✅ Command submission  
✅ Display formatting  
⬜ Real-time updates (subscriptions)  
⬜ Full error handling  
⬜ Reconnection logic  

## Next Steps

- Implement SpacetimeDB subscriptions for real-time updates
- Add proper error handling and retry logic
- Integrate with actual game world (rooms.json)
- Test multiplayer interactions
