[← Back to Main README](README.md)

---

# Multiplayer Skills-Heavy Text Game - Development Roadmap

## Overview

**Current Status:** Phase 1 substantially complete - SpacetimeDB backend operational, Signal and Discord bots fully implemented with DM-only gameplay. Terminal client needs fixing.

**Project Structure:**
```
rs_text_game_test/
├── src/
│   ├── lib.rs                      # Library exports
│   ├── bin/
│   │   ├── terminal_client.rs      # ⚠️ HTTP client (needs query endpoint fix)
│   │   ├── sdk_client.rs           # ✅ SDK demo (works, needs full integration)
│   │   ├── signal_bot.rs           # ✅ Signal Messenger bot (Phase 1 complete)
│   │   └── discord_bot.rs          # ✅ Discord bot (implementation complete)
│   ├── terminal_client/            # HTTP client modules
│   ├── spacetimedb_client/         # Generated SDK bindings (19 files)
│   ├── signal_client/              # Signal bot (webhook + SpacetimeDB HTTP)
│   └── discord_client/             # Discord bot (gateway + SpacetimeDB HTTP)
├── docs/                           # Architecture documentation
├── tests/                          # Test scripts
└── TODO.md                         # This file

text_game_stdb/                     # Separate SpacetimeDB module repo
└── src/lib.rs                      # ✅ Published WASM module
```

**Completed Interfaces:**
- ✅ **Signal Bot** - Webhook-based, DM-only gameplay after group auth
- ✅ **Discord Bot** - Gateway-based, DM-only gameplay after guild auth  
- ⚠️ **Terminal Client** - HTTP-based, has query endpoint issue

Transform the single-player terminal game into a multiplayer, skills-focused text adventure accessible via Discord, Signal, and Terminal interfaces.

## Design Philosophy
- **Skills-heavy gameplay** - Most challenges solvable through skill checks rather than combat
- **Light combat** - Quick, avoidable through stealth/persuasion, skill-based resolution
- **Multiple solution paths** - Different skills provide different approaches
- **Multiplayer collaboration** - Players can cooperate using complementary skills
- **Multi-platform** - Play via Discord, Signal messenger, or terminal client
- **DM-only gameplay** - Keep public channels clean, all game content in direct messages

---

## Phase Overview

### Phase 1: Core Infrastructure (95% Complete)
**Status:** SpacetimeDB backend operational. Discord and Signal bots fully functional with DM-only gameplay. Terminal client needs fixing. Event broadcasting not yet implemented.

**Critical Path:**
1. Fix terminal client query endpoint bug → Enable terminal gameplay
2. Implement tick result feedback → Players see their action outcomes
3. Add event broadcasting → Real-time multi-player interactions

See detailed task breakdown below.

### Phase 2: Skills & Character Systems (Not Started)
Create skill-heavy gameplay mechanics with character classes focused on non-combat solutions.

### Phase 3: World & Content (Not Started)
Build dimensional world with exploration, puzzles, and multi-solution challenges.

### Phase 4: Advanced Features (Not Started)
Add combat, items, magic, and social systems.

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
  - [x] Session struct with interface_type (Terminal/Signal/Discord)
  - [x] ConnectionIdentifiers: phone (Signal), Discord user ID, IP address
  - [x] SpacetimeDB Identity-based authentication
  - [x] Session cached in bot memory for performance
  - [x] Player linked to session in database
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
  - [x] Instant: help, status (bypass tick system)
  - [x] Queued: movement (north/south/east/west/up/down), look, all world-changing actions
  - [x] Implemented in both Signal and Discord bots
- [x] **Inbound Command Envelope:**
  - [x] Simple text commands from Discord/Signal
  - [x] Authentication via `auth PlayerName` command
  - [x] Rate limiting: one queued command per player
  - [x] Command validation before queueing
- [x] **Outbound Response Envelope:**
  - [x] Plain text for Discord and Signal
  - [x] Discord markdown support (bold, code blocks)
  - [x] Emoji integration (✅ ❌ 📧 🎮)
  - [x] Message length limits (Discord: 2000 chars)
  - [x] Error formatting with helpful messages
- [ ] **Event Broadcasting:**
  - [ ] Design event filtering (who needs to see what)
  - [ ] Plan event delivery guarantees
  - [ ] Design event subscription system (room-based, proximity-based, global)
  - [ ] Return tick results to clients
  - [ ] Real-time room updates when players enter/leave

