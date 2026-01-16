//! Guitar Pro file format parsing
//!
//! Supports (with content-based detection):
//! - GP7/GP8 (.gp, .gp7) - Guitar Pro 7+ ZIP/XML format
//! - GP6 (.gpx) - Guitar Pro 6 BCFZ compressed format
//! - GP5 (.gp5) - Guitar Pro 5 binary format (v5.00 and v5.10)
//! - GP4 (.gp4) - Guitar Pro 4 binary format
//! - GP3 (.gp3) - Guitar Pro 3 binary format

mod bcfz;
mod convert;
mod gp5;
mod gp7;
mod types;

pub use convert::convert_gp_to_tab;
pub use gp5::Gp5Parser;
pub use gp7::Gp7Parser;
pub use types::*;

use crate::{Error, Result};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use tracing::debug;

/// Magic bytes for different Guitar Pro formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpFileType {
    /// GP5/GP4/GP3 binary format (starts with "FICHIER GUITAR PRO")
    Binary,
    /// GP7/GP8 ZIP format (starts with "PK")
    Zip,
    /// GP6 BCFZ compressed format (starts with "BCFZ")
    Bcfz,
    /// Unknown format
    Unknown,
}

impl GpFileType {
    /// Detect file type from magic bytes
    pub fn detect<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic).map_err(|e| {
            Error::InvalidFormat(format!("Failed to read file header: {}", e))
        })?;

        Ok(match &magic {
            // ZIP format: "PK\x03\x04" or "PK\x05\x06"
            [0x50, 0x4B, 0x03, 0x04] | [0x50, 0x4B, 0x05, 0x06] => GpFileType::Zip,
            // BCFZ format: "BCFZ"
            [0x42, 0x43, 0x46, 0x5A] => GpFileType::Bcfz,
            // Binary format: length byte + "FIC" (start of "FICHIER GUITAR PRO")
            [len, 0x46, 0x49, 0x43] if *len >= 20 && *len <= 40 => GpFileType::Binary,
            _ => GpFileType::Unknown,
        })
    }

    /// Detect file type from a file path
    pub fn detect_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        Self::detect(&mut reader)
    }
}

/// Parse a Guitar Pro file (auto-detects format)
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<GuitarProFile> {
    let path = path.as_ref();

    // First try content-based detection
    let file_type = GpFileType::detect_file(path)?;
    debug!("Detected Guitar Pro file type: {:?}", file_type);

    match file_type {
        GpFileType::Zip => {
            debug!("Parsing as GP7/GP8 ZIP format");
            Gp7Parser::parse_file(path)
        }
        GpFileType::Bcfz => {
            debug!("Parsing as GP6 BCFZ format");
            parse_bcfz_file(path)
        }
        GpFileType::Binary => {
            debug!("Parsing as GP5/GP4/GP3 binary format");
            Gp5Parser::parse_file(path)
        }
        GpFileType::Unknown => {
            // Fall back to extension-based detection
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            debug!("Unknown magic bytes, falling back to extension: {}", extension);

            match extension.as_str() {
                "gp" | "gp7" | "gp8" => Gp7Parser::parse_file(path),
                "gpx" => parse_bcfz_file(path),
                "gp5" | "gp4" | "gp3" => Gp5Parser::parse_file(path),
                _ => Err(Error::InvalidFormat(format!(
                    "Unknown Guitar Pro format (extension: {})",
                    extension
                ))),
            }
        }
    }
}

/// Parse a BCFZ (Guitar Pro 6) file
fn parse_bcfz_file<P: AsRef<Path>>(path: P) -> Result<GuitarProFile> {
    let file = File::open(path.as_ref())?;
    let mut reader = BufReader::new(file);

    // Decompress BCFZ to get the score.gpif XML
    let xml_content = bcfz::decompress_gpif(&mut reader)?;

    // Parse the XML using GP7 parser
    Gp7Parser::parse_xml(&xml_content)
}

/// Check if a file is a Guitar Pro file (by extension or content)
pub fn is_guitar_pro_file<P: AsRef<Path>>(path: P) -> bool {
    // First check extension
    let extension = path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if matches!(extension.as_str(), "gp" | "gp3" | "gp4" | "gp5" | "gp6" | "gp7" | "gp8" | "gpx") {
        return true;
    }

    // If extension doesn't match, try content detection
    GpFileType::detect_file(&path).map(|t| t != GpFileType::Unknown).unwrap_or(false)
}
