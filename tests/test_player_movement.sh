#!/bin/bash
# test_player_movement.sh
#
# Test script for TestPlayer movement around the map
# 
# Prerequisites:
# 1. SpacetimeDB must be running on localhost:3000
# 2. The text-game module must be published to SpacetimeDB
# 3. A player named "TestPlayer" must already exist
#    (Create it first using: cargo run --features spacetimedb-http --bin terminal_client)
#
# Usage:
#   ./test_player_movement.sh

set -e  # Exit on error

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  TestPlayer Movement Test Runner                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo

# Check if SpacetimeDB is running
echo "🔍 Checking if SpacetimeDB is running..."
if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "❌ Error: SpacetimeDB is not running on localhost:3000"
    echo
    echo "Please start SpacetimeDB first:"
    echo "  spacetime start"
    echo
    exit 1
fi
echo "✅ SpacetimeDB is running"
echo

# Check if the database exists
echo "🔍 Checking if 'text-game' database exists..."
if ! curl -s "http://localhost:3000/database/sql/text-game" -X POST -d "SELECT 1" > /dev/null 2>&1; then
    echo "❌ Error: Database 'text-game' does not exist"
    echo
    echo "Please publish the SpacetimeDB module first:"
    echo "  cd ../text_game_stdb"
    echo "  spacetime publish text-game"
    echo
    echo "Or if the database has a different name, update SpacetimeConfig::default()"
    echo "in src/terminal_client/client.rs"
    echo
    exit 1
fi
echo "✅ Database 'text-game' exists"
echo

# Build the test binary
echo "🔨 Building test binary..."
cargo build --features spacetimedb-http --bin test_player_movement --quiet
echo "✅ Build complete"
echo

# Run the test
echo "🚀 Running TestPlayer movement test..."
echo "════════════════════════════════════════════════════════════════"
echo

./target/debug/test_player_movement

EXIT_CODE=$?

echo
echo "════════════════════════════════════════════════════════════════"

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Test completed successfully!"
else
    echo "❌ Test failed with exit code: $EXIT_CODE"
    echo
    echo "Common issues:"
    echo "  - Player 'TestPlayer' might not exist yet"
    echo "  - Create it first using: cargo run --features spacetimedb-http --bin terminal_client"
    echo "  - Or the game module might not be published to SpacetimeDB"
fi

exit $EXIT_CODE
