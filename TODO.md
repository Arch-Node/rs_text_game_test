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

#### ✅ Task 1: Design Multiplayer Architecture

**Overview:** Define the complete architectural design for transitioning from single-player to multiplayer, including state management, communication protocols, session handling, and synchronization strategies.

**Status:** ✅ Core architecture complete with SpacetimeDB backend

##### 1.1 Analyze Current Single-Player Architecture
- [x] Document current `GameState` structure and lifecycle
- [x] Map all current commands and their state mutations
- [x] Identify which parts of `World` are static vs dynamic
- [x] Review current `Player` struct and what needs to become per-session
- [x] List all current side effects (room changes, item pickups, etc.)
- [x] Document current output/messaging flow

##### 1.2 Define Multiplayer State Model
- [x] **SpacetimeDB Backend Selected:**
  - [x] Database + server combined into one
  - [x] Built-in real-time state synchronization
  - [x] ACID transactions (no race conditions)
  - [x] ~100μs latency, 100k tx/s capacity
  - [x] Automatic persistence and scaling
- [x] **Database Schema Defined:**
  - [x] `player` table: id, name, account_id, class, position_x/y/z, dimension, status, last_action
  - [x] `room` table: id, position_x/y/z, dimension, name, description, exits_json
  - [x] `session` table: id, player_id, interface_type, connection_id, auth_state, identity
  - [x] `queued_command` table: id, player_id, session_id, command, queued_at, priority
  - [x] `command_log` table: audit trail of all commands
- [x] **Multi-Dimensional Coordinate System:**
  - [x] 3D positions (x, y, z) within each dimension
  - [x] Separate spatial planes (material, ethereal, shadow, dream)
  - [x] Dimensions can shift/interact but maintain independent coordinates
  - [x] Coordinate center at (100,100,100) to avoid negative numbers in SQL
  - [x] 6-directional movement (north/south/east/west/up/down)
- [x] **Reducers (Game Logic) Implemented:**
  - [x] `connect_session` - Create new connection
  - [x] `authenticate_player` - Link session to player
  - [x] `submit_command` - Queue command (one per player)
  - [x] `execute_tick` - Process all queued commands (includes z-axis movement)
  - [x] `get_player_info` - Query player state
  - [x] `create_room` - Insert room into database
  - [x] `get_current_room` - Query room at player's position
  - [x] `list_rooms_in_dimension` - Query all rooms in dimension
- [x] **SpacetimeDB Module Published:**
  - [x] Compiled to WASM
  - [x] Published to local server (text-game)
  - [x] Tested with CLI commands
  - [x] 10 test rooms populated
- [x] **Session Management:**
  - [x] Session struct with interface type (Terminal/Signal/SMS)
  - [x] ConnectionIdentifiers support (phone, signal ID, IP, MAC)
  - [x] SpacetimeDB Identity-based authentication
- [x] **Command Queue System:**
  - [x] Tick-based execution (15s production, 3s testing)
  - [x] One queued command per player at a time
  - [x] Instant commands bypass queue (look, inventory, status, help, say)
  - [x] Command validation before queueing
  - [x] Priority queue support
- [x] **SDK Client Bindings:**
  - [x] Generated type-safe Rust client (19 files)
  - [x] Auto-generated from SpacetimeDB module
  - [x] Includes all tables and reducers
  - [x] Demo client compiled and tested

##### 1.3 Design Command/Message Protocol
- [x] **Instant vs Queued Commands:**
  - [x] Instant: look, examine, inventory, status, help, say, tell, emote, who, score, time
  - [x] Queued: movement, actions, anything that changes world state
- [ ] **Inbound Command Envelope:**
  - [ ] Define `CommandRequest` structure for different interfaces
  - [ ] Design command validation and sanitization
  - [ ] Plan rate limiting per session/player (done: one command per player)
- [ ] **Outbound Response Envelope:**
  - [ ] Define message types (narrative, system, chat, combat, error)
  - [ ] Define formatting hints for different interfaces (SMS char limits, etc.)
  - [ ] Plan message batching and chunking strategies
- [ ] **Event Broadcasting:**
  - [ ] Design event filtering (who needs to see what)
  - [ ] Plan event delivery guarantees
  - [ ] Design event subscription system (room-based, proximity-based, global)

