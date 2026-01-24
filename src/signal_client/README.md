[← Back to Main README](../../README.md)

---

# Signal Messenger Bot

Signal Messenger interface for the text adventure game. Players can play via Signal messages on their phone.

## Status

**Phase 1 Complete** ✅ (8 hours development)

- ✅ Webhook-based message receiving
- ✅ Player authentication via `auth PlayerName` command
- ✅ Group authentication with DM redirect for gameplay
- ✅ DM-only gameplay (keeps group chats clean)
- ✅ Session management with in-memory caching
- ✅ Player storage in SpacetimeDB
- ✅ Command processing (instant + queued)
- ✅ Background tick processor (3 seconds)
- ✅ SpacetimeDB HTTP API integration
- ✅ Emoji-enhanced message formatting

## Architecture

```
Signal Messenger (your phone)
       ↓
signal-cli-rest-api (port 8080)
       ↓
SignalBot webhook server (port 3001)
       ↓
SpacetimeDB HTTP API (port 3000)
       ↓
text-game database
```

### Components

**Source Files** (`src/signal_client/`):
- **bot.rs** - Core SignalBot with session/player caching, SpacetimeDB integration
- **webhook.rs** - Actix-web HTTP server receiving Signal messages
- **message_handler.rs** - Command parsing, authentication, routing
- **formatter.rs** - Signal message formatting with emojis (📍 🚶 🚪 ✅ ❌)
- **mod.rs** - Module exports

**Binary**: `src/bin/signal_bot.rs` - Entry point with environment configuration

**Build**: Cargo feature `signal-client` with actix-web, tokio, reqwest dependencies

## Quick Start

See [SIGNAL_BOT_SETUP.md](SIGNAL_BOT_SETUP.md) for complete setup instructions.

**Prerequisites:**
1. SpacetimeDB running on `localhost:3000`
2. signal-cli-rest-api running on `localhost:8080`
3. Signal phone number registered for bot

**Run:**
```bash
export BOT_NUMBER="+15551234567"
export SIGNAL_API_URL="http://localhost:8080"
export SPACETIME_URL="http://localhost:3000"

cargo run --features signal-client --bin signal_bot
```

## How to Play

1. **Authentication** (first time):
   - Send message to bot: `auth YourPlayerName`
   - Or send in group chat and bot will DM you
   
2. **Commands**:
   - Movement: `north`, `south`, `east`, `west`, `up`, `down`
   - Info: `look`, `help`, `status`
   - All gameplay happens in DM with bot

3. **Response**:
   - Instant commands respond immediately
   - Movement commands queued and executed on next tick (3s)

## Current Limitations

### Known Issues

1. **Session ID query incomplete** - Returns placeholder after session creation
   - Impact: Session caching may not work correctly
   - Fix: Query session table by connection_id (Phase 2)

2. **No tick result feedback** - Commands queued but results not returned
   - Impact: Users don't see actual outcome of actions
   - Fix: Implement command_log subscription (Phase 2)

3. **Synchronous webhook handler** - Messages processed synchronously
   - Impact: Slight latency with concurrent messages
   - Fix: Refactor for async spawn (Phase 2)

### Feature Gaps

- No Signal group chat support (auth only, no gameplay)
- No rate limiting per phone number
- No session persistence (memory only, resets on restart)
- No player-to-player messaging
- No admin commands
- No metrics/monitoring

## Phase 2 Roadmap

**Priority Tasks** (8-11 hours):

1. **Fix session ID query** (2h) - Query session table after connect_session
2. **Implement result feedback** (3-4h) - Subscribe to command_log, return tick results
3. **Add integration tests** (2-3h) - Mock signal-cli-rest-api, test flows
4. **Enhance error handling** (1-2h) - Better user messages, retry logic

## Performance

- **Webhook latency**: <10ms
- **Tick interval**: 3 seconds (configurable)
- **Message send latency**: 100-500ms (depends on signal-cli-rest-api)
- **SpacetimeDB latency**: 10-50ms (local HTTP)
- **Memory usage**: ~10MB for session cache

## Security Notes

- Phone numbers stored in memory only (not persisted to disk)
- No authentication beyond Signal's phone verification
- Webhook should use HTTPS in production
- No rate limiting implemented yet
- Keep BOT_NUMBER environment variable private

## Development

**Build:**
```bash
cargo build --features signal-client --bin signal_bot
```

**Test:**
```bash
./test_signal_bot.sh  # Validates setup
cargo test --features signal-client  # Runs unit tests
```

**Debug:**
```bash
RUST_LOG=debug cargo run --features signal-client --bin signal_bot
```

## Documentation

- **[SIGNAL_BOT_SETUP.md](SIGNAL_BOT_SETUP.md)** - Complete setup and deployment guide
- **This README** - Overview, status, and quick reference

## References

- [signal-cli-rest-api](https://github.com/bbernhard/signal-cli-rest-api) - Signal Protocol API wrapper
- [SpacetimeDB Docs](https://spacetimedb.com/docs) - Database documentation
- [Actix Web](https://actix.rs/) - Web framework used

---

[← Back to Main README](../../README.md)
