// terminal_client/display.rs
//
// Display formatting for terminal output

use super::client::PlayerInfo;

pub struct DisplayFormatter;

impl DisplayFormatter {
    /// Format player status for display
    pub fn format_player_status(info: &PlayerInfo) -> String {
        format!(
            "┌─────────────────────────────────────────┐\n\
             │ Player: {:30} │\n\
             │ Class:  {:30} │\n\
             │ Position: ({}, {}, {}) in {:15} │\n\
             │ Status: {:30} │\n\
             └─────────────────────────────────────────┘",
            info.name,
            info.class,
            info.position_x,
            info.position_y,
            info.position_z,
            info.dimension,
            info.status
        )
    }
    
    /// Format command result
    pub fn format_command_result(success: bool, message: &str, is_queued: bool) -> String {
        let prefix = if success {
            if is_queued {
                "⏱ QUEUED"
            } else {
                "✓ OK"
            }
        } else {
            "✗ ERROR"
        };
        
        let status_info = if is_queued {
            "\n  → Command queued for next tick (15 seconds)"
        } else {
            ""
        };
        
        format!("{}: {}{}", prefix, message, status_info)
    }
    
    /// Format error message
    pub fn format_error(error: &str) -> String {
        format!("✗ ERROR: {}", error)
    }
    
    /// Format system message
    pub fn format_system(message: &str) -> String {
        format!("⚙ SYSTEM: {}", message)
    }
    
    /// Display available commands
    pub fn display_help() {
        println!("\n═══ Available Commands ═══\n");
        
        println!("Movement (queued for next tick):");
        println!("  go <direction>     - Move in a direction (north, south, east, west, up, down)");
        println!("  teleport <anchor>  - Teleport to an anchor (if you're a Teleporter)");
        println!("  shift <direction>  - Shift between dimensions (if you're a LayerWalker)\n");
        
        println!("Instant commands (execute immediately):");
        println!("  look               - Look around your current location");
        println!("  examine <target>   - Examine something in detail");
        println!("  inventory          - Check your inventory");
        println!("  status             - Check your character status");
        println!("  help               - Show this help message\n");
        
        println!("Communication:");
        println!("  say <message>      - Say something to nearby players");
        println!("  tell <player> <msg> - Send a private message\n");
        
        println!("System:");
        println!("  quit / exit        - Disconnect from game\n");
    }
    
    /// Clear screen (platform-independent)
    pub fn clear_screen() {
        print!("\x1B[2J\x1B[1;1H");
    }
}
