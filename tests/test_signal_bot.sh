#!/bin/bash
# test_signal_bot.sh
#
# Test script for Signal bot setup validation

set -e

echo "=== Signal Bot Setup Validator ==="
echo

# Check if signal-cli-rest-api is running
echo "1. Checking signal-cli-rest-api..."
if curl -s http://localhost:8080/v1/health > /dev/null 2>&1; then
    echo "   ✅ signal-cli-rest-api is running"
else
    echo "   ❌ signal-cli-rest-api is NOT running"
    echo "      Start it with: docker run -d -p 8080:8080 --name signal-cli-rest-api bbernhard/signal-cli-rest-api:latest"
    exit 1
fi

# Check if SpacetimeDB is running
echo "2. Checking SpacetimeDB..."
if curl -s http://localhost:3000/database/text-game/call > /dev/null 2>&1; then
    echo "   ✅ SpacetimeDB is running"
else
    echo "   ❌ SpacetimeDB is NOT running"
    echo "      Start it with: cd text_game_stdb && spacetime publish text-game"
    exit 1
fi

# Check for BOT_NUMBER environment variable
echo "3. Checking configuration..."
if [ -z "$BOT_NUMBER" ]; then
    echo "   ❌ BOT_NUMBER environment variable not set"
    echo "      Set it with: export BOT_NUMBER=\"+15551234567\""
    exit 1
else
    echo "   ✅ BOT_NUMBER is set: $BOT_NUMBER"
fi

# Try to build the Signal bot
echo "4. Building Signal bot..."
if cargo build --features signal-client --bin signal_bot 2>&1 | grep -q "Finished"; then
    echo "   ✅ Signal bot builds successfully"
else
    echo "   ❌ Signal bot build failed"
    exit 1
fi

# Check if bot can start (don't actually start it, just validate)
echo "5. Validating binary..."
if [ -f "target/debug/signal_bot" ]; then
    echo "   ✅ signal_bot binary exists"
else
    echo "   ❌ signal_bot binary not found"
    exit 1
fi

echo
echo "=== All checks passed! ==="
echo
echo "To start the bot:"
echo "  export BOT_NUMBER=\"$BOT_NUMBER\""
echo "  cargo run --features signal-client --bin signal_bot"
echo
echo "To test sending a message:"
echo "  1. Send a message from your phone to: $BOT_NUMBER"
echo "  2. First message should be your character name"
echo "  3. Try commands like: look, north, help"
echo
