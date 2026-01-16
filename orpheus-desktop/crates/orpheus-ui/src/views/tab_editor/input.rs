//! Keyboard input handling for tab editor
//!
//! Optimized for speed - virtuoso-level entry.

use egui::{Key, Modifiers};
use orpheus_core::tab::{
    BaseDuration, BendAmount, SlideDirection, TapType, WhammyTechnique, DiveDepth,
    PalmMuteIntensity,
};

use super::state::{TabEditorState, EditorMode, ActiveTool, HarmonicTool};

/// Result of handling input
#[derive(Debug, Clone, PartialEq)]
pub enum InputResult {
    /// No action taken
    None,
    /// Action taken, need repaint
    Handled,
    /// Action taken, note entered (for preview)
    NoteEntered {
        /// String number (1-indexed)
        string: u8,
        /// Fret number
        fret: u8,
        /// Velocity (0-127)
        velocity: u8,
    },
    /// Cursor moved to position with note (for preview)
    NoteFocused {
        /// String number (1-indexed)
        string: u8,
        /// Fret number
        fret: u8,
        /// Velocity (0-127)
        velocity: u8,
    },
    /// Exit editor requested
    Exit,
    /// Open file requested
    Open,
    /// Save requested
    Save,
    /// Export MIDI requested
    ExportMidi,
    /// Play/stop requested
    TogglePlayback,
    /// Jump to previous section
    PrevSection,
    /// Jump to next section
    NextSection,
    /// Add section marker at current position
    AddSectionMarker,
    /// Quick toggle practice mode on current measure
    QuickPractice,
    /// Switch to record mode with current tempo/time sig
    SwitchToRecordMode,
    /// Import MIDI file
    ImportMidi,
}

impl TabEditorState {
    /// Handle keyboard input
    pub fn handle_key(&mut self, key: Key, modifiers: Modifiers, current_time_ms: u64) -> InputResult {
        match self.mode {
            EditorMode::Insert => self.handle_insert_key(key, modifiers, current_time_ms),
            EditorMode::Normal => self.handle_normal_key(key, modifiers),
            EditorMode::Visual => self.handle_visual_key(key, modifiers),
            EditorMode::Command => self.handle_command_key(key, modifiers),
        }
    }

