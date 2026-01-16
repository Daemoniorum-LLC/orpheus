//! Native file dialogs using rfd
//!
//! Provides a high-level API for file open/save dialogs with preset filters
//! for common music production file types.

use std::path::PathBuf;
use rfd::{AsyncFileDialog, FileDialog};

/// File filter with extensions
#[derive(Debug, Clone)]
pub struct FileFilter {
    /// Display name
    name: String,
    /// File extensions (without dot)
    extensions: Vec<String>,
}

impl FileFilter {
    /// Create a new file filter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            extensions: Vec::new(),
        }
    }

    /// Add an extension to the filter
    pub fn add_extension(mut self, ext: impl Into<String>) -> Self {
        self.extensions.push(ext.into());
        self
    }

    /// Get filter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get extensions
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }

    // Preset filters

    /// Maestro project files (.maestro, .mst)
    pub fn maestro() -> Self {
        Self::new("Maestro Project")
            .add_extension("maestro")
            .add_extension("mst")
    }

    /// Guitar Pro files (.gp, .gp5, .gpx, .gp4, .gp3)
    pub fn guitar_pro() -> Self {
        Self::new("Guitar Pro")
            .add_extension("gp")
            .add_extension("gp5")
            .add_extension("gpx")
            .add_extension("gp4")
            .add_extension("gp3")
    }

    /// Audio files (.wav, .flac, .mp3, .ogg)
    pub fn audio() -> Self {
        Self::new("Audio Files")
            .add_extension("wav")
            .add_extension("flac")
            .add_extension("mp3")
            .add_extension("ogg")
    }

    /// MIDI files (.mid, .midi)
    pub fn midi() -> Self {
        Self::new("MIDI Files")
            .add_extension("mid")
            .add_extension("midi")
    }

    /// All supported project files
    pub fn all_projects() -> Self {
        Self::new("All Projects")
            .add_extension("maestro")
            .add_extension("mst")
            .add_extension("gp")
            .add_extension("gp5")
            .add_extension("gpx")
    }

    /// All supported files
    pub fn all_supported() -> Self {
        Self::new("All Supported Files")
            .add_extension("maestro")
            .add_extension("mst")
            .add_extension("gp")
            .add_extension("gp5")
            .add_extension("gpx")
            .add_extension("wav")
            .add_extension("flac")
            .add_extension("mp3")
            .add_extension("mid")
            .add_extension("midi")
    }
}

/// Open file dialog builder
#[derive(Debug, Clone)]
pub struct OpenDialog {
    /// Dialog title
    title: String,
    /// File filters
    filters: Vec<FileFilter>,
    /// Starting directory
    start_dir: Option<PathBuf>,
    /// Allow multiple file selection
    multiple: bool,
}

impl Default for OpenDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenDialog {
    /// Create a new open dialog
    pub fn new() -> Self {
        Self {
            title: "Open File".into(),
            filters: Vec::new(),
            start_dir: None,
            multiple: false,
        }
    }

    /// Set dialog title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Add a file filter
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set starting directory
    pub fn start_directory(mut self, dir: Option<PathBuf>) -> Self {
        self.start_dir = dir;
        self
    }

    /// Allow multiple file selection
    pub fn multiple(mut self, allow: bool) -> Self {
        self.multiple = allow;
        self
    }

    /// Get configured filters
    pub fn filters(&self) -> &[FileFilter] {
        &self.filters
    }

    /// Show the dialog synchronously and return selected file(s)
    pub fn show(self) -> Option<Vec<PathBuf>> {
        let mut dialog = FileDialog::new().set_title(&self.title);

        // Add filters
        for filter in &self.filters {
            let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &exts);
        }

        // Set starting directory
        if let Some(dir) = self.start_dir {
            dialog = dialog.set_directory(dir);
        }