##### 1.4 Architecture Pattern Selection
- [x] **Core Pattern Selected:** Tick-based event-driven with command queue
- [x] **Concurrency Strategy:** SpacetimeDB handles this (ACID transactions)
- [x] **Turn-Based System:**
  - [x] 15-second ticks for production (strategic gameplay)
  - [x] 3-second ticks for testing
  - [x] Command queueing with one-per-player limit
- [ ] **Document pros/cons of chosen pattern**

##### 1.5 Session Abstraction Layer
- [x] **ConnectionIdentifiers Implemented:**
  - [x] Signal: Phone number
  - [x] Discord: User ID (snowflake)
  - [x] Terminal: Connection string/IP
  - [x] Stored on Session table in SpacetimeDB
  - [x] Cached in bot memory (HashMap) for performance
- [x] **Interface-Specific Implementation:**
  - [x] Signal bot: send_message() via signal-cli-rest-api HTTP
  - [x] Discord bot: msg.reply() via Serenity gateway
  - [x] Terminal client: HTTP response (needs fixing)
  - [x] Format limits enforced (Discord: 2000 chars)
- [ ] **Multi-Device Support (Future):**
  - [ ] One account, multiple characters
  - [ ] Character selection flow per interface
  - [ ] Handle multiple sessions for same account
  - [ ] Session switching between devices

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
  - [x] Link Identity → Session → Player in database
- [x] **Interface-Specific Auth (Implemented):**
  - [x] Discord: `@Bot auth PlayerName` in guild → DM sent
  - [x] Signal: `auth PlayerName` in group → DM sent
  - [x] Terminal: Direct authentication flow (needs fixing)
  - [x] DM-only gameplay after authentication
- [x] **Player Creation:**
  - [x] Players created in database via `authenticate_player` reducer
  - [x] Player persists across bot restarts
  - [x] Linked to Discord user ID or Signal phone number
- [ ] **Future Account System:**
  - [ ] User registration with password (for terminal)
  - [ ] Account recovery mechanisms
  - [ ] Guest/anonymous sessions

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

#### ⚠️ Task 2: Interface Client Implementation

**Status:** ✅ Signal and Discord complete, ⚠️ Terminal needs fixing

##### 2.1 Signal Messenger Bot (✅ Complete)
- [x] Webhook server using actix-web
- [x] Integration with signal-cli-rest-api
- [x] Message handler with authentication via `auth PlayerName`
- [x] Group authentication with DM redirect
- [x] DM-only gameplay (keeps groups clean)
- [x] Command routing (instant vs queued)
- [x] SpacetimeDB HTTP API integration
- [x] Session and player caching
- [x] Background tick processor (3 seconds)
- [x] Message formatting with emojis
- [x] Complete setup documentation

##### 2.2 Discord Bot (✅ Complete)
- [x] Gateway-based bot using Serenity 0.12
- [x] EventHandler implementation
- [x] Authentication via `@Bot auth PlayerName`
- [x] Guild authentication with auto-DM
- [x] DM-only gameplay (keeps channels clean)
- [x] Message handling for DMs and mentions
- [x] SpacetimeDB HTTP API integration
- [x] Session and player caching (Arc<RwLock<HashMap>>)
- [x] Background tick processor (3 seconds)
- [x] Discord-specific formatting
- [x] Error handling with anyhow::Result
- [x] Complete setup documentation

##### 2.3 Terminal Client (⚠️ Needs Fix)
- [x] HTTP-based client using reqwest
- [x] Authentication flow
- [x] Display formatting
- [x] Input handling
- [ ] **BLOCKER:** Fix query endpoint (`/database/text-game/query` returns 404)
- [ ] Complete game loop
- [ ] Test full flow
- [ ] Proper error messages

##### 2.4 SDK Client (⚠️ Partial)
- [x] SDK bindings generated (19 files)
- [x] Demo client compiled
- [x] Type-safe API confirmed working
- [ ] Full integration into terminal_client
- [ ] Replace HTTP calls with SDK calls
- [ ] Real-time subscriptions
- [ ] Event handling

#### ⚠️ Task 3: Player Entity and Storage

**Status:** ✅ Basic implementation complete, needs expansion

##### 3.1 Player Table (✅ Complete)
- [x] Player table in SpacetimeDB with id, name, account_id
- [x] Position tracking (x, y, z, dimension)
- [x] Status field
- [x] Last action timestamp
- [x] Player creation via `authenticate_player` reducer
- [x] Storage persists across bot restarts

##### 3.2 Player Features (⏳ Future)
- [ ] Expand Player model with class
- [ ] Add inventory system reference
- [ ] Add stats (health, mana, stamina)
- [ ] Add skill levels
- [ ] Add experience points system
- [ ] Add equipped items
- [ ] Create PlayerManager to track all connected players

