# Multiplayer Skills-Heavy Text Game - Development Roadmap

## Overview
Transform the single-player terminal game into a multiplayer, skills-focused text adventure accessible via Signal, terminal, and SMS interfaces.

## Design Philosophy
- **Skills-heavy gameplay** - Most challenges solvable through skill checks rather than combat
- **Light combat** - Quick, avoidable through stealth/persuasion, skill-based resolution
- **Multiple solution paths** - Different skills provide different approaches
- **Multiplayer collaboration** - Players can cooperate using complementary skills
- **Multi-platform** - Play via Signal messenger, terminal client, or text messages

---

## Development Tasks

### Phase 1: Core Infrastructure

#### ✅ Task 0: Project Planning
- [x] Create comprehensive roadmap
- [x] Document design philosophy
- [x] Break down implementation into phases

#### ⬜ Task 1: Design Multiplayer Architecture

**Overview:** Define the complete architectural design for transitioning from single-player to multiplayer, including state management, communication protocols, session handling, and synchronization strategies.

##### 1.1 Analyze Current Single-Player Architecture
- [ ] Document current `GameState` structure and lifecycle
- [ ] Map all current commands and their state mutations
- [ ] Identify which parts of `World` are static vs dynamic
- [ ] Review current `Player` struct and what needs to become per-session
- [ ] List all current side effects (room changes, item pickups, etc.)
- [ ] Document current output/messaging flow

##### 1.2 Define Multiplayer State Model
- [ ] **Shared World State:**
  - [ ] Design `WorldState` struct containing rooms, NPCs, items, global events
  - [ ] Identify mutable shared state (room occupancy, NPC positions, world items)
  - [ ] Determine which state changes are transactional vs eventual
  - [ ] Plan for world state versioning/snapshots
- [ ] **Per-Player State:**
  - [ ] Define `PlayerState` struct (position, inventory, stats, skills, quests)
  - [ ] Separate player-specific views from shared world view
  - [ ] Design player visibility rules (what players can see of each other)
- [ ] **Session Management:**
  - [ ] Design `Session` struct linking connection to player identity
  - [ ] Plan session lifecycle (create, authenticate, resume, timeout, close)
  - [ ] Decide on session storage (in-memory, Redis, database)
  - [ ] Define session metadata (connection time, last activity, interface type)

##### 1.3 Design Command/Message Protocol
- [ ] **Inbound Command Envelope:**
  - [ ] Define `CommandRequest` struct:
    ```rust
    struct CommandRequest {
        session_id: SessionId,
        player_id: PlayerId,
        command: String,
        interface: InterfaceType,  // Terminal, Signal, SMS
        timestamp: DateTime,
        metadata: HashMap<String, String>
    }
    ```
  - [ ] Design command validation and sanitization
  - [ ] Plan rate limiting per session/player
  - [ ] Define command priority levels (immediate vs queued)
- [ ] **Outbound Response Envelope:**
  - [ ] Define `CommandResponse` struct:
    ```rust
    struct CommandResponse {
        session_id: SessionId,
        messages: Vec<Message>,
        events: Vec<GameEvent>,
        errors: Option<Vec<Error>>,
        state_delta: Option<StateDelta>
    }
    ```
  - [ ] Design message types (narrative, system, chat, combat, error)
  - [ ] Define formatting hints for different interfaces
  - [ ] Plan message batching and chunking strategies
- [ ] **Event Broadcasting:**
  - [ ] Define `GameEvent` enum (player_moved, player_joined, player_left, item_taken, npc_dialogue, world_update)
  - [ ] Design event filtering (who needs to see what)
  - [ ] Plan event delivery guarantees (best-effort vs guaranteed)
  - [ ] Design event subscription system (room-based, proximity-based, global)

##### 1.4 Architecture Pattern Selection
- [ ] **Choose Core Pattern:**
  - [ ] Option A: Actor model (one actor per player + world actor)
  - [ ] Option B: Event-driven (command → event → state update → broadcast)
  - [ ] Option C: ECS (Entity Component System) for game objects
  - [ ] Document pros/cons of chosen pattern
- [ ] **Concurrency Strategy:**
  - [ ] Decide on locking strategy (RwLock on world state, per-player locks)
  - [ ] Plan for lock-free data structures where possible
  - [ ] Consider message passing vs shared memory
  - [ ] Design deadlock prevention strategy
- [ ] **Turn-Based vs Real-Time:**
  - [ ] Decision: Hybrid approach (continuous for exploration, turn-based for combat/challenges)
  - [ ] Define tick rate for world updates (1 second, 5 seconds, 10 seconds?)
  - [ ] Plan command queuing and execution order
  - [ ] Design action interruption and priority system

