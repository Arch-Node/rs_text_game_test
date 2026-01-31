#!/bin/bash
# Complete restart script for development
# Rebuilds backend, client, and restarts everything fresh
#
# Usage:
#   ./restart_dev.sh         # Keep existing data
#   ./restart_dev.sh --clear # Clear database and start fresh

set -e  # Exit on error

# Parse arguments
CLEAR_DB=false
if [[ "$1" == "--clear" ]]; then
    CLEAR_DB=true
fi

echo "🔄 === Development Environment Restart ==="
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    echo "⚠️  Warning: .env file not found!"
    echo "Copy .env.example to .env and fill in your tokens:"
    echo "  cp .env.example .env"
    echo "  nano .env"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Step 1: Stop any running SpacetimeDB instances
echo "1️⃣  Stopping SpacetimeDB..."
pkill -f "spacetime start" || true
sleep 2

# Step 2: Start SpacetimeDB in background
echo ""
echo "2️⃣  Starting SpacetimeDB..."
cd ../text_game_stdb || {
    echo "❌ Error: Backend directory not found at ../text_game_stdb"
    echo "Please update the path in this script"
    exit 1
}

spacetime start &
SPACETIME_PID=$!
echo "✅ SpacetimeDB started (PID: $SPACETIME_PID)"

# Wait for SpacetimeDB to be ready
echo "⏳ Waiting for SpacetimeDB to be ready..."
sleep 3

# Step 3: Publish backend
echo ""
echo "3️⃣  Publishing SpacetimeDB backend..."

if [ "$CLEAR_DB" = true ]; then
    echo "📦 Publishing with --clear-database (fresh start)..."
    spacetime publish --clear-database --server local text-game
else
    echo "📦 Publishing and keeping existing data..."
    spacetime publish --server local text-game
fi

if [ $? -ne 0 ]; then
    echo "❌ Failed to publish backend"
    exit 1
fi

echo "✅ Backend published successfully"

# Step 4: Return to client and rebuild
echo ""
echo "4️⃣  Rebuilding Discord client..."
cd - > /dev/null
cargo build --features discord-client --bin discord_bot
if [ $? -ne 0 ]; then
    echo "❌ Failed to build Discord client"
    exit 1
fi

echo "✅ Discord client built successfully"

# Step 5: Run the bot
echo ""
echo "5️⃣  Starting Discord bot..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Load environment variables from .env if it exists
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

exec cargo run --features discord-client --bin discord_bot