#### ✅ Task 4: Persistence and State Management

**Status:** ✅ Complete with SpacetimeDB

##### 4.1 SpacetimeDB Backend (✅ Complete)
- [x] SpacetimeDB CLI installed and running (localhost:3000)
- [x] Module structure with tables and reducers
- [x] **Tables Defined:**
  - [x] `player` - Characters with 4D positions
  - [x] `room` - Locations with 3D coordinates and descriptions
  - [x] `session` - Active connections (Terminal/Signal/Discord)
  - [x] `queued_command` - Commands awaiting tick execution
  - [x] `command_log` - Complete audit trail
- [x] **Reducers Implemented:**
  - [x] `connect_session` - Create new connection
  - [x] `authenticate_player` - Link session to player (creates player in DB)
  - [x] `submit_command` - Queue command (one per player)
  - [x] `execute_tick` - Process all queued commands
  - [x] `get_player_info` - Query player state
  - [x] `create_room` - Insert room into database
  - [x] `get_current_room` - Query room at player's position
  - [x] `list_rooms_in_dimension` - Query all rooms
- [x] Compiled to WASM
- [x] Published to local server (text-game database)
- [x] 10 test rooms populated
- [x] ACID transactions (automatic)
- [x] Automatic persistence

##### 4.2 Client SDK (✅ Generated)
- [x] Type-safe Rust bindings auto-generated
- [x] 19 files with all tables and reducers
- [x] Used by SDK client demo

#### ⚠️ Task 5: Tick System and Event Broadcasting

**Status:** ✅ Tick system works, ⚠️ Results not returned to clients yet

##### 5.1 Tick System (✅ Complete)
- [x] Background tick processor in both bots (3 second interval)
- [x] `execute_tick` reducer processes all queued commands
- [x] One command per player limit
- [x] Command validation
- [x] Movement commands work (north, south, east, west, up, down)
- [x] Room transitions
- [x] Command logging

##### 5.2 Event Broadcasting (⏳ Not Started)
- [ ] Return tick results to clients
- [ ] Real-time room updates
- [ ] Player visibility in same room
- [ ] Event filtering (proximity-based)
- [ ] Cross-dimension awareness
- [ ] SpacetimeDB subscription system
- [ ] Event types (movement, chat, actions, combat, system)

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

### Phase 6: Multiplayer Features

#### ⬜ Task 16: Create Player Interaction Commands
- [ ] Add chat commands (say/tell/whisper)
- [ ] Implement trade system with other players
- [ ] Add examine other players
- [ ] Create follow/group system
- [ ] Add skill-based interactions
  - Teach skills to other players
  - Collaborate on challenges

#### ⬜ Task 17: Create Admin/Game Master Tools
- [ ] Build admin commands for spawning items/NPCs
- [ ] Add teleporting players
- [ ] Enable world modification
- [ ] Create event triggering
- [ ] Add player management
- [ ] Implement server monitoring

---

### Phase 7: Testing & Documentation

#### ⬜ Task 18: Write Comprehensive Tests
- [ ] Add unit tests for skills system
- [ ] Test combat mechanics
- [ ] Test inventory management
- [ ] Test command parsing
- [ ] Add integration tests for multiplayer interactions
- [ ] Test server-client communication
- [ ] Test persistence layer

#### ⬜ Task 19: Create Documentation
- [ ] Document multiplayer architecture
- [ ] Write skills system guide
- [ ] Create player handbook
- [ ] Document API for different interfaces
- [ ] Write deployment guide
- [ ] Create contribution guidelines

---

## Current Status
**Phase 1:** 95% Complete (Terminal client needs fix, event broadcasting pending)  
**Next Steps:** Fix terminal client query endpoint → Implement tick result feedback → Add event broadcasting  
**Date:** January 23, 2026

---

## Task Summary by Phase
- **Phase 1 (Infrastructure):** Tasks 0-5 [95% Complete]
- **Phase 2 (Skills & Classes):** Tasks 6-8 [Not Started]
- **Phase 3 (World & Content):** Tasks 9-14 [Not Started]
- **Phase 4 (Combat):** Task 15 [Not Started]
- **Phase 5 (AI Generation):** Task 16 [Not Started]
- **Phase 6 (Multiplayer Features):** Tasks 16-17 [Not Started]
- **Phase 7 (Testing & Docs):** Tasks 18-19 [Not Started]

---

## Notes
- Focus on skills as primary gameplay mechanic
- Combat should be avoidable and quick when it occurs
- Multiple solution paths encourage different character builds
- Cross-platform accessibility is key to multiplayer engagement

---

[← Back to Main README](README.md)
