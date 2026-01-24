[← Back to Main README](../../README.md)

---

# Signal Integration Plan

## Overview

Add Signal Messenger support to the text adventure game, allowing players to interact via Signal messages. This follows the same architecture as the terminal client but uses Signal as the interface.

## Architecture

```
Signal Messenger
    ↓
Signal Bot (Rust)
    ↓
SpacetimeDB Reducers (connect_session, authenticate_player, submit_command, execute_tick)
    ↓
SpacetimeDB Tables (player, room, session, queued_command, command_log)
```

## Implementation Options

### Option 1: signal-cli-rest-api (Recommended)
Use signal-cli with REST API wrapper for easier integration.

**Pros:**
- HTTP API (similar to our HTTP terminal client)
- Well-tested and stable
- Supports receiving messages via webhook
- Docker container available

**Cons:**
- Requires running signal-cli daemon
- Extra dependency (Java-based signal-cli)

### Option 2: presage (Native Rust)
Pure Rust Signal protocol implementation.

**Pros:**
- Native Rust, no Java dependency
- Direct protocol implementation
- Better performance

**Cons:**
- Less mature than signal-cli
- More complex setup
- Still developing

### Option 3: signal-cli wrapper
Direct wrapper around signal-cli command-line tool.

**Pros:**
- Simple to start with
- Can poll for messages

**Cons:**
- Inefficient (polling)
- Requires signal-cli installed
- No real-time message delivery

## Recommended Approach: signal-cli-rest-api

### Setup Steps

1. **Install signal-cli-rest-api**
   ```bash
   # Via Docker
   docker run -d --name signal-api \
     -p 8080:8080 \
     -v signal-data:/home/.local/share/signal-cli \
     bbernhard/signal-cli-rest-api
   ```

2. **Register Signal Number**
   ```bash
   # Register phone number
   curl -X POST http://localhost:8080/v1/register/+1234567890
   
   # Verify with code received via SMS
   curl -X POST http://localhost:8080/v1/register/+1234567890/verify/123456
   ```

3. **Configure Webhook**
   Point signal-cli-rest-api to our Rust bot's webhook endpoint.

### Rust Bot Architecture

```
src/
  signal_client/
    mod.rs              - Main module
    bot.rs              - Signal bot logic
    webhook.rs          - Webhook receiver (actix-web/axum)
    message_handler.rs  - Process incoming messages
    formatter.rs        - Format game output for Signal
```

## Implementation Plan

### Phase 1: Basic Connection (2-4 hours)

1. **Create signal_client module**
   ```rust
   // src/signal_client/mod.rs
   pub mod bot;
   pub mod webhook;
   pub mod message_handler;
   pub mod formatter;
   ```

2. **Set up webhook server**
   - Use actix-web or axum
   - Receive POST requests from signal-cli-rest-api
   - Parse incoming message JSON

3. **Connect to SpacetimeDB**
   - Use existing HTTP client or SDK
   - Call `connect_session("signal", phone_number)`
   - Store session mapping (phone → session_id)

### Phase 2: Message Processing (2-3 hours)

4. **Implement message handler**
   - Parse Signal message text
   - Map sender phone → session
   - Call `submit_command(session_id, message_text)`

5. **Authentication flow**
   - First message from new number → authenticate_player
   - Prompt for player name if needed
   - Link phone number to player account

### Phase 3: Response Formatting (2-3 hours)

6. **Format output for Signal**
   - Convert game output to Signal-friendly text
   - Handle line breaks and formatting
   - Keep messages under Signal's limits
   - Support reactions for quick responses

7. **Implement ticker/poller**
   - Subscribe to command_log for this player
   - Send game responses back via Signal API
   - Handle async tick execution results

### Phase 4: Testing & Polish (2-3 hours)

8. **Test complete flow**
   - Send commands via Signal
   - Verify SpacetimeDB updates
   - Check responses arrive

9. **Add error handling**
   - Invalid commands
   - Network failures
   - Session timeouts

10. **Documentation**
    - Setup instructions
    - User guide (how to play via Signal)

## Code Structure

### Main Bot Entry Point

```rust
// src/bin/signal_bot.rs
use rust_game_test::signal_client::SignalBot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let bot = SignalBot::new(
        "http://localhost:8080",      // signal-cli-rest-api
        "ws://localhost:3000",         // SpacetimeDB
        "+1234567890",                 // Bot's Signal number
    ).await?;
    
    bot.run().await?;
    Ok(())
}
```

### Webhook Handler

