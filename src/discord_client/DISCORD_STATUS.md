# Discord Bot Status

## Overview

The Discord bot provides a messaging interface to the SpacetimeDB text adventure game through Discord.

**Status:** ✅ **Implementation Complete** (Compilation Verified)

## Architecture

```
Discord Gateway
       ↓
   Discord Bot (Serenity)
       ↓
  SpacetimeDB HTTP API
       ↓
  text-game Database
```

### Components

1. **Discord Bot (`src/discord_client/bot.rs`)**
   - Core bot logic with SpacetimeDB integration
   - Session management (Discord user ID → session_id)
   - Player authentication (Discord user ID → player name)
   - Command submission via HTTP
   - Caching for performance

2. **Message Handler (`src/discord_client/message_handler.rs`)**
   - Processes incoming Discord messages
   - Handles both DMs and @mentions
   - Authentication flow
   - Command routing (help, status, game commands)

3. **Formatter (`src/discord_client/formatter.rs`)**
   - Discord-specific message formatting
   - Uses Discord markdown and emojis
   - 2000 character limit handling
   - Welcome/help/status/error formatting

4. **Bot Binary (`src/bin/discord_bot.rs`)**
   - Entry point for Discord bot
   - EventHandler implementation
   - Background tick processor (3 second interval)
   - Environment configuration

## Features

### ✅ Implemented

- **Authentication Flow**
  - `@Bot auth PlayerName` in any channel or DM
  - Authenticated in guild → bot sends DM for gameplay
  - Keeps public channels clean

- **DM-Only Gameplay**
  - Game commands only work in direct messages
  - Prevents channel spam from location updates
  - Private gameplay experience

- **Game Commands**
  - Movement: `north`, `south`, `east`, `west`, `up`, `down`
  - Info: `look`, `help`, `status`
  - Commands submitted to SpacetimeDB

- **Message Handling**
  - Responds to @mentions in guilds (auth only)
  - Responds to all messages in DMs (full gameplay)
  - Ignores other bots' messages

- **Tick Processing**
  - Background task executes ticks every 3 seconds
  - Matches SpacetimeDB game tick rate

- **Session Management**
  - Automatic session creation
  - Session caching (Discord user → session ID)
  - Player name caching

- **Player Storage**
  - Players stored in SpacetimeDB `player` table
  - Created via `authenticate_player` reducer
  - Persists across bot restarts
  - Linked to Discord user ID via session

- **Error Handling**
  - Graceful error messages
  - Logging with log/env_logger

### 📋 Potential Enhancements

- **Rich Embeds**
  - Use Discord embeds for room descriptions
  - Formatted inventory displays
  - Colored messages based on content

- **Reaction Navigation**
  - React with ⬆️⬇️⬅️➡️ to move
  - Interactive UI without typing commands

- **Server-Specific Prefixes**
  - Custom command prefixes per guild
  - Support for `!command` syntax

- **Admin Commands**
  - Server management
  - Player moderation
  - Game state inspection

- **Help Command Improvements**
  - Context-sensitive help
  - Command examples
  - Tutorial mode

- **Status Dashboard**
  - Player stats display
  - Location history
  - Achievement tracking

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DISCORD_TOKEN` | Yes | - | Bot token from Discord Developer Portal |
| `SPACETIME_URL` | No | `http://localhost:3000` | SpacetimeDB server URL |

### Required Discord Intents

- `GUILD_MESSAGES` - Read messages in servers
- `DIRECT_MESSAGES` - Read direct messages
- `MESSAGE_CONTENT` - Access message content (privileged)

### Required Discord Permissions

- `Send Messages` - Reply to users
- `Read Messages/View Channels` - See messages
- `Read Message History` - Context for commands

## Building

```bash
# Check compilation
cargo check --features discord-client --bin discord_bot

# Build release
cargo build --release --features discord-client --bin discord_bot

# Run
cargo run --features discord-client --bin discord_bot
```

## Dependencies

### Core
- `serenity` 0.12 - Discord library (client, gateway, rustls_backend, model)
- `tokio` 1.x - Async runtime
- `reqwest` 0.11 - HTTP client for SpacetimeDB
- `anyhow` 1.x - Error handling
- `log` 0.4 - Logging
- `env_logger` 0.11 - Log configuration

### Features
- `discord-client` feature enables: `["serenity", "tokio", "reqwest"]`

## Implementation Notes

### Message Flow

**In Guild Channels (Public):**
1. User: `@Bot auth PlayerName`
2. Bot authenticates player
3. Bot replies in channel: "✅ Authenticated! Please send me a DM to start playing"
4. Bot sends DM with welcome message
5. Game commands in guild are rejected → "Game commands only work in DMs"

**In Direct Messages (Private):**
1. Discord message received via gateway
2. EventHandler's `message()` called
3. `handle_message()` checks authentication
4. Command parsed and processed
5. Response sent back to Discord DM

This keeps public channels clean while allowing easy discovery and authentication.

### Tick Processing

Background task runs every 3 seconds:
```rust
tokio::spawn(async move {
    let mut tick_interval = interval(Duration::from_secs(3));
    loop {
        tick_interval.tick().await;
        bot.execute_tick().await;
    }
});
```

### Caching Strategy

Two caches maintained:
- `session_cache`: Discord user ID → SpacetimeDB session ID
- `player_cache`: Discord user ID → player name

Both use `Arc<RwLock<HashMap>>` for concurrent access.

### Error Handling

All errors use `anyhow::Result` for:
- Send + Sync compatibility (required by Serenity)
- Context propagation
- Ergonomic error handling

## Testing

### Manual Testing Checklist

- [ ] Bot connects to Discord
- [ ] Bot responds to @mentions
- [ ] Bot responds in DMs
- [ ] Authentication works (`auth PlayerName`)
- [ ] Help command shows guide
- [ ] Status command shows player info
- [ ] Movement commands work (north, south, etc.)
- [ ] Look command returns room description
- [ ] Tick processor runs in background
- [ ] Multiple users can play simultaneously
- [ ] Sessions persist across bot restarts (via SpacetimeDB)

### Integration Testing

1. Start SpacetimeDB
2. Start Discord bot
3. Message bot in Discord
4. Verify commands execute
5. Check SpacetimeDB logs for activity

## Known Limitations

1. **No Command History** - Each command is independent
2. **No Streaming Output** - Long descriptions sent as single message
3. **No Notification System** - Users must actively check for changes
4. **No Guild-Specific Config** - Same bot behavior in all servers
5. **Basic Error Messages** - Could be more user-friendly

## Comparison to Signal Bot

| Feature | Discord Bot | Signal Bot |
|---------|-------------|------------|
| Architecture | Gateway (WebSocket) | Webhook (HTTP) |
| Authentication | Discord user ID | Signal number |
| Message Delivery | Push via gateway | Pull via webhook |
| Formatting | Discord markdown | Basic text |
| Rich Content | Embeds, reactions | Limited |
| Setup Complexity | Medium (Dev Portal) | High (Signal CLI) |
| Hosting | Requires persistent connection | Stateless webhook |

## Next Steps

1. ✅ Complete implementation
2. ✅ Verify compilation
3. ⏳ Integration testing
4. ⏳ Deploy to test server
5. ⏳ Gather user feedback
6. ⏳ Add rich embeds
7. ⏳ Implement reaction navigation

## Resources

- [Discord Bot Setup Guide](./DISCORD_BOT_SETUP.md)
- [Serenity Documentation](https://docs.rs/serenity/)
- [Discord Developer Portal](https://discord.com/developers/applications)
- [SpacetimeDB Docs](https://spacetimedb.com/docs)
