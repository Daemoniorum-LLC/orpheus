//! Command registry for keyboard shortcuts and command palette

use std::collections::HashMap;

/// Information about a registered command
#[derive(Debug, Clone)]
pub struct CommandInfo {
    /// Unique command ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category for grouping
    pub category: CommandCategory,
    /// Keyboard shortcut (if any)
    pub shortcut: Option<String>,
}

/// Command categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    File,
    Edit,
    View,
    Transport,
    Track,
    Mix,
    Tools,
    Help,
}

impl CommandCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Transport => "Transport",
            Self::Track => "Track",
            Self::Mix => "Mix",
            Self::Tools => "Tools",
            Self::Help => "Help",
        }
    }
}

/// Registry of available commands
pub struct CommandRegistry {
    commands: HashMap<String, CommandInfo>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Register default commands
    fn register_defaults(&mut self) {
        // File commands
        self.register(CommandInfo {
            id: "file.new".into(),
            name: "New Project".into(),
            description: "Create a new project".into(),
            category: CommandCategory::File,
            shortcut: Some("Ctrl+N".into()),
        });
        self.register(CommandInfo {
            id: "file.open".into(),
            name: "Open Project".into(),
            description: "Open an existing project".into(),
            category: CommandCategory::File,
            shortcut: Some("Ctrl+O".into()),
        });
        self.register(CommandInfo {
            id: "file.save".into(),
            name: "Save".into(),
            description: "Save the current project".into(),
            category: CommandCategory::File,
            shortcut: Some("Ctrl+S".into()),
        });
        self.register(CommandInfo {
            id: "file.save_as".into(),
            name: "Save As...".into(),
            description: "Save the project with a new name".into(),
            category: CommandCategory::File,
            shortcut: Some("Ctrl+Shift+S".into()),
        });

        // Edit commands
        self.register(CommandInfo {
            id: "edit.undo".into(),
            name: "Undo".into(),
            description: "Undo the last action".into(),
            category: CommandCategory::Edit,
            shortcut: Some("Ctrl+Z".into()),
        });
        self.register(CommandInfo {
            id: "edit.redo".into(),
            name: "Redo".into(),
            description: "Redo the last undone action".into(),
            category: CommandCategory::Edit,
            shortcut: Some("Ctrl+Shift+Z".into()),
        });

        // Transport commands
        self.register(CommandInfo {
            id: "transport.play".into(),
            name: "Play/Pause".into(),
            description: "Toggle playback".into(),
            category: CommandCategory::Transport,
            shortcut: Some("Space".into()),
        });
        self.register(CommandInfo {
            id: "transport.stop".into(),
            name: "Stop".into(),
            description: "Stop playback and return to start".into(),
            category: CommandCategory::Transport,
            shortcut: Some("Enter".into()),
        });
        self.register(CommandInfo {
            id: "transport.record".into(),
            name: "Record".into(),
            description: "Start/stop recording".into(),
            category: CommandCategory::Transport,
            shortcut: Some("R".into()),
        });

        // View commands
        self.register(CommandInfo {
            id: "view.command_palette".into(),
            name: "Command Palette".into(),
            description: "Open command palette".into(),
            category: CommandCategory::View,
            shortcut: Some("Ctrl+Shift+P".into()),
        });
        self.register(CommandInfo {
            id: "view.toggle_left_panel".into(),
            name: "Toggle Left Panel".into(),
            description: "Show/hide left panel".into(),
            category: CommandCategory::View,
            shortcut: Some("Ctrl+B".into()),
        });
    }

    /// Register a command
    pub fn register(&mut self, info: CommandInfo) {
        self.commands.insert(info.id.clone(), info);
    }

    /// Get command by ID
    pub fn get(&self, id: &str) -> Option<&CommandInfo> {
        self.commands.get(id)
    }

    /// Get all commands
    pub fn all(&self) -> impl Iterator<Item = &CommandInfo> {
        self.commands.values()
    }

    /// Get commands by category
    pub fn by_category(&self, category: CommandCategory) -> Vec<&CommandInfo> {
        self.commands
            .values()
            .filter(|c| c.category == category)
            .collect()
    }

    /// Search commands by name/description
    pub fn search(&self, query: &str) -> Vec<&CommandInfo> {
        let query = query.to_lowercase();
        self.commands
            .values()
            .filter(|c| {
                c.name.to_lowercase().contains(&query)
                    || c.description.to_lowercase().contains(&query)
                    || c.id.to_lowercase().contains(&query)
            })
            .collect()
    }
}
