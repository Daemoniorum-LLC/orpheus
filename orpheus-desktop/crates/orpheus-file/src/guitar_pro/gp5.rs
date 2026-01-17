//! Guitar Pro 5 binary format parser
//!
//! This parser handles GP3, GP4, and GP5 files with a focus on robustness
//! over perfect parsing of all features.

use super::types::*;
use crate::{Error, Result};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use tracing::{debug, trace, warn};

/// GP5 file parser
pub struct Gp5Parser<R: Read + Seek> {
    reader: R,
    version: GpVersion,
    version_str: String,
}

impl Gp5Parser<BufReader<File>> {
    /// Parse a GP5 file from disk
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<GuitarProFile> {
        let file = File::open(path.as_ref())
            .map_err(|e| Error::Io(e))?;
        let reader = BufReader::new(file);
        let mut parser = Gp5Parser::new(reader);
        parser.parse()
    }
}

impl<R: Read + Seek> Gp5Parser<R> {
    /// Create a new parser from a reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            version: GpVersion::Unknown,
            version_str: String::new(),
        }
    }

    /// Parse the entire file
    pub fn parse(&mut self) -> Result<GuitarProFile> {
        let mut gp_file = GuitarProFile::default();

        // Read version header
        let header = self.read_header()?;
        gp_file.version = GpVersion::from_header(&header);
        self.version = gp_file.version;
        self.version_str = header.clone();
        debug!("Parsing Guitar Pro file version: {:?} ({})", gp_file.version, header);

        // Read song info
        gp_file.info = self.read_song_info()?;
        debug!("Song: {} by {}", gp_file.info.title, gp_file.info.artist);

        // GP3: Triplet feel before tempo
        if matches!(self.version, GpVersion::Gp3) {
            let _ = self.read_byte()?; // triplet feel
        }

        // Read lyrics (GP4+)
        if self.is_gp4_or_later() {
            // GP4 has an extra byte before lyrics (possibly triplet feel or other flag)
            if matches!(self.version, GpVersion::Gp4) {
                let _ = self.read_byte()?;
            }
            self.skip_lyrics()?;
        }

        if self.is_gp5() {
            // GP5: Parse with RSE (Realistic Sound Engine) support
            debug!("GP5 format detected ({}) - parsing with RSE support", self.version_str);

            // GP5 v5.10+ has RSE equalizer BEFORE page setup (11 signed bytes)
            // GP5 v5.00 does NOT have this section
            if self.is_gp5_10() {
                debug!("Reading RSE EQ at position: {:?}", self.position());
                for i in 0..11 {
                    let eq = self.read_signed_byte()?;
                    trace!("RSE EQ band {}: {}", i, eq);
                }

                // Skip page setup (GP5 v5.10 specific - has 12 template strings)
                self.skip_gp5_page_setup()?;
            } else if self.is_gp5_00() {
                // GP5 v5.00 has simpler page setup like GP4
                debug!("GP5 v5.00 detected - using simplified page setup");
                self.skip_page_setup()?;
            }

            // Read tempo
            debug!("Reading tempo at position: {:?}", self.position());

            // v5.10 has tempo name string before tempo, v5.00 does not
            if self.is_gp5_10() {
                let tempo_name = self.read_string_int_byte()?;
                debug!("Tempo name: '{}'", tempo_name);
            }

            let tempo_bytes = self.peek_bytes(4)?;
            debug!("Next 4 bytes (tempo): {:02x?}", tempo_bytes);

            gp_file.tempo = self.read_int()? as u16;
            debug!("Tempo: {} BPM", gp_file.tempo);

            // Hide tempo flag (v5.10 only)
            if self.is_gp5_10() && gp_file.tempo > 0 {
                let _ = self.read_bool()?;
            }

            // Key signature
            gp_file.key_signature.key = self.read_signed_byte()?;
            debug!("Key signature: {}", gp_file.key_signature.key);

            // Octave - v5.10 uses 4-byte int, v5.00 uses 1 byte
            if self.is_gp5_10() {
                let octave = self.read_int()?;
                debug!("Octave: {}", octave);
            } else {
                // v5.00: skip 3 bytes of padding after key
                let _ = self.read_bytes(3)?;
                debug!("v5.00: skipped 3 bytes after key");
            }

            // Read MIDI channels
            debug!("Reading MIDI channels at position: {:?}", self.position());
            let _channels = self.read_midi_channels()?;
            debug!("MIDI channels complete at position: {:?}", self.position());

            // GP5 has musical directions (19 int16 values for section markers)
            // Both v5.00 and v5.10 have this section
            debug!("Reading musical directions at position: {:?}", self.position());
            for _ in 0..19 {
                let _ = self.read_short()?;
            }

            // RSE master effect settings (4 bytes)
            debug!("Reading RSE master at position: {:?}", self.position());
            let _ = self.read_int()?;  // accented/ghost volumes or padding

            // v5.00 has 1 extra byte of padding before measures
            if self.is_gp5_00() {
                let _ = self.read_byte()?;
                debug!("v5.00: skipped 1 byte padding before measures");
            }

            // Debug: show what's next
            let peek = self.peek_bytes(16)?;
            debug!("Before measures, next 16 bytes: {:02x?}", peek);

            // Read number of measures
            let num_measures = self.read_int()? as usize;
            debug!("Number of measures: {}", num_measures);

            // Read number of tracks
            let num_tracks = self.read_int()? as usize;
            debug!("Number of tracks: {}", num_tracks);

            // Read measure headers
            debug!("Reading measure headers at position: {:?}", self.position());
            let measure_headers = self.read_measure_headers(num_measures)?;
            gp_file.measures = measure_headers;
            let mh_end = self.position()?;
            debug!("Measure headers complete at position: {}", mh_end);

            // GP5 has variable padding between measure headers and tracks
            // Track format: flags1, flags2, name (41 bytes = 1 len + 40 content), stringCount, tuning...
            // Search for track by looking for string count (6) followed by valid tuning (MIDI notes 20-100)
            let mut scan_pos = mh_end;
            let mut found_tracks = false;
            while scan_pos < mh_end + 2000 {
                if let Ok(peek) = self.peek_bytes(52) {
                    // Look for pattern at offset 43: stringCount (4-8), then valid tuning
                    // offset 0-1: flags, offset 2-42: name (41 bytes), offset 43-46: stringCount
                    if peek[43] >= 4 && peek[43] <= 8 && peek[44] == 0 && peek[45] == 0 && peek[46] == 0 {
                        // Potential string count 4-8
                        let string_count = peek[43];
                        // Check first tuning value (should be MIDI note 20-100)
                        let first_tuning = i32::from_le_bytes([peek[47], peek[48], peek[49], peek[50]]);
                        if first_tuning >= 20 && first_tuning <= 100 {
                            debug!("Found track at position {}: strings={}, first_tuning={}",
                                   scan_pos, string_count, first_tuning);
                            found_tracks = true;
                            break;
                        }
                    }
                }
                let _ = self.read_byte()?;
                scan_pos += 1;
            }
            if !found_tracks {
                warn!("Could not find track data within 2000 bytes of measure headers");
            }
            debug!("Track data at position: {}", scan_pos);

            // Read tracks
            debug!("Reading tracks at position: {:?}", self.position());
            gp_file.tracks = self.read_tracks(num_tracks)?;
            debug!("Tracks complete at position: {:?}", self.position());
            for (i, track) in gp_file.tracks.iter().enumerate() {
                debug!("Track {}: {} ({} strings)", i + 1, track.name, track.strings);
            }

            // GP5 has additional RSE data after tracks - scan to find beat data start
            if let Some(beat_pos) = self.scan_for_beat_data()? {
                debug!("Found beat data at position: {}", beat_pos);
            }

            // Try to read measure-track pairs (note data)
            if let Err(e) = self.read_measure_track_pairs(&mut gp_file.measures, num_tracks) {
                warn!("Could not parse note data: {:?}", e);
                // Continue - we still have useful structure
            }

            return Ok(gp_file);
        }

        // GP3/GP4: Read tempo
        gp_file.tempo = self.read_int()? as u16;
        debug!("Tempo: {} BPM", gp_file.tempo);

        // Read key (GP3 uses int, GP4+ uses byte + padding)
        if matches!(self.version, GpVersion::Gp3) {
            gp_file.key_signature.key = self.read_int()? as i8;
        } else if self.is_gp4_or_later() {
            gp_file.key_signature.key = self.read_byte()? as i8;
            let _ = self.read_bytes(3)?; // padding
        }

        // Read octave (GP4+)
        if self.is_gp4_or_later() {
            let _ = self.read_byte()?;
        }

        // Read MIDI channels
        let _channels = self.read_midi_channels()?;

        // Read number of measures
        let num_measures = self.read_int()? as usize;
        debug!("Number of measures: {}", num_measures);

        // Read number of tracks
        let num_tracks = self.read_int()? as usize;
        debug!("Number of tracks: {}", num_tracks);

        // Read measure headers
        let measure_headers = self.read_measure_headers(num_measures)?;
        gp_file.measures = measure_headers;

        // Read tracks
        gp_file.tracks = self.read_tracks(num_tracks)?;
        for (i, track) in gp_file.tracks.iter().enumerate() {
            debug!("Track {}: {} ({} strings)", i + 1, track.name, track.strings);
        }

        // Try to read measure-track pairs (note data)
        // This is the most complex part and most likely to fail
        if let Err(e) = self.read_measure_track_pairs(&mut gp_file.measures, num_tracks) {
            warn!("Could not parse note data: {:?}", e);
            // Continue with metadata only
        }

        Ok(gp_file)
    }

    // ==================== Version helpers ====================

    fn is_gp5(&self) -> bool {
        matches!(self.version, GpVersion::Gp5)
    }

    /// Check if this is GP5 v5.10 or later (has RSE features)
    fn is_gp5_10(&self) -> bool {
        self.is_gp5() && self.version_str.contains("5.10")
    }

    /// Check if this is GP5 v5.00 (older format without full RSE)
    fn is_gp5_00(&self) -> bool {
        self.is_gp5() && self.version_str.contains("5.00")
    }

    fn is_gp4_or_later(&self) -> bool {
        matches!(self.version, GpVersion::Gp4 | GpVersion::Gp5)
    }

    fn is_gp3(&self) -> bool {
        matches!(self.version, GpVersion::Gp3)
    }

    // ==================== Low-level read functions ====================

    fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)
            .map_err(|e| Error::Io(e))?;
        Ok(buf[0])
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        self.reader.read_exact(&mut buf)
            .map_err(|e| Error::Io(e))?;
        Ok(buf)
    }

    fn peek_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let pos = self.position()?;
        let bytes = self.read_bytes(count)?;
        self.reader.seek(SeekFrom::Start(pos as u64))
            .map_err(|e| Error::Io(e))?;
        Ok(bytes)
    }

    fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_byte()? != 0)
    }

    fn read_signed_byte(&mut self) -> Result<i8> {
        Ok(self.read_byte()? as i8)
    }

    fn read_short(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)
            .map_err(|e| Error::Io(e))?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_int(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)
            .map_err(|e| Error::Io(e))?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_string_byte(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_byte()? as usize;
        let actual_len = len.min(max_len);
        let data = self.read_bytes(max_len)?;
        // Filter out null bytes and invalid UTF-8
        let s: String = data[..actual_len]
            .iter()
            .filter(|&&b| b != 0 && b >= 32 && b < 127)
            .map(|&b| b as char)
            .collect();
        Ok(s)
    }

    fn read_string_int(&mut self) -> Result<String> {
        let len = self.read_int()? as usize;
        if len == 0 || len > 10000 {
            return Ok(String::new());
        }
        let data = self.read_bytes(len)?;
        String::from_utf8(data)
            .or_else(|e| {
                // Fallback: filter valid chars
                Ok(e.into_bytes().iter()
                    .filter(|&&b| b >= 32 && b < 127)
                    .map(|&b| b as char)
                    .collect())
            })
    }

    fn read_string_int_byte(&mut self) -> Result<String> {
        let full_len = self.read_int()?;
        if full_len <= 0 || full_len > 10000 {
            return Ok(String::new());
        }
        let str_len = self.read_byte()? as usize;
        if str_len == 0 {
            // Skip remaining bytes
            if full_len > 1 {
                let _ = self.read_bytes((full_len - 1) as usize)?;
            }
            return Ok(String::new());
        }
        let data = self.read_bytes(str_len.min(full_len as usize - 1))?;
        // Skip padding
        let remaining = (full_len as usize).saturating_sub(str_len + 1);
        if remaining > 0 {
            let _ = self.read_bytes(remaining)?;
        }
        String::from_utf8(data)
            .or_else(|e| {
                Ok(e.into_bytes().iter()
                    .filter(|&&b| b >= 32 && b < 127)
                    .map(|&b| b as char)
                    .collect())
            })
    }

    /// Read RSE string format: 3 padding bytes + 1 length byte + string content
    fn read_rse_string(&mut self) -> Result<String> {
        // RSE strings have format: 00 00 00 LEN + string content
        let _ = self.read_bytes(3)?; // Skip 3 padding bytes
        let len = self.read_byte()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        let data = self.read_bytes(len)?;
        String::from_utf8(data)
            .or_else(|e| {
                Ok(e.into_bytes().iter()
                    .filter(|&&b| b >= 32 && b < 127)
                    .map(|&b| b as char)
                    .collect())
            })
    }

    fn skip(&mut self, count: i64) -> Result<()> {
        self.reader.seek(SeekFrom::Current(count))
            .map_err(|e| Error::Io(e))?;
        Ok(())
    }

    fn position(&mut self) -> Result<u64> {
        self.reader.stream_position()
            .map_err(|e| Error::Io(e))
    }

    /// Scan forward to find where beat data starts
    /// GP5 has additional RSE data after tracks that we need to skip
    fn scan_for_beat_data(&mut self) -> Result<Option<u64>> {
        let start_pos = self.position()?;
        let mut pos = start_pos;

        // Scan up to 2000 bytes to find beat data
        // Beat data starts with: num_beats (int 1-8), then beat_flags (typically 0x02 or 0x40)
        for _ in 0..2000 {
            if let Ok(peek) = self.peek_bytes(8) {
                // Look for pattern: small beat count (1-8) followed by common beat flags
                let beat_count = i32::from_le_bytes([peek[0], peek[1], peek[2], peek[3]]);
                let beat_flags = peek[4];

                // Valid beat count is 1-8, common beat flags are 0x02, 0x40, 0x42, etc
                if beat_count >= 1 && beat_count <= 8 {
                    // Check if this looks like beat data
                    // 0x40 = rest/empty beat, 0x02 = has duration
                    if beat_flags == 0x40 || (beat_flags & 0x02) != 0 {
                        // Additional validation: check for note data pattern after flags
                        // For 0x40 flags, check for note count (should be 0-6)
                        if beat_flags == 0x40 {
                            // Check next bytes for valid note structure
                            let potential_duration = peek[5];
                            // Duration byte is typically 0-16 for whole note to 256th
                            if potential_duration <= 16 || potential_duration == 0 {
                                debug!("Found beat data at position {}: count={}, flags=0x{:02x}",
                                    pos, beat_count, beat_flags);
                                return Ok(Some(pos));
                            }
                        } else if (beat_flags & 0x02) != 0 {
                            // Has duration - check duration value
                            let duration = peek[5];
                            if duration <= 16 {
                                debug!("Found beat data at position {}: count={}, flags=0x{:02x}",
                                    pos, beat_count, beat_flags);
                                return Ok(Some(pos));
                            }
                        }
                    }
                }
            }

            // Move to next byte
            let _ = self.read_byte()?;
            pos += 1;
        }

        // Not found, restore position
        self.reader.seek(SeekFrom::Start(start_pos))
            .map_err(|e| Error::Io(e))?;
        Ok(None)
    }

    // ==================== High-level parse functions ====================

    fn read_header(&mut self) -> Result<String> {
        // GP files start with a version string
        // Format: length byte + string data (30 bytes, null-padded)
        // Total: 31 bytes
        let header = self.read_string_byte(30)?;
        if !header.starts_with("FICHIER GUITAR PRO") {
            return Err(Error::InvalidFormat(format!(
                "Not a Guitar Pro file (header: {})",
                header
            )));
        }
        Ok(header)
    }

    fn read_song_info(&mut self) -> Result<SongInfo> {
        let mut info = SongInfo::default();

        info.title = self.read_string_int_byte().unwrap_or_default();
        info.subtitle = self.read_string_int_byte().unwrap_or_default();
        info.artist = self.read_string_int_byte().unwrap_or_default();
        info.album = self.read_string_int_byte().unwrap_or_default();
        info.author = self.read_string_int_byte().unwrap_or_default(); // Words

        // GP5 has separate music by field
        if self.is_gp5() {
            let _music = self.read_string_int_byte().unwrap_or_default();
        }

        info.copyright = self.read_string_int_byte().unwrap_or_default();
        info.tab_author = self.read_string_int_byte().unwrap_or_default();
        info.instructions = self.read_string_int_byte().unwrap_or_default();

        // Comments (number of lines + lines)
        let num_comments = self.read_int().unwrap_or(0) as usize;
        debug!("Num comments: {}", num_comments);
        for i in 0..num_comments.min(100) {
            if let Ok(comment) = self.read_string_int_byte() {
                debug!("Comment {}: '{}'", i, comment);
                info.comments.push(comment);
            }
        }

        debug!("Song info complete. Position after: {:?}", self.position());
        Ok(info)
    }

    fn skip_lyrics(&mut self) -> Result<()> {
        // Lyrics track number
        let track = self.read_int()?;
        debug!("Lyrics track: {}", track);

        // 5 lyric lines
        for i in 0..5 {
            let start_measure = self.read_int()?;
            debug!("Lyrics line {} start measure: {}", i, start_measure);

            // GP4/GP5 lyrics use int-prefixed string (not int-byte)
            let len = self.read_int()?;
            debug!("Lyrics line {} length: {}", i, len);
            if len > 0 && len < 100000 {
                let _ = self.read_bytes(len as usize)?;
            }
        }
        Ok(())
    }

    fn skip_page_setup(&mut self) -> Result<()> {
        debug!("Page setup at position: {:?}", self.position());

        // Page size
        let width = self.read_int()?;
        let height = self.read_int()?;
        debug!("Page size: {}x{}", width, height);

        // Margins (4 ints)
        for _ in 0..4 {
            let _ = self.read_int()?;
        }

        // Score size
        let _ = self.read_int()?;

        // Header/footer flags
        let flags = self.read_short()?;
        debug!("Page flags: {}", flags);

        // Header/footer strings (varies by GP version)
        // GP5.10 typically has 11 strings
        let num_strings = 11;
        for i in 0..num_strings {
            let s = self.read_string_int_byte()?;
            if !s.is_empty() {
                debug!("Page string {}: {}", i, s);
            }
        }

        debug!("Page setup end at position: {:?}", self.position());
        Ok(())
    }

    fn read_midi_channels(&mut self) -> Result<Vec<MidiChannel>> {
        let mut channels = Vec::new();

        // 64 MIDI channels (4 ports x 16 channels)
        for _ in 0..64 {
            let program = self.read_int()? as u8;
            let volume = self.read_byte()?;
            let pan = self.read_byte()?;
            let chorus = self.read_byte()?;
            let reverb = self.read_byte()?;
            let phaser = self.read_byte()?;
            let tremolo = self.read_byte()?;
            let _ = self.read_bytes(2)?; // padding

            channels.push(MidiChannel {
                program,
                volume,
                pan,
                chorus,
                reverb,
                phaser,
                tremolo,
            });
        }

        Ok(channels)
    }

    fn read_measure_headers(&mut self, count: usize) -> Result<Vec<Measure>> {
        let mut measures = Vec::with_capacity(count);
        let mut current_time_sig = TimeSignature::default();

        for i in 0..count {
            let pos_start = self.position()?;
            let mut measure = Measure {
                number: (i + 1) as u16,
                ..Default::default()
            };

            let flags1 = self.read_byte()?;
            // GP5 has second flag byte
            let flags2 = if self.is_gp5() { self.read_byte()? } else { 0 };

            if i < 3 {
                debug!("Measure {} at pos {}: flags1=0x{:02x} flags2=0x{:02x}", i+1, pos_start, flags1, flags2);
            }

            // Time signature numerator
            if flags1 & 0x01 != 0 {
                current_time_sig.numerator = self.read_byte()?;
            }

            // Time signature denominator
            if flags1 & 0x02 != 0 {
                current_time_sig.denominator = self.read_byte()?;
                // GP5 has extra timeSignatureType byte
                if self.is_gp5() {
                    let _ = self.read_byte()?;
                }
            }

            if flags1 & 0x03 != 0 {
                measure.time_signature = Some(current_time_sig);
            }

            // Repeat start
            measure.repeat_start = flags1 & 0x04 != 0;

            // Repeat end
            if flags1 & 0x08 != 0 {
                measure.repeat_end = self.read_byte()?;
            }

            // Number of alternate endings
            if flags1 & 0x10 != 0 {
                let _ = self.read_byte()?;
            }

            // Marker
            if flags1 & 0x20 != 0 {
                let marker_name = self.read_string_int_byte()?;
                let _ = self.read_int()?; // color
                measure.marker = Some(marker_name);
            }

            // Key signature change
            if flags1 & 0x40 != 0 {
                let _ = self.read_signed_byte()?; // key
                let _ = self.read_byte()?; // minor
            }

            // Double bar
            // Note: flag 0x80 indicates double bar, no extra data needed

            // GP5 extra data based on flags
            if self.is_gp5() {
                // Time signature beam groups
                if flags1 & 0x03 != 0 {
                    let _ = self.read_bytes(4)?;
                }

                // Time signature extension (flags2 & 0x01)
                if flags2 & 0x01 != 0 {
                    let _ = self.read_bytes(4)?;
                }

                // Triplet feel (flags2 & 0x10)
                if flags2 & 0x10 != 0 {
                    let _ = self.read_byte()?;
                }
            }

            measures.push(measure);
        }

        Ok(measures)
    }

    fn read_tracks(&mut self, count: usize) -> Result<Vec<Track>> {
        let mut tracks = Vec::with_capacity(count);

        for i in 0..count {
            let pos = self.position()?;
            debug!("Track {} at position {}", i + 1, pos);

            let mut track = Track {
                number: (i + 1) as u8,
                ..Default::default()
            };

            let flags1 = self.read_byte()?;
            // GP5 has second flags byte
            let flags2 = if self.is_gp5() { self.read_byte()? } else { 0 };
            track.is_drums = flags1 & 0x01 != 0;
            debug!("Track {} flags: 0x{:02x} 0x{:02x}, drums={}", i + 1, flags1, flags2, track.is_drums);

            // GP5: If flags2 has certain bits set, RSE instrument preset data comes BEFORE the name
            // Bits 1 (0x02), 2 (0x04), or 4 (0x10) indicate RSE preset strings before track name
            if self.is_gp5() && (flags2 & 0x16) != 0 {
                // RSE preset structure (GP5 uses 3-byte padding + 1-byte length strings):
                // - 3 bytes padding (00 00 00)
                // - 1 byte string length
                // - string content (preset name like "American Clean - Pi Distortion")
                // - 1 byte unknown (0a)
                // - 3 bytes padding (00 00 00)
                // - 1 byte string length
                // - string content (effect type like "Amp Tones")
                // - 1 byte unknown (88)

                // Read preset name
                let pos_before = self.position()?;
                let preset_name = self.read_rse_string()?;
                debug!("Track {} RSE preset at pos {}: '{}'", i + 1, pos_before, preset_name);

                // Skip separator byte
                let _ = self.read_byte()?;

                // Read effect type name
                let effect_name = self.read_rse_string()?;
                debug!("Track {} RSE effect: '{}'", i + 1, effect_name);

                // Skip unknown byte before track name
                let _ = self.read_byte()?;
            } else if self.is_gp5() && (flags2 & 0x01) != 0 {
                // When only bit 0 is set (no RSE), there's extra track data
                // Skip 2 ints + 2 bytes = 10 bytes of track extension data
                let _ = self.read_int()?;
                let _ = self.read_int()?;
                let _ = self.read_bytes(2)?;
                debug!("Track {} skipped extension data (flags2 & 0x01)", i + 1);
            }

            // Track name (40 bytes content + 1 byte length)
            track.name = self.read_string_byte(40)?;
            debug!("Track {} name: '{}'", i + 1, track.name);

            // Number of strings
            track.strings = self.read_int()? as u8;
            debug!("Track {} strings: {}", i + 1, track.strings);

            // String tuning (7 strings max)
            track.tuning.clear();
            for _ in 0..7 {
                let tuning = self.read_int()? as u8;
                if track.tuning.len() < track.strings as usize {
                    track.tuning.push(tuning);
                }
            }
            debug!("Track {} tuning: {:?}", i + 1, track.tuning);

            // MIDI port
            let _ = self.read_int()?;

            // MIDI channel
            track.channel = self.read_int()? as u8;

            // MIDI channel for effects
            let _ = self.read_int()?;

            // Number of frets
            let _ = self.read_int()?;

            // Capo
            track.capo = self.read_int()? as u8;

            // Track color
            let r = self.read_byte()?;
            let g = self.read_byte()?;
            let b = self.read_byte()?;
            let _ = self.read_byte()?; // alpha/padding
            track.color = (r, g, b);

            // GP5 extra data - highly variable, just skip what we can
            if self.is_gp5() {
                self.skip_track_rse_data(flags2)?;
            }

            tracks.push(track);
        }

        Ok(tracks)
    }

    fn skip_track_rse_data(&mut self, _flags2: u8) -> Result<()> {
        // RSE (Realistic Sound Engine) data is complex and varies
        // Just skip the basic RSE settings
        let pos = self.position()?;
        trace!("Skipping track RSE data at position {}", pos);

        // Basic RSE settings (several ints and bytes)
        for _ in 0..11 {
            let _ = self.read_int()?;
        }

        // Chord diagrams count
        let num_diagrams = self.read_int()?;
        if num_diagrams > 0 && num_diagrams < 50 {
            for _ in 0..num_diagrams {
                // Each diagram is variable size
                let _ = self.read_int()?; // unknown
                let _ = self.read_int()?; // fret
                let _ = self.read_int()?; // position
                let _ = self.read_int()?; // unknown
                let _ = self.read_int()?; // strings mask

                // Fret assignments per string
                for _ in 0..7 {
                    let _ = self.read_byte()?;
                }

                let _ = self.read_int()?; // barre flags

                // Fingerings
                for _ in 0..7 {
                    let _ = self.read_signed_byte()?;
                }
            }
        }

        trace!("Track RSE data complete at position: {:?}", self.position());
        Ok(())
    }

    fn skip_gp5_page_setup(&mut self) -> Result<()> {
        debug!("Skipping GP5 page setup at position: {:?}", self.position());

        // Page dimensions
        let width = self.read_int()?;
        let height = self.read_int()?;
        debug!("Page dimensions: {}x{}", width, height);

        // Margins (left, right, top, bottom)
        for _ in 0..4 {
            let _ = self.read_int()?;
        }

        // Score size percentage
        let _ = self.read_int()?;

        // Header/footer flags (bitmask)
        let flags = self.read_short()?;
        debug!("Page setup flags: 0x{:04x}", flags);

        // Header/footer text strings - GP5 has 12 strings:
        // 0-1: empty header lines, 2: title, 3: subtitle, 4: artist, 5: album,
        // 6: words_by, 7: music_by, 8: words_music, 9: copyright1, 10: copyright2,
        // 11: page_number_format (e.g., "Page %N%/%P%")
        for i in 0..12 {
            let pos_before = self.position()?;
            let s = self.read_string_int_byte()?;
            trace!("Page string {} at pos {}: '{}'", i, pos_before, if s.len() > 30 { &s[..30] } else { &s });
        }

        // After page setup, show next 32 bytes for debugging
        let peek = self.peek_bytes(32)?;
        debug!("After page setup, next 32 bytes: {:02x?}", peek);

        debug!("Page setup complete at position: {:?}", self.position());
        Ok(())
    }

    fn skip_gp5_rse_master(&mut self) -> Result<()> {
        debug!("Skipping GP5 RSE master at position: {:?}", self.position());

        // Master RSE volume and balance
        let _ = self.read_int()?; // RSE master volume
        let _ = self.read_int()?; // Unknown

        // Equalizer settings (11 bands: 8 + 3 for the EQ)
        for _ in 0..11 {
            let _ = self.read_signed_byte()?;
        }

        debug!("RSE master complete at position: {:?}", self.position());
        Ok(())
    }

    fn read_measure_track_pairs(&mut self, measures: &mut [Measure], num_tracks: usize) -> Result<()> {
        let num_voices = if self.is_gp5() { 2 } else { 1 };

        let mut total_beats = 0;
        for (measure_idx, measure) in measures.iter_mut().enumerate() {
            measure.beats = Vec::with_capacity(num_tracks);

            for track_idx in 0..num_tracks {
                let mut track_beats = Vec::new();

                for voice in 0..num_voices {
                    let pos_before = self.position().unwrap_or(0);
                    let num_beats = self.read_int()?;
                    if num_beats < 0 || num_beats > 500 {
                        debug!("Invalid beat count {} at pos {} (M{} T{} V{})",
                            num_beats, pos_before, measure_idx + 1, track_idx + 1, voice + 1);
                        return Err(Error::Parse(format!("Invalid beat count: {}", num_beats)));
                    }

                    if measure_idx < 5 {
                        trace!("M{} T{} V{}: {} beats at pos {}",
                            measure_idx + 1, track_idx + 1, voice + 1, num_beats, pos_before);
                    }

                    for beat_num in 0..num_beats {
                        let beat_pos = self.position().unwrap_or(0);
                        match self.read_beat() {
                            Ok(beat) => {
                                if measure_idx < 3 && track_idx == 0 && beat_num < 2 {
                                    trace!("  Beat {} at pos {}: {} notes, flags parsed",
                                        beat_num + 1, beat_pos, beat.notes.len());
                                }
                                track_beats.push(beat);
                                total_beats += 1;
                            }
                            Err(e) => {
                                trace!("Beat parse error at pos {}: {:?}", beat_pos, e);
                                // Try to recover
                            }
                        }
                    }
                }

                measure.beats.push(TrackBeats {
                    track: (track_idx + 1) as u8,
                    beats: track_beats,
                });

                // GP5: Check for inter-track RSE effect data
                // After tracks with RSE presets, there may be effect settings (16-18 bytes)
                // Check if next value looks like garbage (not a valid beat count pair)
                if self.is_gp5() {
                    if let Ok(peek) = self.peek_bytes(8) {
                        let val1 = i32::from_le_bytes([peek[0], peek[1], peek[2], peek[3]]);
                        let val2 = i32::from_le_bytes([peek[4], peek[5], peek[6], peek[7]]);

                        // If either value is clearly not a beat count (< 0 or > 100),
                        // this is RSE effect data that needs to be skipped
                        let val1_invalid = val1 < 0 || val1 > 100;
                        let val2_invalid = val2 < 0 || val2 > 100;

                        if val1_invalid || (val1 == 0 && val2_invalid) {
                            let inter_pos = self.position().unwrap_or(0);
                            // Skip until we find valid consecutive beat counts
                            // Limit based on observed RSE data sizes (can be 150+ bytes at measure boundaries)
                            let mut skipped = 0;
                            while skipped < 200 {
                                if let Ok(peek) = self.peek_bytes(8) {
                                    let v1 = i32::from_le_bytes([peek[0], peek[1], peek[2], peek[3]]);
                                    let v2 = i32::from_le_bytes([peek[4], peek[5], peek[6], peek[7]]);

                                    // Found valid beat count pair
                                    // Beat counts realistically 0-64 (rarely more)
                                    if v1 >= 0 && v1 <= 64 && v2 >= 0 && v2 <= 64 {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                                let _ = self.read_byte()?;
                                skipped += 1;
                            }
                            if skipped > 0 {
                                trace!("  Skipped {} bytes inter-track RSE at {}", skipped, inter_pos);
                            }
                        }
                    }
                }
            }
        }
        debug!("Total beats parsed: {}", total_beats);

        Ok(())
    }

    fn read_beat(&mut self) -> Result<Beat> {
        let mut beat = Beat::default();
        let start_pos = self.position().unwrap_or(0);

        let flags = self.read_byte()?;
        trace!("Beat at {}: flags=0x{:02x}", start_pos, flags);

        // Dotted note
        beat.dotted = flags & 0x01 != 0;

        // Status is encoded in top 2 bits of flags (bits 6-7)
        // 0x00 = normal, 0x40 = rest, 0x80 = empty
        let is_rest = (flags & 0xC0) != 0;

        // Duration
        let duration_value = self.read_signed_byte()?;
        beat.duration = Duration::from_value(duration_value);
        trace!("  duration={}", duration_value);

        // Tuplet
        if flags & 0x20 != 0 {
            beat.tuplet = Some(self.read_int()? as u8);
        }

        // Chord diagram
        if flags & 0x02 != 0 {
            self.skip_chord_diagram()?;
        }

        // Text
        if flags & 0x04 != 0 {
            beat.text = Some(self.read_string_int_byte()?);
        }

        // Beat effects
        if flags & 0x08 != 0 {
            beat.effects = self.read_beat_effects()?;
        }

        // Mix change
        if flags & 0x10 != 0 {
            self.skip_mix_change()?;
        }

        // String flags and notes (always read string_flags, but only parse notes if not rest)
        let string_flags = self.read_byte()?;
        trace!("  string_flags=0x{:02x}, is_rest={}", string_flags, is_rest);

        if !is_rest {
            for string_num in 0..7 {
                if string_flags & (1 << (6 - string_num)) != 0 {
                    match self.read_note(string_num + 1) {
                        Ok(note) => beat.notes.push(note),
                        Err(e) => {
                            trace!("Note parse error: {:?}", e);
                        }
                    }
                }
            }
        }

        // GP5 extra data - read after notes
        // Format: variable length based on beat content, may include RSE effect data
        // RSE data can contain: preset strings (string_int_byte format) + config bytes
        if self.is_gp5() {
            let extra_pos = self.position().unwrap_or(0);
            let mut extra_bytes = 0;

            // Skip RSE effect data which may include preset strings
            // Pattern: config bytes + 0-2 string_int_byte format strings + more config
            let mut strings_found = 0;
            while extra_bytes < 200 {
                if let Ok(peek) = self.peek_bytes(8) {
                    let val1 = i32::from_le_bytes([peek[0], peek[1], peek[2], peek[3]]);
                    let val2 = i32::from_le_bytes([peek[4], peek[5], peek[6], peek[7]]);

                    // Check for string_int_byte pattern (string length 5-100)
                    // Format: 4 bytes (length+1), 1 byte (actual length), N bytes (string)
                    if val1 >= 5 && val1 <= 100 && strings_found < 2 {
                        // Verify it looks like a string (5th byte should be val1-1)
                        if peek[4] as i32 == val1 - 1 {
                            // Skip the string_int_byte
                            let str_len = val1 as usize;
                            trace!("  GP5 RSE string at {}: {} chars", self.position().unwrap_or(0), str_len - 1);
                            let _ = self.read_bytes(4)?; // length
                            let _ = self.read_bytes(1)?; // actual length byte
                            let _ = self.read_bytes(str_len - 1)?; // string content
                            extra_bytes += 4 + 1 + (str_len - 1);
                            strings_found += 1;
                            continue;
                        }
                    }

                    // Check for valid beat count pair (both values 0-64)
                    // This indicates we've reached the next voice/track's data
                    if val1 >= 0 && val1 <= 64 && val2 >= 0 && val2 <= 64 {
                        break;
                    }
                }
                let _ = self.read_byte()?;
                extra_bytes += 1;
            }
            if extra_bytes > 0 {
                trace!("  GP5 extra at {}: {} bytes (strings: {})", extra_pos, extra_bytes, strings_found);
            }
        }

        trace!("Beat done at pos {}", self.position().unwrap_or(0));
        Ok(beat)
    }

    fn skip_chord_diagram(&mut self) -> Result<()> {
        let flags = self.read_byte()?;

        if flags & 0x01 != 0 {
            // Full chord definition
            let _ = self.read_bool()?; // sharp
            let _ = self.read_bytes(3)?; // padding
            let _ = self.read_byte()?; // root
            let _ = self.read_byte()?; // type
            let _ = self.read_byte()?; // extension
            let _ = self.read_int()?; // bass
            let _ = self.read_int()?; // tonality
            let _ = self.read_bool()?; // add
            let _ = self.read_string_byte(21)?; // name
            let _ = self.read_bytes(4)?; // padding
            let _ = self.read_byte()?; // fifth
            let _ = self.read_byte()?; // ninth
            let _ = self.read_byte()?; // eleventh
            let _ = self.read_int()?; // base fret

            // Frets for each string
            for _ in 0..7 {
                let _ = self.read_int()?;
            }

            // Barres
            let _ = self.read_byte()?; // num barres
            for _ in 0..5 {
                let _ = self.read_byte()?; // barre frets
            }
            for _ in 0..5 {
                let _ = self.read_byte()?; // barre starts
            }
            for _ in 0..5 {
                let _ = self.read_byte()?; // barre ends
            }

            // Omissions and fingerings
            for _ in 0..7 {
                let _ = self.read_byte()?;
            }
            for _ in 0..7 {
                let _ = self.read_signed_byte()?;
            }
            let _ = self.read_bool()?; // show fingering
        }

        Ok(())
    }

    fn read_beat_effects(&mut self) -> Result<BeatEffects> {
        let mut effects = BeatEffects::default();
        let start_pos = self.position().unwrap_or(0);

        let flags1 = self.read_byte()?;
        let flags2 = self.read_byte()?;
        trace!("BeatEffects at {}: flags1=0x{:02x} flags2=0x{:02x}", start_pos, flags1, flags2);

        effects.fade_in = flags1 & 0x10 != 0;

        // Vibrato/slap
        if flags1 & 0x20 != 0 {
            let _ = self.read_byte()?;
        }

        // Tremolo bar
        if flags2 & 0x04 != 0 {
            effects.tremolo_bar = Some(self.read_bend_points()?);
        }

        // Stroke
        if flags1 & 0x40 != 0 {
            let up = self.read_byte()?;
            let down = self.read_byte()?;
            if up > 0 {
                effects.stroke = Some(Stroke::Up(up));
            } else if down > 0 {
                effects.stroke = Some(Stroke::Down(down));
            }
        }

        // Pick stroke
        if flags2 & 0x02 != 0 {
            let _ = self.read_byte()?;
        }

        // Wah (GP5 only)
        if self.is_gp5() && flags2 & 0x08 != 0 {
            let _ = self.read_signed_byte()?;
        }

        trace!("BeatEffects done at {}", self.position().unwrap_or(0));
        Ok(effects)
    }

    fn skip_mix_change(&mut self) -> Result<()> {
        let pos = self.position().unwrap_or(0);
        let instrument = self.read_signed_byte()?;
        trace!("MixChange at {}: instrument={}", pos, instrument);

        // GP5 with all -1 values: The next 6 bytes may all be 0xff (-1) indicating "no change"
        // This is valid data, not garbage

        let volume = self.read_signed_byte()?;
        let pan = self.read_signed_byte()?;
        let chorus = self.read_signed_byte()?;
        let reverb = self.read_signed_byte()?;
        let phaser = self.read_signed_byte()?;
        let tremolo = self.read_signed_byte()?;
        trace!("  volume={}, pan={}, chorus={}, reverb={}, phaser={}, tremolo={}",
            volume, pan, chorus, reverb, phaser, tremolo);

        // Tempo name (GP5)
        if self.is_gp5() {
            let name_pos = self.position().unwrap_or(0);
            let _ = self.read_string_int_byte()?;
            trace!("  tempo name at {}, now at {}", name_pos, self.position().unwrap_or(0));
        }

        let tempo = self.read_int()?;
        trace!("  tempo={}", tempo);

        // Transition durations
        if volume >= 0 { let _ = self.read_byte()?; }
        if pan >= 0 { let _ = self.read_byte()?; }
        if chorus >= 0 { let _ = self.read_byte()?; }
        if reverb >= 0 { let _ = self.read_byte()?; }
        if phaser >= 0 { let _ = self.read_byte()?; }
        if tremolo >= 0 { let _ = self.read_byte()?; }
        if tempo > 0 {
            let _ = self.read_byte()?;
            // GP5: hide tempo flag
            if self.is_gp5() {
                let _ = self.read_byte()?;
            }
        }

        // All tracks flag (GP4+)
        if self.is_gp4_or_later() {
            let _ = self.read_byte()?;
        }

        // GP5 RSE: Additional RSE instrument data when instrument is changed
        // Format: 7 bytes base RSE + 11 bytes settings + optional preset names
        if self.is_gp5() && instrument >= 0 {
            let rse_pos = self.position().unwrap_or(0);

            // Skip base RSE settings (7 bytes)
            let _ = self.read_bytes(7)?;
            trace!("  Skipped GP5 RSE base at {} (7 bytes)", rse_pos);

            // Check for RSE preset data by looking ahead
            // Pattern: 11 bytes of flags/settings, then potentially preset name(s)
            // Preset names are in string_int_byte format (4 bytes length, 1 byte actual, then string)
            if let Ok(peek) = self.peek_bytes(15) {
                let first_int = i32::from_le_bytes([peek[0], peek[1], peek[2], peek[3]]);

                // RSE section typically starts with a small flag (0, 1, 2, 3)
                if first_int >= 0 && first_int <= 3 {
                    // Check if there's a preset name at offset 11
                    // Preset name length would be bytes 11-14 as i32 (typically 5-50 chars)
                    let preset_len = i32::from_le_bytes([peek[11], peek[12], peek[13], peek[14]]);

                    if preset_len > 3 && preset_len < 100 {
                        // Likely has RSE preset names - skip the full RSE block
                        trace!("  RSE preset detected: flag={}, preset_len={} at {}",
                            first_int, preset_len, self.position().unwrap_or(0));

                        // Skip RSE instrument info: 11 bytes settings
                        let _ = self.read_bytes(11)?;

                        // Skip RSE preset name 1 (string_int_byte format)
                        let _ = self.read_string_int_byte()?;

                        // Skip RSE preset name 2 (string_int_byte format)
                        let _ = self.read_string_int_byte()?;

                        trace!("  RSE presets done at {}", self.position().unwrap_or(0));
                    }
                }
            }
        }

        trace!("MixChange done at {}", self.position().unwrap_or(0));
        Ok(())
    }

    fn read_note(&mut self, string: u8) -> Result<Note> {
        let mut note = Note {
            string,
            ..Default::default()
        };
        let pos = self.position().unwrap_or(0);

        let flags = self.read_byte()?;
        trace!("Note at {} for string {}: flags=0x{:02x}", pos, string, flags);

        note.ghost = flags & 0x04 != 0;
        let has_effects = flags & 0x08 != 0;

        // Note type (if 0x20 flag set)
        if flags & 0x20 != 0 {
            let note_type = self.read_byte()?;
            note.tied = note_type == 0x02;
        }

        if self.is_gp3() {
            // GP3: Fret is always present, right after note_type
            let fret = self.read_signed_byte()?;
            note.fret = fret as u8;
            trace!("  fret={} (GP3)", fret);
        } else {
            // GP4/GP5: Duration, velocity, fret, fingering are conditional

            // Duration (GP4+)
            if flags & 0x01 != 0 {
                let _ = self.read_signed_byte()?;
                let _ = self.read_signed_byte()?;
            }

            // Velocity
            if flags & 0x10 != 0 {
                let vel = self.read_signed_byte()?;
                note.velocity = vel as u8;
                trace!("  velocity={}", vel);
            }

            // Fret (only when 0x20 flag is set in GP4/5)
            if flags & 0x20 != 0 {
                let fret = self.read_signed_byte()?;
                note.fret = fret as u8;
                trace!("  fret={}", fret);
            }

            // Fingering (GP4+)
            if flags & 0x80 != 0 {
                let _ = self.read_signed_byte()?;
                let _ = self.read_signed_byte()?;
            }
        }

        // Effects
        if has_effects {
            note.effects = self.read_note_effects()?;
        }

        Ok(note)
    }

    fn read_note_effects(&mut self) -> Result<NoteEffects> {
        let mut effects = NoteEffects::default();

        let flags1 = self.read_byte()?;
        let flags2 = if self.is_gp4_or_later() {
            self.read_byte()?
        } else {
            0
        };

        // Bend
        if flags1 & 0x01 != 0 {
            effects.bend = Some(self.read_bend_points()?);
        }

        // Grace note
        if flags1 & 0x02 != 0 {
            effects.grace_note = Some(self.read_grace_note()?);
        }

        // Tremolo picking
        if flags2 & 0x04 != 0 {
            let _ = self.read_byte()?;
        }

        // Slide
        if flags2 & 0x08 != 0 {
            let slide = self.read_signed_byte()?;
            effects.slide = Some(match slide {
                1 => SlideType::ShiftSlide,
                2 => SlideType::LegatoSlide,
                3 => SlideType::OutDownwards,
                4 => SlideType::OutUpwards,
                -1 => SlideType::IntoFromAbove,
                -2 => SlideType::IntoFromBelow,
                _ => SlideType::ShiftSlide,
            });
        }

        // Harmonic
        if flags2 & 0x10 != 0 {
            let harmonic_type = self.read_byte()?;
            effects.harmonic = Some(match harmonic_type {
                1 => HarmonicType::Natural,
                2 => HarmonicType::Artificial,
                3 => HarmonicType::Pinch,
                4 => HarmonicType::Tap,
                5 => HarmonicType::Semi,
                6 => HarmonicType::Feedback,
                _ => HarmonicType::Natural,
            });

            // GP5 harmonic extra data
            if self.is_gp5() {
                match harmonic_type {
                    2 => {
                        // Artificial: octave + fret
                        let _ = self.read_byte()?;
                        let _ = self.read_byte()?;
                    }
                    4 => {
                        // Tap: fret
                        let _ = self.read_byte()?;
                    }
                    _ => {}
                }
            }
        }

        // Trill
        if flags2 & 0x20 != 0 {
            let fret = self.read_byte()?;
            let speed = self.read_byte()?;
            effects.trill = Some((fret, Duration::from_value(speed as i8)));
        }

        effects.hammer_on = flags1 & 0x02 != 0;
        effects.vibrato = flags2 & 0x40 != 0;

        Ok(effects)
    }

    fn read_bend_points(&mut self) -> Result<Vec<BendPoint>> {
        let _ = self.read_byte()?; // bend type
        let _ = self.read_int()?; // bend value

        let num_points = self.read_int()? as usize;
        let mut points = Vec::with_capacity(num_points.min(50));

        for _ in 0..num_points.min(50) {
            let position = (self.read_int()? / 60) as u8;
            let value = (self.read_int()? / 25) as i16;
            let vibrato = self.read_bool()?;

            points.push(BendPoint {
                position,
                value,
                vibrato,
            });
        }

        Ok(points)
    }

    fn read_grace_note(&mut self) -> Result<GraceNote> {
        let fret = self.read_byte()?;
        let velocity = self.read_byte()?;
        let duration = self.read_byte()?;
        let flags = self.read_byte()?;

        Ok(GraceNote {
            fret,
            velocity,
            duration,
            on_beat: flags & 0x01 != 0,
            dead: flags & 0x02 != 0,
        })
    }
}

/// MIDI channel configuration
#[derive(Debug, Clone, Default)]
pub struct MidiChannel {
    pub program: u8,
    pub volume: u8,
    pub pan: u8,
    pub chorus: u8,
    pub reverb: u8,
    pub phaser: u8,
    pub tremolo: u8,
}