        // Show dialog
        if self.multiple {
            dialog.pick_files()
        } else {
            dialog.pick_file().map(|p| vec![p])
        }
    }

    /// Show the dialog asynchronously
    pub async fn show_async(self) -> Option<Vec<PathBuf>> {
        let mut dialog = AsyncFileDialog::new().set_title(&self.title);

        // Add filters
        for filter in &self.filters {
            let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &exts);
        }

        // Set starting directory
        if let Some(dir) = self.start_dir {
            dialog = dialog.set_directory(dir);
        }

        // Show dialog
        if self.multiple {
            let handles = dialog.pick_files().await?;
            Some(handles.into_iter().map(|h| h.path().to_path_buf()).collect())
        } else {
            let handle = dialog.pick_file().await?;
            Some(vec![handle.path().to_path_buf()])
        }
    }

    /// Convenience: Open project dialog
    pub fn project() -> Self {
        Self::new()
            .title("Open Project")
            .filter(FileFilter::maestro())
            .filter(FileFilter::guitar_pro())
            .start_directory(dirs::document_dir())
    }

    /// Convenience: Import audio dialog
    pub fn import_audio() -> Self {
        Self::new()
            .title("Import Audio")
            .filter(FileFilter::audio())
            .multiple(true)
            .start_directory(dirs::audio_dir().or_else(dirs::document_dir))
    }

    /// Convenience: Import MIDI dialog
    pub fn import_midi() -> Self {
        Self::new()
            .title("Import MIDI")
            .filter(FileFilter::midi())
            .start_directory(dirs::document_dir())
    }
}

/// Save file dialog builder
#[derive(Debug, Clone)]
pub struct SaveDialog {
    /// Dialog title
    title: String,
    /// File filters
    filters: Vec<FileFilter>,
    /// Starting directory
    start_dir: Option<PathBuf>,
    /// Default file name
    default_name: Option<String>,
}

impl Default for SaveDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveDialog {
    /// Create a new save dialog
    pub fn new() -> Self {
        Self {
            title: "Save File".into(),
            filters: Vec::new(),
            start_dir: None,
            default_name: None,
        }
    }

    /// Set dialog title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Add a file filter
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set starting directory
    pub fn start_directory(mut self, dir: Option<PathBuf>) -> Self {
        self.start_dir = dir;
        self
    }

    /// Set default file name
    pub fn default_name(mut self, name: impl Into<String>) -> Self {
        self.default_name = Some(name.into());
        self
    }

    /// Get configured filters
    pub fn filters(&self) -> &[FileFilter] {
        &self.filters
    }

    /// Show the dialog synchronously
    pub fn show(self) -> Option<PathBuf> {
        let mut dialog = FileDialog::new().set_title(&self.title);

        // Add filters
        for filter in &self.filters {
            let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &exts);
        }

        // Set starting directory
        if let Some(dir) = self.start_dir {
            dialog = dialog.set_directory(dir);
        }

        // Set default filename
        if let Some(name) = self.default_name {
            dialog = dialog.set_file_name(&name);
        }

        dialog.save_file()
    }

    /// Show the dialog asynchronously
    pub async fn show_async(self) -> Option<PathBuf> {
        let mut dialog = AsyncFileDialog::new().set_title(&self.title);

        // Add filters
        for filter in &self.filters {
            let exts: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &exts);
        }

        // Set starting directory
        if let Some(dir) = self.start_dir {
            dialog = dialog.set_directory(dir);
        }

        // Set default filename
        if let Some(name) = self.default_name {
            dialog = dialog.set_file_name(&name);
        }

        let handle = dialog.save_file().await?;
        Some(handle.path().to_path_buf())
    }

    /// Convenience: Save project dialog
    pub fn project(project_name: &str) -> Self {
        Self::new()
            .title("Save Project")
            .filter(FileFilter::maestro())
            .default_name(format!("{}.maestro", project_name))
            .start_directory(dirs::document_dir())
    }

    /// Convenience: Export audio dialog
    pub fn export_audio(default_name: &str) -> Self {
        Self::new()
            .title("Export Audio")
            .filter(FileFilter::new("WAV Audio").add_extension("wav"))
            .filter(FileFilter::new("FLAC Audio").add_extension("flac"))
            .default_name(format!("{}.wav", default_name))
            .start_directory(dirs::audio_dir().or_else(dirs::document_dir))
    }

    /// Convenience: Export MIDI dialog
    pub fn export_midi(default_name: &str) -> Self {
        Self::new()
            .title("Export MIDI")
            .filter(FileFilter::midi())
            .default_name(format!("{}.mid", default_name))
            .start_directory(dirs::document_dir())
    }
}