##### 1.4 Architecture Pattern Selection
- [x] **Core Pattern Selected:** Tick-based event-driven with command queue
- [x] **Concurrency Strategy:** SpacetimeDB handles this (ACID transactions)
- [x] **Turn-Based System:**
  - [x] 15-second ticks for production (strategic gameplay)
  - [x] 3-second ticks for testing
  - [x] Command queueing with one-per-player limit
- [ ] **Document pros/cons of chosen pattern**

##### 1.5 Session Abstraction Layer
- [x] **ConnectionIdentifiers Defined:**
  - [x] Signal ID, phone number, IP address, MAC address, device fingerprint
  - [x] Stored on Session (not Player) to support multi-device play
- [ ] **Define Session Interface Trait:**
  - [ ] send_message() with format limits per interface
  - [ ] send_prompt()
  - [ ] get_interface_type()
  - [ ] get_format_limits() (SMS: 160 chars, etc.)
- [ ] **Account vs Character Mapping:**
  - [ ] One account can have multiple characters
  - [ ] Design character selection flow per interface
  - [ ] Handle multiple sessions for same account (different devices)

##### 1.6 State Synchronization Strategy
- [x] **SpacetimeDB Handles Most Sync:**
  - [x] Built-in real-time subscriptions
  - [x] Automatic delta updates
  - [x] ACID transactions prevent conflicts
- [ ] **Define Visibility Rules:**
  - [ ] Location awareness (players in same room/dimension see each other)
  - [ ] Proximity-based event filtering
  - [ ] Cross-dimension awareness rules
  - [ ] Global events (server announcements, world events)
- [ ] **Update Cadence:**
  - [ ] World tick: Every 15 seconds for queued commands
  - [ ] Immediate: Instant commands
  - [ ] Event-driven: Combat, dialogue, skill checks

##### 1.7 Authentication and Identity
- [x] **SpacetimeDB Identity System:**
  - [x] Each session has unique Identity
  - [x] Link Identity → Session → Player
- [ ] **Interface-Specific Auth:**
  - [ ] Terminal: username/password or token
  - [ ] Signal: phone number + registration
  - [ ] SMS: phone number + PIN
- [ ] **Account System Design:**
  - [ ] User registration flow per interface
  - [ ] Account recovery mechanisms
  - [ ] Guest/anonymous sessions (if supported)

##### 1.8 Error Handling and Resilience
- [x] **Command Validation Errors:**
  - [x] Immediate feedback on invalid commands
  - [x] Failed commands logged in command_log table
- [ ] **Error Categories:**
  - [ ] Command parsing errors
  - [ ] Authentication/authorization errors
  - [ ] Network/interface errors
  - [ ] Server/world errors
- [ ] **Graceful Degradation:**
  - [ ] Handle partial service outages
  - [ ] Read-only mode during maintenance
  - [ ] Timeout policies per interface

##### 1.9 Logging and Observability
- [x] **Command Audit Trail:**
  - [x] command_log table tracks all executed commands
  - [x] Includes success/failure and error messages
- [ ] **Structured Logging:**
  - [ ] Define log levels: TRACE, DEBUG, INFO, WARN, ERROR
  - [ ] Required fields: timestamp, session_id, player_id, command, duration
- [ ] **Metrics to Track:**
  - [ ] Active sessions per interface
  - [ ] Commands per second
  - [ ] Average command execution time
  - [ ] Tick execution duration
  - [ ] Error rates by category

##### 1.10 Create Architecture Documentation
- [x] Architecture decision records for SpacetimeDB choice
- [x] SpacetimeDB integration documentation
- [x] Multiplayer state design documentation
- [ ] Sequence diagrams for:
  - [ ] Player connection and authentication
  - [ ] Command processing flow (done: in state design doc)
  - [ ] Event broadcasting
  - [ ] Tick execution
- [ ] Component interaction diagrams
- [ ] API specifications for interface modules

