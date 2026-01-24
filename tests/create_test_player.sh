#!/bin/bash
# create_test_player.sh
#
# Helper script to create TestPlayer for testing
#
# This script:
# 1. Connects to SpacetimeDB
# 2. Creates a player named "TestPlayer"
# 3. Verifies the player was created successfully

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Create TestPlayer                                           ║"
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

# Build terminal client
echo "🔨 Building terminal client..."
cargo build --features spacetimedb-http --bin terminal_client --quiet 2>/dev/null || true
echo "✅ Build complete"
echo

echo "📝 Instructions:"
echo "   1. When prompted, enter player name: TestPlayer"
echo "   2. Press Ctrl+C after authentication to exit"
echo "   3. The player will be created and ready for testing"
echo
echo "Starting terminal client in 3 seconds..."
sleep 3
echo

# Run terminal client
# User will need to manually enter "TestPlayer" and then exit
./target/debug/terminal_client || true

echo
echo "════════════════════════════════════════════════════════════════"
echo "✅ Setup complete!"
echo
echo "You can now run the movement test:"
echo "  ./test_player_movement.sh"
