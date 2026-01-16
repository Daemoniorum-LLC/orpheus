//! Tab Editor View
//!
//! Keyboard-driven tablature editor for virtuoso-speed composition.
//!
//! # Features
//! - Vim-like modes (Normal, Insert, Visual, Command)
//! - Number key entry for frets (0-9 with buffering for 10+)
//! - Technique shortcuts (Shift+H/P/S/B/T/etc)
//! - Duration shortcuts (W/H/Q/E/S/T)
//! - Full undo/redo support
//! - Guitar Pro file import (.gp, .gp5, .gpx, .gp4, .gp3)
//!
//! # Keyboard Reference
//!
//! ## Global Shortcuts (Ctrl+)
//! - `Ctrl+O`: Open file (Guitar Pro import)
//! - `Ctrl+S`: Save file
//! - `Ctrl+Z`: Undo
//! - `Ctrl+Y`: Redo
//! - `Ctrl+C`: Copy
//! - `Ctrl+X`: Cut
//! - `Ctrl+V`: Paste
//! - `Ctrl+M`: Add measure
//! - `Ctrl+T`: Toggle multi-track view
//! - `Ctrl+Tab`: Next track
//!
//! ## Insert Mode (default)
//! - `0-9`: Enter fret number (buffered for multi-digit)
//! - `Enter`: Commit fret or insert beat
//! - `Space`: Rest/skip
//! - `Delete/Backspace`: Delete note
//! - `Arrow keys`: Navigate
//! - `Shift+Arrow`: Navigate measures
//! - `Esc`: Switch to Normal mode
//!
//! ## Technique Shortcuts (Shift+key in Insert mode)
//! - `Shift+H`: Hammer-on
//! - `Shift+P`: Pull-off
//! - `Shift+S`: Slide
//! - `Shift+B`: Bend (cycles through amounts)
//! - `Shift+T`: Tap
//! - `Shift+N`: Natural harmonic
//! - `Shift+I`: Pinch harmonic
//! - `Shift+M`: Palm mute (cycles intensity)
//! - `Shift+V`: Vibrato
//! - `Shift+W`: Whammy/dive bomb
//! - `Shift+L`: Let ring
//!
//! ## Duration Shortcuts
//! - `W`: Whole note
//! - `H`: Half note
//! - `Q`: Quarter note
//! - `E`: Eighth note
//! - `S`: Sixteenth note
//! - `T`: Thirty-second note
//! - `.`: Toggle dotted
//! - `Alt+3`: Toggle triplet

mod state;
mod view;
mod input;
mod render;
mod stage_view;
mod section_analysis;
mod track_groups;

pub use state::{
    TabEditorState, TabCursor, TabSelection, TabClipboard,
    EditorMode, FretBuffer, ActiveTool, HarmonicTool,
    PendingTabAction, PracticeMode, TabPlaybackEvent, TabPlaybackEventType,
    TrackGroup,
};
pub use view::{TabEditorView, TabEditorAction};
pub use input::InputResult;
pub use stage_view::{StageView, StageViewState, StageViewAction, StagePosition, instrument_templates};
pub use section_analysis::{
    SectionAnalysisState, SectionAnalysisDialog, SectionAnalysisAction,
    SectionNavigator, DetectedSection, SectionType, analyze_sections,
};
pub use track_groups::{TrackGroupsPanel, TrackGroupsPanelState, TrackGroupAction};
pub use self::open_file_dialog as open_gp_dialog;
pub use render::{
    TOOLBAR_HEIGHT, STRING_HEIGHT, FRET_WIDTH, MEASURE_HEADER_HEIGHT,
    STATUS_HEIGHT, TRACK_LABEL_WIDTH,
    technique_symbol, tool_indicator, fret_color,
};

/// Open a file dialog and load a Guitar Pro or Maestro file
///
/// Returns `Some(TabEditorState)` if a file was selected and parsed successfully,
/// `None` if the dialog was cancelled or an error occurred.
///
/// Supported formats:
/// - .maestro (Orpheus Tab format)
/// - .gp, .gpx (Guitar Pro 6/7)
/// - .gp5 (Guitar Pro 5)
/// - .gp4, .gp3 (Guitar Pro 3/4)
pub fn open_file_dialog() -> Option<TabEditorState> {
    use orpheus_file::{OpenDialog, FileFilter};

    // Create dialog with tab file filters
    let dialog = OpenDialog::new()
        .title("Open Tab File")
        .filter(FileFilter::new("Orpheus Tab").add_extension("maestro"))
        .filter(FileFilter::guitar_pro())
        .start_directory(dirs::document_dir());

    // Show file dialog (returns Vec<PathBuf>)
    let paths = dialog.show()?;
    let path = paths.into_iter().next()?;

    // Load based on extension
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())?;

    match ext.as_str() {
        // Orpheus native format
        "maestro" => {
            match TabEditorState::load_from_path(&path) {
                Ok(state) => {
                    tracing::info!("Loaded Maestro file: {:?}", path);
                    Some(state)
                }
                Err(e) => {
                    tracing::error!("Failed to load Maestro file {:?}: {}", path, e);
                    None
                }
            }
        }
        // Guitar Pro formats
        "gp" | "gp3" | "gp4" | "gp5" | "gpx" => {
            match TabEditorState::from_gp_file(&path) {
                Ok(state) => {
                    // Note: GP files load without a file_path since they need Save As
                    tracing::info!("Loaded GP file: {:?}", path);
                    Some(state)
                }
                Err(e) => {
                    tracing::error!("Failed to load GP file {:?}: {}", path, e);
                    None
                }
            }
        }
        _ => {
            tracing::warn!("Unsupported file extension: {}", ext);
            None
        }
    }
}

/// Show a save file dialog and return the selected path
///
/// Returns `Some(PathBuf)` if a path was selected, `None` if cancelled.
pub fn save_file_dialog(default_name: Option<&str>) -> Option<std::path::PathBuf> {
    use orpheus_file::{SaveDialog, FileFilter};

    let name = default_name.unwrap_or("Untitled");

    let dialog = SaveDialog::new()
        .title("Save Tab File")
        .filter(FileFilter::new("Orpheus Tab").add_extension("maestro"))
        .default_name(format!("{}.maestro", name))
        .start_directory(dirs::document_dir());

    dialog.show()
}

/// Show a MIDI export dialog and export the document
///
/// Returns `true` if export was successful, `false` if cancelled or failed.
pub fn export_midi_dialog(state: &TabEditorState) -> bool {
    use orpheus_file::{SaveDialog, FileFilter, export_midi, MidiExportOptions};

    let name = state.title().replace("*", "");

    let dialog = SaveDialog::new()
        .title("Export MIDI")
        .filter(FileFilter::midi())
        .default_name(format!("{}.mid", name))
        .start_directory(dirs::document_dir());

    if let Some(path) = dialog.show() {
        let options = MidiExportOptions::default();
        match export_midi(&state.document, &path, &options) {
            Ok(()) => {
                tracing::info!("Exported MIDI to: {:?}", path);
                true
            }
            Err(e) => {
                tracing::error!("Failed to export MIDI: {}", e);
                false
            }
        }
    } else {
        false
    }
}