#### ⬜ Task 2: Add Networking Dependencies
- [ ] Add `tokio` for async runtime
- [ ] Add `spacetimedb` SDK for backend (replaces traditional database + server)
- [ ] Add Signal protocol support (libsignal-service-rs or signal-cli wrapper)
- [ ] Add websocket/TCP libraries for terminal interface (or use SpacetimeDB's built-in networking)
- [ ] Add SMS gateway support (twilio-rs or similar)
- [ ] Define logging levels and structured fields (session, player, command)
- [ ] Choose logging crates and error handling patterns
- [ ] Plan for tracing async flows and server diagnostics
- [ ] Update Cargo.toml with all necessary crates
- [ ] **Note:** SpacetimeDB handles much of the networking/state sync automatically

#### ⬜ Task 3: Create Player Entity System
- [ ] Expand Player model with name/id
- [ ] Add inventory system reference
- [ ] Add stats (health, mana, stamina)
- [ ] Add skill levels
- [ ] Add experience points system
- [ ] Add equipped items
- [ ] Create PlayerManager to track all connected players

#### ⬜ Task 4: Add Persistence Layer
- [ ] **SpacetimeDB Integration** (chosen backend)
  - [ ] Install SpacetimeDB CLI and start local instance
  - [ ] Add `spacetimedb` Rust SDK dependency to Cargo.toml
  - [ ] Define SpacetimeDB tables for game state:
    - [ ] `Player` table (id, name, account_id, class, stats, skills)
    - [ ] `Session` table (session_id, player_id, interface_type, connection_ids, auth_state)
    - [ ] `WorldState` table (room_occupancy, dynamic_items, npc_positions)
    - [ ] `CommandLog` table (audit trail of all commands)
  - [ ] Create SpacetimeDB reducers (stored procedures):
    - [ ] `authenticate_session` - Validate and create session
    - [ ] `submit_command` - Queue player command
    - [ ] `execute_tick` - Process all queued commands
    - [ ] `broadcast_event` - Send game events to relevant sessions
  - [ ] Design schema for player data with automatic sync
  - [ ] Implement world state saving with ACID guarantees
  - [ ] Add inventory/skills/progress persistence
  - [ ] Enable server restart recovery (automatic with SpacetimeDB)
  - [ ] Configure SpacetimeDB subscriptions for real-time client updates
- [ ] **Why SpacetimeDB:**
  - Built-in real-time state synchronization (no custom protocol needed)
  - ACID transactions solve race conditions (two players taking same item)
  - Game logic runs in database as WASM modules (100μs latency)
  - Automatic persistence and scaling (no Docker/K8s complexity)
  - Perfect fit for multiplayer games (used by BitCraft MMORPG)
  - Client SDKs auto-generated for Rust/TypeScript/C#
**Build SpacetimeDB Module** (instead of traditional server)
  - [ ] Define module structure with tables and reducers
  - [ ] Implement world state management as SpacetimeDB tables
  - [ ] Add player connection handling via SpacetimeDB subscriptions
  - [ ] Create command routing system as reducers
  - [ ] Implement state persistence (automatic with SpacetimeDB)
  - [ ] Add event broadcasting using SpacetimeDB's subscription system
  - [ ] Create tick system for world updates (reducer scheduled at intervals)
  - [ ] Compile module to WASM
  - [ ] Publish to local SpacetimeDB instance
  - [ ] Test real-time sync and state updates
- [ ] **Alternative: Build custom adapter layer**
  - [ ] If SpacetimeDB's model doesn't fit tick-based architecture
  - [ ] Create thin layer between SpacetimeDB and existing multiplayer code
  - [ ] Use SpacetimeDB primarily as persistent store with pub/sub
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

#### ✅ Task 18: Signal Messenger Interface (Phase 1 Complete)

**Status:** Basic functionality working, see [SIGNAL_STATUS.md](../SIGNAL_STATUS.md)

**Completed:**
- [x] Create Signal bot with signal-cli-rest-api integration
- [x] Implement webhook server (actix-web on port 3001)
- [x] Message parsing and command routing
- [x] Add command execution via SpacetimeDB
- [x] Format responses with emojis (📍 🚶 ✅ ❌)
- [x] Manage player sessions via phone numbers (in-memory cache)
- [x] Authentication flow (phone → character name)
- [x] Background tick processor (3 seconds)
- [x] Complete setup documentation

**Phase 2 TODO:**
- [ ] Query session table after creation
- [ ] Implement command_log subscription
- [ ] Support Signal group chats
- [ ] Rate limiting per phone number
- [ ] Database-backed session persistence

**Documentation:** [SIGNAL_BOT_SETUP.md](../SIGNAL_BOT_SETUP.md), [SIGNAL_STATUS.md](../SIGNAL_STATUS.md)

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
