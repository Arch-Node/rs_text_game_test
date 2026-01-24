// terminal_client/input.rs
//
// Input handling for terminal client

use std::io::{self, Write};

pub struct InputHandler;

impl InputHandler {
    /// Prompt for a line of input
    pub fn prompt() -> io::Result<String> {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_string())
    }
    
    /// Check if command is a quit command
    pub fn is_quit_command(input: &str) -> bool {
        matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q")
    }
    
    /// Check if command is a help command
    pub fn is_help_command(input: &str) -> bool {
        matches!(input.to_lowercase().as_str(), "help" | "?" | "h")
    }
    
    /// Check if command is a status command
    pub fn is_status_command(input: &str) -> bool {
        matches!(input.to_lowercase().as_str(), "status" | "stat" | "stats")
    }
}
