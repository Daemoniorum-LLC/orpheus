//! MIDI Input Panel
//!
//! Allows selecting and connecting to MIDI input devices,
//! and shows real-time MIDI activity.

use egui::{Color32, RichText, Rounding, Stroke, Ui, Vec2};
use std::time::Instant;

/// Information about a MIDI input device
#[derive(Debug, Clone)]
pub struct MidiDevice {
    /// Device index
    pub index: usize,
    /// Device name
    pub name: String,
}

/// Recent MIDI activity entry
#[derive(Debug, Clone)]
pub struct MidiActivityEntry {
    /// Message description
    pub description: String,
    /// Timestamp when received
    pub time: Instant,
    /// Message type color
    pub color: Color32,
}

/// State for the MIDI input panel
#[derive(Debug, Clone)]
pub struct MidiInputState {
    /// Available devices
    pub available_devices: Vec<MidiDevice>,
    /// Currently connected device index
    pub connected_device: Option<usize>,
    /// Connection status message
    pub status_message: String,
    /// Is currently scanning for devices
    pub scanning: bool,
    /// Recent MIDI activity (circular buffer)
    pub activity: Vec<MidiActivityEntry>,
    /// Max activity entries to keep
    pub max_activity: usize,
    /// Show activity monitor
    pub show_activity: bool,
    /// MIDI learn mode (waiting for input to map)
    pub learn_mode: bool,
    /// Last received note (for visual feedback)
    pub last_note: Option<(u8, u8)>, // (note, velocity)
    /// Channel filter (None = all channels)
    pub channel_filter: Option<u8>,
}

impl MidiInputState {
    pub fn new() -> Self {
        Self {
            available_devices: Vec::new(),
            connected_device: None,
            status_message: "Not connected".to_string(),
            scanning: false,
            activity: Vec::new(),
            max_activity: 50,
            show_activity: true,
            learn_mode: false,
            last_note: None,
            channel_filter: None,
        }
    }

    /// Add an activity entry
    pub fn add_activity(&mut self, description: String, color: Color32) {
        self.activity.push(MidiActivityEntry {
            description,
            time: Instant::now(),
            color,
        });

        // Trim old entries
        if self.activity.len() > self.max_activity {
            self.activity.remove(0);
        }
    }

    /// Clear old activity entries (older than 5 seconds)
    pub fn cleanup_activity(&mut self) {
        let now = Instant::now();
        self.activity
            .retain(|entry| now.duration_since(entry.time).as_secs() < 5);
    }

    /// Set connected device
    pub fn set_connected(&mut self, index: usize, name: &str) {
        self.connected_device = Some(index);
        self.status_message = format!("Connected: {}", name);
    }

    /// Set disconnected
    pub fn set_disconnected(&mut self) {
        self.connected_device = None;
        self.status_message = "Not connected".to_string();
    }

    /// Set error message
    pub fn set_error(&mut self, message: String) {
        self.status_message = format!("Error: {}", message);
    }
}

impl Default for MidiInputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be triggered by the MIDI input panel
#[derive(Debug, Clone)]
pub enum MidiInputAction {
    /// Scan for available devices
    ScanDevices,
    /// Connect to a device by index
    Connect(usize),
    /// Disconnect from current device
    Disconnect,
    /// Toggle MIDI learn mode
    ToggleLearnMode,
    /// Set channel filter
    SetChannelFilter(Option<u8>),
    /// Clear activity log
    ClearActivity,
}

/// MIDI input panel
pub struct MidiInputPanel<'a> {
    state: &'a mut MidiInputState,
}

impl<'a> MidiInputPanel<'a> {
    pub fn new(state: &'a mut MidiInputState) -> Self {
        Self { state }
    }

