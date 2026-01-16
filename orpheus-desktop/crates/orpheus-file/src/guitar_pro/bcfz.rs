//! BCFZ (Guitar Pro 6) decompression
//!
//! BCFZ is a custom LZ-based compression format used by Guitar Pro 6 (.gpx files).
//! Structure:
//! - 4 bytes: "BCFZ" magic
//! - 4 bytes: uncompressed size (little-endian)
//! - Bit stream with LZ-compressed data
//!
//! Decompression algorithm:
//! - Read 1 bit: if 0 = literal bytes, if 1 = back-reference
//! - If literal: read 2 bits (LE) for byte count, read that many bytes
//! - If back-ref: read 4 bits (BE) for word_size, read offset and length (LE)
//!
//! The result is BCFS data - a sector-based virtual filesystem.

use crate::{Error, Result};
use std::cmp;
use std::io::Read;
use tracing::{debug, trace};

const BCFZ_MAGIC: &[u8; 4] = b"BCFZ";
const SECTOR_SIZE: usize = 0x1000; // 4096 bytes

/// Bit stream reader for BCFZ decompression
struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8, // 0-7, counts from MSB (bit 7 down to 0)
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    /// Check if there's more data to read
    fn has_data(&self) -> bool {
        self.byte_pos < self.data.len()
    }

    /// Read a single bit (MSB first within each byte)
    fn read_bit(&mut self) -> Option<u8> {
        if self.byte_pos >= self.data.len() {
            return None;
        }

        let byte = self.data[self.byte_pos];
        // Read from MSB to LSB (bit 7, 6, 5, ... 0)
        let bit = (byte >> (7 - self.bit_pos)) & 1;

        self.bit_pos += 1;
        if self.bit_pos >= 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }

        Some(bit)
    }

    /// Read multiple bits in big-endian order (MSB first)
    fn read_bits(&mut self, count: usize) -> Option<usize> {
        let mut value: usize = 0;
        for _ in 0..count {
            let bit = self.read_bit()?;
            value = (value << 1) | (bit as usize);
        }
        Some(value)
    }

    /// Read multiple bits in little-endian/reversed order (LSB first)
    fn read_bits_reversed(&mut self, count: usize) -> Option<usize> {
        let mut value: usize = 0;
        for i in 0..count {
            let bit = self.read_bit()?;
            value |= (bit as usize) << i;
        }
        Some(value)
    }

    /// Read a full byte from the bit stream
    fn read_byte(&mut self) -> Option<u8> {
        self.read_bits(8).map(|v| v as u8)
    }

    /// Read multiple bytes from the bit stream
    fn read_bytes(&mut self, count: usize) -> Option<Vec<u8>> {
        let mut bytes = Vec::with_capacity(count);
        for _ in 0..count {
            bytes.push(self.read_byte()?);
        }
        Some(bytes)
    }
}

/// Decompress a BCFZ file and extract the score.gpif content
pub fn decompress_gpif<R: Read>(reader: &mut R) -> Result<String> {
    // First decompress BCFZ to get BCFS data
    let bcfs_data = decompress(reader)?;

    // Then extract files from BCFS
    let files = extract_bcfs(&bcfs_data)?;

    // Find score.gpif
    for (name, content) in &files {
        trace!("BCFS file: '{}' ({} bytes)", name, content.len());
        if name.ends_with("score.gpif") || name == "score.gpif" || name.contains("score.gpif") {
            return Ok(String::from_utf8_lossy(content).to_string());
        }
    }

    // Debug: list all files found
    debug!("Files found in BCFS archive:");
    for (name, content) in &files {
        debug!("  '{}' ({} bytes)", name, content.len());
    }

    Err(Error::InvalidFormat(format!(
        "score.gpif not found in BCFS archive (found {} files)",
        files.len()
    )))
}

