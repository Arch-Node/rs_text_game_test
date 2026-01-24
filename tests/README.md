# Tests

This directory contains all test scripts and test binaries.

## Test Scripts

### Shell Scripts (in tests/)
- **test_backend.sh** - Tests backend SpacetimeDB reducers
- **test_signal_bot.sh** - Tests Signal messenger bot integration
- **create_test_player.sh** - Helper script to create test players
- **test_player_movement.sh** - Tests player movement commands
- **test_terminal_tick.sh** - Tests terminal client tick execution

### Test Binaries (in tests/bin/)
- **test_sdk_movement.rs** - SDK client movement test (WebSocket)
- **test_sdk_tick.rs** - SDK client tick execution test (WebSocket)
- **test_player_movement.rs** - HTTP client movement test
- **test_terminal_tick.rs** - Terminal client tick test

## Running Tests

### End-to-End Test Suite
Run all tests from the project root:
```bash
./end_to_end_test.sh
```

### Individual SDK Tests
```bash
# Test movement with WebSocket SDK
cargo run --features spacetimedb-sdk-client --bin test_sdk_movement

# Test tick execution with WebSocket SDK
cargo run --features spacetimedb-sdk-client --bin test_sdk_tick
```

### Individual HTTP Tests
```bash
# Test movement with HTTP client
cargo run --features spacetimedb-http --bin test_player_movement

# Test terminal tick
cargo run --features spacetimedb-http --bin test_terminal_tick
```

### Shell Script Tests
```bash
cd tests/

# Test backend
./test_backend.sh

# Test Signal bot
./test_signal_bot.sh

# Test player movement
./test_player_movement.sh

# Test terminal tick
./test_terminal_tick.sh
```

## Test Data

Test data (rooms, etc.) is stored in `/test_data/` and can be loaded with:
```bash
./load_test_data.sh
```

## Documentation

See [../docs/TESTING_GUIDE.md](../docs/TESTING_GUIDE.md) for detailed testing documentation.
