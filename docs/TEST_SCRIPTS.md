# Test Scripts Quick Reference

## Overview

| Script | Purpose | Prerequisites | Duration |
|--------|---------|---------------|----------|
| `end_to_end_test.sh` | **Complete test suite** | SpacetimeDB running, text_game_stdb directory | ~30 sec |
| `test_terminal_tick.sh` | Test tick execution with auto-generated player | Database published | ~25 sec |
| `test_player_movement.sh` | Test TestPlayer movement | TestPlayer exists, database published | ~20 sec |
| `create_test_player.sh` | Create TestPlayer manually | Database published | Manual |

## Usage Patterns

### First Time Setup
```bash
# Start SpacetimeDB
spacetime start

# Run complete test (does everything)
./end_to_end_test.sh
```

### Quick Tick Test
```bash
# Tests command queue and tick execution
./test_terminal_tick.sh
```

### Movement Testing
```bash
# Create TestPlayer first
./create_test_player.sh

# Then test movement
./test_player_movement.sh
```

### Interactive Testing
```bash
# Manual terminal client
cargo run --features spacetimedb-http --bin terminal_client
```

## Test Script Details

### end_to_end_test.sh ⭐ Recommended

**What it does:**
1. ✅ Checks prerequisites (CLI, server, module directory)
2. ✅ Publishes SpacetimeDB module from ../text_game_stdb
3. ✅ Builds test binaries
4. ✅ Runs tick test (connection, player creation, commands)
5. ✅ Tests player movement
6. ✅ Queries player data to verify persistence
7. ✅ Offers cleanup options

**When to use:**
- First time setup
- After making changes to SpacetimeDB module
- To verify entire system end-to-end
- CI/CD pipelines

**Output:**
- Color-coded status messages
- Detailed logs in /tmp/*.log
- Summary of passed/failed tests

### test_terminal_tick.sh

**What it does:**
- Creates temporary player (TestPlayer_XXXX)
- Submits 5 test commands (look, north, look, inventory, south)
- Waits 3.5s for tick after each command
- Queries command_log for results
- Shows position changes

**When to use:**
- Testing tick execution
- Verifying command queue
- Testing result feedback
- Quick validation

### test_player_movement.sh

**What it does:**
- Logs in as existing "TestPlayer"
- Executes movement sequence: N→E→S→W
- Includes look commands
- Tracks position changes
- Calculates distance traveled
- Verifies return to start

**When to use:**
- Testing persistent player state
- Verifying movement mechanics
- Testing room transitions
- After room/exit changes

### create_test_player.sh

**What it does:**
- Launches terminal_client
- Prompts for player name
- Guides you to create "TestPlayer"

**When to use:**
- Before running movement test
- Creating test accounts manually
- First time setup

## Binary Commands

Run tests directly with cargo:

```bash
# Tick test
cargo run --features spacetimedb-http --bin test_terminal_tick

# Movement test
cargo run --features spacetimedb-http --bin test_player_movement

# Interactive client
cargo run --features spacetimedb-http --bin terminal_client
```

## Troubleshooting

### 404 Error
**Problem:** Database doesn't exist  
**Solution:** Run `./end_to_end_test.sh` or manually publish:
```bash
cd ../text_game_stdb
spacetime publish text-game
```

### "TestPlayer might not exist"
**Problem:** Movement test needs existing player  
**Solution:**
```bash
./create_test_player.sh
# Or let end_to_end_test.sh create players automatically
```

### "SpacetimeDB is not running"
**Problem:** Server not started  
**Solution:**
```bash
# In another terminal
spacetime start
```

### Compilation Errors
**Problem:** Dependencies or features missing  
**Solution:**
```bash
# Rebuild everything
cargo clean
cargo build --features spacetimedb-http
```

## Expected Results

### Successful end_to_end_test.sh:
```
✅ Prerequisites check
✅ Module publishing
✅ Binary compilation
✅ Connection and commands
✅ Player movement
✅ Data persistence

╔══════════════════════════════════════════════════════════════╗
║  ALL TESTS PASSED ✓                                         ║
╚══════════════════════════════════════════════════════════════╝
```

### Successful tick test:
```
✅ Connected! Session ID: 12345
✅ Authenticated! Player ID: 67890
Test 1/5: Command "look"
   ✅ Command submitted
   ⏰ Waiting for tick...
   ✅ Command succeeded
```

### Successful movement test:
```
📍 Position changed: (0, 0, 0) → (0, 1, 0)
✅ Command succeeded
📏 Distance from start: 0 tiles
✅ Returned to starting position!
```

## Log Files

Tests create temporary log files:
- `/tmp/publish_output.log` - Module publish output
- `/tmp/tick_test.log` - Tick test results
- `/tmp/movement_test.log` - Movement test results
- `/tmp/player_query.log` - Database query results

Check these if tests fail for detailed error messages.

## CI/CD Integration

For automated testing:

```bash
#!/bin/bash
# ci_test.sh

# Start SpacetimeDB in background
spacetime start &
SPACETIME_PID=$!
sleep 5

# Run tests
./end_to_end_test.sh
TEST_RESULT=$?

# Cleanup
kill $SPACETIME_PID

exit $TEST_RESULT
```

## Documentation Links

- [TESTING_GUIDE.md](TESTING_GUIDE.md) - Complete testing documentation
- [SPACETIMEDB_SETUP.md](SPACETIMEDB_SETUP.md) - SpacetimeDB setup and troubleshooting
- [README.md](README.md) - Project overview
- [TODO.md](TODO.md) - Development roadmap
