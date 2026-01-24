# Signal Bot Setup Guide

This guide covers setting up the Signal Messenger interface for Layered Realms.

## Overview

The Signal bot allows players to interact with the game via Signal Messenger. The architecture is:

```
Signal Messenger → signal-cli-rest-api → Webhook Server → SpacetimeDB
                                         (signal_bot)
```

## Prerequisites

1. **SpacetimeDB running** on localhost:3000
2. **signal-cli-rest-api** installed and configured
3. **Signal phone number** registered for the bot

## Step 1: Install signal-cli-rest-api

### Option A: Docker (Recommended)

```bash
docker run -d \
  --name signal-cli-rest-api \
  -p 8080:8080 \
  -v ~/signal-cli-config:/home/.local/share/signal-cli \
  bbernhard/signal-cli-rest-api:latest
```

### Option B: Native Installation

See https://github.com/bbernhard/signal-cli-rest-api for detailed instructions.

## Step 2: Register Signal Phone Number

You need a phone number to register as the bot. This can be:
- A dedicated number (recommended for production)
- A VOIP number (like Google Voice)
- A spare SIM card

Register the number:

```bash
# Using Docker
docker exec signal-cli-rest-api \
  curl -X POST http://localhost:8080/v1/register/{YOUR_NUMBER}

# Follow the SMS verification process
docker exec signal-cli-rest-api \
  curl -X POST http://localhost:8080/v1/register/{YOUR_NUMBER}/verify/{CODE}
```

Replace `{YOUR_NUMBER}` with your phone number in international format (e.g., `+15551234567`).

## Step 3: Configure Webhook

Configure signal-cli-rest-api to send incoming messages to your bot:

```bash
# Edit the configuration to point to your webhook endpoint
# This depends on your signal-cli-rest-api setup
# The webhook URL should be: http://YOUR_SERVER:3001/signal/webhook
```

If running locally, you might need to use ngrok or similar for testing:

```bash
# Install ngrok
brew install ngrok  # or download from ngrok.com

# Expose your local webhook
ngrok http 3001

# Use the ngrok URL in signal-cli-rest-api config
# Example: https://abc123.ngrok.io/signal/webhook
```

## Step 4: Build and Run the Bot

```bash
# Build the Signal bot binary
cargo build --features signal-client --bin signal_bot

# Set environment variables
export BOT_NUMBER="+15551234567"          # Your registered Signal number
export SIGNAL_API_URL="http://localhost:8080"
export SPACETIME_URL="http://localhost:3000"

# Run the bot
cargo run --features signal-client --bin signal_bot
```

The bot will:
- Start webhook server on `0.0.0.0:3001`
- Listen at `/signal/webhook` for incoming messages
- Execute ticks every 3 seconds to process commands
- Send responses via signal-cli-rest-api

## Step 5: Test the Bot

1. Send a message from your personal Signal to the bot's number
2. The first message should be your character name
3. The bot will authenticate you and respond with welcome message
4. Send commands like `north`, `look`, `status`, etc.

Example conversation:

```
You: TestPlayer
Bot: 👋 Welcome, TestPlayer! You are in Town Square.
     Commands: north, south, east, west, up, down, look, help
     Send a command to begin your adventure!

You: look
Bot: 📍 Town Square
     A bustling central plaza.
     🚪 Exits: north, east

You: north
Bot: Command queued. Waiting for next tick...
     [After tick: 🚶 You moved north to North Street]
```

## Configuration

### Environment Variables

- `BOT_NUMBER`: Signal phone number (required, format: `+15551234567`)
- `SIGNAL_API_URL`: signal-cli-rest-api endpoint (default: `http://localhost:8080`)
- `SPACETIME_URL`: SpacetimeDB server endpoint (default: `http://localhost:3000`)

### Logging

Set log level via `RUST_LOG` environment variable:

```bash
export RUST_LOG=info          # Standard logging
export RUST_LOG=debug         # Detailed logging
export RUST_LOG=signal_bot=trace  # Maximum detail for bot
```

## Architecture Details

### Session Management

- Each Signal phone number gets a unique session in SpacetimeDB
- Sessions are cached in-memory for faster lookup
- Session interface type is `"signal"`

### Command Processing

1. **Instant Commands** (local processing):
   - help, look, status, inventory
   - Respond immediately without SpacetimeDB tick

2. **Queued Commands** (tick processing):
   - Movement: north, south, east, west, up, down
   - Submitted to SpacetimeDB via `submit_command` reducer
   - Processed during next tick (every 3 seconds)
   - Response sent after tick execution

### Message Flow

```
Signal User → signal-cli-rest-api → POST /signal/webhook
                                     ↓
                                  webhook.rs parses payload
                                     ↓
                                  message_handler.rs processes command
                                     ↓
                                  bot.rs calls SpacetimeDB reducers
                                     ↓
                                  bot.rs sends response via Signal API
                                     ↓
                                  signal-cli-rest-api → Signal User
```

## Troubleshooting

### Bot not receiving messages

1. Check signal-cli-rest-api is running: `curl http://localhost:8080/v1/health`
2. Check webhook server is running: `curl http://localhost:3001/health`
3. Verify webhook is configured in signal-cli-rest-api
4. Check logs for incoming webhook calls: `RUST_LOG=debug cargo run...`

### Messages not being sent

1. Verify bot number is registered: `curl http://localhost:8080/v1/accounts`
2. Test sending directly via API:
   ```bash
   curl -X POST http://localhost:8080/v2/send \
     -H "Content-Type: application/json" \
     -d '{
       "number": "+15551234567",
       "recipients": ["+15559999999"],
       "message": "Test"
     }'
   ```
3. Check Signal API logs for errors

### SpacetimeDB connection issues

1. Verify SpacetimeDB is running: `curl http://localhost:3000/database/text-game/call`
2. Check database exists: `spacetime ls`
3. Verify module is published: `spacetime logs text-game`

### Authentication failures

1. Check session table: `spacetime sql text-game "SELECT * FROM session"`
2. Verify player table: `spacetime sql text-game "SELECT * FROM player"`
3. Check logs for authentication errors

## Security Considerations

1. **Phone Number Privacy**: Bot's phone number will be visible to players
2. **Rate Limiting**: Currently no rate limiting implemented
3. **Authentication**: Phone number based - no additional auth required
4. **HTTPS**: Use HTTPS for production webhook endpoint
5. **Environment Variables**: Never commit BOT_NUMBER or credentials

## Production Deployment

For production use:

1. Use a dedicated server with static IP
2. Configure HTTPS for webhook endpoint
3. Use systemd or similar to manage the bot process
4. Set up monitoring and alerts
5. Configure signal-cli-rest-api for production mode
6. Consider rate limiting and abuse prevention
7. Regular backups of signal-cli configuration

## Next Steps

- Implement subscription mechanism for command results
- Add support for group chats
- Add rich formatting (bold, italic, etc.)
- Implement player-to-player messaging
- Add rate limiting per phone number
- Create admin commands for moderation
