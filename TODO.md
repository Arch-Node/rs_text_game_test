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

#### ⬜ Task 1a: Define Multiplayer State Model
- [ ] Define authoritative GameState for multiple players
- [ ] Identify shared vs per-player state
- [ ] Add player/session identifiers to core models
- [ ] Plan world tick and update cadence

#### ⬜ Task 1b: Define Command/Message Protocol
- [ ] Define inbound command envelope (player/session, input, metadata)
- [ ] Define outbound response envelope (messages, errors, events)
- [ ] Standardize command routing for all interfaces
- [ ] Map output formats for terminal, Signal, and SMS

#### ⬜ Task 1c: Define Session Abstraction
- [ ] Create session interface for terminal, Signal, and SMS
- [ ] Decide how sessions map to players (account vs character)
- [ ] Plan connection lifecycle (connect, idle, disconnect, reconnect)

#### ⬜ Task 1d: Logging & Error Strategy
- [ ] Define logging levels and structured fields (session, player, command)
- [ ] Choose logging crates and error handling patterns
- [ ] Plan for tracing async flows and server diagnostics

#### ⬜ Task 1: Design Multiplayer Architecture
- [ ] Design server-client architecture with shared game state
- [ ] Decide: turn-based vs real-time mechanics
- [ ] Plan state synchronization strategy
- [ ] Design message broadcasting system
- [ ] Plan player sessions/authentication
- [ ] Consider concurrent player management

#### ⬜ Task 3: Create Player Entity System
- [ ] Expand Player model with name/id
- [ ] Add inventory system
- [ ] Add stats (health, mana, stamina)
- [ ] Add skill levels
- [ ] Add experience points system
- [ ] Add equipped items
- [ ] Create PlayerManager to track all connected players

#### ⬜ Task 2: Add Networking Dependencies
- [ ] Add `tokio` for async runtime
- [ ] Add Signal protocol support (libsignal-service-rs or signal-cli wrapper)
- [ ] Add websocket/TCP libraries for terminal interface
- [ ] Add SMS gateway support (twilio-rs or similar)
- [ ] Update Cargo.toml with all necessary crates

#### ⬜ Task 16: Add Persistence Layer
- [ ] Choose database (SQLite for simple, PostgreSQL for production)
- [ ] Design schema for player data
- [ ] Implement world state saving
- [ ] Add inventory/skills/progress persistence
- [ ] Enable server restart recovery
- [ ] Add save/load operations

#### ⬜ Task 9: Create Server Module
- [ ] Build game server core
- [ ] Implement world state management
- [ ] Add player connection handling
- [ ] Create command routing system
- [ ] Implement state persistence
- [ ] Add event broadcasting to all clients
- [ ] Create tick system for world updates

---

### Phase 2: Skills & Character Systems

#### ⬜ Task 4: Design Skills System
- [ ] Create skill categories (exploration, crafting, social, magic, survival, etc.)
- [ ] Implement skill levels and experience
- [ ] Add skill check mechanics
- [ ] Distinguish passive vs active skills
- [ ] Design skill-based progression
- [ ] Make skills the primary gameplay mechanic
- [ ] Create skill tree structure

#### ⬜ Task 5: Implement Character Classes
- [ ] Create Scholar class (research/knowledge focus)
- [ ] Create Diplomat class (social/persuasion focus)
- [ ] Create Artificer class (crafting/invention focus)
- [ ] Create Scout class (exploration/survival focus)
- [ ] Create Mystic class (magical skills focus)
- [ ] Define unique skill bonuses per class
- [ ] Balance class abilities

#### ⬜ Task 19: Add Skill Training Mechanics
- [ ] Implement learn-by-doing system (XP from skill use)
- [ ] Create skill trainers (NPCs that teach)
- [ ] Add skill books/scrolls
- [ ] Create practice activities
- [ ] Design skill trees with prerequisites
- [ ] Balance skill progression rates

---

### Phase 3: Game Content Systems

#### ⬜ Task 6: Create Inventory and Item System
- [ ] Design Item model with types (consumables, tools, equipment, quest items)
- [ ] Add weight/encumbrance system
- [ ] Implement durability mechanics
- [ ] Add quality levels
- [ ] Create inventory management with capacity limits
- [ ] Add item usage mechanics
- [ ] Implement equipping/unequipping system
- [ ] Build item creation/builder functions for procedural generation

#### ⬜ Task 8: Add NPC System
- [ ] Create NPC models
- [ ] Build NPC creation/builder functions for procedural generation
- [ ] Implement dialogue trees
- [ ] Add skill-based interaction checks (persuasion, intimidation, deception)
- [ ] Create trading system
- [ ] Add quest giver functionality
- [ ] Implement reputation tracking per NPC/faction