##### 1.5 Session Abstraction Layer
- [ ] **Define Session Interface:**
  ```rust
  trait SessionInterface {
      fn send_message(&self, msg: Message) -> Result<()>;
      fn send_prompt(&self) -> Result<()>;
      fn get_interface_type(&self) -> InterfaceType;
      fn get_format_limits(&self) -> FormatLimits;  // SMS char limit, etc.
  }
  ```
- [ ] **Map Interface Types:**
  - [ ] Terminal: full formatting, color, real-time
  - [ ] Signal: moderate length, async, notification support
  - [ ] SMS: strict char limits, highest latency, most concise
- [ ] **Account vs Character Mapping:**
  - [ ] Decision: One account can have multiple characters
  - [ ] Design character selection flow per interface
  - [ ] Plan character switching without disconnecting
  - [ ] Handle multiple sessions for same account (different devices)

##### 1.6 State Synchronization Strategy
- [ ] **Visibility and Awareness:**
  - [ ] Define "location awareness" (players in same room see each other)
  - [ ] Design proximity-based event filtering
  - [ ] Plan for "global" events (server announcements, world events)
  - [ ] Handle delayed sync for SMS users (summary-based updates)
- [ ] **State Change Propagation:**
  - [ ] Design delta-based updates (only send what changed)
  - [ ] Plan full state refresh scenarios (reconnect, teleport)
  - [ ] Handle optimistic updates with rollback
  - [ ] Design conflict resolution (two players take same item)
- [ ] **Update Cadence:**
  - [ ] World tick: Every 5-10 seconds for passive events
  - [ ] Immediate: Player commands affecting others
  - [ ] Batched: Periodic summaries for SMS interface
  - [ ] Event-driven: Combat, dialogue, skill checks

##### 1.7 Authentication and Identity
- [ ] **Basic Auth Strategy:**
  - [ ] Terminal: username/password or token-based
  - [ ] Signal: phone number as identity (pre-registered)
  - [ ] SMS: phone number with PIN or initial registration flow
- [ ] **Security Considerations:**
  - [ ] Plan password hashing (argon2, bcrypt)
  - [ ] Design token/session key generation
  - [ ] Plan for session hijacking prevention
  - [ ] Define admin/moderator authentication
- [ ] **Account System:**
  - [ ] Design user registration flow per interface
  - [ ] Plan account recovery mechanisms
  - [ ] Handle guest/anonymous sessions (if supported)

##### 1.8 Error Handling and Resilience
- [ ] **Error Categories:**
  - [ ] Command parsing errors
  - [ ] Authentication/authorization errors
  - [ ] State mutation errors (invalid action, race condition)
  - [ ] Network/interface errors
  - [ ] Server/world errors
- [ ] **Graceful Degradation:**
  - [ ] Handle partial service outages (SMS down, database slow)
  - [ ] Plan for read-only mode during maintenance
  - [ ] Design queue backpressure handling
  - [ ] Define timeout policies per interface

##### 1.9 Logging and Observability
- [ ] **Structured Logging:**
  - [ ] Define log levels: TRACE, DEBUG, INFO, WARN, ERROR
  - [ ] Required fields: timestamp, session_id, player_id, command, duration
  - [ ] Optional fields: interface_type, room_id, error_details
- [ ] **Metrics to Track:**
  - [ ] Active sessions per interface
  - [ ] Commands per second
  - [ ] Average command execution time
  - [ ] State synchronization lag
  - [ ] Error rates by category
- [ ] **Tracing:**
  - [ ] Plan distributed tracing for async flows
  - [ ] Design request correlation IDs
  - [ ] Trace command from receipt → execution → broadcast

##### 1.10 Create Architecture Documentation
- [ ] Write architecture decision records (ADRs) for key choices
- [ ] Create sequence diagrams for:
  - [ ] Player connection and authentication
  - [ ] Command processing flow
  - [ ] Event broadcasting
  - [ ] State synchronization
- [ ] Document data flow diagrams
- [ ] Create component interaction diagrams
- [ ] Write API specifications for internal modules
- [ ] Define coding standards and patterns to follow

#### ⬜ Task 2: Add Networking Dependencies
- [ ] Add `tokio` for async runtime
- [ ] Add Signal protocol support (libsignal-service-rs or signal-cli wrapper)
- [ ] Add websocket/TCP libraries for terminal interface
- [ ] Add SMS gateway support (twilio-rs or similar)
- [ ] Define logging levels and structured fields (session, player, command)
- [ ] Choose logging crates and error handling patterns
- [ ] Plan for tracing async flows and server diagnostics
- [ ] Update Cargo.toml with all necessary crates

