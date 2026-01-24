# Cross-Client Event Broadcasting

## Overview

The multiplayer event broadcasting system enables real-time awareness between players. When one player performs an action, nearby players receive instant notifications via WebSocket callbacks.

## Architecture

### Event Broadcaster Module (`src/event_broadcaster.rs`)

The event broadcaster provides:

1. **GameEvent enum** - Types of events that can be broadcast:
   - `PlayerMoved`: Position changes (with from/to coordinates)
   - `PlayerJoined`: New player enters the area
   - `PlayerLeft`: Player disconnects
   - `PlayerAction`: Command execution (non-movement)

2. **Event Monitoring Setup** - Registers SpacetimeDB table callbacks:
   - `player.on_update()`: Detects position changes
   - `player.on_insert()`: Detects new players joining
   - `command_log.on_insert()`: Detects player actions

3. **Event Formatting** - Distance-based filtering and message formatting:
   - Only shows events within 5 units for movements/joins
   - Only shows actions within 3 units
   - Different dimension = no notification

### Client Integration

#### Terminal Client

```rust
// During authentication
let (event_tx, event_rx) = mpsc::channel(100);
setup_event_monitoring(conn, player_name, event_tx);
self.event_rx = Some(event_rx);

// Poll for events in game loop
if let Some(message) = client.poll_event() {
    println!("{}", message);
}
```

#### Signal Bot

```rust
// During authentication
let (event_tx, event_rx) = mpsc::channel(100);
setup_event_monitoring(conn, player_name, event_tx.clone());

// Spawn background task to forward events to Signal
tokio::spawn(async move {
    while let Some(event) = event_rx.recv().await {
        if let Some(message) = format_event(&event, current_pos) {
            bot.send_message(&phone, &message).await;
        }
    }
});
```

#### Discord Bot

Similar to Signal bot - events spawn background task that forwards formatted messages to Discord channels.

## How It Works

### 1. Background WebSocket Processor

Each client spawns a background task via `tokio::spawn(conn.run_async())` that:
- Maintains persistent WebSocket connection to SpacetimeDB
- Receives table update notifications
- Triggers registered callbacks

### 2. Table Subscriptions

The SDK automatically subscribes to:
- `player` table: Position updates, new players
- `command_log` table: Command execution events
- `session` table: Connection/disconnection events

### 3. Callback Execution

When SpacetimeDB publishes a table update:
1. Background processor receives the notification
2. Filters callbacks registered for that table
3. Executes callback with new/old row data
4. Callback sends event to mpsc channel

### 4. Event Delivery

Each client has an event channel:
- Terminal: Polls `try_recv()` in game loop
- Signal/Discord: Background task `recv().await` and forwards to messaging API

### 5. Distance-Based Filtering

Events are filtered by proximity:
- **Movements/Joins**: 5 unit radius
- **Actions**: 3 unit radius
- **Different dimensions**: Not shown

Distance calculated via Euclidean formula:
```rust
distance = sqrt((x2-x1)² + (y2-y1)² + (z2-z1)²)
```

## Example Flow

### Player A moves north:

1. **Reducer Execution**
   ```
   execute_tick() updates Player A's position from (100,100,100) to (100,101,100)
   ```

2. **Database Update**
   ```
   SpacetimeDB commits transaction, updates player table
   ```

3. **WebSocket Broadcast**
   ```
   SpacetimeDB sends table update to all connected clients
   ```

