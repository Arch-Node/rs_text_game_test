// heartbeat.rs
//
// World heartbeat system for periodic updates (30 second tick)

use std::time::{Duration, Instant};
use crate::models::GameState;
use crate::output::MessageBuffer;

/// Heartbeat interval (30 seconds)
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Tracks when the last heartbeat occurred
#[derive(Debug)]
pub struct HeartbeatTimer {
    last_beat: Instant,
    tick_count: u64,
}

impl HeartbeatTimer {
    /// Create a new heartbeat timer starting now
    pub fn new() -> Self {
        Self {
            last_beat: Instant::now(),
            tick_count: 0,
        }
    }

    /// Check if it's time for a heartbeat, and process if needed
    /// Returns true if a heartbeat occurred
    pub fn check_and_beat(&mut self, state: &mut GameState, output: &mut MessageBuffer) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_beat);

        if elapsed >= HEARTBEAT_INTERVAL {
            self.last_beat = now;
            self.tick_count += 1;
            self.process_heartbeat(state, output);
            true
        } else {
            false
        }
    }

    /// Get the current tick count
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Get time until next heartbeat
    pub fn time_until_next(&self) -> Duration {
        let elapsed = Instant::now().duration_since(self.last_beat);
        HEARTBEAT_INTERVAL.saturating_sub(elapsed)
    }

    /// Process a single heartbeat - this is where dynamic world updates happen
    fn process_heartbeat(&self, state: &mut GameState, output: &mut MessageBuffer) {
        // TODO: Add dynamic world behavior here
        // Examples:
        // - NPCs moving between rooms
        // - Random events occurring
        // - Resource regeneration
        // - Day/night cycle updates
        // - Weather changes
        // - Multiplayer synchronization

        // For now, just a debug message (can be removed or made optional)
        if cfg!(debug_assertions) {
            output.add_line(format!("[Heartbeat #{}: World updated]", self.tick_count));
        }

        // Call world update logic
        update_world(state, output);
    }
}

impl Default for HeartbeatTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Main world update function - called every heartbeat
/// This is where you'll add dynamic world behavior
fn update_world(state: &mut GameState, _output: &mut MessageBuffer) {
    // Placeholder for future dynamic world updates
    // Examples of what you might add:
    
    // 1. Update all NPCs (when you add them)
    // for npc in &mut state.world.npcs {
    //     npc.update();
    // }
    
    // 2. Process room events
    // for room in state.world.rooms_by_coord.values_mut() {
    //     if let Some(event) = room.pending_event.take() {
    //         event.execute();
    //     }
    // }
    
    // 3. Regenerate resources
    // for room in state.world.rooms_by_coord.values_mut() {
    //     room.regenerate_resources();
    // }
    
    // 4. Advance time/weather/day-night cycle
    // state.world.time.advance(30);
    
    let _ = state; // Suppress unused warning for now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_timer_creation() {
        let timer = HeartbeatTimer::new();
        assert_eq!(timer.tick_count(), 0);
    }

    #[test]
    fn test_time_until_next() {
        let timer = HeartbeatTimer::new();
        let remaining = timer.time_until_next();
        assert!(remaining <= HEARTBEAT_INTERVAL);
        assert!(remaining.as_secs() <= 30);
    }
}
