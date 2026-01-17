//! Virtual keyboard panel for playing synths with computer keyboard
//!
//! Maps QWERTY keyboard to musical notes:
//! - Bottom row (ZXCVBNM): C3-B3 (white keys)
//! - Second row (ASDFGHJKL): C4-B4 (white keys)
//! - Third row (QWERTYUIOP): C5-B5 (white keys)
//! - Numbers hold black keys (sharps/flats)

use egui::{Color32, Key, Rect, Rounding, Sense, Stroke, Ui, Vec2};

/// The selected instrument for the virtual keyboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyboardInstrument {
    #[default]
    Piano,
    Guitar,
    Bass,
    Drums,
}

impl std::fmt::Display for KeyboardInstrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardInstrument::Piano => write!(f, "Piano"),
            KeyboardInstrument::Guitar => write!(f, "Guitar"),
            KeyboardInstrument::Bass => write!(f, "Bass"),
            KeyboardInstrument::Drums => write!(f, "Drums"),
        }
    }
}

/// Actions from the virtual keyboard
#[derive(Debug, Clone)]
pub enum VirtualKeyboardAction {
    /// Play a note (MIDI note number, velocity)
    NoteOn(u8, f32),
    /// Release a note
    NoteOff(u8),
    /// Trigger a drum hit
    DrumHit(DrumType, f32),
    /// Pluck a guitar string (string 0-5, fret, velocity)
    GuitarPluck(u8, u8, f32),
    /// Bass note on
    BassNoteOn(u8, f32),
    /// Bass note off
    BassNoteOff(u8),
}

/// Drum types for the drum machine
#[derive(Debug, Clone, Copy)]
pub enum DrumType {
    Kick,
    Snare,
    HiHatClosed,
    HiHatOpen,
    Crash,
    Ride,
    Tom1,
    Tom2,
    Tom3,
}

/// State for the virtual keyboard
pub struct VirtualKeyboardState {
    /// Currently selected instrument
    pub instrument: KeyboardInstrument,
    /// Current octave offset (0 = middle C at A key)
    pub octave: i8,
    /// Velocity (0.0 - 1.0)
    pub velocity: f32,
    /// Currently held notes (for visual feedback)
    pub held_notes: Vec<u8>,
    /// Whether the keyboard is enabled
    pub enabled: bool,
    /// Sustain pedal state
    pub sustain: bool,
}

impl Default for VirtualKeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualKeyboardState {
    pub fn new() -> Self {
        Self {
            instrument: KeyboardInstrument::Piano,
            octave: 0,
            velocity: 0.8,
            held_notes: Vec::new(),
            enabled: true,
            sustain: false,
        }
    }
}

/// Virtual keyboard panel
pub struct VirtualKeyboardPanel<'a> {
    state: &'a mut VirtualKeyboardState,
}

impl<'a> VirtualKeyboardPanel<'a> {
    pub fn new(state: &'a mut VirtualKeyboardState) -> Self {
        Self { state }
    }

