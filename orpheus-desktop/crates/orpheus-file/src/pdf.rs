//! PDF tab sheet export
//!
//! Renders tablature documents as professional-looking PDF sheet music.

use std::io::BufWriter;
use std::path::Path;

use printpdf::{
    BuiltinFont, Color, IndirectFontRef, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference, Point, Rgb,
};

use orpheus_core::tab::{Instrument, TabBeat, TabDocument, TabMeasure, Technique};

/// PDF export options
#[derive(Debug, Clone)]
pub struct PdfExportOptions {
    /// Page size (Letter, A4, etc.)
    pub page_size: PageSize,
    /// Page margins in mm
    pub margin_mm: f32,
    /// Title font size in points
    pub title_font_size: f32,
    /// Tablature font size in points
    pub tab_font_size: f32,
    /// String height in mm
    pub string_height_mm: f32,
    /// Fret width in mm
    pub fret_width_mm: f32,
    /// Show tempo marking
    pub show_tempo: bool,
    /// Show time signature
    pub show_time_signature: bool,
    /// Show track names
    pub show_track_names: bool,
    /// Show measure numbers
    pub show_measure_numbers: bool,
    /// Include technique annotations
    pub show_techniques: bool,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            page_size: PageSize::Letter,
            margin_mm: 20.0,
            title_font_size: 24.0,
            tab_font_size: 10.0,
            string_height_mm: 4.0,
            fret_width_mm: 8.0,
            show_tempo: true,
            show_time_signature: true,
            show_track_names: true,
            show_measure_numbers: true,
            show_techniques: true,
        }
    }
}

/// Standard page sizes
#[derive(Debug, Clone, Copy)]
pub enum PageSize {
    /// US Letter (8.5 x 11 inches)
    Letter,
    /// A4 (210 x 297 mm)
    A4,
    /// Legal (8.5 x 14 inches)
    Legal,
    /// Custom size (width x height in mm)
    Custom(f32, f32),
}

impl PageSize {
    /// Get page dimensions in mm
    pub fn dimensions_mm(&self) -> (f32, f32) {
        match self {
            Self::Letter => (215.9, 279.4),
            Self::A4 => (210.0, 297.0),
            Self::Legal => (215.9, 355.6),
            Self::Custom(w, h) => (*w, *h),
        }
    }
}

/// Export result
#[derive(Debug)]
pub struct PdfExportResult {
    /// Path to the exported PDF file
    pub path: std::path::PathBuf,
    /// Number of pages
    pub page_count: usize,
    /// Total measures exported
    pub measure_count: usize,
    /// Total tracks exported
    pub track_count: usize,
}

