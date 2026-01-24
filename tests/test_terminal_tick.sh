#!/bin/bash
# test_terminal_tick.sh
#
# Test script for terminal client tick functionality
# 
# Prerequisites:
# 1. SpacetimeDB must be running on localhost:3000
# 2. The text-game module must be published to SpacetimeDB
#
# Usage:
#   ./test_terminal_tick.sh

set -e  # Exit on error

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Terminal Client Tick Test Runner                           ║"
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
cargo build --features spacetimedb-http --bin test_terminal_tick --quiet
echo "✅ Build complete"
echo

# Run the test
echo "🚀 Running terminal tick test..."
echo "════════════════════════════════════════════════════════════════"
echo

./target/debug/test_terminal_tick

echo
echo "════════════════════════════════════════════════════════════════"
echo "✅ Test script complete!"
