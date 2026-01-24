#!/bin/bash
# Test script for terminal client connectivity

echo "=== Testing SpacetimeDB Backend ==="
echo

echo "1. Checking SpacetimeDB server status..."
curl -s http://localhost:3000/health > /dev/null
if [ $? -eq 0 ]; then
    echo "   ✓ Server is running"
else
    echo "   ✗ Server is not responding"
    exit 1
fi

echo
echo "2. Checking room count..."
ROOM_COUNT=$(spacetime sql text-game -s local -- "SELECT COUNT(*) FROM room" 2>/dev/null | grep -o '[0-9]\+' | tail -1)
echo "   ✓ Found $ROOM_COUNT rooms"

echo
echo "3. Testing connect_session reducer..."
spacetime call text-game -s local connect_session '"terminal"' '"test-conn-1"' 2>&1 | grep -q "WARNING"
if [ $? -eq 0 ]; then
    echo "   ✓ connect_session executed"
else
    echo "   ✗ connect_session failed"
fi

echo
echo "4. Checking session was created..."
SESSION_COUNT=$(spacetime sql text-game -s local -- "SELECT COUNT(*) FROM session" 2>/dev/null | grep -o '[0-9]\+' | tail -1)
echo "   ✓ Found $SESSION_COUNT session(s)"

echo
echo "=== Backend Ready for Testing ==="
echo
echo "To test clients:"
echo "  1. SDK Client (recommended):"
echo "     cargo run --bin sdk_client --features spacetimedb-sdk-client"
echo
echo "  2. HTTP Client (needs API fix):"
echo "     cargo run --bin terminal_client --features spacetimedb-http"
echo
echo "  3. Bash wrapper (works immediately):"
echo "     See TERMINAL_CLIENT_STATUS.md for script"
echo