/// Export a tab document to PDF
pub fn export_pdf<P: AsRef<Path>>(
    document: &TabDocument,
    path: P,
    options: &PdfExportOptions,
) -> Result<PdfExportResult, String> {
    let path = path.as_ref();

    // Calculate page dimensions
    let (page_width, page_height) = options.page_size.dimensions_mm();
    let usable_width = page_width - 2.0 * options.margin_mm;

    // Get document title from metadata
    let title = &document.metadata.title;

    // Get time signature from first measure or default
    let time_sig = document.measures.first()
        .and_then(|m| m.time_signature)
        .unwrap_or_default();
    let beats_per_measure = time_sig.numerator as f32;
    let measure_width = beats_per_measure * options.fret_width_mm * 4.0; // Assume quarter note = 4 fret widths
    let measures_per_line = ((usable_width - 20.0) / measure_width).floor() as usize;
    let measures_per_line = measures_per_line.max(1);

    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        title,
        Mm(page_width),
        Mm(page_height),
        "Layer 1",
    );

    // Get fonts
    let title_font = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;
    let text_font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let tab_font = doc.add_builtin_font(BuiltinFont::CourierBold).map_err(|e| e.to_string())?;

    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut page_count = 1;

    // Draw title on first page
    draw_title(&current_layer, &document.metadata.title, page_width, page_height, options, &title_font);

    // Draw tempo and time signature
    let mut y_pos = page_height - options.margin_mm - 30.0;

    if options.show_tempo {
        let tempo_text = format!("Tempo: {:.0} BPM", document.tempo_map.base_tempo);
        draw_text(&current_layer, &tempo_text, options.margin_mm, y_pos, &text_font, 10.0);
    }

    if options.show_time_signature {
        let ts_text = format!("{}/{}", time_sig.numerator, time_sig.denominator);
        draw_text(&current_layer, &ts_text, options.margin_mm + 80.0, y_pos, &text_font, 10.0);
    }

    y_pos -= 15.0;

    // Helper closure to add a new page
    let start_new_page = |doc: &PdfDocumentReference, page_count: &mut usize, page_width: f32, page_height: f32, title: &str, options: &PdfExportOptions, text_font: &IndirectFontRef| -> PdfLayerReference {
        let (page, layer) = doc.add_page(Mm(page_width), Mm(page_height), format!("Page {}", *page_count + 1));
        *page_count += 1;
        let layer_ref = doc.get_page(page).get_layer(layer);

        // Draw page header (title and page number)
        draw_text(&layer_ref, title, options.margin_mm, page_height - options.margin_mm - 5.0, text_font, 10.0);
        draw_text(&layer_ref, &format!("Page {}", *page_count), page_width - options.margin_mm - 20.0, page_height - options.margin_mm - 5.0, text_font, 10.0);

        layer_ref
    };

    // Draw each track
    for (_track_idx, track) in document.tracks.iter().enumerate() {
        // Track header
        if options.show_track_names {
            // Check if we need a new page for track header
            if y_pos < options.margin_mm + 30.0 {
                current_layer = start_new_page(&doc, &mut page_count, page_width, page_height, &document.metadata.title, options, &text_font);
                y_pos = page_height - options.margin_mm - 20.0;
            }

            draw_text(
                &current_layer,
                &track.name,
                options.margin_mm,
                y_pos,
                &text_font,
                12.0,
            );
            y_pos -= 10.0;
        }

        // Get string count
        let string_count = match &track.instrument {
            Instrument::StringedInstrument(s) => s.string_count,
            _ => continue, // Skip non-stringed instruments
        };

        // Track height
        let track_height = string_count as f32 * options.string_height_mm;

        // Draw measures for this track
        let mut measure_in_line = 0;
        let mut current_x = options.margin_mm;

        for (measure_idx, measure) in document.measures.iter().enumerate() {
            // Find beats for this track
            let track_beats = measure.track_beats.iter()
                .find(|tb| tb.track_id == track.id);

            // Calculate actual measure width
            let beat_count = track_beats.map(|tb| tb.beats.len()).unwrap_or(4);
            let actual_measure_width = beat_count as f32 * options.fret_width_mm;

            // Check if we need to wrap to next line
            if measure_in_line >= measures_per_line {
                measure_in_line = 0;
                current_x = options.margin_mm;
                y_pos -= track_height + 10.0;

                // Check if we need a new page
                if y_pos < options.margin_mm + track_height + 20.0 {
                    current_layer = start_new_page(&doc, &mut page_count, page_width, page_height, &document.metadata.title, options, &text_font);
                    y_pos = page_height - options.margin_mm - 20.0;
                }
            }

            // Draw measure
            draw_measure(
                &current_layer,
                measure,
                track_beats.map(|tb| tb.beats.as_slice()).unwrap_or(&[]),
                measure_idx,
                current_x,
                y_pos,
                actual_measure_width,
                string_count,
                options,
                &tab_font,
            );

            current_x += actual_measure_width + 2.0;
            measure_in_line += 1;
        }

        y_pos -= track_height + 20.0;
    }

    // Write PDF to file
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).map_err(|e| e.to_string())?;

    Ok(PdfExportResult {
        path: path.to_path_buf(),
        page_count,
        measure_count: document.measures.len(),
        track_count: document.tracks.len(),
    })
}

/// Draw the document title
fn draw_title(
    layer: &PdfLayerReference,
    title: &str,
    page_width: f32,
    page_height: f32,
    options: &PdfExportOptions,
    font: &IndirectFontRef,
) {
    let text_x = page_width / 2.0 - (title.len() as f32 * options.title_font_size * 0.3);
    let text_y = page_height - options.margin_mm - 10.0;

    layer.use_text(
        title,
        options.title_font_size,
        Mm(text_x),
        Mm(text_y),
        font,
    );
}