    /// Handle key in insert mode (note entry)
    fn handle_insert_key(&mut self, key: Key, modifiers: Modifiers, current_time_ms: u64) -> InputResult {
        // Escape -> Normal mode
        if key == Key::Escape {
            self.mode = EditorMode::Normal;
            self.active_tool = ActiveTool::None;
            self.fret_buffer.clear();
            self.status = "NORMAL".to_string();
            return InputResult::Handled;
        }

        // Ctrl shortcuts
        if modifiers.ctrl {
            return self.handle_ctrl_shortcuts(key, modifiers);
        }

        // Number keys -> fret entry
        if let Some(digit) = key_to_digit(key) {
            self.fret_buffer.push(digit, current_time_ms);

            // Check if we should auto-commit
            if self.fret_buffer.should_commit(current_time_ms, self.fret_count()) {
                if let Some(fret) = self.fret_buffer.take() {
                    let string = self.cursor.string;
                    let velocity = self.current_velocity;
                    self.enter_note(fret);
                    return InputResult::NoteEntered { string, fret, velocity };
                }
            }
            return InputResult::Handled;
        }

        // Enter -> commit fret buffer or advance
        if key == Key::Enter {
            if let Some(fret) = self.fret_buffer.take() {
                let string = self.cursor.string;
                let velocity = self.current_velocity;
                self.enter_note(fret);
                return InputResult::NoteEntered { string, fret, velocity };
            } else {
                // Add new beat
                self.insert_beat();
                self.cursor.right(self.beats_in_measure());
            }
            return InputResult::Handled;
        }

        // Navigation - preview note if landing on one
        match key {
            Key::ArrowLeft => {
                self.fret_buffer.clear();
                if modifiers.shift {
                    // Previous measure
                    if self.cursor.measure > 0 {
                        self.cursor.measure -= 1;
                        self.cursor.beat = 0;
                    }
                } else {
                    self.cursor.left();
                }
                return self.navigation_result();
            }
            Key::ArrowRight => {
                self.fret_buffer.clear();
                if modifiers.shift {
                    // Next measure
                    if self.cursor.measure < self.document.measures.len().saturating_sub(1) {
                        self.cursor.measure += 1;
                        self.cursor.beat = 0;
                    }
                } else {
                    self.cursor.right(self.beats_in_measure());
                }
                return self.navigation_result();
            }
            Key::ArrowUp => {
                self.fret_buffer.clear();
                self.cursor.up();
                return self.navigation_result();
            }
            Key::ArrowDown => {
                self.fret_buffer.clear();
                self.cursor.down(self.string_count());
                return self.navigation_result();
            }
            Key::Home => {
                self.cursor.beat = 0;
                return InputResult::Handled;
            }
            Key::End => {
                self.cursor.beat = self.beats_in_measure().saturating_sub(1);
                return InputResult::Handled;
            }
            Key::PageUp => {
                self.cursor.measure = self.cursor.measure.saturating_sub(4);
                self.cursor.beat = 0;
                return InputResult::Handled;
            }
            Key::PageDown => {
                self.cursor.measure = (self.cursor.measure + 4)
                    .min(self.document.measures.len().saturating_sub(1));
                self.cursor.beat = 0;
                return InputResult::Handled;
            }
            _ => {}
        }

        // Delete/Backspace -> delete note
        if key == Key::Delete || key == Key::Backspace {
            self.delete_note();
            if key == Key::Backspace {
                self.cursor.left();
            }
            return InputResult::Handled;
        }

        // Space -> enter rest (not just skip)
        if key == Key::Space {
            self.fret_buffer.clear();
            self.enter_rest();
            return InputResult::Handled;
        }

        // Section navigation works in Insert mode too (Ctrl+[ and Ctrl+])
        if modifiers.ctrl {
            if key == Key::OpenBracket {
                return InputResult::PrevSection;
            }
            if key == Key::CloseBracket {
                return InputResult::NextSection;
            }
        }

        // Duration shortcuts
        match key {
            Key::W => { self.set_duration(BaseDuration::Whole); return InputResult::Handled; }
            Key::H if !modifiers.shift => { self.set_duration(BaseDuration::Half); return InputResult::Handled; }
            Key::Q => { self.set_duration(BaseDuration::Quarter); return InputResult::Handled; }
            Key::E => { self.set_duration(BaseDuration::Eighth); return InputResult::Handled; }
            Key::S if !modifiers.shift => { self.set_duration(BaseDuration::Sixteenth); return InputResult::Handled; }
            Key::T if !modifiers.shift => { self.set_duration(BaseDuration::ThirtySecond); return InputResult::Handled; }
            _ => {}
        }

        // Modifier keys (hold to apply technique)
        if key == Key::Period {
            self.toggle_dotted();
            return InputResult::Handled;
        }

        // Technique toggle shortcuts (Shift+key)
        if modifiers.shift {
            match key {
                Key::H => {
                    // Hammer-on
                    self.active_tool = if matches!(self.active_tool, ActiveTool::HammerOn) {
                        ActiveTool::None
                    } else {
                        ActiveTool::HammerOn
                    };
                    self.status = if matches!(self.active_tool, ActiveTool::HammerOn) {
                        "Hammer-On ON".to_string()
                    } else {
                        "Hammer-On OFF".to_string()
                    };
                    return InputResult::Handled;
                }
                Key::P => {
                    // Pull-off
                    self.active_tool = if matches!(self.active_tool, ActiveTool::PullOff) {
                        ActiveTool::None
                    } else {
                        ActiveTool::PullOff
                    };
                    self.status = if matches!(self.active_tool, ActiveTool::PullOff) {
                        "Pull-Off ON".to_string()
                    } else {
                        "Pull-Off OFF".to_string()
                    };
                    return InputResult::Handled;
                }
                Key::S => {
                    // Slide up
                    self.active_tool = if matches!(self.active_tool, ActiveTool::Slide(_)) {
                        ActiveTool::None
                    } else {
                        ActiveTool::Slide(SlideDirection::Up)
                    };
                    self.status = format!("Slide: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::B => {
                    // Bend (cycle through amounts)
                    self.active_tool = match self.active_tool {
                        ActiveTool::Bend(BendAmount::Half) => ActiveTool::Bend(BendAmount::Full),
                        ActiveTool::Bend(BendAmount::Full) => ActiveTool::Bend(BendAmount::OneAndHalf),
                        ActiveTool::Bend(BendAmount::OneAndHalf) => ActiveTool::Bend(BendAmount::Two),
                        ActiveTool::Bend(BendAmount::Two) => ActiveTool::None,
                        _ => ActiveTool::Bend(BendAmount::Half),
                    };
                    self.status = format!("Bend: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::T => {
                    // Tap
                    self.active_tool = if matches!(self.active_tool, ActiveTool::Tap(_)) {
                        ActiveTool::None
                    } else {
                        ActiveTool::Tap(TapType::RightHand)
                    };
                    self.status = format!("Tap: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::N => {
                    // Natural harmonic
                    self.active_tool = if matches!(self.active_tool, ActiveTool::Harmonic(HarmonicTool::Natural)) {
                        ActiveTool::None
                    } else {
                        ActiveTool::Harmonic(HarmonicTool::Natural)
                    };
                    self.status = format!("Harmonic: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::I => {
                    // Pinch harmonic
                    self.active_tool = if matches!(self.active_tool, ActiveTool::Harmonic(HarmonicTool::Pinch)) {
                        ActiveTool::None
                    } else {
                        ActiveTool::Harmonic(HarmonicTool::Pinch)
                    };
                    self.status = format!("Pinch: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::M => {
                    // Palm mute (cycle intensity)
                    self.active_tool = match self.active_tool {
                        ActiveTool::PalmMute(PalmMuteIntensity::Light) => {
                            ActiveTool::PalmMute(PalmMuteIntensity::Medium)
                        }
                        ActiveTool::PalmMute(PalmMuteIntensity::Medium) => {
                            ActiveTool::PalmMute(PalmMuteIntensity::Heavy)
                        }
                        ActiveTool::PalmMute(PalmMuteIntensity::Heavy) => {
                            ActiveTool::PalmMute(PalmMuteIntensity::Chug)
                        }
                        ActiveTool::PalmMute(PalmMuteIntensity::Chug) => {
                            ActiveTool::None
                        }
                        _ => ActiveTool::PalmMute(PalmMuteIntensity::Light),
                    };
                    self.status = format!("Palm Mute: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::V => {
                    // Vibrato
                    self.active_tool = if matches!(self.active_tool, ActiveTool::Vibrato) {
                        ActiveTool::None
                    } else {
                        ActiveTool::Vibrato
                    };
                    self.status = format!("Vibrato: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::W => {
                    // Whammy dive bomb
                    self.active_tool = if matches!(self.active_tool, ActiveTool::Whammy(_)) {
                        ActiveTool::None
                    } else {
                        ActiveTool::Whammy(WhammyTechnique::DiveBomb(DiveDepth::Slack))
                    };
                    self.status = format!("Whammy: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::L => {
                    // Let ring
                    self.active_tool = if matches!(self.active_tool, ActiveTool::LetRing) {
                        ActiveTool::None
                    } else {
                        ActiveTool::LetRing
                    };
                    self.status = format!("Let Ring: {:?}", self.active_tool);
                    return InputResult::Handled;
                }
                Key::X => {
                    // Dead note (muted) - enter dead note at current position
                    self.enter_dead_note();
                    self.status = "Dead note entered".to_string();
                    return InputResult::Handled;
                }
                Key::G => {
                    // Ghost note - toggle ghost on current note
                    if self.toggle_ghost_note() {
                        self.status = "Ghost note toggled".to_string();
                    } else {
                        self.status = "No note at cursor".to_string();
                    }
                    return InputResult::Handled;
                }
                _ => {}
            }
        }

        // Triplet toggle
        if key == Key::Num3 && modifiers.alt {
            self.toggle_triplet();
            return InputResult::Handled;
        }

        // F1 or ? for technique help
        if key == Key::F1 || (key == Key::Slash && modifiers.shift) {
            self.show_technique_help = !self.show_technique_help;
            return InputResult::Handled;
        }

        InputResult::None
    }

    /// Handle key in normal mode (vim-like navigation)
    fn handle_normal_key(&mut self, key: Key, modifiers: Modifiers) -> InputResult {
        // i -> Insert mode
        if key == Key::I {
            self.mode = EditorMode::Insert;
            self.status = "INSERT".to_string();
            return InputResult::Handled;
        }

        // v -> Visual mode
        if key == Key::V {
            self.mode = EditorMode::Visual;
            self.selection = Some(super::state::TabSelection::new(self.cursor));
            self.status = "VISUAL".to_string();
            return InputResult::Handled;
        }

        // : -> Command mode
        if key == Key::Semicolon && modifiers.shift {
            self.mode = EditorMode::Command;
            self.status = ":".to_string();
            return InputResult::Handled;
        }

        // Vim-style navigation
        match key {
            Key::H | Key::ArrowLeft => {
                self.cursor.left();
                return InputResult::Handled;
            }
            Key::L | Key::ArrowRight => {
                self.cursor.right(self.beats_in_measure());
                return InputResult::Handled;
            }
            Key::K | Key::ArrowUp => {
                self.cursor.up();
                return InputResult::Handled;
            }
            Key::J | Key::ArrowDown => {
                self.cursor.down(self.string_count());
                return InputResult::Handled;
            }
            Key::Num0 | Key::Home => {
                self.cursor.beat = 0;
                return InputResult::Handled;
            }
            Key::End => {
                self.cursor.beat = self.beats_in_measure().saturating_sub(1);
                return InputResult::Handled;
            }
            _ => {}
        }

        // x -> delete note
        if key == Key::X {
            self.delete_note();
            return InputResult::Handled;
        }

        // dd -> delete beat
        if key == Key::D {
            // Would need double-key detection
            self.delete_beat();
            return InputResult::Handled;
        }

        // u -> undo
        if key == Key::U {
            self.undo();
            return InputResult::Handled;
        }

        // Ctrl+R -> redo
        if key == Key::R && modifiers.ctrl {
            self.redo();
            return InputResult::Handled;
        }

        // Ctrl shortcuts
        if modifiers.ctrl {
            return self.handle_ctrl_shortcuts(key, modifiers);
        }

        // Space -> toggle playback (in normal mode)
        if key == Key::Space {
            return InputResult::TogglePlayback;
        }

        // Section navigation: [ and ]
        if key == Key::OpenBracket {
            return InputResult::PrevSection;
        }
        if key == Key::CloseBracket {
            return InputResult::NextSection;
        }

        InputResult::None
    }

    /// Handle key in visual mode
    fn handle_visual_key(&mut self, key: Key, modifiers: Modifiers) -> InputResult {
        // Escape -> back to normal
        if key == Key::Escape {
            self.mode = EditorMode::Normal;
            self.selection = None;
            self.status = "NORMAL".to_string();
            return InputResult::Handled;
        }

        // Navigation extends selection
        match key {
            Key::H | Key::ArrowLeft => {
                self.cursor.left();
                if let Some(ref mut sel) = self.selection {
                    sel.end = self.cursor;
                }
                return InputResult::Handled;
            }
            Key::L | Key::ArrowRight => {
                self.cursor.right(self.beats_in_measure());
                if let Some(ref mut sel) = self.selection {
                    sel.end = self.cursor;
                }
                return InputResult::Handled;
            }
            Key::K | Key::ArrowUp => {
                self.cursor.up();
                if let Some(ref mut sel) = self.selection {
                    sel.end = self.cursor;
                }
                return InputResult::Handled;
            }
            Key::J | Key::ArrowDown => {
                self.cursor.down(self.string_count());
                if let Some(ref mut sel) = self.selection {
                    sel.end = self.cursor;
                }
                return InputResult::Handled;
            }
            _ => {}
        }

        // y -> copy
        if key == Key::Y {
            self.copy();
            self.mode = EditorMode::Normal;
            self.selection = None;
            return InputResult::Handled;
        }

        // d -> cut
        if key == Key::D {
            self.cut();
            self.mode = EditorMode::Normal;
            self.selection = None;
            return InputResult::Handled;
        }

        InputResult::None
    }

    /// Handle key in command mode
    fn handle_command_key(&mut self, key: Key, _modifiers: Modifiers) -> InputResult {
        // Escape -> back to normal
        if key == Key::Escape {
            self.mode = EditorMode::Normal;
            self.status = "NORMAL".to_string();
            return InputResult::Handled;
        }

        // For now, just exit command mode on any key
        self.mode = EditorMode::Normal;
        InputResult::None
    }

    /// Handle Ctrl+ shortcuts
    fn handle_ctrl_shortcuts(&mut self, key: Key, modifiers: Modifiers) -> InputResult {
        match key {
            Key::O => InputResult::Open,
            Key::S => InputResult::Save,
            Key::E => InputResult::ExportMidi,
            Key::Z => { self.undo(); InputResult::Handled }
            Key::Y => { self.redo(); InputResult::Handled }
            Key::C => { self.copy(); InputResult::Handled }
            Key::X => { self.cut(); InputResult::Handled }
            Key::V => { self.paste(); InputResult::Handled }
            Key::A => {
                // Select all in measure
                self.select_all();
                InputResult::Handled
            }
            Key::M => {
                // Add measure
                self.add_measure();
                InputResult::Handled
            }
            Key::T => {
                // Toggle multi-track view
                self.toggle_multi_track_view();
                InputResult::Handled
            }
            Key::Tab => {
                // Ctrl+Shift+Tab -> Previous track
                // Ctrl+Tab -> Next track
                if modifiers.shift {
                    self.prev_track();
                } else {
                    self.next_track();
                }
                InputResult::Handled
            }
            Key::B => {
                // Ctrl+B -> Add section marker at cursor
                InputResult::AddSectionMarker
            }
            Key::P => {
                // Ctrl+P -> Quick practice mode toggle on current measure
                InputResult::QuickPractice
            }
            Key::R => {
                // Ctrl+R -> Switch to record mode with current settings
                InputResult::SwitchToRecordMode
            }
            Key::I => {
                // Ctrl+I -> Import MIDI file
                InputResult::ImportMidi
            }
            _ => InputResult::None,
        }
    }
}

/// Convert Key to digit character
fn key_to_digit(key: Key) -> Option<char> {
    match key {
        Key::Num0 => Some('0'),
        Key::Num1 => Some('1'),
        Key::Num2 => Some('2'),
        Key::Num3 => Some('3'),
        Key::Num4 => Some('4'),
        Key::Num5 => Some('5'),
        Key::Num6 => Some('6'),
        Key::Num7 => Some('7'),
        Key::Num8 => Some('8'),
        Key::Num9 => Some('9'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_digit() {
        assert_eq!(key_to_digit(Key::Num0), Some('0'));
        assert_eq!(key_to_digit(Key::Num9), Some('9'));
        assert_eq!(key_to_digit(Key::A), None);
    }
}
