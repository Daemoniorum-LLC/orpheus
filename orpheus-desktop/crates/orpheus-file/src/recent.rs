//! Recent files tracking
//!
//! Maintains a list of recently opened files with metadata.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;

/// Maximum number of recent files to track
const MAX_RECENT_FILES: usize = 20;

/// A recent file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    /// File path
    pub path: PathBuf,
    /// Display name (file name without extension)
    pub name: String,
    /// Last opened timestamp
    pub last_opened: DateTime<Utc>,
    /// Number of times opened
    pub open_count: u32,
    /// Whether the file still exists
    #[serde(skip)]
    pub exists: bool,
}

impl RecentFile {
    /// Create a new recent file entry
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Self {
            exists: path.exists(),
            path,
            name,
            last_opened: Utc::now(),
            open_count: 1,
        }
    }

    /// Update the entry (called when file is opened again)
    pub fn touch(&mut self) {
        self.last_opened = Utc::now();
        self.open_count += 1;
        self.exists = self.path.exists();
    }

    /// Check if file still exists
    pub fn refresh_exists(&mut self) {
        self.exists = self.path.exists();
    }
}

/// Recent files list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFiles {
    /// List of recent files (most recent first)
    files: Vec<RecentFile>,
    /// Maximum entries to keep
    #[serde(skip, default = "default_max_entries")]
    max_entries: usize,
}

fn default_max_entries() -> usize {
    MAX_RECENT_FILES
}

impl Default for RecentFiles {
    fn default() -> Self {
        Self::new()
    }
}

impl RecentFiles {
    /// Create a new empty recent files list
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            max_entries: MAX_RECENT_FILES,
        }
    }

    /// Create with custom max entries
    pub fn with_max_entries(max: usize) -> Self {
        Self {
            files: Vec::new(),
            max_entries: max,
        }
    }

    /// Add or update a file in the recent list
    pub fn add(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();

        // Check if already in list
        if let Some(pos) = self.files.iter().position(|f| f.path == path) {
            // Update and move to front
            self.files[pos].touch();
            let entry = self.files.remove(pos);
            self.files.insert(0, entry);
        } else {
            // Add new entry at front
            self.files.insert(0, RecentFile::new(path));
        }

        // Trim to max size
        self.files.truncate(self.max_entries);
    }

    /// Remove a file from the list
    pub fn remove(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        self.files.retain(|f| f.path != path);
    }

    /// Clear all recent files
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// Get all recent files
    pub fn files(&self) -> &[RecentFile] {
        &self.files
    }

    /// Get existing files only
    pub fn existing_files(&self) -> Vec<&RecentFile> {
        self.files.iter().filter(|f| f.exists).collect()
    }

    /// Get file count
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Refresh existence status for all files
    pub fn refresh(&mut self) {
        for file in &mut self.files {
            file.refresh_exists();
        }
    }

    /// Remove files that no longer exist
    pub fn prune_missing(&mut self) {
        self.refresh();
        self.files.retain(|f| f.exists);
    }

    /// Get the most recently opened file
    pub fn most_recent(&self) -> Option<&RecentFile> {
        self.files.first()
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load from file
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut recent: RecentFiles = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        recent.max_entries = MAX_RECENT_FILES;
        recent.refresh();
        Ok(recent)
    }

    /// Load or create new
    pub fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_default()
    }
}

/// Thread-safe recent files manager
pub struct RecentFilesManager {
    /// Recent files data
    data: RwLock<RecentFiles>,
    /// Path to persist recent files
    persist_path: Option<PathBuf>,
}

impl RecentFilesManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            data: RwLock::new(RecentFiles::new()),
            persist_path: None,
        }
    }

    /// Create with persistence path
    pub fn with_persistence(path: PathBuf) -> Self {
        let data = RecentFiles::load_or_default(&path);
        Self {
            data: RwLock::new(data),
            persist_path: Some(path),
        }
    }

    /// Create with default persistence (in config directory)
    pub fn with_default_persistence() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("orpheus");
            let _ = std::fs::create_dir_all(&app_dir);
            let path = app_dir.join("recent_files.json");
            Self::with_persistence(path)
        } else {
            Self::new()
        }
    }

    /// Add a file
    pub fn add(&self, path: impl AsRef<Path>) {
        self.data.write().add(path);
        self.save();
    }

    /// Remove a file
    pub fn remove(&self, path: impl AsRef<Path>) {
        self.data.write().remove(path);
        self.save();
    }

    /// Clear all
    pub fn clear(&self) {
        self.data.write().clear();
        self.save();
    }

    /// Get files (cloned)
    pub fn files(&self) -> Vec<RecentFile> {
        self.data.read().files.clone()
    }

    /// Get existing files only
    pub fn existing_files(&self) -> Vec<RecentFile> {
        self.data.read().existing_files().into_iter().cloned().collect()
    }

    /// Get count
    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }

    /// Get most recent
    pub fn most_recent(&self) -> Option<RecentFile> {
        self.data.read().most_recent().cloned()
    }

    /// Refresh and prune
    pub fn refresh(&self) {
        self.data.write().refresh();
    }

    /// Prune missing files
    pub fn prune_missing(&self) {
        self.data.write().prune_missing();
        self.save();
    }

    /// Save to disk (called automatically on modifications)
    fn save(&self) {
        if let Some(ref path) = self.persist_path {
            let _ = self.data.read().save(path);
        }
    }

    /// Explicitly flush to disk
    pub fn flush(&self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.persist_path {
            self.data.read().save(path)
        } else {
            Ok(())
        }
    }
}

