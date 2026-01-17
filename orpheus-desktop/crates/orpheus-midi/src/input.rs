//! MIDI input handling
//!
//! Provides device enumeration and real-time MIDI input via midir.

use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::Arc;
#[allow(unused_imports)]
use parking_lot::Mutex;
#[cfg(feature = "midi-io")]
use tracing::{info, warn};

use crate::message::MidiMessage;
use crate::Error;

/// Information about a MIDI input device
#[derive(Debug, Clone)]
pub struct MidiInputDevice {
    /// Device index
    pub index: usize,
    /// Device name
    pub name: String,
}

/// MIDI input port handle
pub struct MidiInputPort {
    /// Port name
    name: String,
    /// Receiver for incoming MIDI messages
    receiver: Receiver<TimestampedMidiMessage>,
    /// Connection handle (kept alive)
    #[cfg(feature = "midi-io")]
    _connection: midir::MidiInputConnection<()>,
    /// Flag for mock ports
    #[cfg(not(feature = "midi-io"))]
    _mock: bool,
}

/// A MIDI message with timestamp
#[derive(Debug, Clone)]
pub struct TimestampedMidiMessage {
    /// The MIDI message
    pub message: MidiMessage,
    /// Timestamp in microseconds from connection start
    pub timestamp: u64,
}

/// List available MIDI input devices
#[cfg(feature = "midi-io")]
pub fn list_input_devices() -> Result<Vec<MidiInputDevice>, Error> {
    let midi_in = midir::MidiInput::new("Orpheus MIDI Scanner")
        .map_err(|e| Error::ConnectionError(e.to_string()))?;

    let ports = midi_in.ports();
    let devices: Vec<MidiInputDevice> = ports
        .iter()
        .enumerate()
        .filter_map(|(index, port)| {
            midi_in.port_name(port).ok().map(|name| MidiInputDevice { index, name })
        })
        .collect();

    Ok(devices)
}

/// List available MIDI input devices (mock version)
#[cfg(not(feature = "midi-io"))]
pub fn list_input_devices() -> Result<Vec<MidiInputDevice>, Error> {
    // Return empty list in mock mode
    Ok(Vec::new())
}

/// Open a MIDI input port by device index
#[cfg(feature = "midi-io")]
pub fn open_input(device_index: usize) -> Result<MidiInputPort, Error> {
    let midi_in = midir::MidiInput::new("Orpheus MIDI Input")
        .map_err(|e| Error::ConnectionError(e.to_string()))?;

    let ports = midi_in.ports();
    let port = ports
        .get(device_index)
        .ok_or_else(|| Error::DeviceNotFound(format!("Device index {} not found", device_index)))?;

    let port_name = midi_in
        .port_name(port)
        .unwrap_or_else(|_| "Unknown".to_string());

    let (tx, rx) = bounded::<TimestampedMidiMessage>(1024);

    let connection = midi_in
        .connect(
            port,
            "orpheus-midi-in",
            move |timestamp, data, _| {
                if let Some(message) = MidiMessage::from_bytes(data) {
                    let msg = TimestampedMidiMessage { message, timestamp };
                    if tx.send(msg).is_err() {
                        warn!("MIDI message queue full, dropping message");
                    }
                }
            },
            (),
        )
        .map_err(|e| Error::ConnectionError(e.to_string()))?;

    info!("Opened MIDI input: {}", port_name);

    Ok(MidiInputPort {
        name: port_name,
        receiver: rx,
        _connection: connection,
    })
}

/// Open a MIDI input port (mock version)
#[cfg(not(feature = "midi-io"))]
pub fn open_input(_device_index: usize) -> Result<MidiInputPort, Error> {
    Err(Error::ConnectionError(
        "MIDI I/O not available (compile with midi-io feature)".to_string(),
    ))
}

/// Open a MIDI input port by name
#[cfg(feature = "midi-io")]
pub fn open_input_by_name(name: &str) -> Result<MidiInputPort, Error> {
    let devices = list_input_devices()?;
    let device = devices
        .iter()
        .find(|d| d.name.contains(name))
        .ok_or_else(|| Error::DeviceNotFound(format!("No device matching '{}' found", name)))?;
    open_input(device.index)
}

/// Open a MIDI input port by name (mock version)
#[cfg(not(feature = "midi-io"))]
pub fn open_input_by_name(_name: &str) -> Result<MidiInputPort, Error> {
    Err(Error::ConnectionError(
        "MIDI I/O not available (compile with midi-io feature)".to_string(),
    ))
}

impl MidiInputPort {
    /// Get the port name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Try to receive a MIDI message (non-blocking)
    pub fn try_recv(&self) -> Option<TimestampedMidiMessage> {
        self.receiver.try_recv().ok()
    }

    /// Receive a MIDI message (blocking)
    pub fn recv(&self) -> Option<TimestampedMidiMessage> {
        self.receiver.recv().ok()
    }

    /// Get the receiver for use with crossbeam select
    pub fn receiver(&self) -> &Receiver<TimestampedMidiMessage> {
        &self.receiver
    }