4. **Callback Trigger (Player B's client)**
   ```
   player.on_update() fires:
   - old_player: position (100,100,100)
   - new_player: position (100,101,100)
   - Generates PlayerMoved event
   ```

5. **Event Filtering**
   ```
   format_event() checks:
   - Same dimension? ✓
   - Within 5 units? ✓ (distance = 1)
   - Returns: "👤 Alice moved north"
   ```

6. **Delivery to Player B**
   ```
   Terminal: Prints to console
   Signal: Sends message to phone
   Discord: Posts to channel
   ```

## Event Types in Detail

### PlayerMoved
```rust
GameEvent::PlayerMoved {
    player_name: String,
    from_x, from_y, from_z: i32,
    to_x, to_y, to_z: i32,
    dimension: String,
}
```
**Trigger:** `player.on_update()` when position changes  
**Display:** "👤 Alice moved north" (direction calculated from coordinates)  
**Range:** 5 units

### PlayerJoined
```rust
GameEvent::PlayerJoined {
    player_name: String,
    position_x, position_y, position_z: i32,
    dimension: String,
}
```
**Trigger:** `player.on_insert()`  
**Display:** "✨ Bob joined the area"  
**Range:** 5 units

### PlayerAction
```rust
GameEvent::PlayerAction {
    player_name: String,
    action: String, // command text
    position_x, position_y, position_z: i32,
    dimension: String,
}
```
**Trigger:** `command_log.on_insert()`  
**Display:** "🎮 Alice used: look around"  
**Range:** 3 units (more intimate)

### PlayerLeft
```rust
GameEvent::PlayerLeft {
    player_name: String,
}
```
**Trigger:** Future implementation (session disconnect detection)  
**Display:** "👋 Bob left the game"  
**Range:** Global (no filtering)

## Performance Characteristics

### Latency
- **WebSocket delivery:** ~10-50ms
- **Callback execution:** <1ms
- **Event formatting:** <1ms
- **Total player-to-player:** ~20-100ms

### Scalability
- **Per-player overhead:** 1 WebSocket connection + 3 table subscriptions
- **Event fanout:** O(n) where n = players in proximity
- **Memory:** ~1KB per event in channel buffer (100 event buffer = 100KB)

### Optimization Opportunities
1. **Spatial indexing**: Group players by grid cells to reduce callback executions
2. **Event batching**: Send multiple events in one message
3. **Client-side prediction**: Show local actions immediately, confirm with server
4. **Selective subscriptions**: Only subscribe to tables for current dimension

## Testing

### Unit Tests
```bash
cargo test event_broadcaster --features spacetimedb-sdk-client
```

### Integration Tests
```bash
# Run multiplayer event test
./tests/test_multiplayer_events.sh

# Expected output:
# - Two players connect
# - Movement events broadcast
# - Distance filtering applied
# - Callbacks fire instantly
```

### Manual Testing
1. Start two terminal clients in separate windows
2. Authenticate with different player names
3. Move one player with `north`, `south`, etc.
4. Observe other player receives "👤 Alice moved north" notification

## Future Enhancements

### Planned Features
- [ ] Room-based event channels (only broadcast to players in same room)
- [ ] Event priorities (movement < action < combat)
- [ ] Event replay (catch up on missed events after reconnect)
- [ ] Custom event filters per player (mute notifications, keywords)
- [ ] Event aggregation ("3 players entered the room")

### Technical Improvements
- [ ] Add `player.on_delete()` for proper PlayerLeft events
- [ ] Implement session timeout detection
- [ ] Add event ACK/confirmation system
- [ ] Create event history table for debugging
- [ ] Add metrics (events sent, latency, callback execution time)

## Troubleshooting

### Events not appearing?

1. **Check player distance:**
   ```rust
   // In terminal client
   client.get_player_info(&self_player_id); // Your position
   client.get_players_in_dimension(&dimension); // Other players
   ```

2. **Verify WebSocket connection:**
   - Look for "Connected with real-time subscriptions" message
   - Check SpacetimeDB logs for connection events

3. **Test callback registration:**
   - Add `log::debug!()` in event_broadcaster.rs callbacks
   - Confirm callbacks fire when tables update

4. **Check event channel:**
   - Ensure `poll_event()` or background receiver is running
   - Check for channel buffer overflow (increase from 100)

### Performance issues?

1. **Too many events:** Increase filter distances (reduce radius)
2. **Slow delivery:** Check network latency to SpacetimeDB
3. **Memory usage:** Reduce event channel buffer size
4. **CPU usage:** Profile callback execution time

## Related Documentation

- [SpacetimeDB SDK Documentation](../docs/spacetimedb-integration.md)
- [WebSocket Architecture](../docs/architecture-analysis.md)
- [Multiplayer State Design](../docs/multiplayer-state-design.md)
- [Terminal Client Guide](../src/terminal_client/README.md)
