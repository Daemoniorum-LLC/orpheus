//! # Orpheus File
//!
//! File format handling for Orpheus - .maestro, Guitar Pro, audio files.
//!
//! ## Features
//!
//! - `dialogs`: Enable native file dialogs (requires GTK/portal on Linux)
//!
//! ## Modules
//!
//! - [`dialog`]: Native file dialogs using rfd (requires `dialogs` feature)
//! - [`project`]: .maestro project file serialization
//! - [`recent`]: Recent files tracking
//! - [`guitar_pro`]: Guitar Pro file parsing
//! - [`audio`]: Audio file handling
//! - [`midi_import`]: MIDI file import and conversion to tabs

#[cfg(feature = "dialogs")]
pub mod dialog;

pub mod project;
pub mod recent;
pub mod guitar_pro;
pub mod audio;
pub mod midi;
pub mod midi_import;
pub mod pdf;

/// Result type for file operations
pub type Result<T> = std::result::Result<T, Error>;

/// File error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

// Re-export commonly used types
pub use guitar_pro::{GuitarProFile, parse_file as parse_guitar_pro, is_guitar_pro_file};
pub use audio::{
    AudioFile, AudioFormat, AudioMetadata, WaveformData, WaveformPeak,
    is_audio_file, get_audio_metadata, detect_format,
};

// Re-export dialog types (when feature enabled)
#[cfg(feature = "dialogs")]
pub use dialog::{
    FileFilter, OpenDialog, SaveDialog,
    pick_directory, pick_directory_async,
};

// Re-export project types
pub use project::{
    ProjectSerializer, ProjectFileType,
    load_project, save_as_json, load_from_json, detect_project_type,
};

// Re-export recent files types
pub use recent::{
    RecentFile, RecentFiles, RecentFilesManager,
};

// Re-export MIDI types
pub use midi::{
    export_midi, export_midi_bytes, MidiExportOptions,
};

// Re-export MIDI import types
pub use midi_import::{
    import_midi, import_midi_bytes, MidiImportOptions, MidiImportResult,
    ImportInstrumentType, FretPreference,
};

// Re-export PDF types
pub use pdf::{
    export_pdf, PdfExportOptions, PdfExportResult, PageSize,
};
