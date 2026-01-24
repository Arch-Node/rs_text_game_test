// main.rs
//
// Text-Based Layered World Engine
//
// Entry point and main game loop

mod models;
mod file_format;
mod exits;
mod world;
mod commands;
mod output;
mod heartbeat;

#[cfg(test)]
mod heartbeat_examples;

use std::io::{self, Write};

use models::{GameState, Player, PlayerClass};
use exits::build_exit_alias_map;
use world::load_world_from_rooms_json;
use commands::{parse_command, execute_command};
use output::{MessageBuffer, OutputMode};
use heartbeat::HeartbeatTimer;

fn prompt_line() -> io::Result<String> {
    print!("> ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf)
}

fn main() {
    // Build exit alias map once
    let exit_alias_map = build_exit_alias_map();

    // Load world from rooms.json
    // TODO: make path configurable (CLI arg); for now, fixed.
    let world = match load_world_from_rooms_json("rooms.json") {
        Ok(w) => w,
        Err(e) => {
            eprintln!("World load failed: {e}");
            std::process::exit(1);
        }
    };

    // TODO: ask user to choose class; for now, default to LayerWalker
    let player = Player {
        class: PlayerClass::LayerWalker,
        pos: world.start,
    };

    let mut state = GameState {
        world,
        player,
        is_running: true,
    };

    // Create output buffer for terminal mode
    let mut output = MessageBuffer::new(OutputMode::TerminalAndBuffer);

    // Create heartbeat timer for periodic world updates
    let mut heartbeat = HeartbeatTimer::new();

    // Intro
    println!("Welcome to {} (v{})", state.world.meta.world_name, state.world.meta.version);
    execute_command(&mut state, commands::Command::Help, &exit_alias_map, &mut output);
    execute_command(&mut state, commands::Command::Look, &exit_alias_map, &mut output);

    // Main loop
    while state.is_running {
        // Check for heartbeat and process world updates
        heartbeat.check_and_beat(&mut state, &mut output);

        let line = match prompt_line() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Input error: {e}");
                break;
            }
        };

        if let Some(cmd) = parse_command(&line) {
            // Clear previous output
            output.clear();
            
            // Execute command (output is accumulated in buffer and printed)
            execute_command(&mut state, cmd, &exit_alias_map, &mut output);
        }
    }

    println!("Goodbye.");
}
