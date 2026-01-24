#!/bin/bash
# Load test data into SpacetimeDB

set -e

echo "🔄 Loading test data into SpacetimeDB..."
echo ""

# Check if database exists
if ! spacetime sql text-game --server local -- "SELECT * FROM room" >/dev/null 2>&1; then
    echo "❌ Database 'text-game' not found. Please publish the module first:"
    echo "   spacetime publish -s local -y text-game"
    exit 1
fi

echo "📦 Database found: text-game"
echo ""

# Load rooms from JSON
ROOMS_FILE="test_data/rooms.json"

if [ ! -f "$ROOMS_FILE" ]; then
    echo "❌ Rooms file not found: $ROOMS_FILE"
    exit 1
fi

echo "🏠 Loading rooms..."

# Starting Hall (100, 100, 100)
echo "   Creating: The Starting Hall"
spacetime call text-game create_room 100 100 100 '"material"' '"The Starting Hall"' '"A grand hall with marble floors and tall pillars. Torches line the walls, casting flickering shadows. Exits lead north, south, east, and west."' '"{\"north\":{\"x\":100,\"y\":101,\"z\":100},\"south\":{\"x\":100,\"y\":99,\"z\":100},\"east\":{\"x\":101,\"y\":100,\"z\":100},\"west\":{\"x\":99,\"y\":100,\"z\":100}}"' --server local >/dev/null 2>&1

# Northern Courtyard (100, 101, 100)
echo "   Creating: The Northern Courtyard"
spacetime call text-game create_room 100 101 100 '"material"' '"The Northern Courtyard"' '"An open courtyard with a fountain in the center. The sound of water echoes off the surrounding walls. A path leads back south."' '"{\"south\":{\"x\":100,\"y\":100,\"z\":100}}"' --server local >/dev/null 2>&1

# Eastern Garden (101, 100, 100)
echo "   Creating: The Eastern Garden"
spacetime call text-game create_room 101 100 100 '"material"' '"The Eastern Garden"' '"A lush garden filled with exotic plants and flowers. The air is fragrant with their perfume. A path leads back west."' '"{\"west\":{\"x\":100,\"y\":100,\"z\":100}}"' --server local >/dev/null 2>&1

# Southern Passage (100, 99, 100)
echo "   Creating: The Southern Passage"
spacetime call text-game create_room 100 99 100 '"material"' '"The Southern Passage"' '"A narrow stone passage leading deeper into the structure. The walls are damp and cold. A path leads back north."' '"{\"north\":{\"x\":100,\"y\":100,\"z\":100}}"' --server local >/dev/null 2>&1

# Western Library (99, 100, 100)
echo "   Creating: The Western Library"
spacetime call text-game create_room 99 100 100 '"material"' '"The Western Library"' '"A cozy library with shelves upon shelves of ancient books. A reading desk sits near a window. A path leads back east."' '"{\"east\":{\"x\":100,\"y\":100,\"z\":100}}"' --server local >/dev/null 2>&1

echo ""
echo "✅ Test data loaded successfully!"
echo ""

# Verify rooms were created
ROOM_COUNT=$(spacetime sql text-game --server local -- "SELECT * FROM room" 2>/dev/null | grep -v "WARNING" | grep -v "position_x" | grep -v "^$" | wc -l)
echo "📊 Total rooms in database: $ROOM_COUNT"
echo ""
echo "To view rooms:"
echo "  spacetime sql text-game --server local -- \"SELECT name, position_x, position_y, position_z FROM room\""
