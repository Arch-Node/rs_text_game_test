# Rust Text Adventure Game Engine

A modular, extensible text-based adventure game engine written in Rust. Features a layered world system with multiple z-levels, player classes with distinct abilities, and a flexible output system designed for both terminal and GUI interfaces.

> **🚧 Development Roadmap:** This project is being transformed into a multiplayer, skills-heavy game with support for Signal, terminal, and SMS interfaces. See [TODO.md](TODO.md) for the complete development roadmap and current status.

## Project Structure

```
rust_game_test/
├── Cargo.toml              # Project configuration and dependencies
├── rooms.json              # World data (rooms, layers, anchors)
├── src/                    # Source code directory
│   ├── main.rs            # Entry point and game loop (~60 lines)
│   ├── models.rs          # Core data structures (~110 lines)
│   ├── file_format.rs     # JSON deserialization structs (~80 lines)
│   ├── exits.rs           # Exit definitions and aliases (~70 lines)
│   ├── world.rs           # World loading and validation (~180 lines)
│   ├── commands.rs        # Command parsing and execution (~220 lines)
│   ├── output.rs          # Flexible message output system (~120 lines)
│   └── examples.rs        # Usage examples for different contexts
└── target/                # Build artifacts (generated)
```

## Module Overview

### `main.rs` - Entry Point
- Initializes the game engine
- Sets up the terminal game loop
- Handles user input prompts
- Minimal glue code to tie modules together

### `models.rs` - Core Data Structures
**Runtime game state:**
- `Coord` - 3D coordinates (w=layer, x,y=position)
- `Player` - Player state and position
- `PlayerClass` - Teleporter or LayerWalker abilities
- `GameState` - Complete game state container
- `World` - Validated world data with lookup maps
- `Room` - Individual room with exits and description
- `ExitSpec` - Exit destination specification
- `Anchor` - Teleportation anchor points
- `Layer` - Layer metadata (scale, name)
- `Meta` - World metadata

### `file_format.rs` - JSON Deserialization
**Serde structs for loading `rooms.json`:**
- `RoomsFile` - Root JSON structure
- `FileRoom`, `FileLayer`, `FileAnchor` - JSON representations
- `FileExitSpec` - Supports relative (`to_rel`) or absolute (`to_abs`) exits
- Keeps file format separate from runtime models

### `exits.rs` - Exit System
**Exit definitions and handling:**
- `ExitKind` - Directional, Teleport, LayerShift, Special
- `ExitDef` - Exit definitions with aliases
- `EXIT_DEFS` - Configurable exit types (north, south, window, mousehole, etc.)
- `build_exit_alias_map()` - Creates command alias lookup
- `resolve_exit_target()` - Converts exit specs to coordinates

### `world.rs` - World Loading & Validation
**World initialization:**
- `load_world_from_rooms_json()` - Loads and parses JSON
- `validate_world()` - Ensures world consistency:
  - Start position exists
  - All exits point to valid rooms
  - Anchors reference existing rooms
  - Layer scales are valid
- Converts file format structs to runtime models

### `commands.rs` - Command System
**Command parsing and execution:**
- `Command` enum - All available commands
- `parse_command()` - Parses user input to commands
- `execute_command()` - Routes commands to handlers
- Action handlers:
  - `do_look()` - Display current room
  - `do_go()` - Move through exits
  - `do_teleport()` - Teleport to anchors (Teleporter class)
  - `do_shift()` - Shift between layers (LayerWalker class, TODO)
  - `do_where()` - Show current position
  - `do_help()` - Display available commands

### `output.rs` - Flexible Output System
**Message accumulation for portability:**
- `MessageBuffer` - Accumulates game output
- `OutputMode`:
  - `BufferOnly` - Store messages (GUI/web/testing)
  - `TerminalAndBuffer` - Print AND store (terminal play)
  - `TerminalOnly` - Print without buffering
- Enables non-terminal interfaces (GUI, web, automated testing)

### `examples.rs` - Usage Examples
Demonstrates how to use the engine in different contexts:
- Silent execution for testing
- GUI integration patterns
- Web API response generation

## Building and Running

### Prerequisites
```bash
# Install Rust and Cargo
sudo apt install cargo
# or use rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build
```bash
cd rust_game_test
cargo build          # Debug build
cargo build --release # Optimized release build
```

### Run
```bash
cargo run
```

### Test
```bash
cargo test
```

## Game Features

### Layered World System
- Multiple z-layers (w coordinate)
- Different scales per layer (macrocell concept)
- Layer-specific room networks

### Player Classes
- **Teleporter**: Can teleport to anchors within same layer
- **LayerWalker**: Can shift between layers (planned feature)

### Exits
- Explicit exit system (only listed exits are valid)
- Relative exits (dx, dy within same layer)
- Absolute exits (full w, x, y coordinates)
- Configurable aliases (n/north, e/east, etc.)
- Special exits (window, mousehole, custom)

### Commands
```
look | l              View current room
where                 Show coordinates
go <exit>             Move through an exit
n, s, e, w           Directional shortcuts
teleport <anchor>     Teleport to anchor (Teleporter only)
shift up|down         Change layers (LayerWalker only, TODO)
help | ?              Show commands
quit                  Exit game
```

## Extending the Game

### Adding New Rooms
Edit `rooms.json`:
```json
{
  "id": "new_room",
  "pos": { "w": 0, "x": 2, "y": 0 },
  "name": "New Room",
  "desc": "A freshly created room.",
  "exits": {
    "west": { "to_rel": { "dx": -1, "dy": 0 } }
  }
}
```

### Adding New Exit Types
Edit `EXIT_DEFS` in `src/exits.rs`:
```rust
ExitDef { 
    id: "portal", 
    kind: ExitKind::Special, 
    aliases: &["portal", "enter portal"] 
}
```

### Adding New Commands
1. Add variant to `Command` enum in `commands.rs`
2. Handle in `parse_command()`
3. Implement handler function (e.g., `do_my_command()`)
4. Route in `execute_command()`

### Creating a GUI Frontend
```rust
use output::{MessageBuffer, OutputMode};

let mut output = MessageBuffer::new(OutputMode::BufferOnly);
execute_command(&mut state, cmd, &exit_alias_map, &mut output);

// Display in GUI
for message in output.get_messages() {
    gui_text_area.append(message);
}
```

## Design Principles

1. **Separation of Concerns**: Each module has a single, clear responsibility
2. **Format Independence**: File format separate from runtime models
3. **Output Flexibility**: Works in terminal, GUI, web, or headless contexts
4. **Type Safety**: Rust's type system prevents invalid game states
5. **Explicit Over Implicit**: Rooms must explicitly define available exits
6. **Validation First**: World is validated at load time, not runtime

## Future Enhancements

- [ ] Implement layer shifting with macrocell snapping
- [ ] Add inventory system
- [ ] Add NPC and conversation system
- [ ] Add items and object interaction
- [ ] Add save/load game state
- [ ] Create GUI frontend (egui, iced, or web)
- [ ] Add scripting for room behaviors
- [ ] Multiplayer support

## Learning Rust Notes

This project demonstrates several Rust concepts:
- Module organization and visibility
- Ownership and borrowing (passing `&mut GameState`)
- Error handling with `Result<T, E>`
- Enums and pattern matching
- Trait implementations
- JSON deserialization with Serde
- Hash maps for fast lookups
- String handling and formatting

## License

This is a learning project - feel free to use and modify as needed.