/// Pick a directory
pub fn pick_directory(title: &str, start_dir: Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = FileDialog::new().set_title(title);
    if let Some(dir) = start_dir {
        dialog = dialog.set_directory(dir);
    }
    dialog.pick_folder()
}

/// Pick a directory asynchronously
pub async fn pick_directory_async(title: &str, start_dir: Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = AsyncFileDialog::new().set_title(title);
    if let Some(dir) = start_dir {
        dialog = dialog.set_directory(dir);
    }
    let handle = dialog.pick_folder().await?;
    Some(handle.path().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_filter_creation() {
        let filter = FileFilter::new("Maestro Project")
            .add_extension("maestro")
            .add_extension("mst");

        assert_eq!(filter.name(), "Maestro Project");
        assert_eq!(filter.extensions(), &["maestro", "mst"]);
    }

    #[test]
    fn test_file_filter_presets() {
        let maestro = FileFilter::maestro();
        assert_eq!(maestro.extensions(), &["maestro", "mst"]);

        let gp = FileFilter::guitar_pro();
        assert!(gp.extensions().contains(&"gp5".to_string()));
        assert!(gp.extensions().contains(&"gpx".to_string()));

        let audio = FileFilter::audio();
        assert!(audio.extensions().contains(&"wav".to_string()));
        assert!(audio.extensions().contains(&"flac".to_string()));

        let midi = FileFilter::midi();
        assert!(midi.extensions().contains(&"mid".to_string()));
    }

    #[test]
    fn test_open_dialog_config() {
        let dialog = OpenDialog::new()
            .title("Open Project")
            .filter(FileFilter::maestro())
            .filter(FileFilter::guitar_pro())
            .start_directory(dirs::document_dir());

        assert_eq!(dialog.filters().len(), 2);
    }

    #[test]
    fn test_open_dialog_multiple() {
        let dialog = OpenDialog::new()
            .title("Import Audio")
            .filter(FileFilter::audio())
            .multiple(true);

        assert!(dialog.multiple);
        assert_eq!(dialog.filters().len(), 1);
    }

    #[test]
    fn test_save_dialog_config() {
        let dialog = SaveDialog::new()
            .title("Save Project")
            .filter(FileFilter::maestro())
            .default_name("My Song.maestro");

        assert_eq!(dialog.filters().len(), 1);
        assert_eq!(dialog.default_name, Some("My Song.maestro".to_string()));
    }

    #[test]
    fn test_convenience_dialogs() {
        let open_project = OpenDialog::project();
        assert_eq!(open_project.filters().len(), 2);

        let import_audio = OpenDialog::import_audio();
        assert!(import_audio.multiple);

        let save_project = SaveDialog::project("Test Song");
        assert!(save_project.default_name.unwrap().contains("Test Song"));

        let export_audio = SaveDialog::export_audio("Mix");
        assert_eq!(export_audio.filters().len(), 2);
    }

    #[test]
    fn test_all_supported_filter() {
        let all = FileFilter::all_supported();
        assert!(all.extensions().len() >= 10);
        assert!(all.extensions().contains(&"maestro".to_string()));
        assert!(all.extensions().contains(&"wav".to_string()));
        assert!(all.extensions().contains(&"mid".to_string()));
    }
}
