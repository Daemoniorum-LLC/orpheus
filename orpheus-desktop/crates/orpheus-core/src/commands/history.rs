//! Command history for undo/redo

use super::{BoxedCommand, CommandContext};
use tracing::{debug, trace};

/// Maximum number of commands to keep in history
const MAX_HISTORY_SIZE: usize = 100;

/// Command history manager
pub struct CommandHistory {
    /// Undo stack
    undo_stack: Vec<BoxedCommand>,
    /// Redo stack
    redo_stack: Vec<BoxedCommand>,
    /// Whether to merge similar commands
    merge_enabled: bool,
    /// Time of last command (for merge timeout)
    last_command_time: std::time::Instant,
    /// Merge timeout in milliseconds
    merge_timeout_ms: u64,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            merge_enabled: true,
            last_command_time: std::time::Instant::now(),
            merge_timeout_ms: 500,
        }
    }

    /// Execute a command and add it to history
    pub fn execute(&mut self, mut command: BoxedCommand, ctx: &mut CommandContext) -> crate::Result<()> {
        // Execute the command
        command.execute(ctx)?;
        debug!("Executed command: {}", command.name());

        // Add to undo stack
        self.undo_stack.push(command);

        // Trim history if too large
        while self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.remove(0);
        }

        // Clear redo stack on new command
        self.redo_stack.clear();
        self.last_command_time = std::time::Instant::now();

        Ok(())
    }

    /// Undo the last command
    pub fn undo(&mut self, ctx: &mut CommandContext) -> crate::Result<bool> {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo(ctx)?;
            debug!("Undid command: {}", command.name());
            self.redo_stack.push(command);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redo the last undone command
    pub fn redo(&mut self, ctx: &mut CommandContext) -> crate::Result<bool> {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(ctx)?;
            debug!("Redid command: {}", command.name());
            self.undo_stack.push(command);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the name of the next undo command
    pub fn undo_name(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.name())
    }

    /// Get the name of the next redo command
    pub fn redo_name(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.name())
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get undo stack size
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Enable or disable command merging
    pub fn set_merge_enabled(&mut self, enabled: bool) {
        self.merge_enabled = enabled;
    }
}