#### ⬜ Task 14: Create Quest System
- [ ] Design quest framework
- [ ] Create skill-based quest objectives
- [ ] Implement multiple solution paths
- [ ] Add quest tracking
- [ ] Design rewards system (skill XP, items, reputation)
- [ ] Create quest chains
- [ ] Add dynamic quest generation

#### ⬜ Task 13: Add Crafting System
- [ ] Design recipe system requiring specific skill levels
- [ ] Implement resource gathering
- [ ] Create item creation mechanics
- [ ] Add quality based on skill level
- [ ] Enable experimentation/discovery of new recipes
- [ ] Balance crafting progression

#### ⬜ Task 15: Implement Puzzle and Skill Challenges
- [ ] Create lockpicking (requires dexterity skill)
- [ ] Add research puzzles (requires knowledge skills)
- [ ] Implement social challenges (persuasion/deception)
- [ ] Create environmental puzzles (perception/investigation)
- [ ] Balance challenge difficulty

#### ⬜ Task 18: Implement Time and World Events
- [ ] Add day/night cycle
- [ ] Create weather effects that impact skills
  - Rain affects tracking
  - Darkness affects perception
- [ ] Add scheduled events
- [ ] Implement dynamic world changes based on player actions

---

### Phase 4: Combat (Lightweight)

#### ⬜ Task 7: Implement Combat System (Light)
- [ ] Design initiative based on perception skill
- [ ] Create attack/defense rolls using weapon/armor skills
- [ ] Add option to avoid combat through stealth/persuasion skills
- [ ] Implement status effects
- [ ] Add escape mechanics
- [ ] Keep combat quick and secondary to skills
- [ ] Build monster/enemy creation functions for procedural generation

---

### Phase 5: AI-Powered Content Generation

#### ⬜ Task 23: Integrate AI for Dynamic Content
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

#### ⬜ Task 11: Implement Terminal Interface (Refactor Existing)
- [ ] Refactor current terminal interface to connect to server
- [ ] Add connection handling
- [ ] Implement async input/output
- [ ] Add multiplayer awareness (see other players)
- [ ] Improve formatting for better readability
- [ ] Handle disconnection/reconnection

#### ⬜ Task 10: Implement Signal Interface
- [ ] Create Signal protocol client/bot
- [ ] Use libsignal or signal-cli wrapper
- [ ] Implement message parsing
- [ ] Add command execution
- [ ] Format responses appropriately
- [ ] Manage player sessions via phone numbers

#### ⬜ Task 12: Implement Text/SMS Interface
- [ ] Create text message interface using Twilio or similar
- [ ] Implement SMS parsing
- [ ] Handle response chunking (SMS length limits)
- [ ] Add session management
- [ ] Create command shortcuts for mobile

---

### Phase 7: Multiplayer Features

#### ⬜ Task 17: Create Player Interaction Commands
- [ ] Add chat commands (say/tell/whisper)
- [ ] Implement trade system with other players
- [ ] Add examine other players
- [ ] Create follow/group system
- [ ] Add skill-based interactions
  - Teach skills to other players
  - Collaborate on challenges

#### ⬜ Task 20: Create Admin/Game Master Tools
- [ ] Build admin commands for spawning items/NPCs
- [ ] Add teleporting players
- [ ] Enable world modification
- [ ] Create event triggering
- [ ] Add player management
- [ ] Implement server monitoring

---

### Phase 8: Testing & Documentation

#### ⬜ Task 21: Write Comprehensive Tests
- [ ] Add unit tests for skills system
- [ ] Test combat mechanics
- [ ] Test inventory management
- [ ] Test command parsing
- [ ] Add integration tests for multiplayer interactions
- [ ] Test server-client communication
- [ ] Test persistence layer

#### ⬜ Task 22: Create Documentation
- [ ] Document multiplayer architecture
- [ ] Write skills system guide
- [ ] Create player handbook
- [ ] Document API for different interfaces
- [ ] Write deployment guide
- [ ] Create contribution guidelines

---

## Current Status
**Phase:** Planning Complete  
**Next Steps:** Begin Phase 1 with Task 3 (Player Entity System) or Task 4 (Skills System)  
**Date:** January 23, 2026

---

## Notes
- Focus on skills as primary gameplay mechanic
- Combat should be avoidable and quick when it occurs
- Multiple solution paths encourage different character builds
- Cross-platform accessibility is key to multiplayer engagement
