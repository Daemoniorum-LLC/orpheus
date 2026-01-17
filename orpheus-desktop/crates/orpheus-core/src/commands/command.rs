//! Command trait and context

use crate::{AppState, MixerState, Project, TransportState};
use std::any::Any;

/// Context provided to commands for execution
pub struct CommandContext<'a> {
    /// Project data
    pub project: &'a mut Project,
    /// Application state
    pub state: &'a mut AppState,
    /// Transport state
    pub transport: &'a mut TransportState,
    /// Mixer state
    pub mixer: &'a mut MixerState,
}

/// A command that can be executed, undone, and potentially merged
pub trait Command: Send + Sync {
    /// Display name for the command (shown in Edit menu)
    fn name(&self) -> &str;

    /// Execute the command
    fn execute(&mut self, ctx: &mut CommandContext) -> crate::Result<()>;

    /// Undo the command
    fn undo(&mut self, ctx: &mut CommandContext) -> crate::Result<()>;

    /// Whether this command can be merged with another
    fn can_merge(&self, _other: &dyn Command) -> bool {
        false
    }

    /// Merge another command into this one (for coalescing rapid changes)
    fn merge(&mut self, _other: Box<dyn Command>) -> bool {
        false
    }

    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Cast to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Boxed command type
pub type BoxedCommand = Box<dyn Command>;
