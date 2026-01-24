# Discord Bot Setup Guide

This guide covers setting up the Discord bot interface for the text adventure game.

## Overview

The Discord bot allows players to interact with the game via Discord messages. The architecture is:

```
Discord Messages → Discord Bot → SpacetimeDB
```

## Prerequisites

1. **SpacetimeDB running** on localhost:3000
2. **Discord Bot Token** from Discord Developer Portal

## Step 1: Create a Discord Bot

### 1. Go to Discord Developer Portal

Visit [https://discord.com/developers/applications](https://discord.com/developers/applications)

### 2. Create New Application

- Click "New Application"
- Give it a name (e.g., "Text Adventure Bot")
- Accept the terms and create

### 3. Create Bot User

- Go to the "Bot" tab in the left sidebar
- Click "Add Bot"
- Confirm by clicking "Yes, do it!"

### 4. Get Bot Token

- Under the "Bot" section, click "Reset Token"
- Copy the token (you'll need this later)
- **Keep this token secret!** Don't share it or commit it to git

### 5. Configure Bot Permissions

Under the "Bot" tab:
- Enable "MESSAGE CONTENT INTENT" (required to read message content)
- Under "Privileged Gateway Intents", enable:
  - Message Content Intent ✅

### 6. Invite Bot to Your Server

1. Go to "OAuth2" → "URL Generator"
2. Under "SCOPES", select:
   - `bot`
3. Under "BOT PERMISSIONS", select:
   - Send Messages
   - Read Messages/View Channels
   - Read Message History
4. Copy the generated URL at the bottom
5. Open the URL in your browser and select a server to add the bot to

## Step 2: Build and Run the Bot

### Set Environment Variables

```bash
# Discord bot token (required)
export DISCORD_TOKEN="your-bot-token-here"

# SpacetimeDB URL (optional, defaults to localhost:3000)
export SPACETIME_URL="http://localhost:3000"
```

### Build the Bot

```bash
cargo build --features discord-client --bin discord_bot
```

### Run the Bot

```bash
cargo run --features discord-client --bin discord_bot
```

The bot will:
- Connect to Discord with your bot token
- Listen for messages mentioning the bot or in DMs
- Execute ticks every 3 seconds to process commands
- Send responses via Discord messages

## Step 3: Using the Bot

### Authentication

Authenticate in any channel or DM:

**In a server channel:**
```
@YourBot auth PlayerName
```

**In DMs:**
```
auth PlayerName
```

✨ **After authenticating in a channel, the bot will send you a DM.** All gameplay happens in DMs to keep public channels clean.

### Game Commands

📬 **Game commands only work in DMs:**

**Movement:**
- `north` (or `n`, `s`, `e`, `w`, `u`, `d`)

**Info:**
- `look` - Look around current room
- `status` - Show your status
- `help` - Show available commands

**Example DM session:**
```
You: auth Alice
Bot: ✅ Welcome Alice! ...

You: look
Bot: 🏛️ Center Room
You are in a central chamber...

You: north
Bot: ✅ Command queued: north
```

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DISCORD_TOKEN` | Yes | - | Your Discord bot token |
| `SPACETIME_URL` | No | `http://localhost:3000` | SpacetimeDB server URL |

### Tick Rate

The bot processes commands every 3 seconds. This matches the SpacetimeDB tick rate for the game.

To change the tick rate, edit `src/bin/discord_bot.rs`:

```rust
let mut tick_interval = interval(Duration::from_secs(3)); // Change 3 to your desired seconds
```

## Testing

### Manual Testing

1. Ensure SpacetimeDB is running:
   ```bash
   spacetime start
   ```

2. Start the Discord bot:
   ```bash
   cargo run --features discord-client --bin discord_bot
   ```

3. In Discord, message your bot:
   ```
   @YourBot auth TestPlayer
   @YourBot help
   @YourBot look
   @YourBot north
   ```

### Check Logs

The bot logs all activity. Look for:
- ✅ "Discord bot connected as..."
- ✅ "Bot is ready to receive commands!"
- User authentication messages
- Command submissions
- Tick executions

## Troubleshooting

### Bot doesn't respond

1. **Check intents:** Make sure "Message Content Intent" is enabled in Discord Developer Portal
2. **Check token:** Ensure `DISCORD_TOKEN` is set correctly
3. **Check bot permissions:** Bot needs "Send Messages" and "Read Messages" permissions in the channel
4. **Check logs:** Look for error messages in the bot's console output

### "Authentication failed"

- Ensure SpacetimeDB is running on the correct URL
- Check SpacetimeDB logs for errors
- Verify the `text-game` database is published

### Commands not executing

- Commands are tick-based and process every 3 seconds
- Check if you're authenticated (run `@YourBot status`)
- Ensure you're using valid commands (run `@YourBot help`)

## Production Deployment

### Security Best Practices

1. **Never commit your bot token** to git
2. Use environment variables or secret management
3. Run bot as a non-root user
4. Keep dependencies updated

### Running as a Service

#### systemd (Linux)

Create `/etc/systemd/system/discord-bot.service`:

```ini
[Unit]
Description=Text Adventure Discord Bot
After=network.target

[Service]
Type=simple
User=gamebot
WorkingDirectory=/opt/text-adventure
Environment="DISCORD_TOKEN=your-token-here"
Environment="SPACETIME_URL=http://localhost:3000"
ExecStart=/opt/text-adventure/target/release/discord_bot
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable discord-bot
sudo systemctl start discord-bot
sudo systemctl status discord-bot
```

### Docker

```dockerfile
FROM rust:1.76 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features discord-client --bin discord_bot

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/discord_bot /usr/local/bin/
ENV SPACETIME_URL=http://spacetimedb:3000
CMD ["discord_bot"]
```

## Architecture

### Message Flow

1. User sends message in Discord
2. Discord API → Bot receives message via gateway
3. Bot checks authentication status
4. Bot processes command or handles auth
5. Bot submits to SpacetimeDB
6. Bot sends response back to Discord

### Tick Processing

The bot runs a background task that executes ticks every 3 seconds:
- Calls SpacetimeDB's `execute_tick` reducer
- Processes all queued commands
- Results are cached in SpacetimeDB

### Caching

The bot caches:
- Session IDs (Discord user ID → session_id)
- Player names (Discord user ID → player name)

This reduces API calls to SpacetimeDB.

## Next Steps

- Add support for server-specific prefixes
- Implement command cooldowns
- Add rich embeds for room descriptions
- Add reaction-based navigation
- Implement admin commands

## Resources

- [Discord Developer Portal](https://discord.com/developers/applications)
- [Serenity Documentation](https://docs.rs/serenity/)
- [SpacetimeDB Documentation](https://spacetimedb.com/docs)
