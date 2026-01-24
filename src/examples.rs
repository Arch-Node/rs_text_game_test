// Example: Using the game engine in different contexts
//
// This shows how the MessageBuffer system enables non-terminal interfaces

#[cfg(test)]
mod examples {
    use crate::models::{GameState, Player, PlayerClass};
    use crate::world::load_world_from_rooms_json;
    use crate::exits::build_exit_alias_map;
    use crate::commands::{parse_command, execute_command};
    use crate::output::{MessageBuffer, OutputMode};

    /// Example: Running the game silently for testing or automation
    #[allow(dead_code)]
    fn example_buffer_only() {
        let exit_alias_map = build_exit_alias_map();
        let world = load_world_from_rooms_json("rooms.json").unwrap();
        let player = Player {
            class: PlayerClass::LayerWalker,
            pos: world.start,
        };
        let mut state = GameState {
            world,
            player,
            is_running: true,
        };

        // Create buffer in BufferOnly mode - no terminal output
        let mut output = MessageBuffer::new(OutputMode::BufferOnly);

        // Execute commands silently
        if let Some(cmd) = parse_command("look") {
            execute_command(&mut state, cmd, &exit_alias_map, &mut output);
        }

        // Get the accumulated output
        let messages = output.get_messages();
        assert!(!messages.is_empty());
        println!("Game produced {} lines of output", messages.len());

        // Or get as a single string
        let full_text = output.get_text();
        println!("Full output:\n{}", full_text);
    }

    /// Example: GUI application could process messages for display
    #[allow(dead_code)]
    fn example_gui_processing() {
        // In a GUI app, you'd use BufferOnly mode
        let mut output = MessageBuffer::new(OutputMode::BufferOnly);
        
        // ... execute commands ...
        
        // Then retrieve and display in GUI widgets
        for message in output.get_messages() {
            // gui_text_widget.append_line(message);
            println!("GUI would display: {}", message);
        }
        
        output.clear(); // Ready for next command
    }

    /// Example: Web API could return JSON responses
    #[allow(dead_code)]
    fn example_web_api() -> String {
        let mut output = MessageBuffer::new(OutputMode::BufferOnly);
        
        // ... execute commands ...
        
        // Convert to JSON response
        let messages = output.get_messages();
        format!(r#"{{"messages": {:?}, "count": {}}}"#, messages, messages.len())
    }
}
