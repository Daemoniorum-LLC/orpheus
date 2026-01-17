//! Project file serialization (.maestro format)
//!
//! The .maestro format is a compressed JSON file with the following structure:
//! - Magic bytes: "MSTR" (4 bytes)
//! - Version: u16 (2 bytes)
//! - Flags: u16 (2 bytes)
//! - Compressed JSON data (flate2)

use std::io::{Read, Write};
use std::path::Path;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use orpheus_core::project::Project;

use crate::Error;

/// Magic bytes for .maestro files
const MAGIC: &[u8; 4] = b"MSTR";

/// Current format version
const VERSION: u16 = 1;

/// Format flags
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum FormatFlags {
    /// No special flags
    None = 0,
    /// File contains embedded audio
    EmbeddedAudio = 1,
    /// File is encrypted
    Encrypted = 2,
}

/// Project file header
#[derive(Debug, Clone)]
pub struct FileHeader {
    /// Format version
    pub version: u16,
    /// Format flags
    pub flags: u16,
}

impl Default for FileHeader {
    fn default() -> Self {
        Self {
            version: VERSION,
            flags: FormatFlags::None as u16,
        }
    }
}

/// Project serializer for .maestro format
pub struct ProjectSerializer;

impl ProjectSerializer {
    /// Serialize a project to bytes (compressed)
    pub fn to_bytes(project: &Project) -> Result<Vec<u8>, Error> {
        // Serialize to JSON first
        let json = serde_json::to_string(project)
            .map_err(|e| Error::Parse(format!("Failed to serialize project: {}", e)))?;

        // Create output buffer with header
        let mut output = Vec::new();

        // Write magic
        output.extend_from_slice(MAGIC);

        // Write version
        output.extend_from_slice(&VERSION.to_le_bytes());

        // Write flags
        output.extend_from_slice(&(FormatFlags::None as u16).to_le_bytes());

        // Compress and write JSON
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json.as_bytes())
            .map_err(|e| Error::Io(e))?;
        let compressed = encoder.finish()
            .map_err(|e| Error::Io(e))?;

        output.extend_from_slice(&compressed);

        Ok(output)
    }

    /// Deserialize a project from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Project, Error> {
        if data.len() < 8 {
            return Err(Error::InvalidFormat("File too small".into()));
        }

        // Verify magic
        if &data[0..4] != MAGIC {
            return Err(Error::InvalidFormat("Invalid magic bytes".into()));
        }

        // Read version
        let version = u16::from_le_bytes([data[4], data[5]]);
        if version > VERSION {
            return Err(Error::UnsupportedFormat(
                format!("File version {} is newer than supported version {}", version, VERSION)
            ));
        }

        // Read flags
        let _flags = u16::from_le_bytes([data[6], data[7]]);

        // Decompress JSON
        let mut decoder = GzDecoder::new(&data[8..]);
        let mut json = String::new();
        decoder.read_to_string(&mut json)
            .map_err(|e| Error::Parse(format!("Failed to decompress: {}", e)))?;

        // Parse JSON
        let project: Project = serde_json::from_str(&json)
            .map_err(|e| Error::Parse(format!("Failed to parse project: {}", e)))?;

        Ok(project)
    }

    /// Save project to file
    pub fn save(project: &Project, path: &Path) -> Result<(), Error> {
        let bytes = Self::to_bytes(project)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load project from file
    pub fn load(path: &Path) -> Result<Project, Error> {
        if !path.exists() {
            return Err(Error::NotFound(path.display().to_string()));
        }

        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    /// Check if a file is a valid .maestro file
    pub fn is_maestro_file(path: &Path) -> bool {
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut magic = [0u8; 4];
            if file.read_exact(&mut magic).is_ok() {
                return &magic == MAGIC;
            }
        }
        false
    }

    /// Get file header without loading full project
    pub fn read_header(path: &Path) -> Result<FileHeader, Error> {
        if !path.exists() {
            return Err(Error::NotFound(path.display().to_string()));
        }

        let mut file = std::fs::File::open(path)?;
        let mut header_bytes = [0u8; 8];
        file.read_exact(&mut header_bytes)
            .map_err(|e| Error::Io(e))?;

        if &header_bytes[0..4] != MAGIC {
            return Err(Error::InvalidFormat("Invalid magic bytes".into()));
        }

        Ok(FileHeader {
            version: u16::from_le_bytes([header_bytes[4], header_bytes[5]]),
            flags: u16::from_le_bytes([header_bytes[6], header_bytes[7]]),
        })
    }
}

/// Save project as uncompressed JSON (for debugging/compatibility)
pub fn save_as_json(project: &Project, path: &Path) -> Result<(), Error> {
    let json = serde_json::to_string_pretty(project)
        .map_err(|e| Error::Parse(format!("Failed to serialize: {}", e)))?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load project from uncompressed JSON
pub fn load_from_json(path: &Path) -> Result<Project, Error> {
    let content = std::fs::read_to_string(path)?;
    let project: Project = serde_json::from_str(&content)
        .map_err(|e| Error::Parse(format!("Failed to parse: {}", e)))?;
    Ok(project)
}

/// Detect project file type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectFileType {
    /// Native .maestro format
    Maestro,
    /// Plain JSON (legacy/debug)
    Json,
    /// Guitar Pro format
    GuitarPro,
    /// Unknown format
    Unknown,
}

