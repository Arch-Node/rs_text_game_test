#!/bin/bash
# Test script for multiplayer event broadcasting
#
# This script creates two players and demonstrates that they can see
# each other's actions in real-time via WebSocket subscriptions.

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎮 Multiplayer Event Broadcasting Test${NC}"
echo "This test demonstrates real-time cross-client event notifications"
echo ""

# Clean up any existing test players
echo -e "${GREEN}Cleaning up previous test data...${NC}"
spacetime call text-game clean_test_data || true

# Create two test players in nearby locations
echo -e "${GREEN}Creating Player 1 (Alice) at position (100, 100, 100)...${NC}"
cargo run --bin test_sdk_movement --features spacetimedb-sdk-client -- Alice &
PID1=$!

sleep 2

echo -e "${GREEN}Creating Player 2 (Bob) at position (101, 100, 100)...${NC}"
cargo run --bin test_sdk_movement --features spacetimedb-sdk-client -- Bob &
PID2=$!

# Wait for both to complete
wait $PID1
wait $PID2

echo ""
echo -e "${BLUE}✨ Test Complete${NC}"
echo ""
echo "Expected behavior:"
echo "  - Alice and Bob both connect via WebSocket"
echo "  - When Alice moves, Bob receives a 'PlayerMoved' event"
echo "  - When Bob moves, Alice receives a 'PlayerMoved' event"
echo "  - Events are filtered by proximity (within 5 units)"
echo "  - Background WebSocket processor delivers callbacks instantly"
echo ""
echo "Event types supported:"
echo "  • PlayerMoved: Another player changed position"
echo "  • PlayerJoined: New player entered the area"
echo "  • PlayerAction: Another player used a command"
echo "  • PlayerLeft: Player disconnected"