impl Default for RecentFilesManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_recent_file_creation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.maestro");
        std::fs::write(&path, b"test").unwrap();

        let recent = RecentFile::new(path.clone());
        assert_eq!(recent.name, "test");
        assert!(recent.exists);
        assert_eq!(recent.open_count, 1);
    }

    #[test]
    fn test_add_recent_file() {
        let mut recent = RecentFiles::new();

        recent.add("/path/to/file1.maestro");
        recent.add("/path/to/file2.maestro");

        assert_eq!(recent.len(), 2);
        // Most recent should be first
        assert_eq!(recent.files()[0].path, PathBuf::from("/path/to/file2.maestro"));
    }

    #[test]
    fn test_update_existing() {
        let mut recent = RecentFiles::new();

        recent.add("/path/to/file.maestro");
        recent.add("/path/to/other.maestro");
        recent.add("/path/to/file.maestro"); // Re-add first file

        assert_eq!(recent.len(), 2);
        // Re-added file should be first
        assert_eq!(recent.files()[0].path, PathBuf::from("/path/to/file.maestro"));
        // Open count should be 2
        assert_eq!(recent.files()[0].open_count, 2);
    }

    #[test]
    fn test_max_entries() {
        let mut recent = RecentFiles::with_max_entries(5);

        for i in 0..10 {
            recent.add(format!("/path/to/file{}.maestro", i));
        }

        assert_eq!(recent.len(), 5);
        // Most recent (9) should be first
        assert_eq!(recent.files()[0].path, PathBuf::from("/path/to/file9.maestro"));
    }

    #[test]
    fn test_remove() {
        let mut recent = RecentFiles::new();

        recent.add("/path/to/file1.maestro");
        recent.add("/path/to/file2.maestro");
        recent.remove("/path/to/file1.maestro");

        assert_eq!(recent.len(), 1);
        assert_eq!(recent.files()[0].path, PathBuf::from("/path/to/file2.maestro"));
    }

    #[test]
    fn test_clear() {
        let mut recent = RecentFiles::new();

        recent.add("/path/to/file1.maestro");
        recent.add("/path/to/file2.maestro");
        recent.clear();

        assert!(recent.is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let save_path = dir.path().join("recent.json");

        let mut recent = RecentFiles::new();
        recent.add("/path/to/file1.maestro");
        recent.add("/path/to/file2.maestro");
        recent.save(&save_path).unwrap();

        let loaded = RecentFiles::load(&save_path).unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_most_recent() {
        let mut recent = RecentFiles::new();

        assert!(recent.most_recent().is_none());

        recent.add("/path/to/file.maestro");
        assert!(recent.most_recent().is_some());
        assert_eq!(recent.most_recent().unwrap().path, PathBuf::from("/path/to/file.maestro"));
    }

    #[test]
    fn test_manager_basic() {
        let manager = RecentFilesManager::new();

        manager.add("/path/to/file.maestro");
        assert_eq!(manager.len(), 1);

        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_manager_persistence() {
        let dir = tempdir().unwrap();
        let persist_path = dir.path().join("recent.json");

        // Create and populate
        {
            let manager = RecentFilesManager::with_persistence(persist_path.clone());
            manager.add("/path/to/file1.maestro");
            manager.add("/path/to/file2.maestro");
            assert_eq!(manager.len(), 2, "Manager should have 2 files before flush");
            manager.flush().expect("Failed to flush recent files");
        }

        // Verify file was created
        assert!(persist_path.exists(), "Recent files not saved to disk");

        // Load again via manager
        let manager = RecentFilesManager::with_persistence(persist_path);
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_existing_files() {
        let dir = tempdir().unwrap();
        let existing = dir.path().join("exists.maestro");
        std::fs::write(&existing, b"test").unwrap();

        let mut recent = RecentFiles::new();
        recent.add(&existing);
        recent.add("/path/to/nonexistent.maestro");

        assert_eq!(recent.len(), 2);
        assert_eq!(recent.existing_files().len(), 1);
    }
}