    /// Drain all pending messages
    pub fn drain(&self) -> Vec<TimestampedMidiMessage> {
        let mut messages = Vec::new();
        while let Some(msg) = self.try_recv() {
            messages.push(msg);
        }
        messages
    }
}

/// MIDI input manager that handles multiple devices
pub struct MidiInputManager {
    /// Active input ports
    ports: Arc<Mutex<Vec<MidiInputPort>>>,
    /// Combined message receiver (from all ports)
    /// Reserved for future async message forwarding
    #[allow(dead_code)]
    combined_rx: Receiver<TimestampedMidiMessage>,
    /// Combined message sender
    /// Reserved for future async message forwarding
    #[allow(dead_code)]
    combined_tx: Sender<TimestampedMidiMessage>,
}

impl MidiInputManager {
    /// Create a new MIDI input manager
    pub fn new() -> Self {
        let (tx, rx) = bounded(4096);
        Self {
            ports: Arc::new(Mutex::new(Vec::new())),
            combined_rx: rx,
            combined_tx: tx,
        }
    }

    /// List available devices
    pub fn list_devices(&self) -> Result<Vec<MidiInputDevice>, Error> {
        list_input_devices()
    }

    /// Open a device by index
    pub fn open_device(&self, index: usize) -> Result<(), Error> {
        let port = open_input(index)?;
        self.ports.lock().push(port);
        Ok(())
    }

    /// Open a device by name (partial match)
    pub fn open_device_by_name(&self, name: &str) -> Result<(), Error> {
        let port = open_input_by_name(name)?;
        self.ports.lock().push(port);
        Ok(())
    }

    /// Get number of open ports
    pub fn port_count(&self) -> usize {
        self.ports.lock().len()
    }

    /// Poll all ports and combine messages
    pub fn poll(&self) -> Vec<TimestampedMidiMessage> {
        let ports = self.ports.lock();
        let mut messages = Vec::new();
        for port in ports.iter() {
            messages.extend(port.drain());
        }
        messages
    }

    /// Try to receive from combined channel
    pub fn try_recv(&self) -> Option<TimestampedMidiMessage> {
        self.combined_rx.try_recv().ok()
    }

    /// Close all ports
    pub fn close_all(&self) {
        self.ports.lock().clear();
    }
}

impl Default for MidiInputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock MIDI input port for testing
///
/// Allows programmatic injection of MIDI messages for integration testing
/// without requiring actual MIDI hardware.
pub struct MockMidiInputPort {
    /// Port name
    name: String,
    /// Sender for injecting messages
    sender: Sender<TimestampedMidiMessage>,
    /// Receiver for reading messages
    receiver: Receiver<TimestampedMidiMessage>,
}

impl MockMidiInputPort {
    /// Create a new mock MIDI input port
    pub fn new(name: impl Into<String>) -> Self {
        let (tx, rx) = bounded(1024);
        Self {
            name: name.into(),
            sender: tx,
            receiver: rx,
        }
    }

    /// Get the port name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Inject a MIDI message (for testing)
    pub fn inject(&self, message: MidiMessage, timestamp: u64) {
        let msg = TimestampedMidiMessage { message, timestamp };
        let _ = self.sender.send(msg);
    }

    /// Inject a note on message
    pub fn inject_note_on(&self, channel: u8, note: u8, velocity: u8, timestamp: u64) {
        self.inject(
            MidiMessage::NoteOn { channel, note, velocity },
            timestamp,
        );
    }

    /// Inject a note off message
    pub fn inject_note_off(&self, channel: u8, note: u8, velocity: u8, timestamp: u64) {
        self.inject(
            MidiMessage::NoteOff { channel, note, velocity },
            timestamp,
        );
    }

    /// Inject a control change message
    pub fn inject_cc(&self, channel: u8, control: u8, value: u8, timestamp: u64) {
        self.inject(
            MidiMessage::ControlChange { channel, control, value },
            timestamp,
        );
    }

    /// Try to receive a MIDI message (non-blocking)
    pub fn try_recv(&self) -> Option<TimestampedMidiMessage> {
        self.receiver.try_recv().ok()
    }

    /// Receive a MIDI message (blocking)
    pub fn recv(&self) -> Option<TimestampedMidiMessage> {
        self.receiver.recv().ok()
    }

    /// Get the receiver for use with crossbeam select
    pub fn receiver(&self) -> &Receiver<TimestampedMidiMessage> {
        &self.receiver
    }

    /// Drain all pending messages
    pub fn drain(&self) -> Vec<TimestampedMidiMessage> {
        let mut messages = Vec::new();
        while let Some(msg) = self.try_recv() {
            messages.push(msg);
        }
        messages
    }

