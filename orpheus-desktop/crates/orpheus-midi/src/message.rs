//! MIDI message types and parsing
//!
//! Provides strongly-typed MIDI message handling.

use serde::{Deserialize, Serialize};

/// MIDI channel (0-15)
pub type Channel = u8;

/// MIDI note number (0-127)
pub type Note = u8;

/// MIDI velocity (0-127)
pub type Velocity = u8;

/// MIDI control change number (0-127)
pub type ControlNumber = u8;

/// MIDI control value (0-127)
pub type ControlValue = u8;

/// MIDI program number (0-127)
pub type Program = u8;

/// Pitch bend value (-8192 to 8191)
pub type PitchBend = i16;

/// MIDI message types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiMessage {
    /// Note On event
    NoteOn {
        channel: Channel,
        note: Note,
        velocity: Velocity,
    },
    /// Note Off event
    NoteOff {
        channel: Channel,
        note: Note,
        velocity: Velocity,
    },
    /// Polyphonic aftertouch
    PolyAftertouch {
        channel: Channel,
        note: Note,
        pressure: u8,
    },
    /// Control Change
    ControlChange {
        channel: Channel,
        control: ControlNumber,
        value: ControlValue,
    },
    /// Program Change
    ProgramChange {
        channel: Channel,
        program: Program,
    },
    /// Channel Aftertouch
    ChannelAftertouch {
        channel: Channel,
        pressure: u8,
    },
    /// Pitch Bend
    PitchBend {
        channel: Channel,
        value: PitchBend,
    },
    /// System exclusive (variable length)
    SysEx(Vec<u8>),
    /// Timing clock
    TimingClock,
    /// Start
    Start,
    /// Continue
    Continue,
    /// Stop
    Stop,
    /// Active sensing
    ActiveSensing,
    /// System reset
    SystemReset,
}

impl MidiMessage {
    /// Parse a MIDI message from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        let status = bytes[0];
        let msg_type = status & 0xF0;
        let channel = status & 0x0F;

        match msg_type {
            0x80 if bytes.len() >= 3 => Some(MidiMessage::NoteOff {
                channel,
                note: bytes[1] & 0x7F,
                velocity: bytes[2] & 0x7F,
            }),
            0x90 if bytes.len() >= 3 => {
                let velocity = bytes[2] & 0x7F;
                // Note On with velocity 0 is treated as Note Off
                if velocity == 0 {
                    Some(MidiMessage::NoteOff {
                        channel,
                        note: bytes[1] & 0x7F,
                        velocity: 0,
                    })
                } else {
                    Some(MidiMessage::NoteOn {
                        channel,
                        note: bytes[1] & 0x7F,
                        velocity,
                    })
                }
            }
            0xA0 if bytes.len() >= 3 => Some(MidiMessage::PolyAftertouch {
                channel,
                note: bytes[1] & 0x7F,
                pressure: bytes[2] & 0x7F,
            }),
            0xB0 if bytes.len() >= 3 => Some(MidiMessage::ControlChange {
                channel,
                control: bytes[1] & 0x7F,
                value: bytes[2] & 0x7F,
            }),
            0xC0 if bytes.len() >= 2 => Some(MidiMessage::ProgramChange {
                channel,
                program: bytes[1] & 0x7F,
            }),
            0xD0 if bytes.len() >= 2 => Some(MidiMessage::ChannelAftertouch {
                channel,
                pressure: bytes[1] & 0x7F,
            }),
            0xE0 if bytes.len() >= 3 => {
                let lsb = bytes[1] as i16 & 0x7F;
                let msb = bytes[2] as i16 & 0x7F;
                let value = ((msb << 7) | lsb) - 8192;
                Some(MidiMessage::PitchBend { channel, value })
            }
            0xF0 => match status {
                0xF0 => {
                    // SysEx - find end
                    let end = bytes.iter().position(|&b| b == 0xF7)?;
                    Some(MidiMessage::SysEx(bytes[1..end].to_vec()))
                }
                0xF8 => Some(MidiMessage::TimingClock),
                0xFA => Some(MidiMessage::Start),
                0xFB => Some(MidiMessage::Continue),
                0xFC => Some(MidiMessage::Stop),
                0xFE => Some(MidiMessage::ActiveSensing),
                0xFF => Some(MidiMessage::SystemReset),
                _ => None,
            },
            _ => None,
        }
    }

    /// Convert message to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            MidiMessage::NoteOff {
                channel,
                note,
                velocity,
            } => vec![0x80 | (channel & 0x0F), note & 0x7F, velocity & 0x7F],
            MidiMessage::NoteOn {
                channel,
                note,
                velocity,
            } => vec![0x90 | (channel & 0x0F), note & 0x7F, velocity & 0x7F],
            MidiMessage::PolyAftertouch {
                channel,
                note,
                pressure,
            } => vec![0xA0 | (channel & 0x0F), note & 0x7F, pressure & 0x7F],
            MidiMessage::ControlChange {
                channel,
                control,
                value,
            } => vec![0xB0 | (channel & 0x0F), control & 0x7F, value & 0x7F],
            MidiMessage::ProgramChange { channel, program } => {
                vec![0xC0 | (channel & 0x0F), program & 0x7F]
            }
            MidiMessage::ChannelAftertouch { channel, pressure } => {
                vec![0xD0 | (channel & 0x0F), pressure & 0x7F]
            }
            MidiMessage::PitchBend { channel, value } => {
                let biased = (value + 8192) as u16;
                let lsb = (biased & 0x7F) as u8;
                let msb = ((biased >> 7) & 0x7F) as u8;
                vec![0xE0 | (channel & 0x0F), lsb, msb]
            }
            MidiMessage::SysEx(data) => {
                let mut bytes = Vec::with_capacity(data.len() + 2);
                bytes.push(0xF0);
                bytes.extend(data);
                bytes.push(0xF7);
                bytes
            }
            MidiMessage::TimingClock => vec![0xF8],
            MidiMessage::Start => vec![0xFA],
            MidiMessage::Continue => vec![0xFB],
            MidiMessage::Stop => vec![0xFC],
            MidiMessage::ActiveSensing => vec![0xFE],
            MidiMessage::SystemReset => vec![0xFF],
        }
    }

    /// Check if this is a channel message (has a channel number)
    pub fn is_channel_message(&self) -> bool {
        matches!(
            self,
            MidiMessage::NoteOn { .. }
                | MidiMessage::NoteOff { .. }
                | MidiMessage::PolyAftertouch { .. }
                | MidiMessage::ControlChange { .. }
                | MidiMessage::ProgramChange { .. }
                | MidiMessage::ChannelAftertouch { .. }
                | MidiMessage::PitchBend { .. }
        )
    }

    /// Get the channel for channel messages
    pub fn channel(&self) -> Option<Channel> {
        match self {
            MidiMessage::NoteOn { channel, .. }
            | MidiMessage::NoteOff { channel, .. }
            | MidiMessage::PolyAftertouch { channel, .. }
            | MidiMessage::ControlChange { channel, .. }
            | MidiMessage::ProgramChange { channel, .. }
            | MidiMessage::ChannelAftertouch { channel, .. }
            | MidiMessage::PitchBend { channel, .. } => Some(*channel),
            _ => None,
        }
    }

    /// Convert MIDI note number to note name
    pub fn note_name(note: Note) -> &'static str {
        const NAMES: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        NAMES[(note % 12) as usize]
    }

    /// Get the octave for a MIDI note
    pub fn note_octave(note: Note) -> i8 {
        (note / 12) as i8 - 1
    }
}