/// Draw text at position
fn draw_text(
    layer: &PdfLayerReference,
    text: &str,
    x: f32,
    y: f32,
    font: &IndirectFontRef,
    font_size: f32,
) {
    layer.use_text(text, font_size, Mm(x), Mm(y), font);
}

/// Draw a measure
fn draw_measure(
    layer: &PdfLayerReference,
    _measure: &TabMeasure,
    beats: &[TabBeat],
    measure_idx: usize,
    x: f32,
    y: f32,
    width: f32,
    string_count: u8,
    options: &PdfExportOptions,
    font: &IndirectFontRef,
) {
    let height = string_count as f32 * options.string_height_mm;

    // Set line color to black
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));

    // Draw string lines
    for i in 0..string_count {
        let string_y = y - (i as f32 * options.string_height_mm);
        draw_line(layer, x, string_y, x + width, string_y, 0.3);
    }

    // Draw bar lines
    draw_line(layer, x, y, x, y - height + options.string_height_mm, 0.5);
    draw_line(layer, x + width, y, x + width, y - height + options.string_height_mm, 0.5);

    // Draw measure number
    if options.show_measure_numbers {
        layer.use_text(
            &format!("{}", measure_idx + 1),
            8.0,
            Mm(x),
            Mm(y + 3.0),
            font,
        );
    }

    // Draw fret numbers
    let beat_width = if beats.is_empty() {
        width / 4.0
    } else {
        width / beats.len() as f32
    };

    for (beat_idx, beat) in beats.iter().enumerate() {
        if beat.is_rest {
            continue;
        }

        let beat_x = x + beat_idx as f32 * beat_width + beat_width / 2.0;

        for note in &beat.notes {
            let note_y = y - (note.string as f32 - 1.0) * options.string_height_mm;
            let fret_text = format!("{}", note.fret);

            // Draw white background for better readability
            layer.use_text(
                &fret_text,
                options.tab_font_size,
                Mm(beat_x - 1.5),
                Mm(note_y - 1.5),
                font,
            );

            // Draw technique annotation
            if options.show_techniques && !note.techniques.is_empty() {
                let tech_text = technique_abbreviation(&note.techniques);
                if !tech_text.is_empty() {
                    layer.use_text(
                        &tech_text,
                        6.0,
                        Mm(beat_x + 2.0),
                        Mm(note_y + 1.0),
                        font,
                    );
                }
            }
        }
    }
}

/// Draw a line
fn draw_line(layer: &PdfLayerReference, x1: f32, y1: f32, x2: f32, y2: f32, width: f32) {
    layer.set_outline_thickness(width);

    let points = vec![
        (Point::new(Mm(x1), Mm(y1)), false),
        (Point::new(Mm(x2), Mm(y2)), false),
    ];

    layer.add_line(printpdf::Line {
        points,
        is_closed: false,
    });
}

/// Get technique abbreviation for display
fn technique_abbreviation(techniques: &[Technique]) -> String {
    let abbrevs: Vec<&str> = techniques
        .iter()
        .filter_map(|t| match t {
            Technique::HammerOn => Some("h"),
            Technique::PullOff => Some("p"),
            Technique::LegatoSlide(_) => Some("/"),
            Technique::ShiftSlide(_) => Some("s"),
            Technique::Bend(_) => Some("b"),
            Technique::Vibrato(_) => Some("~"),
            Technique::NaturalHarmonic => Some("NH"),
            Technique::PinchHarmonic => Some("PH"),
            Technique::Tap(_) => Some("T"),
            Technique::PalmMute(_) => Some("PM"),
            Technique::LetRing => Some("lr"),
            _ => None,
        })
        .collect();

    abbrevs.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size_dimensions() {
        let letter = PageSize::Letter.dimensions_mm();
        assert!((letter.0 - 215.9).abs() < 0.1);
        assert!((letter.1 - 279.4).abs() < 0.1);

        let a4 = PageSize::A4.dimensions_mm();
        assert!((a4.0 - 210.0).abs() < 0.1);
        assert!((a4.1 - 297.0).abs() < 0.1);
    }

    #[test]
    fn test_default_options() {
        let options = PdfExportOptions::default();
        assert_eq!(options.margin_mm, 20.0);
        assert!(options.show_tempo);
        assert!(options.show_time_signature);
    }
}