/// Decompress BCFZ data to get BCFS filesystem data
pub fn decompress<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    // Read and verify magic
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic).map_err(|e| {
        Error::InvalidFormat(format!("Failed to read BCFZ magic: {}", e))
    })?;

    if &magic != BCFZ_MAGIC {
        return Err(Error::InvalidFormat(format!(
            "Invalid BCFZ magic: {:?} (expected {:?})",
            magic, BCFZ_MAGIC
        )));
    }

    // Read uncompressed size
    let mut size_bytes = [0u8; 4];
    reader.read_exact(&mut size_bytes).map_err(|e| {
        Error::InvalidFormat(format!("Failed to read BCFZ size: {}", e))
    })?;
    let uncompressed_size = u32::from_le_bytes(size_bytes) as usize;
    debug!("BCFZ uncompressed size: {} bytes", uncompressed_size);

    // Read all remaining compressed data
    let mut compressed_data = Vec::new();
    reader.read_to_end(&mut compressed_data)?;
    debug!("BCFZ compressed data: {} bytes", compressed_data.len());

    // Decompress using bit stream
    let mut output = Vec::with_capacity(uncompressed_size);
    let mut bits = BitReader::new(&compressed_data);

    while output.len() < uncompressed_size && bits.has_data() {
        // Read 1 bit to determine chunk type
        let Some(control_bit) = bits.read_bit() else {
            break;
        };

        if control_bit == 0 {
            // Uncompressed chunk: read 2 bits (LE/reversed) for length
            let Some(len) = bits.read_bits_reversed(2) else {
                break;
            };

            // Read 'len' bytes directly
            if len > 0 {
                let Some(bytes) = bits.read_bytes(len) else {
                    break;
                };
                output.extend(bytes);
            }
        } else {
            // Compressed chunk: read 4 bits (BE) for word_size
            let Some(word_size) = bits.read_bits(4) else {
                break;
            };

            if word_size == 0 {
                continue;
            }

            // Read offset and length using word_size bits (LE/reversed)
            let Some(offset) = bits.read_bits_reversed(word_size) else {
                break;
            };
            let Some(length) = bits.read_bits_reversed(word_size) else {
                break;
            };

            // Copy from back-reference
            if offset > 0 && length > 0 && output.len() >= offset {
                let source_start = output.len() - offset;
                let copy_len = cmp::min(length, offset);

                // Copy bytes (may need to handle overlapping copies)
                for i in 0..copy_len {
                    let byte = output[source_start + i];
                    output.push(byte);
                }
            }
        }
    }

    debug!(
        "BCFZ decompressed: {} bytes (expected {})",
        output.len(),
        uncompressed_size
    );

    Ok(output)
}

/// Extract files from BCFS filesystem data
///
/// BCFS is a sector-based virtual filesystem:
/// - Sectors are 4096 bytes
/// - File headers have magic value 2 at offset +4 (not +0!)
/// - Filename at offset +8 (null-terminated)
/// - File size at offset +0x8C (adjusted)
/// - Block indices at offset +0x94 (adjusted)
fn extract_bcfs(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    let data_len = data.len();
    let mut files = Vec::new();

    trace!("Extracting files from BCFS ({} bytes)", data_len);

    // The BCFS structure:
    // - Sector 0: "BCFS" header
    // - Sector 1+: File entries or data blocks
    // - File entry has: +4 = magic (2), +8 = filename, +0x90 = size, +0x98 = block indices

    let mut offset = 0usize;

    while offset + SECTOR_SIZE <= data_len {
        offset += SECTOR_SIZE;

        if offset + 8 > data_len {
            break;
        }

        // Check for file header magic (value 2) at offset +4
        let header_magic = i32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);

        if header_magic != 2 {
            continue;
        }

        // File entry found
        let filename_offset = offset + 8;
        let filesize_offset = offset + 0x90;
        let block_index_offset = offset + 0x98;

        // Read filename (127 bytes max, null-terminated)
        if filename_offset + 127 > data_len {
            continue;
        }
        let filename_bytes = &data[filename_offset..filename_offset + 127];
        let filename = String::from_utf8_lossy(filename_bytes)
            .trim_end_matches('\0')
            .to_string();

        // Read file size
        if filesize_offset + 4 > data_len {
            continue;
        }
        let file_size = i32::from_le_bytes([
            data[filesize_offset],
            data[filesize_offset + 1],
            data[filesize_offset + 2],
            data[filesize_offset + 3],
        ]) as usize;

        trace!("BCFS file: '{}' ({} bytes)", filename, file_size);

        // Read block indices and collect file data
        let mut file_data = Vec::new();
        let mut block_count = 0usize;

        loop {
            let block_ptr_offset = block_index_offset + (4 * block_count);
            if block_ptr_offset + 4 > data_len {
                break;
            }

            let block_index = i32::from_le_bytes([
                data[block_ptr_offset],
                data[block_ptr_offset + 1],
                data[block_ptr_offset + 2],
                data[block_ptr_offset + 3],
            ]);

            if block_index == 0 {
                break;
            }

            // Read block data
            let block_offset = (block_index as usize) * SECTOR_SIZE;
            if block_offset + SECTOR_SIZE <= data_len {
                file_data.extend_from_slice(&data[block_offset..block_offset + SECTOR_SIZE]);
            }

            block_count += 1;

            // Safety limit
            if block_count > 1000 {
                break;
            }
        }

        // Truncate to actual file size
        if file_size > 0 && file_size <= file_data.len() {
            file_data.truncate(file_size);
            files.push((filename, file_data));
        } else if file_size == 0 && !filename.is_empty() {
            files.push((filename, Vec::new()));
        }
    }

    debug!("Extracted {} files from BCFS", files.len());
    Ok(files)
}
