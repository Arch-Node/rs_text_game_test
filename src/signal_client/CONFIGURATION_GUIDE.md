# Configuration Guide

## Overview

The Signal bot supports multiple configuration methods:

1. **Environment Variables** (required, simplest)
2. **TOML Configuration File** (optional, advanced)
3. **Database Configuration** (optional, persistent)

## Method 1: Environment Variables (Recommended for MVP)

**Current approach - works great for getting started:**

```bash
export BOT_NUMBER="+15551234567"
export SIGNAL_API_URL="http://localhost:8080"
export SPACETIME_URL="http://localhost:3000"
```

**Pros:**
- Simple and straightforward
- Works immediately
- Standard deployment practice
- Secure (not in version control)

**Cons:**
- Bot restart required to change settings
- No per-user rate limiting
- No persistent preferences

## Method 2: TOML Configuration File (Optional)

**For advanced configuration:**

```bash
# Copy example
cp bot_config.toml.example bot_config.toml

# Edit your settings
vim bot_config.toml

# Run bot (will load bot_config.toml if it exists)
cargo run --features signal-client --bin signal_bot
```

**When to use:**
- You want version-controlled bot settings
- You need advanced features (rate limiting, custom messages)
- Running multiple bot instances with different configs
- You want detailed control over behavior

**Pros:**
- More configuration options
- Can be version controlled (except secrets)
- Easy to manage multiple environments (dev/staging/prod)

**Cons:**
- Requires implementation (not currently supported)
- Still need environment variables for secrets
- More complex deployment

## Method 3: Database Configuration (Future)

**For production deployments with multiple bots:**

This would store bot configuration in SpacetimeDB tables:
- `bot_config` - Bot settings and metadata
- `bot_rate_limit` - Per-user message tracking
- `player_preferences` - User-specific settings

**When to use:**
- Running multiple bot instances
- Need persistent rate limiting across restarts
- Want admin dashboard to manage bots
- Need user preferences (emoji on/off, language, etc.)

**Implementation:**
See [bot_config_addition.rs](../text_game_stdb/bot_config_addition.rs) for table definitions.

## Current Status

✅ **Method 1 (Env Vars)** - Fully implemented and working
⚠️ **Method 2 (TOML)** - Example file provided, not implemented
⚠️ **Method 3 (Database)** - Schema designed, not implemented

## Recommendation: Start with Environment Variables

For your current Phase 1 implementation, **environment variables are perfect**. They're:
- Simple to use
- Secure
- Standard practice
- Sufficient for single bot instance

**When to add TOML or database config:**
- When you need rate limiting (Phase 2)
- When running multiple bot instances
- When you want user preferences
- When you need hot-reload of settings

## Adding Database Configuration (Phase 2+)

If you want to add database-backed configuration:

### 1. Add tables to SpacetimeDB module

```bash
cd text_game_stdb

# Copy the table definitions from bot_config_addition.rs
# Add them to src/lib.rs

# Republish module
spacetime publish text-game
```

### 2. Regenerate SDK bindings

```bash
spacetime generate --lang rust \
  --out-dir ../rs_text_game_test/src/spacetimedb_client \
  --project-path .
```

### 3. Update bot to use database config

```rust
// In bot.rs, on startup:
async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
    // Register bot in database
    self.register_bot().await?;
    
    // Load rate limits
    self.load_rate_limits().await?;
    
    Ok(())
}
```

## Configuration Priority

If multiple methods are implemented, priority would be:

1. **Environment variables** (highest - deployment specific)
2. **Database configuration** (medium - runtime changes)
3. **TOML file** (lowest - defaults)

Example:
```
BOT_NUMBER env var → overrides → bot_config.toml value → overrides → database default
```

## Security Best Practices

**Never commit:**
- BOT_NUMBER or phone numbers
- API keys or tokens
- bot_config.toml with secrets

**Do commit:**
- bot_config.toml.example
- Default values and structure
- Documentation

**Use .gitignore:**
```
bot_config.toml
.env
```

## Example Deployment Scenarios

### Development (Local)
```bash
export BOT_NUMBER="+15551234567"
cargo run --features signal-client --bin signal_bot
```

### Staging (Docker)
```dockerfile
FROM rust:latest
ENV BOT_NUMBER="+15551234567"
ENV SIGNAL_API_URL="http://signal-api:8080"
CMD ["./signal_bot"]
```

### Production (Kubernetes)
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: signal-bot-config
data:
  bot-number: <base64-encoded-phone>

---
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
      - name: signal-bot
        env:
        - name: BOT_NUMBER
          valueFrom:
            secretKeyRef:
              name: signal-bot-config
              key: bot-number
```

## Summary

**For now (Phase 1):** Stick with environment variables. They're simple, secure, and sufficient.

**For later (Phase 2+):** Add database configuration when you need:
- Rate limiting
- User preferences
- Multi-bot coordination
- Hot configuration reloads

**Optional (anytime):** Add TOML configuration if you want more advanced settings without modifying code.