```rust
// src/signal_client/webhook.rs
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;

#[derive(Deserialize)]
struct SignalMessage {
    envelope: MessageEnvelope,
}

#[derive(Deserialize)]
struct MessageEnvelope {
    source: String,           // Phone number
    #[serde(rename = "dataMessage")]
    data_message: DataMessage,
}

#[derive(Deserialize)]
struct DataMessage {
    message: String,          // Text content
    timestamp: u64,
}

async fn handle_webhook(
    msg: web::Json<SignalMessage>,
    bot: web::Data<SignalBot>,
) -> HttpResponse {
    let sender = &msg.envelope.source;
    let text = &msg.envelope.data_message.message;
    
    // Process message
    bot.handle_message(sender, text).await;
    
    HttpResponse::Ok().finish()
}

pub async fn start_webhook_server(bot: SignalBot) -> Result<(), std::io::Error> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(bot.clone()))
            .route("/signal/webhook", web::post().to(handle_webhook))
    })
    .bind("0.0.0.0:3001")?
    .run()
    .await
}
```

### Message Handler

```rust
// src/signal_client/message_handler.rs
impl SignalBot {
    pub async fn handle_message(&self, sender: &str, text: &str) {
        // Get or create session
        let session_id = self.get_or_create_session(sender).await;
        
        // Submit command to SpacetimeDB
        self.spacetime_client
            .submit_command(session_id, text)
            .await;
        
        // Tick will process asynchronously
        // Response will come via subscription callback
    }
    
    async fn get_or_create_session(&self, phone: &str) -> u64 {
        // Check cache
        if let Some(session_id) = self.session_cache.get(phone) {
            return *session_id;
        }
        
        // Create new session
        self.spacetime_client
            .connect_session("signal", phone)
            .await;
        
        // Store in cache
        // Return session_id
    }
}
```

### Response Sender

```rust
// src/signal_client/bot.rs
impl SignalBot {
    pub async fn send_message(&self, recipient: &str, text: &str) {
        let url = format!("{}/v2/send", self.signal_api_url);
        
        let payload = json!({
            "number": self.bot_number,
            "recipients": [recipient],
            "message": text,
        });
        
        self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .expect("Failed to send Signal message");
    }
    
    // Subscribe to command results
    pub async fn watch_responses(&self) {
        // Subscribe to command_log where player matches session
        // When new result arrives, send via Signal
    }
}
```

## Configuration

### Cargo.toml

```toml
[dependencies]
# Existing dependencies...
actix-web = { version = "4", optional = true }
# or
axum = { version = "0.7", optional = true }

[features]
signal-client = ["actix-web", "tokio", "reqwest"]
# or
signal-client = ["axum", "tokio", "reqwest"]

[[bin]]
name = "signal_bot"
path = "src/bin/signal_bot.rs"
required-features = ["signal-client"]
```

## User Experience

### Playing via Signal

1. **First Contact**
   ```
   User: "hello"
   Bot: "Welcome to the Layered Realms! What's your character name?"
   User: "Aragorn"
   Bot: "Welcome, Aragorn! You are in Town Square. A bustling central plaza..."
   ```

2. **Commands**
   ```
   User: "north"
   Bot: "Moving north... (waiting for tick)"
   [After tick executes]
   Bot: "You moved north. You are now in Market Street. Vendors line the busy road..."
   ```

3. **Multiple Commands**
   ```
   User: "look"
   Bot: "Market Street. Vendors line the busy road..."
   User: "inventory"
   Bot: "You are carrying: ..."
   ```

## Security Considerations

1. **Authentication**
   - Phone number verification via Signal registration
   - Optional: PIN/password for account recovery
   - Rate limiting per phone number

2. **Privacy**
   - Don't log phone numbers
   - Use hashed identifiers for internal tracking
   - Clear data on request (GDPR)

3. **Abuse Prevention**
   - Rate limit commands per session
   - Block spam patterns
   - Admin controls for banning

## Testing Strategy

1. **Unit Tests**
   - Message parsing
   - Command routing
   - Response formatting

2. **Integration Tests**
   - Mock signal-cli-rest-api
   - Test full message flow
   - Verify SpacetimeDB updates

3. **Manual Testing**
   - Send real Signal messages
   - Test on multiple phones
   - Verify tick timing

## Deployment

### Development
```bash
# Terminal 1: Start SpacetimeDB
spacetime start

# Terminal 2: Start signal-cli-rest-api
docker run -d -p 8080:8080 -v signal-data:/home/.local/share/signal-cli bbernhard/signal-cli-rest-api

# Terminal 3: Run Signal bot
cargo run --bin signal_bot --features signal-client
```

### Production
- Docker Compose with all services
- Persistent volumes for signal-cli data
- Environment variables for configuration
- Health checks and auto-restart

## Timeline Estimate

- **Setup & Basic Connection:** 2-4 hours
- **Message Processing:** 2-3 hours
- **Response Formatting:** 2-3 hours
- **Testing & Polish:** 2-3 hours

**Total:** 8-13 hours for MVP

## Next Steps

1. Install signal-cli-rest-api
2. Register a Signal number for the bot
3. Create `src/signal_client/` module
4. Implement webhook receiver
5. Connect to SpacetimeDB
6. Test end-to-end flow
7. Add response polling/subscription
8. Polish and document

---

**Status:** Ready to implement
**Dependencies:** SpacetimeDB running, signal-cli-rest-api, phone number for bot
**Blocked by:** None

---

[← Back to Main README](../../README.md)
