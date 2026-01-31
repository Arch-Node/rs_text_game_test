#!/bin/bash
# Run the Discord bot with environment variables loaded from .env

# Check if .env exists
if [ ! -f .env ]; then
    echo "Error: .env file not found!"
    echo "Copy .env.example to .env and fill in your tokens:"
    echo "  cp .env.example .env"
    echo "  nano .env"
    exit 1
fi

# Load environment variables from .env
export $(cat .env | grep -v '^#' | xargs)

# Run the bot
echo "Starting Discord bot..."
echo "SpacetimeDB: ${SPACETIME_URL}"
cargo run --features discord-client --bin discord_bot
