// commands.rs
//
// Command parsing and execution

use std::collections::HashMap;
use crate::models::{GameState, Room};
use crate::exits::resolve_exit_target;
use crate::output::MessageBuffer;

/// Game commands
#[derive(Debug)]
pub enum Command {
    Look,
    Where,
    Help,
    Quit,
    /// go <exit_name_or_alias>
    Go { raw: String },
    /// teleport <anchor_id>
    Teleport { anchor_id: String },
    /// shift up/down (LayerWalker only)
    Shift { dir: ShiftDir },
}

/// Direction for layer shifting
#[derive(Debug)]
pub enum ShiftDir {
    Up,
    Down,
}

/// Parse user input into a command
pub fn parse_command(line: &str) -> Option<Command> {
    let s = line.trim();
    if s.is_empty() {
        return None;
    }
    let lower = s.to_lowercase();

    // Simple tokenization MVP
    let parts: Vec<&str> = lower.split_whitespace().collect();
    let (verb, rest) = (parts[0], &parts[1..]);

    match verb {
        "look" | "l" => Some(Command::Look),
        "where" | "pos" => Some(Command::Where),
        "help" | "h" | "?" => Some(Command::Help),
        "quit" | "exit" => Some(Command::Quit),

        "go" | "move" => {
            if rest.is_empty() {
                Some(Command::Help)
            } else {
                Some(Command::Go { raw: rest.join(" ") })
            }
        }

        // Allow directional shortcut: "n", "north", "e", etc. as "go <that>"
        "n" | "north" | "s" | "south" | "e" | "east" | "w" | "west" | "u" | "up" | "d" | "down" => {
            Some(Command::Go { raw: verb.to_string() })
        }

        "teleport" | "tp" => {
            if rest.is_empty() {
                Some(Command::Help)
            } else {
                Some(Command::Teleport { anchor_id: rest[0].to_string() })
            }
        }

        "shift" => {
            // shift up/down
            if rest.is_empty() {
                Some(Command::Help)
            } else {
                match rest[0] {
                    "up" | "u" => Some(Command::Shift { dir: ShiftDir::Up }),
                    "down" | "d" => Some(Command::Shift { dir: ShiftDir::Down }),
                    _ => Some(Command::Help),
                }
            }
        }

        _ => Some(Command::Help),
    }
}

/// Execute a command, modifying game state and writing to output
pub fn execute_command(
    state: &mut GameState,
    cmd: Command,
    exit_alias_map: &HashMap<String, String>,
    output: &mut MessageBuffer,
) {
    match cmd {
        Command::Look => do_look(state, output),
        Command::Where => do_where(state, output),
        Command::Help => do_help(state, output),
        Command::Quit => state.is_running = false,
        Command::Go { raw } => do_go(state, raw, exit_alias_map, output),
        Command::Teleport { anchor_id } => do_teleport(state, &anchor_id, output),
        Command::Shift { dir } => do_shift(state, dir, output),
    }
}

fn do_look(state: &GameState, output: &mut MessageBuffer) {
    let room = current_room(state);
    output.add_blank_line();
    output.add_line(&room.name);
    output.add_line("-".repeat(room.name.len()));
    output.add_line(&room.desc);

    // Print exits (explicit exits only)
    if room.exits.is_empty() {
        output.add_blank_line();
        output.add_line("Exits: (none)");
    } else {
        let mut keys: Vec<String> = room.exits.keys().map(|k| k.to_string()).collect();
        keys.sort();
        output.add_blank_line();
        output.add_line(format!("Exits: {}", keys.join(", ")));
    }
    output.add_blank_line();
}

fn do_where(state: &GameState, output: &mut MessageBuffer) {
    let p = state.player.pos;
    let layer_info = state.world.layers.get(&p.layer);
    let layer_name = layer_info
        .map(|l| format!(" (layer: {}, scale={})", l.name, l.scale))
        .unwrap_or_default();
    
    output.add_line(format!(
        "You are at layer={}, x={}, y={}, z={}{}",
        p.layer, p.x, p.y, p.z, layer_name
    ));
}

fn do_help(state: &GameState, output: &mut MessageBuffer) {
    let class = state.player.class;
    output.add_blank_line();
    output.add_line("Commands:");
    output.add_line("  look | l");
    output.add_line("  where");
    output.add_line("  go <exit>   (or just n/s/e/w/u/d)");
    if class.can_teleport() {
        output.add_line("  teleport <anchor_id> | tp <anchor_id>");
    }
    if class.can_shift_layers() {
        output.add_line("  shift up|down");
    }
    output.add_line("  help | ?");
    output.add_line("  quit");
    output.add_blank_line();
}

fn do_go(
    state: &mut GameState,
    raw: String,
    exit_alias_map: &HashMap<String, String>,
    output: &mut MessageBuffer,
) {
    let room = current_room(state).clone();

    // Normalize input -> exit_id (via alias map); if unknown, try as-is
    let key = raw.trim().to_lowercase();
    let exit_id = exit_alias_map.get(&key).cloned().unwrap_or_else(|| key.clone());

    // Try the normalized exit_id first, then fall back to the original key
    let spec = room.exits.get(&exit_id)
        .or_else(|| room.exits.get(&key));
    
    let spec = match spec {
        Some(s) => s,
        None => {
            output.add_line(format!("You can't go that way (no exit: '{}').", raw.trim()));
            return;
        }
    };

    // Resolve destination
    let target = match resolve_exit_target(room.pos, spec) {
        Ok(t) => t,
        Err(e) => {
            output.add_line(format!("That exit seems broken: {e}"));
            return;
        }
    };

    // Apply move if target exists (should, due to validation)
    if state.world.rooms_by_coord.contains_key(&target) {
        state.player.pos = target;
        do_look(state, output);
    } else {
        output.add_line("You can't go that way (destination missing).");
    }
}

fn do_teleport(state: &mut GameState, anchor_id: &str, output: &mut MessageBuffer) {
    if !state.player.class.can_teleport() {
        output.add_line("You don't have the ability to teleport.");
        return;
    }

    let a = match state.world.anchors_by_id.get(anchor_id) {
        Some(a) => a,
        None => {
            output.add_line(format!("Unknown anchor '{}'.", anchor_id));
            return;
        }
    };

    if a.pos.layer != state.player.pos.layer {
        output.add_line(format!(
            "You can't teleport between layers (anchor is in layer={}, you are in layer={}).",
            a.pos.layer, state.player.pos.layer
        ));
        return;
    }

    state.player.pos = a.pos;
    do_look(state, output);
}

fn do_shift(state: &mut GameState, dir: ShiftDir, output: &mut MessageBuffer) {
    if !state.player.class.can_shift_layers() {
        output.add_line("You don't have the ability to shift between layers.");
        return;
    }

    // TODO: Implement layer shifting with nearest-macrocell snapping.
    // Planned behavior:
    // 1) Determine target w (w+1 or w-1)
    // 2) Map current coord -> "world space" using scale[from]
    // 3) Map "world space" -> target layer coords using scale[to] (round)
    // 4) Snap to nearest existing room in target layer (Manhattan distance)
    // 5) Move player if found, else error
    let _ = dir;
    output.add_line("(shift not implemented yet)"); // placeholder
}

/// Get the current room the player is in
fn current_room(state: &GameState) -> &Room {
    state
        .world
        .rooms_by_coord
        .get(&state.player.pos)
        .expect("Player position must always reference a valid room")
}