    /// Show the panel and return any action
    pub fn show(&mut self, ui: &mut Ui) -> Option<MidiInputAction> {
        let mut action = None;

        // Cleanup old activity
        self.state.cleanup_activity();

        // Header
        ui.horizontal(|ui| {
            ui.heading("🎹 MIDI Input");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Learn mode toggle
                let learn_text = if self.state.learn_mode { "🔴 Learning" } else { "Learn" };
                if ui.selectable_label(self.state.learn_mode, learn_text).clicked() {
                    action = Some(MidiInputAction::ToggleLearnMode);
                }
            });
        });

        ui.separator();

        // Connection status
        ui.horizontal(|ui| {
            let status_color = if self.state.connected_device.is_some() {
                Color32::from_rgb(100, 200, 100)
            } else {
                Color32::from_gray(150)
            };
            let status_icon = if self.state.connected_device.is_some() { "●" } else { "○" };
            ui.label(RichText::new(status_icon).color(status_color));
            ui.label(&self.state.status_message);
        });

        ui.add_space(4.0);

        // Scan button
        ui.horizontal(|ui| {
            if ui.button(if self.state.scanning { "⟳ Scanning..." } else { "🔍 Scan Devices" }).clicked() {
                if !self.state.scanning {
                    action = Some(MidiInputAction::ScanDevices);
                }
            }

            if self.state.connected_device.is_some() {
                if ui.button("⏹ Disconnect").clicked() {
                    action = Some(MidiInputAction::Disconnect);
                }
            }
        });

        ui.add_space(8.0);

        // Device list
        if !self.state.available_devices.is_empty() {
            ui.label(RichText::new("Available Devices:").strong());

            egui::ScrollArea::vertical()
                .max_height(120.0)
                .show(ui, |ui| {
                    for device in &self.state.available_devices {
                        let is_connected = self.state.connected_device == Some(device.index);

                        let response = ui.allocate_ui(Vec2::new(ui.available_width(), 32.0), |ui| {
                            let rect = ui.available_rect_before_wrap();

                            // Background
                            let bg_color = if is_connected {
                                Color32::from_rgb(50, 80, 50)
                            } else if ui.rect_contains_pointer(rect) {
                                Color32::from_gray(50)
                            } else {
                                Color32::from_gray(35)
                            };

                            ui.painter().rect_filled(rect, Rounding::same(4.0), bg_color);

                            if is_connected {
                                ui.painter().rect_stroke(
                                    rect,
                                    Rounding::same(4.0),
                                    Stroke::new(1.0, Color32::from_rgb(100, 200, 100)),
                                );
                            }

                            ui.horizontal(|ui| {
                                ui.add_space(8.0);
                                let icon = if is_connected { "✓" } else { "🎹" };
                                ui.label(icon);
                                ui.label(&device.name);
                            });

                            ui.interact(rect, ui.id().with(device.index), egui::Sense::click())
                        });

                        if response.inner.clicked() && !is_connected {
                            action = Some(MidiInputAction::Connect(device.index));
                        }
                    }
                });
        } else if !self.state.scanning {
            ui.label(RichText::new("No devices found. Click 'Scan Devices' to search.").weak());
        }

        ui.add_space(8.0);
        ui.separator();

        // Channel filter
        ui.horizontal(|ui| {
            ui.label("Channel:");
            let current = self.state.channel_filter.map(|c| c.to_string()).unwrap_or_else(|| "All".to_string());

            egui::ComboBox::from_id_salt("midi_channel_filter")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.state.channel_filter.is_none(), "All").clicked() {
                        action = Some(MidiInputAction::SetChannelFilter(None));
                    }
                    for ch in 1..=16u8 {
                        if ui.selectable_label(self.state.channel_filter == Some(ch), ch.to_string()).clicked() {
                            action = Some(MidiInputAction::SetChannelFilter(Some(ch)));
                        }
                    }
                }).response.on_hover_text("Filter incoming MIDI to specific channel (1-16)");
        });

        ui.add_space(4.0);

        // Last note display
        if let Some((note, velocity)) = self.state.last_note {
            let note_name = note_to_name(note);
            ui.horizontal(|ui| {
                ui.label("Last note:");
                ui.label(RichText::new(format!("{} (vel: {})", note_name, velocity))
                    .color(Color32::from_rgb(100, 200, 255))
                    .strong());
            });
        }

        ui.add_space(8.0);

        // Activity monitor
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.state.show_activity, "Show Activity")
                .on_hover_text("Display incoming MIDI messages in real-time");
            if self.state.show_activity {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Clear")
                        .on_hover_text("Clear activity log")
                        .clicked()
                    {
                        action = Some(MidiInputAction::ClearActivity);
                    }
                });
            }
        });

        if self.state.show_activity {
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &self.state.activity {
                        let age = Instant::now().duration_since(entry.time);
                        let alpha = ((5.0 - age.as_secs_f32()) / 5.0).clamp(0.3, 1.0);

                        let color = Color32::from_rgba_unmultiplied(
                            entry.color.r(),
                            entry.color.g(),
                            entry.color.b(),
                            (alpha * 255.0) as u8,
                        );

                        ui.label(RichText::new(&entry.description).small().color(color));
                    }

                    if self.state.activity.is_empty() {
                        ui.label(RichText::new("No recent activity").weak().small());
                    }
                });
        }

        action
    }
}

/// Convert MIDI note number to note name
fn note_to_name(note: u8) -> String {
    let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i8 - 1;
    let name = names[(note % 12) as usize];
    format!("{}{}", name, octave)
}

/// Get color for MIDI message type
pub fn midi_message_color(msg_type: &str) -> Color32 {
    match msg_type {
        "NoteOn" => Color32::from_rgb(100, 200, 100),
        "NoteOff" => Color32::from_rgb(200, 100, 100),
        "CC" => Color32::from_rgb(100, 150, 255),
        "PitchBend" => Color32::from_rgb(255, 200, 100),
        "Aftertouch" => Color32::from_rgb(200, 150, 255),
        _ => Color32::from_gray(180),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = MidiInputState::new();
        assert!(state.connected_device.is_none());
        assert!(state.available_devices.is_empty());
        assert!(!state.learn_mode);
    }

    #[test]
    fn test_add_activity() {
        let mut state = MidiInputState::new();
        state.add_activity("Note On C4".to_string(), Color32::GREEN);
        assert_eq!(state.activity.len(), 1);
    }

    #[test]
    fn test_activity_limit() {
        let mut state = MidiInputState::new();
        state.max_activity = 5;

        for i in 0..10 {
            state.add_activity(format!("Note {}", i), Color32::WHITE);
        }

        assert_eq!(state.activity.len(), 5);
    }

    #[test]
    fn test_note_to_name() {
        assert_eq!(note_to_name(60), "C4");
        assert_eq!(note_to_name(69), "A4");
        assert_eq!(note_to_name(0), "C-1");
        assert_eq!(note_to_name(127), "G9");
    }

    #[test]
    fn test_set_connected() {
        let mut state = MidiInputState::new();
        state.set_connected(0, "USB MIDI Keyboard");
        assert_eq!(state.connected_device, Some(0));
        assert!(state.status_message.contains("USB MIDI Keyboard"));
    }

    #[test]
    fn test_set_disconnected() {
        let mut state = MidiInputState::new();
        state.set_connected(0, "Test");
        state.set_disconnected();
        assert!(state.connected_device.is_none());
    }
}