/// Detect project file type from path
pub fn detect_project_type(path: &Path) -> ProjectFileType {
    // Check by magic bytes first
    if ProjectSerializer::is_maestro_file(path) {
        return ProjectFileType::Maestro;
    }

    // Check by extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "maestro" | "mst" => ProjectFileType::Maestro,
            "json" => ProjectFileType::Json,
            "gp" | "gp5" | "gpx" | "gp4" | "gp3" => ProjectFileType::GuitarPro,
            _ => ProjectFileType::Unknown,
        }
    } else {
        ProjectFileType::Unknown
    }
}

/// Load project from any supported format
pub fn load_project(path: &Path) -> Result<Project, Error> {
    match detect_project_type(path) {
        ProjectFileType::Maestro => ProjectSerializer::load(path),
        ProjectFileType::Json => load_from_json(path),
        ProjectFileType::GuitarPro => {
            // Would need to convert from Guitar Pro format
            Err(Error::UnsupportedFormat("Guitar Pro import not yet implemented".into()))
        }
        ProjectFileType::Unknown => {
            Err(Error::UnsupportedFormat(format!(
                "Unknown file format: {}",
                path.extension().and_then(|e| e.to_str()).unwrap_or("(no extension)")
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orpheus_core::project::{Track, TrackType};
    use tempfile::tempdir;

    #[test]
    fn test_project_round_trip() {
        let mut project = Project::new("Test Song");
        project.add_track(Track::new("Lead Guitar", TrackType::Midi));
        project.add_track(Track::new("Rhythm Guitar", TrackType::Midi));
        project.tempo = 140.0;

        let bytes = ProjectSerializer::to_bytes(&project).unwrap();
        let loaded = ProjectSerializer::from_bytes(&bytes).unwrap();

        assert_eq!(loaded.metadata.name, "Test Song");
        assert_eq!(loaded.track_count(), 2);
        assert!((loaded.tempo - 140.0).abs() < 0.01);
    }

    #[test]
    fn test_save_and_load_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.maestro");

        let mut project = Project::new("File Test");
        project.add_track(Track::audio("Audio Track"));

        ProjectSerializer::save(&project, &path).unwrap();
        assert!(path.exists());

        let loaded = ProjectSerializer::load(&path).unwrap();
        assert_eq!(loaded.metadata.name, "File Test");
        assert_eq!(loaded.track_count(), 1);
    }

    #[test]
    fn test_compression() {
        let mut project = Project::new("Compression Test");
        // Add multiple tracks to make data larger
        for i in 0..20 {
            project.add_track(Track::midi(format!("Track {}", i)));
        }

        let bytes = ProjectSerializer::to_bytes(&project).unwrap();
        let json = serde_json::to_string(&project).unwrap();

        // Compressed should be smaller than raw JSON
        assert!(bytes.len() < json.len());
    }

    #[test]
    fn test_header_reading() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("header_test.maestro");

        let project = Project::new("Header Test");
        ProjectSerializer::save(&project, &path).unwrap();

        let header = ProjectSerializer::read_header(&path).unwrap();
        assert_eq!(header.version, VERSION);
        assert_eq!(header.flags, FormatFlags::None as u16);
    }

    #[test]
    fn test_is_maestro_file() {
        let dir = tempdir().unwrap();
        let maestro_path = dir.path().join("test.maestro");
        let json_path = dir.path().join("test.json");

        let project = Project::new("Type Test");

        // Save as .maestro
        ProjectSerializer::save(&project, &maestro_path).unwrap();
        assert!(ProjectSerializer::is_maestro_file(&maestro_path));

        // Save as JSON
        save_as_json(&project, &json_path).unwrap();
        assert!(!ProjectSerializer::is_maestro_file(&json_path));
    }

    #[test]
    fn test_detect_project_type() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("detect.maestro");

        let project = Project::new("Detect Test");
        ProjectSerializer::save(&project, &path).unwrap();

        assert_eq!(detect_project_type(&path), ProjectFileType::Maestro);

        // By extension
        assert_eq!(
            detect_project_type(Path::new("foo.gp5")),
            ProjectFileType::GuitarPro
        );
        assert_eq!(
            detect_project_type(Path::new("bar.json")),
            ProjectFileType::Json
        );
    }

    #[test]
    fn test_load_project_dispatch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("dispatch.maestro");

        let mut project = Project::new("Dispatch Test");
        project.add_track(Track::midi("Test Track"));
        ProjectSerializer::save(&project, &path).unwrap();

        let loaded = load_project(&path).unwrap();
        assert_eq!(loaded.metadata.name, "Dispatch Test");
    }

    #[test]
    fn test_json_fallback() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("fallback.json");

        let project = Project::new("JSON Fallback");
        save_as_json(&project, &path).unwrap();

        let loaded = load_project(&path).unwrap();
        assert_eq!(loaded.metadata.name, "JSON Fallback");
    }

    #[test]
    fn test_invalid_magic() {
        let result = ProjectSerializer::from_bytes(b"NOT_MSTR_FILE");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_too_small() {
        let result = ProjectSerializer::from_bytes(b"MSTR");
        assert!(result.is_err());
    }
}