    /// Process keyboard input and return any triggered actions
    pub fn process_input(&mut self, ctx: &egui::Context) -> Vec<VirtualKeyboardAction> {
        if !self.state.enabled {
            return Vec::new();
        }

        let mut actions = Vec::new();

        ctx.input(|i| {
            // Piano/keyboard note mappings
            // Bottom row: Z=C3, X=D3, C=E3, V=F3, B=G3, N=A3, M=B3
            // Middle row: A=C4, S=D4, D=E4, F=F4, G=G4, H=A4, J=B4, K=C5
            // Top row: Q=C5, W=D5, E=E5, R=F5, T=G5, Y=A5, U=B5
            // Black keys on number row: 2=C#, 3=D#, 5=F#, 6=G#, 7=A#

            let base_octave = 4 + self.state.octave;
            let velocity = self.state.velocity;

            match self.state.instrument {
                KeyboardInstrument::Piano | KeyboardInstrument::Bass => {
                    // White keys - middle row (main octave)
                    let white_keys = [
                        (Key::A, 0),  // C
                        (Key::S, 2),  // D
                        (Key::D, 4),  // E
                        (Key::F, 5),  // F
                        (Key::G, 7),  // G
                        (Key::H, 9),  // A
                        (Key::J, 11), // B
                        (Key::K, 12), // C (next octave)
                        (Key::L, 14), // D (next octave)
                    ];

                    // Black keys - top row
                    let black_keys = [
                        (Key::W, 1),  // C#
                        (Key::E, 3),  // D#
                        (Key::T, 6),  // F#
                        (Key::Y, 8),  // G#
                        (Key::U, 10), // A#
                        (Key::O, 13), // C# (next octave)
                    ];

                    // Lower octave - bottom row
                    let lower_keys = [
                        (Key::Z, -12), // C (octave below)
                        (Key::X, -10), // D
                        (Key::C, -8),  // E
                        (Key::V, -7),  // F
                        (Key::B, -5),  // G
                        (Key::N, -3),  // A
                        (Key::M, -1),  // B
                    ];

                    for (key, semitone) in white_keys.iter().chain(black_keys.iter()).chain(lower_keys.iter()) {
                        let note = ((base_octave * 12) + semitone) as u8;

                        if i.key_pressed(*key) {
                            if !self.state.held_notes.contains(&note) {
                                self.state.held_notes.push(note);
                                if self.state.instrument == KeyboardInstrument::Piano {
                                    actions.push(VirtualKeyboardAction::NoteOn(note, velocity));
                                } else {
                                    actions.push(VirtualKeyboardAction::BassNoteOn(note, velocity));
                                }
                            }
                        }
                        if i.key_released(*key) {
                            self.state.held_notes.retain(|&n| n != note);
                            if !self.state.sustain {
                                if self.state.instrument == KeyboardInstrument::Piano {
                                    actions.push(VirtualKeyboardAction::NoteOff(note));
                                } else {
                                    actions.push(VirtualKeyboardAction::BassNoteOff(note));
                                }
                            }
                        }
                    }
                }
                KeyboardInstrument::Drums => {
                    // Drum pad mappings
                    let drum_keys = [
                        (Key::A, DrumType::Kick),
                        (Key::S, DrumType::Snare),
                        (Key::D, DrumType::HiHatClosed),
                        (Key::F, DrumType::HiHatOpen),
                        (Key::G, DrumType::Crash),
                        (Key::H, DrumType::Ride),
                        (Key::Z, DrumType::Tom1),
                        (Key::X, DrumType::Tom2),
                        (Key::C, DrumType::Tom3),
                    ];

                    for (key, drum) in drum_keys.iter() {
                        if i.key_pressed(*key) {
                            actions.push(VirtualKeyboardAction::DrumHit(*drum, velocity));
                        }
                    }
                }
                KeyboardInstrument::Guitar => {
                    // Guitar: number keys = frets, letter keys = strings
                    // Strings: E=0, A=1, D=2, G=3, B=4, e=5
                    let string_keys = [
                        (Key::Z, 0u8), // Low E
                        (Key::X, 1),   // A
                        (Key::C, 2),   // D
                        (Key::V, 3),   // G
                        (Key::B, 4),   // B
                        (Key::N, 5),   // High e
                    ];

                    // Current fret from number keys (0-9)
                    let mut fret = 0u8;
                    for (key, f) in [
                        (Key::Num1, 1), (Key::Num2, 2), (Key::Num3, 3),
                        (Key::Num4, 4), (Key::Num5, 5), (Key::Num6, 6),
                        (Key::Num7, 7), (Key::Num8, 8), (Key::Num9, 9),
                    ] {
                        if i.key_down(key) {
                            fret = f;
                            break;
                        }
                    }

                    for (key, string) in string_keys.iter() {
                        if i.key_pressed(*key) {
                            actions.push(VirtualKeyboardAction::GuitarPluck(*string, fret, velocity));
                        }
                    }
                }
            }

            // Octave shift with [ and ]
            if i.key_pressed(Key::OpenBracket) && self.state.octave > -3 {
                self.state.octave -= 1;
            }
            if i.key_pressed(Key::CloseBracket) && self.state.octave < 3 {
                self.state.octave += 1;
            }

            // Sustain pedal with space (when not in transport mode)
            // Note: This conflicts with play/pause, so we'll use Tab instead
            if i.key_pressed(Key::Tab) {
                self.state.sustain = true;
            }
            if i.key_released(Key::Tab) {
                self.state.sustain = false;
                // Release all sustained notes
                if self.state.instrument == KeyboardInstrument::Piano {
                    for note in self.state.held_notes.drain(..) {
                        actions.push(VirtualKeyboardAction::NoteOff(note));
                    }
                }
            }
        });

        actions
    }