#### ⬜ Task 3: Create Player Entity System
- [ ] Expand Player model with name/id
- [ ] Add inventory system reference
- [ ] Add stats (health, mana, stamina)
- [ ] Add skill levels
- [ ] Add experience points system
- [ ] Add equipped items
- [ ] Create PlayerManager to track all connected players

#### ⬜ Task 4: Add Persistence Layer
- [ ] Choose database (SQLite for simple, PostgreSQL for production)
- [ ] Design schema for player data
- [ ] Implement world state saving
- [ ] Add inventory/skills/progress persistence
- [ ] Enable server restart recovery
- [ ] Add save/load operations

#### ⬜ Task 5: Create Server Module
- [ ] Build game server core
- [ ] Implement world state management
- [ ] Add player connection handling
- [ ] Create command routing system
- [ ] Implement state persistence
- [ ] Add event broadcasting to all clients
- [ ] Create tick system for world updates

---

### Phase 2: Skills & Character Systems

#### ⬜ Task 6: Design Skills System
- [ ] Create skill categories (exploration, crafting, social, magic, survival, etc.)
- [ ] Implement skill levels and experience
- [ ] Add skill check mechanics
- [ ] Distinguish passive vs active skills
- [ ] Design skill-based progression
- [ ] Make skills the primary gameplay mechanic
- [ ] Create skill tree structure

#### ⬜ Task 7: Implement Character Classes
- [ ] Create Scholar class (research/knowledge focus)
- [ ] Create Diplomat class (social/persuasion focus)
- [ ] Create Artificer class (crafting/invention focus)
- [ ] Create Scout class (exploration/survival focus)
- [ ] Create Mystic class (magical skills focus)
- [ ] Define unique skill bonuses per class
- [ ] Balance class abilities

#### ⬜ Task 8: Add Skill Training Mechanics
- [ ] Implement learn-by-doing system (XP from skill use)
- [ ] Create skill trainers (NPCs that teach)
- [ ] Add skill books/scrolls
- [ ] Create practice activities
- [ ] Design skill trees with prerequisites
- [ ] Balance skill progression rates

---

### Phase 3: Game Content Systems

#### ⬜ Task 9: Create Inventory and Item System
- [ ] Design Item model with types (consumables, tools, equipment, quest items)
- [ ] Add weight/encumbrance system
- [ ] Implement durability mechanics
- [ ] Add quality levels
- [ ] Create inventory management with capacity limits
- [ ] Add item usage mechanics
- [ ] Implement equipping/unequipping system
- [ ] Build item creation/builder functions for procedural generation

#### ⬜ Task 10: Add NPC System
- [ ] Create NPC models
- [ ] Build NPC creation/builder functions for procedural generation
- [ ] Implement dialogue trees
- [ ] Add skill-based interaction checks (persuasion, intimidation, deception)
- [ ] Create trading system
- [ ] Add quest giver functionality
- [ ] Implement reputation tracking per NPC/faction

#### ⬜ Task 11: Create Quest System
- [ ] Design quest framework
- [ ] Create skill-based quest objectives
- [ ] Implement multiple solution paths
- [ ] Add quest tracking
- [ ] Design rewards system (skill XP, items, reputation)
- [ ] Create quest chains
- [ ] Add dynamic quest generation

#### ⬜ Task 12: Add Crafting System
- [ ] Design recipe system requiring specific skill levels
- [ ] Implement resource gathering
- [ ] Create item creation mechanics
- [ ] Add quality based on skill level
- [ ] Enable experimentation/discovery of new recipes
- [ ] Balance crafting progression

#### ⬜ Task 13: Implement Puzzle and Skill Challenges
- [ ] Create lockpicking (requires dexterity skill)
- [ ] Add research puzzles (requires knowledge skills)
- [ ] Implement social challenges (persuasion/deception)
- [ ] Create environmental puzzles (perception/investigation)
- [ ] Balance challenge difficulty

#### ⬜ Task 14: Implement Time and World Events
- [ ] Add day/night cycle
- [ ] Create weather effects that impact skills
  - Rain affects tracking
  - Darkness affects perception
- [ ] Add scheduled events
- [ ] Implement dynamic world changes based on player actions

---

### Phase 4: Combat (Lightweight)

