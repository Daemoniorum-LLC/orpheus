//! Guitar Pro 7/6 (.gp, .gpx) file parser
//!
//! GP7 and GP6 files are ZIP archives containing:
//! - Content/score.gpif - Main score data in XML format
//! - Content/BinaryStylesheet - Binary styling data (not parsed)
//! - Various audio/preview files (not parsed)

use super::types::*;
use crate::{Error, Result};
use quick_xml::events::{BytesCData, BytesText, Event};
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::{debug, warn};
use zip::ZipArchive;

/// GP7/GP6 file parser
pub struct Gp7Parser;

impl Gp7Parser {
    /// Parse a GP7/GP6 file from disk
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<GuitarProFile> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        Self::parse_reader(reader)
    }

    /// Parse a GP7/GP6 file from a reader
    pub fn parse_reader<R: Read + std::io::Seek>(reader: R) -> Result<GuitarProFile> {
        let mut archive = ZipArchive::new(reader).map_err(|e| {
            Error::InvalidFormat(format!("Failed to open GP7/GP6 archive: {}", e))
        })?;

        // Find and extract score.gpif
        let gpif_content = Self::extract_gpif(&mut archive)?;

        // Parse the XML
        Self::parse_gpif(&gpif_content)
    }

    /// Parse GPIF XML content directly (used by BCFZ parser)
    pub fn parse_xml(xml: &str) -> Result<GuitarProFile> {
        Self::parse_gpif(xml)
    }

    /// Extract score.gpif from the ZIP archive
    fn extract_gpif<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Result<String> {
        // Try common paths for score.gpif
        let paths = ["Content/score.gpif", "score.gpif", "content/score.gpif"];

        for path in paths {
            if let Ok(mut file) = archive.by_name(path) {
                let mut content = String::new();
                file.read_to_string(&mut content).map_err(|e| {
                    Error::InvalidFormat(format!("Failed to read score.gpif: {}", e))
                })?;
                debug!("Found score.gpif at {} ({} bytes)", path, content.len());
                return Ok(content);
            }
        }

        // List available files for debugging
        let files: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .collect();

        Err(Error::InvalidFormat(format!(
            "score.gpif not found in archive. Available files: {:?}",
            files
        )))
    }

    /// Parse the GPIF XML content
    fn parse_gpif(xml: &str) -> Result<GuitarProFile> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut gp_file = GuitarProFile::default();
        let mut ctx = ParseContext::default();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    ctx.path.push(tag.clone());

                    // Extract id and name attributes
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        if key == "id" {
                            ctx.current_id = value.parse().ok();
                        } else if key == "name" && tag == "Property" {
                            // GP8 format: <Property name="Fret"> or <Property name="String">
                            ctx.current_property_name = Some(value);
                        }
                    }

                    Self::handle_start(&mut ctx);
                }
                Ok(Event::End(ref e)) => {
                    Self::handle_end(&mut gp_file, &mut ctx);
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "Property" {
                        ctx.current_property_name = None;
                    }
                    ctx.path.pop();
                    ctx.current_id = None;
                }
                Ok(Event::Text(e)) => {
                    let text = Self::decode_text(&e);
                    if !text.is_empty() {
                        ctx.text_buffer = text;
                    }
                }
                Ok(Event::CData(e)) => {
                    let text = Self::decode_cdata(&e);
                    if !text.is_empty() {
                        ctx.text_buffer = text;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    ctx.path.push(tag);

                    // Handle empty elements with ref attributes
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        if key == "ref" {
                            if let Ok(ref_id) = value.parse::<u32>() {
                                Self::handle_reference(&mut ctx, ref_id);
                            }
                        }
                    }

                    ctx.path.pop();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warn!("XML parse error at position {}: {:?}", reader.buffer_position(), e);
                }
                _ => {}
            }
            buf.clear();
        }

        // Build the final structure from parsed data
        Self::build_file(&mut gp_file, &ctx);

        Ok(gp_file)
    }

    fn decode_text(e: &BytesText) -> String {
        e.unescape().unwrap_or_default().to_string()
    }

    fn decode_cdata(e: &BytesCData) -> String {
        String::from_utf8_lossy(e.as_ref()).to_string()
    }

    /// Handle element start
    fn handle_start(ctx: &mut ParseContext) {
        let path = ctx.path.join("/");

        match path.as_str() {
            "GPIF/Tracks/Track" => {
                ctx.current_track = Some(TrackBuilder::new(ctx.current_id.unwrap_or(0)));
            }
            "GPIF/MasterBars/MasterBar" => {
                ctx.current_master_bar = Some(MasterBarBuilder::default());
            }
            "GPIF/Notes/Note" => {
                ctx.current_note = Some(NoteBuilder::new(ctx.current_id.unwrap_or(0)));
            }
            "GPIF/Rhythms/Rhythm" => {
                ctx.current_rhythm = Some(RhythmBuilder::new(ctx.current_id.unwrap_or(0)));
            }
            "GPIF/Beats/Beat" => {
                ctx.current_beat = Some(BeatBuilder::new(ctx.current_id.unwrap_or(0)));
            }
            "GPIF/Voices/Voice" => {
                ctx.current_voice = Some(VoiceBuilder::new(ctx.current_id.unwrap_or(0)));
            }
            "GPIF/Bars/Bar" => {
                ctx.current_bar = Some(BarBuilder::new(ctx.current_id.unwrap_or(0)));
            }
            _ => {}
        }

        // Clear text buffer for new element
        ctx.text_buffer.clear();
    }

    /// Handle element end - process collected text
    fn handle_end(gp_file: &mut GuitarProFile, ctx: &mut ParseContext) {
        let path = ctx.path.join("/");
        let text = ctx.text_buffer.trim().to_string();

        match path.as_str() {
            // Version
            "GPIF/GPVersion" => {
                if text.starts_with('7') {
                    gp_file.version = GpVersion::Gp7;
                } else if text.starts_with('6') {
                    gp_file.version = GpVersion::Gp6;
                }
            }

            // Score metadata
            "GPIF/Score/Title" => gp_file.info.title = text,
            "GPIF/Score/SubTitle" => gp_file.info.subtitle = text,
            "GPIF/Score/Artist" => gp_file.info.artist = text,
            "GPIF/Score/Album" => gp_file.info.album = text,
            "GPIF/Score/Words" => gp_file.info.author = text.clone(),
            "GPIF/Score/Music" => {
                if gp_file.info.author.is_empty() {
                    gp_file.info.author = text;
                }
            }
            "GPIF/Score/Copyright" => gp_file.info.copyright = text,
            "GPIF/Score/Tabber" => gp_file.info.tab_author = text,
            "GPIF/Score/Instructions" => gp_file.info.instructions = text,
            "GPIF/Score/Notices" => {
                if !text.is_empty() {
                    gp_file.info.comments.push(text);
                }
            }

            // Track parsing - complete track and store it
            "GPIF/Tracks/Track" => {
                if let Some(track) = ctx.current_track.take() {
                    ctx.tracks.push(track);
                }
            }
            "GPIF/Tracks/Track/Name" => {
                if let Some(ref mut track) = ctx.current_track {
                    track.name = text;
                }
            }
            "GPIF/Tracks/Track/Color" => {
                // Color is space-separated RGB: "117 201 227"
                if let Some(ref mut track) = ctx.current_track {
                    let parts: Vec<u8> = text.split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if parts.len() >= 3 {
                        track.color = (parts[0], parts[1], parts[2]);
                    }
                }
            }

            // MasterBar parsing
            "GPIF/MasterBars/MasterBar" => {
                if let Some(bar) = ctx.current_master_bar.take() {
                    ctx.master_bars.push(bar);
                }
            }
            "GPIF/MasterBars/MasterBar/Time" => {
                if let Some(ref mut bar) = ctx.current_master_bar {
                    if let Some((num, denom)) = text.split_once('/') {
                        bar.time_signature = Some(TimeSignature {
                            numerator: num.parse().unwrap_or(4),
                            denominator: denom.parse().unwrap_or(4),
                        });
                    }
                }
            }
            "GPIF/MasterBars/MasterBar/Bars" => {
                if let Some(ref mut bar) = ctx.current_master_bar {
                    bar.bar_ids = text.split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                }
            }
            "GPIF/MasterBars/MasterBar/Section/Text" => {
                if let Some(ref mut bar) = ctx.current_master_bar {
                    bar.marker = Some(text);
                }
            }

            // Note parsing
            "GPIF/Notes/Note" => {
                if let Some(note) = ctx.current_note.take() {
                    ctx.notes.push(note);
                }
            }
            // GP7 format: direct children
            "GPIF/Notes/Note/String" => {
                if let Some(ref mut note) = ctx.current_note {
                    note.string = text.parse().unwrap_or(1);
                }
            }
            "GPIF/Notes/Note/Fret" => {
                if let Some(ref mut note) = ctx.current_note {
                    note.fret = text.parse().unwrap_or(0);
                }
            }
            "GPIF/Notes/Note/Velocity" => {
                if let Some(ref mut note) = ctx.current_note {
                    note.velocity = text.parse().unwrap_or(100);
                }
            }
            // GP8 format: nested in Properties/Property
            // <Property name="Fret"><Fret>6</Fret></Property>
            "GPIF/Notes/Note/Properties/Property/Fret" => {
                if ctx.current_property_name.as_deref() == Some("Fret") {
                    if let Some(ref mut note) = ctx.current_note {
                        note.fret = text.parse().unwrap_or(0);
                    }
                }
            }
            // <Property name="String"><String>1</String></Property>
            "GPIF/Notes/Note/Properties/Property/String" => {
                if ctx.current_property_name.as_deref() == Some("String") {
                    if let Some(ref mut note) = ctx.current_note {
                        // GP8 uses 0-indexed strings
                        note.string = text.parse::<u8>().unwrap_or(0) + 1;
                    }
                }
            }

            // Rhythm parsing
            "GPIF/Rhythms/Rhythm" => {
                if let Some(rhythm) = ctx.current_rhythm.take() {
                    ctx.rhythms.push(rhythm);
                }
            }
            "GPIF/Rhythms/Rhythm/NoteValue" => {
                if let Some(ref mut rhythm) = ctx.current_rhythm {
                    rhythm.duration = match text.as_str() {
                        "Whole" => Duration::Whole,
                        "Half" => Duration::Half,
                        "Quarter" => Duration::Quarter,
                        "Eighth" => Duration::Eighth,
                        "16th" => Duration::Sixteenth,
                        "32nd" => Duration::ThirtySecond,
                        "64th" => Duration::SixtyFourth,
                        _ => Duration::Quarter,
                    };
                }
            }

            // Beat parsing
            "GPIF/Beats/Beat" => {
                if let Some(beat) = ctx.current_beat.take() {
                    ctx.beats.push(beat);
                }
            }
            "GPIF/Beats/Beat/Notes" => {
                if let Some(ref mut beat) = ctx.current_beat {
                    beat.note_ids = text.split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                }
            }

            // Voice parsing
            "GPIF/Voices/Voice" => {
                if let Some(voice) = ctx.current_voice.take() {
                    ctx.voices.push(voice);
                }
            }
            "GPIF/Voices/Voice/Beats" => {
                if let Some(ref mut voice) = ctx.current_voice {
                    voice.beat_ids = text.split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                }
            }

            // Bar parsing
            "GPIF/Bars/Bar" => {
                if let Some(bar) = ctx.current_bar.take() {
                    ctx.bars.push(bar);
                }
            }
            "GPIF/Bars/Bar/Voices" => {
                if let Some(ref mut bar) = ctx.current_bar {
                    bar.voice_ids = text.split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                }
            }

            // Tempo from automation
            "GPIF/MasterTrack/Automations/Automation/Value" => {
                // Tempo automation value is like "162 2"
                if ctx.path.iter().any(|p| p == "Automation") {
                    if let Some(tempo_str) = text.split_whitespace().next() {
                        if let Ok(tempo) = tempo_str.parse::<u16>() {
                            if gp_file.tempo == 120 { // Only set if not already set
                                gp_file.tempo = tempo;
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }

    /// Handle references (like rhythm refs in beats)
    fn handle_reference(ctx: &mut ParseContext, ref_id: u32) {
        let current_element = ctx.path.last().map(|s| s.as_str());

        if current_element == Some("Rhythm") {
            if let Some(ref mut beat) = ctx.current_beat {
                beat.rhythm_id = Some(ref_id);
            }
        }
    }

    /// Build the final GuitarProFile from parsed data
    fn build_file(gp_file: &mut GuitarProFile, ctx: &ParseContext) {
        // Build tracks
        for track_builder in &ctx.tracks {
            gp_file.tracks.push(track_builder.build());
        }

        // Build rhythms lookup
        let rhythms: HashMap<u32, Duration> = ctx
            .rhythms
            .iter()
            .map(|r| (r.id, r.duration))
            .collect();

        // Build notes lookup
        let notes: HashMap<u32, Note> = ctx
            .notes
            .iter()
            .map(|n| (n.id, n.build()))
            .collect();

        // Build beats lookup
        let beats: HashMap<u32, Beat> = ctx
            .beats
            .iter()
            .map(|b| {
                let mut beat = Beat::default();
                beat.duration = rhythms.get(&b.rhythm_id.unwrap_or(0)).copied().unwrap_or(Duration::Quarter);
                beat.notes = b.note_ids.iter().filter_map(|id| notes.get(id).cloned()).collect();
                (b.id, beat)
            })
            .collect();

        // Build voices lookup
        let voices: HashMap<u32, Vec<Beat>> = ctx
            .voices
            .iter()
            .map(|v| {
                let voice_beats: Vec<Beat> = v.beat_ids.iter()
                    .filter_map(|id| beats.get(id).cloned())
                    .collect();
                (v.id, voice_beats)
            })
            .collect();

        // Build bars lookup (bar id -> beats)
        let bars: HashMap<u32, Vec<Beat>> = ctx
            .bars
            .iter()
            .map(|b| {
                let bar_beats: Vec<Beat> = b.voice_ids.iter()
                    .flat_map(|id| voices.get(id).cloned().unwrap_or_default())
                    .collect();
                (b.id, bar_beats)
            })
            .collect();

        // Build measures from master bars
        for (i, master_bar) in ctx.master_bars.iter().enumerate() {
            let mut measure = Measure::default();
            measure.number = (i + 1) as u16;
            measure.time_signature = master_bar.time_signature;
            measure.marker = master_bar.marker.clone();
            measure.repeat_start = master_bar.repeat_start;
            measure.repeat_end = master_bar.repeat_end;
            measure.tempo = master_bar.tempo;

            // Add beats for each track
            for (track_idx, track) in gp_file.tracks.iter().enumerate() {
                let track_beats = TrackBeats {
                    track: track.number,
                    beats: master_bar.bar_ids.get(track_idx)
                        .and_then(|id| bars.get(id))
                        .cloned()
                        .unwrap_or_default(),
                };
                measure.beats.push(track_beats);
            }

            gp_file.measures.push(measure);
        }

        debug!(
            "Parsed GP7 file: {} tracks, {} measures, {} notes, {} beats",
            gp_file.tracks.len(),
            gp_file.measures.len(),
            ctx.notes.len(),
            ctx.beats.len(),
        );
    }
}

/// Parse context for building the file structure
#[derive(Default)]
struct ParseContext {
    path: Vec<String>,
    current_id: Option<u32>,
    text_buffer: String,

    /// Current property name being parsed (for GP8 Property-based format)
    current_property_name: Option<String>,

    // Builders for current elements
    current_track: Option<TrackBuilder>,
    current_master_bar: Option<MasterBarBuilder>,
    current_note: Option<NoteBuilder>,
    current_rhythm: Option<RhythmBuilder>,
    current_beat: Option<BeatBuilder>,
    current_voice: Option<VoiceBuilder>,
    current_bar: Option<BarBuilder>,

    // Completed elements
    tracks: Vec<TrackBuilder>,
    master_bars: Vec<MasterBarBuilder>,
    notes: Vec<NoteBuilder>,
    rhythms: Vec<RhythmBuilder>,
    beats: Vec<BeatBuilder>,
    voices: Vec<VoiceBuilder>,
    bars: Vec<BarBuilder>,
}

/// Builder for Track
struct TrackBuilder {
    id: u32,
    name: String,
    color: (u8, u8, u8),
    strings: u8,
    tuning: Vec<u8>,
    capo: u8,
    is_drums: bool,
}

impl TrackBuilder {
    fn new(id: u32) -> Self {
        Self {
            id,
            name: String::new(),
            color: (255, 0, 0),
            strings: 6,
            tuning: vec![64, 59, 55, 50, 45, 40],
            capo: 0,
            is_drums: false,
        }
    }

    fn build(&self) -> Track {
        Track {
            number: (self.id + 1) as u8,
            name: self.name.clone(),
            strings: self.strings,
            tuning: self.tuning.clone(),
            channel: 0,
            program: 25,
            is_drums: self.is_drums,
            volume: 100,
            pan: 64,
            capo: self.capo,
            color: self.color,
        }
    }
}

/// Builder for MasterBar
#[derive(Default)]
struct MasterBarBuilder {
    time_signature: Option<TimeSignature>,
    tempo: Option<u16>,
    marker: Option<String>,
    repeat_start: bool,
    repeat_end: u8,
    bar_ids: Vec<u32>,
}

/// Builder for Note
struct NoteBuilder {
    id: u32,
    string: u8,
    fret: u8,
    velocity: u8,
    tied: bool,
    ghost: bool,
    effects: NoteEffects,
}

impl NoteBuilder {
    fn new(id: u32) -> Self {
        Self {
            id,
            string: 1,
            fret: 0,
            velocity: 100,
            tied: false,
            ghost: false,
            effects: NoteEffects::default(),
        }
    }

    fn build(&self) -> Note {
        Note {
            string: self.string,
            fret: self.fret,
            velocity: self.velocity,
            tied: self.tied,
            ghost: self.ghost,
            effects: self.effects.clone(),
        }
    }
}

/// Builder for Rhythm
struct RhythmBuilder {
    id: u32,
    duration: Duration,
}

impl RhythmBuilder {
    fn new(id: u32) -> Self {
        Self {
            id,
            duration: Duration::Quarter,
        }
    }
}

/// Builder for Beat
struct BeatBuilder {
    id: u32,
    rhythm_id: Option<u32>,
    note_ids: Vec<u32>,
}

impl BeatBuilder {
    fn new(id: u32) -> Self {
        Self {
            id,
            rhythm_id: None,
            note_ids: Vec::new(),
        }
    }
}

/// Builder for Voice
struct VoiceBuilder {
    id: u32,
    beat_ids: Vec<u32>,
}

impl VoiceBuilder {
    fn new(id: u32) -> Self {
        Self {
            id,
            beat_ids: Vec::new(),
        }
    }
}

/// Builder for Bar
struct BarBuilder {
    id: u32,
    voice_ids: Vec<u32>,
}

impl BarBuilder {
    fn new(id: u32) -> Self {
        Self {
            id,
            voice_ids: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_gpif() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<GPIF>
  <GPVersion>7</GPVersion>
  <Score>
    <Title><![CDATA[Test Song]]></Title>
    <Artist><![CDATA[Test Artist]]></Artist>
    <Album><![CDATA[Test Album]]></Album>
  </Score>
  <Tracks>
    <Track id="0">
      <Name><![CDATA[Lead Guitar]]></Name>
      <Color>255 128 0</Color>
    </Track>
  </Tracks>
  <MasterBars>
    <MasterBar>
      <Time>4/4</Time>
      <Bars>0</Bars>
    </MasterBar>
  </MasterBars>
  <Bars>
    <Bar id="0">
      <Voices>0</Voices>
    </Bar>
  </Bars>
  <Voices>
    <Voice id="0">
      <Beats>0</Beats>
    </Voice>
  </Voices>
  <Beats>
    <Beat id="0">
      <Rhythm ref="0"/>
      <Notes>0</Notes>
    </Beat>
  </Beats>
  <Notes>
    <Note id="0">
      <String>1</String>
      <Fret>5</Fret>
    </Note>
  </Notes>
  <Rhythms>
    <Rhythm id="0">
      <NoteValue>Quarter</NoteValue>
    </Rhythm>
  </Rhythms>
</GPIF>"#;

        let result = Gp7Parser::parse_gpif(xml);
        assert!(result.is_ok());

        let gp_file = result.unwrap();
        assert_eq!(gp_file.version, GpVersion::Gp7);
        assert_eq!(gp_file.info.title, "Test Song");
        assert_eq!(gp_file.info.artist, "Test Artist");
        assert_eq!(gp_file.tracks.len(), 1);
        assert_eq!(gp_file.tracks[0].name, "Lead Guitar");
        assert_eq!(gp_file.tracks[0].color, (255, 128, 0));
        assert_eq!(gp_file.measures.len(), 1);
    }
}
