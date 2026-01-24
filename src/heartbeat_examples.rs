// Example: Testing heartbeat system
//
// Shows how the heartbeat can be used for world updates

#[cfg(test)]
mod heartbeat_examples {
    use crate::heartbeat::HeartbeatTimer;
    use crate::models::{GameState, Player, PlayerClass};
    use crate::world::load_world_from_rooms_json;
    use crate::output::{MessageBuffer, OutputMode};

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn example_wait_for_heartbeat() {
        use std::thread;
        use std::time::Duration;

        // Set up game
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

        let mut output = MessageBuffer::new(OutputMode::BufferOnly);
        let mut heartbeat = HeartbeatTimer::new();

        println!("Waiting 31 seconds for heartbeat...");
        
        // Wait for heartbeat to trigger
        thread::sleep(Duration::from_secs(31));
        
        let beat_occurred = heartbeat.check_and_beat(&mut state, &mut output);
        assert!(beat_occurred, "Heartbeat should have occurred after 31 seconds");
        
        println!("Heartbeat triggered successfully!");
        println!("Tick count: {}", heartbeat.tick_count());
    }

    #[test]
    fn example_immediate_check() {
        // Set up game
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

        let mut output = MessageBuffer::new(OutputMode::BufferOnly);
        let mut heartbeat = HeartbeatTimer::new();

        // Check immediately - should not trigger
        let beat_occurred = heartbeat.check_and_beat(&mut state, &mut output);
        assert!(!beat_occurred, "Heartbeat should not occur immediately");
        assert_eq!(heartbeat.tick_count(), 0);
    }
}

/// Example of what dynamic world updates might look like in the future
#[allow(dead_code)]
mod future_examples {
    use crate::models::GameState;
    use crate::output::MessageBuffer;

    /// Example: NPC movement during heartbeat
    #[allow(dead_code)]
    fn example_npc_movement(_state: &mut GameState, _output: &mut MessageBuffer) {
        // Future implementation:
        // for npc in &mut state.world.npcs {
        //     if npc.should_move() {
        //         let new_pos = npc.calculate_next_position();
        //         npc.move_to(new_pos);
        //         if npc.pos == state.player.pos {
        //             output.add_line(format!("{} enters the room.", npc.name));
        //         }
        //     }
        // }
    }

    /// Example: Time-based events
    #[allow(dead_code)]
    fn example_timed_events(_state: &mut GameState, _output: &mut MessageBuffer) {
        // Future implementation:
        // state.world.game_time += 30; // seconds
        // 
        // if state.world.game_time % 3600 == 0 {
        //     // Every hour
        //     output.add_line("You hear a clock chime in the distance.");
        // }
        //
        // if state.world.is_night() {
        //     // Night-time events
        //     for room in state.world.rooms_by_coord.values_mut() {
        //         if room.has_tag("outdoor") {
        //             room.description_modifier = "It's dark outside.";
        //         }
        //     }
        // }
    }

    /// Example: Resource regeneration
    #[allow(dead_code)]
    fn example_resource_regen(_state: &mut GameState, _output: &mut MessageBuffer) {
        // Future implementation:
        // for room in state.world.rooms_by_coord.values_mut() {
        //     if let Some(resource) = &mut room.resource {
        //         resource.regenerate(5); // Regenerate 5 units per heartbeat
        //         if resource.is_full() && state.player.pos == room.pos {
        //             output.add_line(format!("{} has fully regenerated.", resource.name));
        //         }
        //     }
        // }
    }
}