    /// Show the virtual keyboard UI
    pub fn show(&mut self, ui: &mut Ui) -> Vec<VirtualKeyboardAction> {
        let mut actions = Vec::new();

        ui.vertical(|ui| {
            // Header with instrument selector
            ui.horizontal(|ui| {
                ui.heading("Keyboard");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Enable toggle
                    let enable_text = if self.state.enabled { "ON" } else { "OFF" };
                    let enable_color = if self.state.enabled {
                        Color32::from_rgb(46, 204, 113)
                    } else {
                        Color32::from_rgb(231, 76, 60)
                    };
                    if ui.button(egui::RichText::new(enable_text).color(enable_color)).clicked() {
                        self.state.enabled = !self.state.enabled;
                    }
                });
            });

            ui.separator();

            // Instrument selector
            ui.horizontal(|ui| {
                ui.label("Instrument:");
                for inst in [KeyboardInstrument::Piano, KeyboardInstrument::Bass, KeyboardInstrument::Guitar, KeyboardInstrument::Drums] {
                    if ui.selectable_label(self.state.instrument == inst, inst.to_string()).clicked() {
                        self.state.instrument = inst;
                    }
                }
            });

            // Octave and velocity controls
            ui.horizontal(|ui| {
                ui.label("Octave:");
                if ui.small_button("-").on_hover_text("Lower octave (Z)").clicked() && self.state.octave > -3 {
                    self.state.octave -= 1;
                }
                ui.label(format!("{:+}", self.state.octave));
                if ui.small_button("+").on_hover_text("Raise octave (X)").clicked() && self.state.octave < 3 {
                    self.state.octave += 1;
                }

                ui.add_space(16.0);

                ui.label("Velocity:");
                ui.add(egui::Slider::new(&mut self.state.velocity, 0.1..=1.0).show_value(false))
                    .on_hover_text("Note velocity (how hard keys are pressed)");
                ui.label(format!("{:.0}%", self.state.velocity * 100.0));
            });

            ui.add_space(8.0);

            // Visual keyboard
            let available_width = ui.available_width();
            let key_height = 60.0;
            let white_key_width = available_width / 10.0;
            let black_key_width = white_key_width * 0.6;

            let (rect, _response) = ui.allocate_exact_size(Vec2::new(available_width, key_height), Sense::hover());

            let painter = ui.painter_at(rect);

            // Draw white keys
            let white_notes = [0, 2, 4, 5, 7, 9, 11, 12, 14, 16]; // C D E F G A B C D E
            for (i, &semitone) in white_notes.iter().enumerate() {
                let base = (4 + self.state.octave) * 12;
                let note = (base + semitone) as u8;
                let is_pressed = self.state.held_notes.contains(&note);

                let key_rect = Rect::from_min_size(
                    rect.min + Vec2::new(i as f32 * white_key_width, 0.0),
                    Vec2::new(white_key_width - 2.0, key_height),
                );

                let color = if is_pressed {
                    Color32::from_rgb(100, 180, 255)
                } else {
                    Color32::from_rgb(240, 240, 240)
                };

                painter.rect_filled(key_rect, Rounding::same(3.0), color);
                painter.rect_stroke(key_rect, Rounding::same(3.0), Stroke::new(1.0, Color32::DARK_GRAY));

                // Key label
                let labels = ["A", "S", "D", "F", "G", "H", "J", "K", "L", ";"];
                if i < labels.len() {
                    painter.text(
                        key_rect.center_bottom() - Vec2::new(0.0, 8.0),
                        egui::Align2::CENTER_CENTER,
                        labels[i],
                        egui::FontId::proportional(10.0),
                        Color32::DARK_GRAY,
                    );
                }
            }

            // Draw black keys
            let black_positions: [(f32, i8); 5] = [(0.7, 1), (1.7, 3), (3.7, 6), (4.7, 8), (5.7, 10)]; // positions and semitones
            for (pos, semitone) in black_positions {
                let base = (4 + self.state.octave) * 12;
                let note = (base + semitone) as u8;
                let is_pressed = self.state.held_notes.contains(&note);

                let key_rect = Rect::from_min_size(
                    rect.min + Vec2::new(pos as f32 * white_key_width - black_key_width / 2.0, 0.0),
                    Vec2::new(black_key_width, key_height * 0.6),
                );

                let color = if is_pressed {
                    Color32::from_rgb(80, 140, 200)
                } else {
                    Color32::from_rgb(40, 40, 40)
                };

                painter.rect_filled(key_rect, Rounding::same(2.0), color);

                // Key label
                let labels = ["W", "E", "T", "Y", "U"];
                let idx = black_positions.iter().position(|&(p, _)| (p - pos).abs() < 0.1).unwrap_or(0);
                if idx < labels.len() {
                    painter.text(
                        key_rect.center_bottom() - Vec2::new(0.0, 4.0),
                        egui::Align2::CENTER_CENTER,
                        labels[idx],
                        egui::FontId::proportional(9.0),
                        Color32::WHITE,
                    );
                }
            }

            ui.add_space(8.0);

            // Help text
            match self.state.instrument {
                KeyboardInstrument::Piano | KeyboardInstrument::Bass => {
                    ui.label(egui::RichText::new("Keys: ASDFGHJKL = notes, WETYU = sharps, ZXC... = lower octave, [ ] = octave shift").small());
                }
                KeyboardInstrument::Drums => {
                    ui.label(egui::RichText::new("Keys: A=Kick S=Snare D/F=HiHat G=Crash H=Ride ZXC=Toms").small());
                }
                KeyboardInstrument::Guitar => {
                    ui.label(egui::RichText::new("Keys: ZXCVBN = strings (low to high), 1-9 = frets").small());
                }
            }
        });

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_state_default() {
        let state = VirtualKeyboardState::new();
        assert_eq!(state.instrument, KeyboardInstrument::Piano);
        assert_eq!(state.octave, 0);
        assert!(state.enabled);
        assert!(state.held_notes.is_empty());
    }

    #[test]
    fn test_instrument_display() {
        assert_eq!(KeyboardInstrument::Piano.to_string(), "Piano");
        assert_eq!(KeyboardInstrument::Drums.to_string(), "Drums");
    }
}