/// Common MIDI Control Change numbers
pub mod cc {
    /// Bank Select MSB
    pub const BANK_SELECT_MSB: u8 = 0;
    /// Modulation Wheel
    pub const MODULATION: u8 = 1;
    /// Breath Controller
    pub const BREATH: u8 = 2;
    /// Foot Controller
    pub const FOOT: u8 = 4;
    /// Portamento Time
    pub const PORTAMENTO_TIME: u8 = 5;
    /// Data Entry MSB
    pub const DATA_ENTRY_MSB: u8 = 6;
    /// Volume
    pub const VOLUME: u8 = 7;
    /// Balance
    pub const BALANCE: u8 = 8;
    /// Pan
    pub const PAN: u8 = 10;
    /// Expression
    pub const EXPRESSION: u8 = 11;
    /// Sustain Pedal
    pub const SUSTAIN: u8 = 64;
    /// Portamento On/Off
    pub const PORTAMENTO: u8 = 65;
    /// Sostenuto Pedal
    pub const SOSTENUTO: u8 = 66;
    /// Soft Pedal
    pub const SOFT: u8 = 67;
    /// All Sound Off
    pub const ALL_SOUND_OFF: u8 = 120;
    /// Reset All Controllers
    pub const RESET_ALL: u8 = 121;
    /// All Notes Off
    pub const ALL_NOTES_OFF: u8 = 123;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_on_parsing() {
        let msg = MidiMessage::from_bytes(&[0x90, 60, 100]);
        assert!(matches!(
            msg,
            Some(MidiMessage::NoteOn {
                channel: 0,
                note: 60,
                velocity: 100
            })
        ));
    }

    #[test]
    fn test_note_on_channel_parsing() {
        let msg = MidiMessage::from_bytes(&[0x93, 64, 80]);
        assert!(matches!(
            msg,
            Some(MidiMessage::NoteOn {
                channel: 3,
                note: 64,
                velocity: 80
            })
        ));
    }