    /// Check if there are pending messages
    pub fn has_pending(&self) -> bool {
        !self.receiver.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices_does_not_panic() {
        // May return empty list on CI without MIDI devices
        let result = list_input_devices();
        assert!(result.is_ok());
    }

    #[test]
    fn test_midi_input_manager_creation() {
        let manager = MidiInputManager::new();
        assert_eq!(manager.port_count(), 0);
    }

    #[test]
    fn test_midi_input_manager_list_devices() {
        let manager = MidiInputManager::new();
        let devices = manager.list_devices();
        assert!(devices.is_ok());
    }

    #[test]
    fn test_open_nonexistent_device() {
        let manager = MidiInputManager::new();
        let result = manager.open_device(9999);
        // Should fail gracefully (no panic)
        assert!(result.is_err());
    }

    #[test]
    fn test_open_device_by_nonexistent_name() {
        let manager = MidiInputManager::new();
        let result = manager.open_device_by_name("NonexistentMIDIDevice12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_poll_empty() {
        let manager = MidiInputManager::new();
        let messages = manager.poll();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_close_all() {
        let manager = MidiInputManager::new();
        manager.close_all();
        assert_eq!(manager.port_count(), 0);
    }

    #[test]
    fn test_timestamped_message() {
        let msg = TimestampedMidiMessage {
            message: MidiMessage::NoteOn {
                channel: 0,
                note: 60,
                velocity: 100,
            },
            timestamp: 12345,
        };
        assert_eq!(msg.timestamp, 12345);
        assert!(matches!(msg.message, MidiMessage::NoteOn { .. }));
    }

    // Mock MIDI input tests - no hardware required

    #[test]
    fn test_mock_port_creation() {
        let mock = MockMidiInputPort::new("Test Port");
        assert_eq!(mock.name(), "Test Port");
        assert!(!mock.has_pending());
    }

    #[test]
    fn test_mock_port_inject_and_receive() {
        let mock = MockMidiInputPort::new("Test");

        // Inject a note on
        mock.inject_note_on(0, 60, 100, 1000);
        assert!(mock.has_pending());

        // Receive the message
        let msg = mock.try_recv().expect("Should receive message");
        assert_eq!(msg.timestamp, 1000);
        assert!(matches!(
            msg.message,
            MidiMessage::NoteOn { channel: 0, note: 60, velocity: 100 }
        ));

        assert!(!mock.has_pending());
    }

    #[test]
    fn test_mock_port_multiple_messages() {
        let mock = MockMidiInputPort::new("Test");

        // Inject multiple messages
        mock.inject_note_on(0, 60, 100, 0);
        mock.inject_note_on(0, 64, 90, 100);
        mock.inject_note_on(0, 67, 80, 200);
        mock.inject_note_off(0, 60, 0, 500);
        mock.inject_note_off(0, 64, 0, 600);
        mock.inject_note_off(0, 67, 0, 700);

        // Drain all messages
        let messages = mock.drain();
        assert_eq!(messages.len(), 6);

        // Verify order and content
        assert!(matches!(messages[0].message, MidiMessage::NoteOn { note: 60, .. }));
        assert!(matches!(messages[1].message, MidiMessage::NoteOn { note: 64, .. }));
        assert!(matches!(messages[2].message, MidiMessage::NoteOn { note: 67, .. }));
        assert!(matches!(messages[3].message, MidiMessage::NoteOff { note: 60, .. }));
    }

    #[test]
    fn test_mock_port_cc_messages() {
        let mock = MockMidiInputPort::new("Test");

        // Inject control changes (e.g., volume and modulation)
        mock.inject_cc(0, 7, 100, 0);  // Volume
        mock.inject_cc(0, 1, 64, 10);  // Modulation

        let messages = mock.drain();
        assert_eq!(messages.len(), 2);

        assert!(matches!(
            messages[0].message,
            MidiMessage::ControlChange { channel: 0, control: 7, value: 100 }
        ));
        assert!(matches!(
            messages[1].message,
            MidiMessage::ControlChange { channel: 0, control: 1, value: 64 }
        ));
    }

    #[test]
    fn test_mock_port_direct_inject() {
        let mock = MockMidiInputPort::new("Test");

        // Use direct inject for less common messages
        mock.inject(
            MidiMessage::PitchBend { channel: 0, value: 8192 },
            0,
        );

        let msg = mock.try_recv().expect("Should receive");
        assert!(matches!(
            msg.message,
            MidiMessage::PitchBend { channel: 0, value: 8192 }
        ));
    }

    // Integration test with real MIDI device
    #[test]
    #[ignore] // Run with `cargo test -- --ignored` on machine with MIDI
    fn test_real_midi_input() {
        let devices = list_input_devices().expect("Failed to list devices");
        println!("Available MIDI input devices:");
        for device in &devices {
            println!("  {}: {}", device.index, device.name);
        }

        if !devices.is_empty() {
            let port = open_input(0).expect("Failed to open first device");
            println!("Opened: {}", port.name());

            // Wait for a message
            println!("Waiting for MIDI input (5 seconds)...");
            let start = std::time::Instant::now();
            while start.elapsed() < std::time::Duration::from_secs(5) {
                if let Some(msg) = port.try_recv() {
                    println!("Received: {:?} at {}", msg.message, msg.timestamp);
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}
