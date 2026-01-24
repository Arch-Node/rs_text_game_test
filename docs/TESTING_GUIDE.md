# Terminal Client Testing Guide

This directory contains test scripts for the terminal client and multiplayer tick functionality.

📋 **Quick Reference:** See [TEST_SCRIPTS.md](TEST_SCRIPTS.md) for a comparison table of all test scripts.

## ⚠️ First Time Setup

**If you get a 404 error**, the SpacetimeDB module hasn't been published yet. See **[SPACETIMEDB_SETUP.md](SPACETIMEDB_SETUP.md)** for complete setup instructions.

### Quick Setup (Automated)

Run the complete end-to-end test that does everything:
```bash
# This will publish the module and run all tests
./end_to_end_test.sh
```

### Manual Setup

```bash
# 1. Start SpacetimeDB
spacetime start

# 2. Publish the game module
cd ../text_game_stdb
spacetime publish text-game

# 3. Verify it exists
cd ../rs_text_game_test
spacetime list
```

## Test Scripts Overview

### 0. Complete End-to-End Test (Recommended)
**File:** [end_to_end_test.sh](end_to_end_test.sh)

Comprehensive test suite that automates everything:
- ✅ Checks prerequisites (SpacetimeDB CLI, server running)
- ✅ Publishes the SpacetimeDB module
- ✅ Builds all test binaries
- ✅ Tests connection and player creation
- ✅ Tests command submission and tick execution
- ✅ Tests player movement around the map
- ✅ Verifies data persistence
- ✅ Offers cleanup options

**Usage:**
```bash
./end_to_end_test.sh
```

This is the **best way to verify your entire setup** from scratch.

### 1. Automated Tick Test
**File:** [test_terminal_tick.sh](test_terminal_tick.sh)  
**Binary:** `test_terminal_tick`

Tests the complete tick workflow with an auto-generated test player:
- Creates a temporary player (TestPlayer_XXXX)
- Submits multiple commands (look, north, east, south, west, inventory)
- Waits for tick execution
- Queries command results from command_log
- Verifies position changes

**Usage:**
```bash
./test_terminal_tick.sh
```

### 2. Player Movement Test
**File:** [test_player_movement.sh](test_player_movement.sh)  
**Binary:** `test_player_movement`

Tests map navigation with a persistent TestPlayer:
- Logs in as "TestPlayer"
- Executes movement sequence (N→E→S→W)
- Shows position changes after each move
- Verifies return to starting position
- Displays final distance traveled

**Usage:**
```bash
# First create TestPlayer (see below)
./create_test_player.sh

# Then run the movement test
./test_player_movement.sh
```

### 3. Create TestPlayer Helper
**File:** [create_test_player.sh](create_test_player.sh)

Helper script to create the "TestPlayer" character:
- Launches terminal_client
- Prompts you to enter "TestPlayer" as the name
- Creates the player in SpacetimeDB

**Usage:**
```bash
./create_test_player.sh
# Enter "TestPlayer" when prompted
# Press Ctrl+C after authentication
```

## Prerequisites

All tests require:
1. **SpacetimeDB running** on `localhost:3000`
   ```bash
   spacetime start
   ```

2. **Game module published** to SpacetimeDB
   ```bash
   cd text_game_stdb
   spacetime publish text-game
   ```

## Quick Start

```bash
# 1. Start SpacetimeDB
spacetime start

# 2. Run automated tick test (no player needed)
./test_terminal_tick.sh

# 3. Create TestPlayer for movement test
./create_test_player.sh

# 4. Run movement test
./test_player_movement.sh
```

## Manual Testing

You can also run the binaries directly:

```bash
# Automated tick test
cargo run --features spacetimedb-http --bin test_terminal_tick

# Player movement test
cargo run --features spacetimedb-http --bin test_player_movement

# Interactive terminal client
cargo run --features spacetimedb-http --bin terminal_client
```

## What Each Test Validates

### test_terminal_tick
✅ SpacetimeDB connection  
✅ Player authentication  
✅ Command submission to queue  
✅ Tick execution (3.5s wait)  
✅ Result polling from command_log  
✅ State persistence

### test_player_movement
✅ Persistent player login  
✅ Map navigation (N/S/E/W)  
✅ Position tracking  
✅ Room transitions  
✅ Movement validation  
✅ Distance calculation

## Troubleshooting

### "SpacetimeDB is not running"
```bash
# Start SpacetimeDB
spacetime start
```

### "Player 'TestPlayer' might not exist"
```bash
# Create the player first
./create_test_player.sh
```

### "Authentication failed"
- Make sure the game module is published:
  ```bash
  cd text_game_stdb
  spacetime publish text-game
  ```

### "Command failed" or "No results in command_log"
- Verify the tick system is running in the SpacetimeDB module
- Check that execute_tick reducer is being called
- Ensure command_log table exists and is writable

## Test Output Examples

### Successful Tick Test:
```
✅ Step 1: Connected! Session ID: 12345
✅ Step 2: Authenticated! Player ID: 67890
📊 Command submitted: look
⏰ Waiting for tick execution...
✅ Command succeeded
```

### Successful Movement Test:
```
📍 Position changed: (0, 0, 0) → (0, 1, 0)
✅ Command succeeded
📏 Distance from start: 0 tiles
✅ Returned to starting position!
```

## Architecture Notes

These tests demonstrate the complete multiplayer architecture:
- **Client → SpacetimeDB HTTP API** (submit_command reducer)
- **Command Queue** (queued_command table)
- **Tick Execution** (execute_tick reducer)
- **Result Logging** (command_log table)
- **State Updates** (player table, room changes)

The 3.5 second delay between command submission and result checking aligns with the expected tick interval in the SpacetimeDB backend.