#### ⬜ Task 15: Implement Combat System (Light)
- [ ] Design initiative based on perception skill
- [ ] Create attack/defense rolls using weapon/armor skills
- [ ] Add option to avoid combat through stealth/persuasion skills
- [ ] Implement status effects
- [ ] Add escape mechanics
- [ ] Keep combat quick and secondary to skills
- [ ] Build monster/enemy creation functions for procedural generation

---

### Phase 5: AI-Powered Content Generation

#### ⬜ Task 16: Integrate AI for Dynamic Content
- [ ] Research local AI options (llama.cpp, mistral, etc.) vs cloud APIs (OpenAI, Anthropic, etc.)
- [ ] Add AI client dependencies to Cargo.toml
- [ ] Create AI service layer with abstraction for local/cloud switching
- [ ] Implement AI-powered item generation
  - Generate item names, descriptions, properties
  - Ensure balanced stats based on item tier/quality
  - Validate generated items for game balance
- [ ] Implement AI-powered monster/enemy generation
  - Generate creature descriptions and abilities
  - Create appropriate stat blocks
  - Assign skill-based behaviors
- [ ] Implement AI-powered NPC dialogue generation
  - Generate contextual dialogue for key NPCs
  - Create personality-consistent responses
  - Handle dynamic conversation flows
- [ ] Add prompt templates for content generation
- [ ] Implement content caching to reduce AI calls
- [ ] Add configuration for AI model selection and parameters
- [ ] Build safety filters to prevent inappropriate content
- [ ] Create fallback systems for when AI is unavailable

---

### Phase 6: Network Interfaces

#### ⬜ Task 17: Implement Terminal Interface (Refactor Existing)
- [ ] Refactor current terminal interface to connect to server
- [ ] Add connection handling
- [ ] Implement async input/output
- [ ] Add multiplayer awareness (see other players)
- [ ] Improve formatting for better readability
- [ ] Handle disconnection/reconnection

#### ⬜ Task 18: Implement Signal Interface
- [ ] Create Signal protocol client/bot
- [ ] Use libsignal or signal-cli wrapper
- [ ] Implement message parsing
- [ ] Add command execution
- [ ] Format responses appropriately
- [ ] Manage player sessions via phone numbers

#### ⬜ Task 19: Implement Text/SMS Interface
- [ ] Create text message interface using Twilio or similar
- [ ] Implement SMS parsing
- [ ] Handle response chunking (SMS length limits)
- [ ] Add session management
- [ ] Create command shortcuts for mobile

---

### Phase 7: Multiplayer Features

#### ⬜ Task 20: Create Player Interaction Commands
- [ ] Add chat commands (say/tell/whisper)
- [ ] Implement trade system with other players
- [ ] Add examine other players
- [ ] Create follow/group system
- [ ] Add skill-based interactions
  - Teach skills to other players
  - Collaborate on challenges

#### ⬜ Task 21: Create Admin/Game Master Tools
- [ ] Build admin commands for spawning items/NPCs
- [ ] Add teleporting players
- [ ] Enable world modification
- [ ] Create event triggering
- [ ] Add player management
- [ ] Implement server monitoring

---

### Phase 8: Testing & Documentation

#### ⬜ Task 22: Write Comprehensive Tests
- [ ] Add unit tests for skills system
- [ ] Test combat mechanics
- [ ] Test inventory management
- [ ] Test command parsing
- [ ] Add integration tests for multiplayer interactions
- [ ] Test server-client communication
- [ ] Test persistence layer

#### ⬜ Task 23: Create Documentation
- [ ] Document multiplayer architecture
- [ ] Write skills system guide
- [ ] Create player handbook
- [ ] Document API for different interfaces
- [ ] Write deployment guide
- [ ] Create contribution guidelines

---

## Current Status
**Phase:** Planning Complete  
**Next Steps:** Begin Phase 1 with Task 1 (Multiplayer Architecture) or Task 6 (Skills System)  
**Date:** January 23, 2026

---

## Task Summary
- **Total Tasks:** 23 (plus Task 0 completed)
- **Phase 1 (Infrastructure):** Tasks 1-5
- **Phase 2 (Skills & Classes):** Tasks 6-8
- **Phase 3 (Content Systems):** Tasks 9-14
- **Phase 4 (Combat):** Task 15
- **Phase 5 (AI Generation):** Task 16
- **Phase 6 (Network Interfaces):** Tasks 17-19
- **Phase 7 (Multiplayer):** Tasks 20-21
- **Phase 8 (Testing & Docs):** Tasks 22-23

---

## Notes
- Focus on skills as primary gameplay mechanic
- Combat should be avoidable and quick when it occurs
- Multiple solution paths encourage different character builds
- Cross-platform accessibility is key to multiplayer engagement
