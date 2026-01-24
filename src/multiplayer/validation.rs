// multiplayer/validation.rs
//
// RUST LEARNING: Pattern Matching and Error Handling
// - `match` is Rust's pattern matching (like switch but more powerful)
// - Must handle all enum variants (compiler enforced)
// - Can destructure enum variants to get their data
// - `if let` is shorthand for matching one pattern

use crate::commands::Command;
use crate::models::Coord;
use super::state::MultiplayerGameState;
use super::player::{Player, PlayerStatus};

/// Check if a command is instant (executes immediately without queueing)
/// 
/// RUST LEARNING: Pattern matching with exhaustive checks
/// - matches!() macro creates a boolean from pattern matching
/// - Can match multiple patterns with | (OR)
/// - Instant commands: observation, info, communication (not world-changing)
pub fn is_instant_command(command: &Command) -> bool {
    matches!(
        command,
        Command::Look | Command::Where | Command::Help
    )
}

/// Result of command validation
/// 
/// RUST LEARNING: Struct with Vec fields
/// - Vec<String> is growable array of owned strings
/// - Default trait provides `::default()` method
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Whether the command can be executed
    pub is_valid: bool,
    
    /// Error messages (why command failed)
    pub errors: Vec<String>,
    
    /// Warning messages (command can execute but with caveats)
    pub warnings: Vec<String>,
    
    /// Whether this command should execute instantly (no queueing)
    pub is_instant: bool,
}

impl ValidationResult {
    /// Create a successful validation
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            is_instant: false,
        }
    }
    
    /// Create a successful instant validation
    pub fn ok_instant() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            is_instant: true,
        }
    }
    
    /// Create a failed validation with error
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            errors: vec![message.into()],
            warnings: Vec::new(),
            is_instant: false,
        }
    }
    
    /// Add an error message
    pub fn add_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
        self.is_valid = false;
    }
    
    /// Add a warning message
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }
}

/// Validate a command before queueing
/// 
/// RUST LEARNING: Pattern Matching
/// - `match` examines the Command enum and executes code for each variant
/// - Can destructure variants: `Command::Go { raw }` extracts the raw field
/// - Underscore patterns like `Command::Look` ignore data
pub fn validate_command(
    state: &MultiplayerGameState,
    player: &Player,
    command: &Command,
) -> ValidationResult {
    // Check if this is an instant command (observation/info)
    if is_instant_command(command) {
        // Instant commands always valid and bypass status checks
        return ValidationResult::ok_instant();
    }
    
    // For queued commands, validate normally
    let mut result = ValidationResult::ok();
    
    // RUST LEARNING: Pattern matching on enum
    // Each arm of match handles a different Command variant
    match command {
        Command::Go { raw } => {
            validate_go(state, player, raw, &mut result);
        }
        
        Command::Teleport { anchor_id } => {
            validate_teleport(state, player, anchor_id, &mut result);
        }
        
        Command::Shift { dir: _ } => {
            validate_shift(state, player, &mut result);
        }
        
        Command::Look | Command::Where | Command::Help => {
            // Already handled above as instant commands
        }
        
        Command::Quit => {
            result.add_warning("You will disconnect on next tick");
        }
    }
    
    // Check player status (can't act if in combat, trading, etc.)
    if !player.can_act() {
        result.add_error(format!("Cannot act while {:?}", player.status));
    }
    
    result
}

/// Validate a Go command
fn validate_go(
    state: &MultiplayerGameState,
    player: &Player,
    exit_name: &str,
    result: &mut ValidationResult,
) {
    // Get current room
    // RUST LEARNING: Option handling with if let
    // if let Some(x) = option_value { ... } only executes if Some
    let Some(room) = state.world.rooms_by_coord.get(&player.pos) else {
        result.add_error("Current room not found");
        return;
    };
    
    // Normalize exit name (convert to lowercase, apply aliases)
    let exit_name_lower = exit_name.trim().to_lowercase();
    
    // Check if exit exists
    // RUST LEARNING: contains_key checks without borrowing the value
    if !room.exits.contains_key(&exit_name_lower) {
        result.add_error(format!("No exit '{}' in current room", exit_name));
        return;
    }
    
    // Check player status
    if player.status == PlayerStatus::InCombat {
        result.add_error("Cannot move while in combat");
    }
}

/// Validate a Teleport command
fn validate_teleport(
    state: &MultiplayerGameState,
    player: &Player,
    anchor_id: &str,
    result: &mut ValidationResult,
) {
    // Check class ability
    if !player.class.can_teleport() {
        result.add_error("Your class cannot teleport");
        return;
    }
    
    // Check if anchor exists
    let Some(anchor) = state.world.anchors_by_id.get(anchor_id) else {
        result.add_error(format!("Unknown anchor '{}'", anchor_id));
        return;
    };
    
    // Check same layer restriction
    if anchor.pos.layer != player.pos.layer {
        result.add_error("Cannot teleport between layers");
    }
}

/// Validate a Shift command
fn validate_shift(
    state: &MultiplayerGameState,
    player: &Player,
    result: &mut ValidationResult,
) {
    // Check class ability
    if !player.class.can_shift_layers() {
        result.add_error("Your class cannot shift layers");
        return;
    }
    
    // TODO: Add more validation
    // - Check if target layer exists
    // - Check if destination room exists
    // - Check if shift is on cooldown
    result.add_warning("Layer shifting not fully implemented");
}

// TODO: Add validation for future commands:
// - Combat actions
// - Trading
// - Item usage
// - Skill checks
// - Social interactions
