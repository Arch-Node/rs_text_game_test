// multiplayer/command_queue.rs
//
// RUST LEARNING: Collections and Ordering
// - Vec<T> is a growable array (like ArrayList in Java)
// - Sorting uses Ord trait (implement or derive)
// - `std::mem::take` swaps value with default, useful for draining collections

use std::time::Instant;
use crate::commands::Command;
use super::types::{PlayerId, SessionId};
use super::validation::ValidationResult;

/// Priority levels for commands
/// 
/// RUST LEARNING: Enum with explicit discriminants
/// - We can assign numeric values to enum variants
/// - These implement Ord automatically in declaration order
/// - Higher number = higher priority when sorted
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    System = 3,
}

/// A command that has been validated and queued for execution
/// 
/// RUST LEARNING: Struct with various field types
/// - Instant for timestamps (monotonic, doesn't go backwards)
/// - Command enum from existing crate::commands
/// - ValidationResult struct from validation module
#[derive(Debug)]
pub struct QueuedCommand {
    /// When this command was queued
    pub queued_at: Instant,
    
    /// Which player issued this command
    pub player_id: PlayerId,
    
    /// Session that sent the command
    pub session_id: SessionId,
    
    /// The actual parsed command
    pub command: Command,
    
    /// Execution priority
    pub priority: CommandPriority,
    
    /// Validation result (warnings, etc.)
    pub validation: ValidationResult,
}

/// A command that failed validation
#[derive(Debug)]
pub struct FailedCommand {
    pub player_id: PlayerId,
    pub session_id: SessionId,
    pub command_text: String,
    pub reason: String,
    pub timestamp: Instant,
}

/// Command queue system
/// 
/// RUST LEARNING: Collections for different purposes
/// - `pending`: Vec for queued commands (will be sorted and executed)
/// - `priority`: Vec for high-priority commands (execute first)
/// - `failed`: Vec for logging failed attempts
#[derive(Debug, Default)]
pub struct CommandQueue {
    /// Commands waiting to execute on next tick
    pub pending: Vec<QueuedCommand>,
    
    /// Priority commands (execute first)
    pub priority: Vec<QueuedCommand>,
    
    /// Failed validations (for logging/feedback)
    pub failed: Vec<FailedCommand>,
}

impl CommandQueue {
    /// Create a new empty command queue
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a command to the queue
    /// 
    /// RUST LEARNING: Taking ownership
    /// - `command: QueuedCommand` takes ownership (not borrowing)
    /// - Command is moved into the queue
    /// - Caller can't use it anymore
    pub fn enqueue(&mut self, command: QueuedCommand) {
        match command.priority {
            CommandPriority::System | CommandPriority::High => {
                self.priority.push(command);
            }
            _ => {
                self.pending.push(command);
            }
        }
    }
    
    /// Record a failed command
    pub fn record_failure(&mut self, failed: FailedCommand) {
        self.failed.push(failed);
    }
    
    /// Get all commands ready for execution (sorted by priority)
    /// 
    /// RUST LEARNING: std::mem::take
    /// - Swaps value with default (empty Vec)
    /// - Takes ownership without requiring &mut Vec
    /// - Leaves empty Vec in place
    /// - Perfect for "draining" a collection
    pub fn drain_commands(&mut self) -> Vec<QueuedCommand> {
        // RUST LEARNING: Combine priority and normal queues
        let mut all_commands = std::mem::take(&mut self.priority);
        all_commands.append(&mut std::mem::take(&mut self.pending));
        
        // RUST LEARNING: Sorting
        // - sort_by_key takes a function that extracts sort key from each item
        // - |cmd| is a closure (anonymous function)
        // - Sorts in ascending order (Low -> Normal -> High -> System)
        all_commands.sort_by_key(|cmd| cmd.priority);
        
        // Reverse to get highest priority first
        all_commands.reverse();
        
        all_commands
    }
    
    /// Get count of pending commands
    pub fn pending_count(&self) -> usize {
        self.pending.len() + self.priority.len()
    }
    
    /// Clear all queues
    pub fn clear(&mut self) {
        self.pending.clear();
        self.priority.clear();
        self.failed.clear();
    }
    
    /// Remove old commands that have timed out
    /// 
    /// RUST LEARNING: retain method
    /// - Keeps only elements for which the predicate returns true
    /// - In-place filtering
    /// - |cmd| is closure with parameter cmd
    pub fn remove_old_commands(&mut self, timeout: std::time::Duration) {
        let now = Instant::now();
        
        self.pending.retain(|cmd| {
            now.duration_since(cmd.queued_at) < timeout
        });
        
        self.priority.retain(|cmd| {
            now.duration_since(cmd.queued_at) < timeout
        });
    }
    
    /// Get commands for a specific player
    pub fn commands_for_player(&self, player_id: PlayerId) -> Vec<&QueuedCommand> {
        // RUST LEARNING: Iterator chains
        // - chain() combines two iterators
        // - filter() keeps only matching elements
        // - collect() gathers into a collection
        self.pending
            .iter()
            .chain(self.priority.iter())
            .filter(|cmd| cmd.player_id == player_id)
            .collect()
    }
    
    /// Check if a player already has a command queued
    /// 
    /// RUST LEARNING: Iterator method `any()`
    /// - Returns true if any element matches the predicate
    /// - Short-circuits (stops as soon as it finds a match)
    /// - More efficient than collecting and checking length
    pub fn player_has_queued_command(&self, player_id: PlayerId) -> bool {
        self.pending.iter().any(|cmd| cmd.player_id == player_id)
            || self.priority.iter().any(|cmd| cmd.player_id == player_id)
    }
    
    /// Get count of commands for a specific player
    pub fn player_command_count(&self, player_id: PlayerId) -> usize {
        self.commands_for_player(player_id).len()
    }
}

// TODO: Add rate limiting (max commands per player per tick)
// NOTE: Command throttling implemented - one command per player at a time
// TODO: Add command history/logging
// TODO: Add command undo/rollback support (for testing)