    #[test]
    fn test_note_on_velocity_zero_is_note_off() {
        let msg = MidiMessage::from_bytes(&[0x90, 60, 0]);
        assert!(matches!(
            msg,
            Some(MidiMessage::NoteOff {
                channel: 0,
                note: 60,
                velocity: 0
            })
        ));
    }

    #[test]
    fn test_note_off_parsing() {
        let msg = MidiMessage::from_bytes(&[0x80, 60, 64]);
        assert!(matches!(
            msg,
            Some(MidiMessage::NoteOff {
                channel: 0,
                note: 60,
                velocity: 64
            })
        ));
    }

    #[test]
    fn test_control_change_parsing() {
        let msg = MidiMessage::from_bytes(&[0xB0, 7, 100]);
        assert!(matches!(
            msg,
            Some(MidiMessage::ControlChange {
                channel: 0,
                control: 7,
                value: 100
            })
        ));
    }

    #[test]
    fn test_pitch_bend_center() {
        let msg = MidiMessage::from_bytes(&[0xE0, 0x00, 0x40]);
        assert!(matches!(
            msg,
            Some(MidiMessage::PitchBend {
                channel: 0,
                value: 0
            })
        ));
    }

    #[test]
    fn test_pitch_bend_max() {
        let msg = MidiMessage::from_bytes(&[0xE0, 0x7F, 0x7F]);
        assert!(matches!(
            msg,
            Some(MidiMessage::PitchBend {
                channel: 0,
                value: 8191
            })
        ));
    }

    #[test]
    fn test_pitch_bend_min() {
        let msg = MidiMessage::from_bytes(&[0xE0, 0x00, 0x00]);
        assert!(matches!(
            msg,
            Some(MidiMessage::PitchBend {
                channel: 0,
                value: -8192
            })
        ));
    }

    #[test]
    fn test_program_change() {
        let msg = MidiMessage::from_bytes(&[0xC0, 25]);
        assert!(matches!(
            msg,
            Some(MidiMessage::ProgramChange {
                channel: 0,
                program: 25
            })
        ));
    }

    #[test]
    fn test_timing_clock() {
        let msg = MidiMessage::from_bytes(&[0xF8]);
        assert!(matches!(msg, Some(MidiMessage::TimingClock)));
    }

    #[test]
    fn test_start_stop_continue() {
        assert!(matches!(
            MidiMessage::from_bytes(&[0xFA]),
            Some(MidiMessage::Start)
        ));
        assert!(matches!(
            MidiMessage::from_bytes(&[0xFC]),
            Some(MidiMessage::Stop)
        ));
        assert!(matches!(
            MidiMessage::from_bytes(&[0xFB]),
            Some(MidiMessage::Continue)
        ));
    }

    #[test]
    fn test_roundtrip_note_on() {
        let original = MidiMessage::NoteOn {
            channel: 5,
            note: 72,
            velocity: 110,
        };
        let bytes = original.to_bytes();
        let parsed = MidiMessage::from_bytes(&bytes);
        assert_eq!(parsed, Some(original));
    }

    #[test]
    fn test_roundtrip_pitch_bend() {
        for value in [-8192i16, -4096, 0, 4096, 8191] {
            let original = MidiMessage::PitchBend { channel: 0, value };
            let bytes = original.to_bytes();
            let parsed = MidiMessage::from_bytes(&bytes);
            assert_eq!(parsed, Some(original));
        }
    }

    #[test]
    fn test_channel_extraction() {
        let msg = MidiMessage::NoteOn {
            channel: 9,
            note: 36,
            velocity: 100,
        };
        assert_eq!(msg.channel(), Some(9));

        let msg = MidiMessage::TimingClock;
        assert_eq!(msg.channel(), None);
    }

    #[test]
    fn test_note_name() {
        assert_eq!(MidiMessage::note_name(60), "C");
        assert_eq!(MidiMessage::note_name(61), "C#");
        assert_eq!(MidiMessage::note_name(69), "A");
    }

    #[test]
    fn test_note_octave() {
        assert_eq!(MidiMessage::note_octave(60), 4); // C4
        assert_eq!(MidiMessage::note_octave(69), 4); // A4
        assert_eq!(MidiMessage::note_octave(21), 0); // A0
    }

    #[test]
    fn test_sysex_parsing() {
        let bytes = [0xF0, 0x7E, 0x00, 0x06, 0x01, 0xF7];
        let msg = MidiMessage::from_bytes(&bytes);
        assert!(matches!(msg, Some(MidiMessage::SysEx(data)) if data == vec![0x7E, 0x00, 0x06, 0x01]));
    }

    #[test]
    fn test_empty_bytes_returns_none() {
        assert!(MidiMessage::from_bytes(&[]).is_none());
    }

    #[test]
    fn test_truncated_message_returns_none() {
        assert!(MidiMessage::from_bytes(&[0x90]).is_none());
        assert!(MidiMessage::from_bytes(&[0x90, 60]).is_none());
    }
}
