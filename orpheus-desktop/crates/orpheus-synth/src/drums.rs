//! Drum synthesis module
//!
//! Synthesizes common drum sounds using basic waveforms:
//! - Kick drum: Low frequency sine with pitch envelope
//! - Snare: Noise burst with bandpass filter + tone component
//! - Hi-hat: High-pass filtered noise with fast decay
//! - Toms: Mid-range sine with pitch envelope
//! - Crash/Ride: Complex noise with metallic resonance

use rand::Rng;

/// Drum types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrumType {
    Kick,
    /// Tight kick for metal (faster attack, shorter decay)
    MetalKick,
    Snare,
    ClosedHiHat,
    OpenHiHat,
    /// Pedal hi-hat (foot-controlled)
    PedalHiHat,
    HighTom,
    MidTom,
    LowTom,
    /// Floor tom
    FloorTom,
    Crash,
    Ride,
    /// Ride bell (different articulation)
    RideBell,
    /// China cymbal (essential for metal)
    China,
    /// Splash cymbal
    Splash,
    Clap,
    Rimshot,
    /// Ghost note snare (very soft)
    GhostSnare,
    Cowbell,
}

impl DrumType {
    /// Get MIDI note number for this drum (General MIDI mapping)
    pub fn midi_note(&self) -> u8 {
        match self {
            Self::Kick => 36,
            Self::MetalKick => 35, // Acoustic bass drum
            Self::Snare => 38,
            Self::ClosedHiHat => 42,
            Self::OpenHiHat => 46,
            Self::PedalHiHat => 44,
            Self::HighTom => 50,
            Self::MidTom => 47,
            Self::LowTom => 45,
            Self::FloorTom => 43,
            Self::Crash => 49,
            Self::Ride => 51,
            Self::RideBell => 53,
            Self::China => 52,
            Self::Splash => 55,
            Self::Clap => 39,
            Self::Rimshot => 37,
            Self::GhostSnare => 38, // Same as snare, different velocity
            Self::Cowbell => 56,
        }
    }

    /// Get from MIDI note number
    pub fn from_midi(note: u8) -> Option<Self> {
        match note {
            35 => Some(Self::MetalKick),
            36 => Some(Self::Kick),
            37 => Some(Self::Rimshot),
            38 => Some(Self::Snare),
            39 => Some(Self::Clap),
            42 => Some(Self::ClosedHiHat),
            43 => Some(Self::FloorTom),
            44 => Some(Self::PedalHiHat),
            45 => Some(Self::LowTom),
            46 => Some(Self::OpenHiHat),
            47 => Some(Self::MidTom),
            49 => Some(Self::Crash),
            50 => Some(Self::HighTom),
            51 => Some(Self::Ride),
            52 => Some(Self::China),
            53 => Some(Self::RideBell),
            55 => Some(Self::Splash),
            56 => Some(Self::Cowbell),
            _ => None,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Kick => "Kick",
            Self::MetalKick => "Metal Kick",
            Self::Snare => "Snare",
            Self::ClosedHiHat => "Closed HH",
            Self::OpenHiHat => "Open HH",
            Self::PedalHiHat => "Pedal HH",
            Self::HighTom => "High Tom",
            Self::MidTom => "Mid Tom",
            Self::LowTom => "Low Tom",
            Self::FloorTom => "Floor Tom",
            Self::Crash => "Crash",
            Self::Ride => "Ride",
            Self::RideBell => "Ride Bell",
            Self::China => "China",
            Self::Splash => "Splash",
            Self::Clap => "Clap",
            Self::Rimshot => "Rimshot",
            Self::GhostSnare => "Ghost Snare",
            Self::Cowbell => "Cowbell",
        }
    }
}

/// Single drum voice synthesizer
pub struct DrumVoice {
    /// Sample rate
    sample_rate: u32,
    /// Current phase
    phase: f32,
    /// Current amplitude envelope
    amplitude: f32,
    /// Pitch envelope
    pitch_env: f32,
    /// Noise state
    noise_state: f32,
    /// Filter state (for hi-pass/band-pass)
    filter_state: f32,
    /// Second filter state
    filter_state2: f32,
    /// Time counter
    time: f32,
    /// Is voice active
    active: bool,
    /// Drum type being played
    drum_type: DrumType,
    /// Velocity
    velocity: f32,
}

impl DrumVoice {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            amplitude: 0.0,
            pitch_env: 0.0,
            noise_state: 0.0,
            filter_state: 0.0,
            filter_state2: 0.0,
            time: 0.0,
            active: false,
            drum_type: DrumType::Kick,
            velocity: 0.0,
        }
    }

    /// Trigger a drum hit
    pub fn trigger(&mut self, drum: DrumType, velocity: f32) {
        self.drum_type = drum;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.phase = 0.0;
        self.amplitude = 1.0;
        self.pitch_env = 1.0;
        self.time = 0.0;
        self.filter_state = 0.0;
        self.filter_state2 = 0.0;
        self.active = true;
    }

    /// Check if voice is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Generate next sample
    pub fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let sample = match self.drum_type {
            DrumType::Kick => self.synth_kick(),
            DrumType::MetalKick => self.synth_metal_kick(),
            DrumType::Snare => self.synth_snare(),
            DrumType::GhostSnare => self.synth_ghost_snare(),
            DrumType::ClosedHiHat => self.synth_closed_hihat(),
            DrumType::OpenHiHat => self.synth_open_hihat(),
            DrumType::PedalHiHat => self.synth_pedal_hihat(),
            DrumType::HighTom => self.synth_tom(200.0),
            DrumType::MidTom => self.synth_tom(150.0),
            DrumType::LowTom => self.synth_tom(100.0),
            DrumType::FloorTom => self.synth_tom(80.0),
            DrumType::Crash => self.synth_crash(),
            DrumType::Ride => self.synth_ride(),
            DrumType::RideBell => self.synth_ride_bell(),
            DrumType::China => self.synth_china(),
            DrumType::Splash => self.synth_splash(),
            DrumType::Clap => self.synth_clap(),
            DrumType::Rimshot => self.synth_rimshot(),
            DrumType::Cowbell => self.synth_cowbell(),
        };

        self.time += 1.0 / self.sample_rate as f32;

        // Check if decayed
        if self.amplitude < 0.001 {
            self.active = false;
        }

        sample * self.velocity
    }

    fn synth_kick(&mut self) -> f32 {
        // Kick: pitch sweep from ~150Hz to ~50Hz with sine wave
        let start_freq = 150.0;
        let end_freq = 50.0;

        // Pitch envelope (fast decay)
        self.pitch_env *= 0.997;
        let freq = end_freq + (start_freq - end_freq) * self.pitch_env;

        // Phase increment
        let phase_inc = freq / self.sample_rate as f32;
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Sine wave
        let tone = (self.phase * std::f32::consts::TAU).sin();

        // Add a bit of click at the start
        let click = if self.time < 0.005 {
            (1.0 - self.time / 0.005) * self.generate_noise() * 0.3
        } else {
            0.0
        };

        // Amplitude envelope
        self.amplitude *= 0.9995;

        (tone + click) * self.amplitude * 0.9
    }

    /// Metal kick - tighter, punchier, faster attack for double bass
    fn synth_metal_kick(&mut self) -> f32 {
        // Metal kick: faster pitch sweep, more click, tighter decay
        let start_freq = 180.0;  // Higher start
        let end_freq = 55.0;

        // Very fast pitch envelope (2ms sweep)
        self.pitch_env *= 0.99;  // Much faster than regular kick
        let freq = end_freq + (start_freq - end_freq) * self.pitch_env;

        let phase_inc = freq / self.sample_rate as f32;
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Sine with slight saturation for punch
        let tone = (self.phase * std::f32::consts::TAU).sin();
        let saturated = (tone * 1.5).tanh();

        // Strong beater click at start
        let click = if self.time < 0.003 {
            (1.0 - self.time / 0.003) * self.generate_noise() * 0.5
        } else {
            0.0
        };

        // Tight amplitude envelope (much faster decay)
        self.amplitude *= 0.998;

        (saturated + click) * self.amplitude * 0.95
    }

    fn synth_snare(&mut self) -> f32 {
        // Snare: tone component + noise component

        // Tone (around 200 Hz with fast decay)
        let freq = 200.0 * (1.0 + self.pitch_env * 0.5);
        self.pitch_env *= 0.99;

        let phase_inc = freq / self.sample_rate as f32;
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let tone = (self.phase * std::f32::consts::TAU).sin();

        // Noise component with bandpass
        let noise = self.generate_noise();
        // Simple highpass to remove low frequencies
        self.filter_state = self.filter_state * 0.85 + noise * 0.15;
        let filtered_noise = noise - self.filter_state;

        // Mix tone and noise
        let tone_decay = (-self.time * 50.0).exp();
        let noise_decay = (-self.time * 15.0).exp();

        self.amplitude = (tone_decay + noise_decay).min(1.0);

        (tone * tone_decay * 0.4 + filtered_noise * noise_decay * 0.6) * self.amplitude
    }

    /// Ghost snare - very soft, used for fills and grooves
    fn synth_ghost_snare(&mut self) -> f32 {
        // Similar to snare but much softer with less tone
        let freq = 220.0;
        let phase_inc = freq / self.sample_rate as f32;
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let tone = (self.phase * std::f32::consts::TAU).sin();

        let noise = self.generate_noise();
        self.filter_state = self.filter_state * 0.8 + noise * 0.2;
        let filtered_noise = noise - self.filter_state;

        // Very fast decay
        let tone_decay = (-self.time * 80.0).exp();
        let noise_decay = (-self.time * 40.0).exp();

        self.amplitude = (tone_decay + noise_decay).min(1.0);

        // Much softer mix
        (tone * tone_decay * 0.2 + filtered_noise * noise_decay * 0.3) * self.amplitude * 0.5
    }

    fn synth_closed_hihat(&mut self) -> f32 {
        // Closed hi-hat: high-frequency noise with very fast decay
        let noise = self.generate_noise();

        // High-pass filter (makes it brighter)
        self.filter_state = self.filter_state * 0.3 + noise * 0.7;
        let filtered = noise - self.filter_state;

        // Add some metallic resonance
        let resonance = (self.phase * std::f32::consts::TAU * 8000.0 / self.sample_rate as f32).sin() * 0.2;
        self.phase += 1.0 / self.sample_rate as f32;

        // Very fast decay
        self.amplitude *= 0.995;

        (filtered * 0.8 + resonance * 0.2) * self.amplitude * 0.7
    }

    fn synth_open_hihat(&mut self) -> f32 {
        // Open hi-hat: similar to closed but longer decay
        let noise = self.generate_noise();

        self.filter_state = self.filter_state * 0.3 + noise * 0.7;
        let filtered = noise - self.filter_state;

        let resonance = (self.phase * std::f32::consts::TAU * 7500.0 / self.sample_rate as f32).sin() * 0.15;
        self.phase += 1.0 / self.sample_rate as f32;

        // Slower decay than closed
        self.amplitude *= 0.9992;

        (filtered * 0.85 + resonance * 0.15) * self.amplitude * 0.6
    }

    /// Pedal hi-hat - foot-controlled "chick" sound
    fn synth_pedal_hihat(&mut self) -> f32 {
        let noise = self.generate_noise();

        // Very tight, muted sound
        self.filter_state = self.filter_state * 0.2 + noise * 0.8;
        let filtered = noise - self.filter_state;

        // Very fast decay
        self.amplitude *= 0.99;

        filtered * self.amplitude * 0.5
    }

    fn synth_tom(&mut self, base_freq: f32) -> f32 {
        // Tom: pitched drum with some noise
        let freq = base_freq * (1.0 + self.pitch_env * 0.5);
        self.pitch_env *= 0.998;

        let phase_inc = freq / self.sample_rate as f32;
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let tone = (self.phase * std::f32::consts::TAU).sin();

        // Small noise component
        let noise = self.generate_noise() * 0.1;

        self.amplitude *= 0.9993;

        (tone * 0.9 + noise) * self.amplitude * 0.8
    }

    fn synth_crash(&mut self) -> f32 {
        // Crash: complex noise with long decay
        let noise = self.generate_noise();

        // Multiple resonant frequencies for metallic sound
        let res1 = (self.time * 2500.0 * std::f32::consts::TAU).sin() * 0.15;
        let res2 = (self.time * 3700.0 * std::f32::consts::TAU).sin() * 0.1;
        let res3 = (self.time * 5100.0 * std::f32::consts::TAU).sin() * 0.08;

        // High-pass filter
        self.filter_state = self.filter_state * 0.4 + noise * 0.6;
        let filtered = noise - self.filter_state;

        // Long decay
        self.amplitude *= 0.9998;

        (filtered * 0.7 + res1 + res2 + res3) * self.amplitude * 0.5
    }

    fn synth_ride(&mut self) -> f32 {
        // Ride: cleaner metallic sound than crash
        let noise = self.generate_noise() * 0.5;

        // Strong resonant component
        let res1 = (self.time * 3000.0 * std::f32::consts::TAU).sin() * 0.3;
        let res2 = (self.time * 4500.0 * std::f32::consts::TAU).sin() * 0.2;

        // High-pass
        self.filter_state = self.filter_state * 0.5 + noise * 0.5;
        let filtered = noise - self.filter_state;

        // Moderate decay
        self.amplitude *= 0.9995;

        (filtered * 0.5 + res1 + res2) * self.amplitude * 0.6
    }

    /// Ride bell - ping on the bell of the ride
    fn synth_ride_bell(&mut self) -> f32 {
        // More tonal, bell-like sound
        let res1 = (self.time * 2800.0 * std::f32::consts::TAU).sin() * 0.4;
        let res2 = (self.time * 4200.0 * std::f32::consts::TAU).sin() * 0.3;
        let res3 = (self.time * 5600.0 * std::f32::consts::TAU).sin() * 0.2;

        // Little noise
        let noise = self.generate_noise() * 0.1;

        // Moderate decay
        self.amplitude *= 0.9996;

        (res1 + res2 + res3 + noise) * self.amplitude * 0.7
    }

    /// China cymbal - trashy, aggressive sound (essential for metal)
    fn synth_china(&mut self) -> f32 {
        let noise = self.generate_noise();

        // Inharmonic, trashy resonances
        let res1 = (self.time * 1800.0 * std::f32::consts::TAU).sin() * 0.2;
        let res2 = (self.time * 2700.0 * std::f32::consts::TAU).sin() * 0.15;
        let res3 = (self.time * 4100.0 * std::f32::consts::TAU).sin() * 0.15;
        let res4 = (self.time * 5900.0 * std::f32::consts::TAU).sin() * 0.1;

        // Bandpass for trashy character
        self.filter_state = self.filter_state * 0.5 + noise * 0.5;
        self.filter_state2 = self.filter_state2 * 0.7 + self.filter_state * 0.3;
        let filtered = self.filter_state - self.filter_state2;

        // Long decay with initial burst
        let burst = if self.time < 0.01 { 1.5 } else { 1.0 };
        self.amplitude *= 0.9997;

        (filtered * 0.5 + res1 + res2 + res3 + res4) * self.amplitude * burst * 0.55
    }

    /// Splash cymbal - short, bright accent
    fn synth_splash(&mut self) -> f32 {
        let noise = self.generate_noise();

        // Bright resonances
        let res1 = (self.time * 4500.0 * std::f32::consts::TAU).sin() * 0.25;
        let res2 = (self.time * 6200.0 * std::f32::consts::TAU).sin() * 0.2;

        // High-pass
        self.filter_state = self.filter_state * 0.25 + noise * 0.75;
        let filtered = noise - self.filter_state;

        // Quick decay (shorter than crash)
        self.amplitude *= 0.9993;

        (filtered * 0.6 + res1 + res2) * self.amplitude * 0.5
    }

    fn synth_clap(&mut self) -> f32 {
        // Clap: multiple noise bursts
        let noise = self.generate_noise();

        // Bandpass filter
        self.filter_state = self.filter_state * 0.7 + noise * 0.3;
        self.filter_state2 = self.filter_state2 * 0.7 + self.filter_state * 0.3;
        let filtered = self.filter_state - self.filter_state2;

        // Multiple attacks (simulates hands hitting)
        let attack_times = [0.0, 0.015, 0.030, 0.045];
        let mut env = 0.0;
        for &t in &attack_times {
            if self.time >= t {
                let decay = (-(self.time - t) * 30.0).exp();
                env += decay;
            }
        }
        env = env.min(1.0);

        self.amplitude = env;

        filtered * self.amplitude * 0.7
    }

    fn synth_rimshot(&mut self) -> f32 {
        // Rimshot: short click with some tone
        let freq = 300.0;
        let phase_inc = freq / self.sample_rate as f32;
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let tone = (self.phase * std::f32::consts::TAU).sin();
        let click = if self.time < 0.002 { self.generate_noise() } else { 0.0 };

        // Very fast decay
        self.amplitude *= 0.99;

        (tone * 0.5 + click * 0.5) * self.amplitude * 0.8
    }

    fn synth_cowbell(&mut self) -> f32 {
        // Cowbell: two detuned square-ish waves
        let freq1 = 587.0; // D5
        let freq2 = 845.0; // Slightly sharp G#5

        let phase_inc1 = freq1 / self.sample_rate as f32;
        let phase_inc2 = freq2 / self.sample_rate as f32;

        self.phase += phase_inc1;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        self.filter_state += phase_inc2;
        if self.filter_state >= 1.0 {
            self.filter_state -= 1.0;
        }

        // Pulse waves
        let pulse1 = if self.phase < 0.5 { 1.0 } else { -1.0 };
        let pulse2 = if self.filter_state < 0.5 { 1.0 } else { -1.0 };

        // Moderate decay
        self.amplitude *= 0.9993;

        (pulse1 * 0.5 + pulse2 * 0.5) * self.amplitude * 0.5
    }

    fn generate_noise(&mut self) -> f32 {
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() * 2.0 - 1.0
    }
}

/// Polyphonic drum machine
pub struct DrumMachine {
    /// Multiple voices for polyphony
    voices: Vec<DrumVoice>,
    /// Sample rate
    sample_rate: u32,
    /// Master volume
    volume: f32,
}

impl DrumMachine {
    /// Create a new drum machine with given polyphony
    pub fn new(sample_rate: u32, polyphony: usize) -> Self {
        let mut voices = Vec::with_capacity(polyphony);
        for _ in 0..polyphony {
            voices.push(DrumVoice::new(sample_rate));
        }

        Self {
            voices,
            sample_rate,
            volume: 0.8,
        }
    }

    /// Create with default settings (8-voice polyphony)
    pub fn default_machine(sample_rate: u32) -> Self {
        Self::new(sample_rate, 8)
    }

    /// Create with metal settings (16-voice polyphony for blast beats)
    pub fn metal_machine(sample_rate: u32) -> Self {
        Self::new(sample_rate, 16)
    }

    /// Create with high polyphony for complex patterns (24 voices)
    pub fn high_polyphony(sample_rate: u32) -> Self {
        Self::new(sample_rate, 24)
    }

    /// Trigger a drum hit
    pub fn trigger(&mut self, drum: DrumType, velocity: f32) {
        // Find a free voice or steal the first one
        let voice_idx = self.voices
            .iter()
            .position(|v| !v.is_active())
            .unwrap_or(0);

        if let Some(voice) = self.voices.get_mut(voice_idx) {
            voice.trigger(drum, velocity);
        }
    }

    /// Trigger from MIDI note
    pub fn trigger_midi(&mut self, note: u8, velocity: u8) {
        if let Some(drum) = DrumType::from_midi(note) {
            self.trigger(drum, velocity as f32 / 127.0);
        }
    }

    /// Set master volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Generate next mono sample
    pub fn next_sample(&mut self) -> f32 {
        let mut output = 0.0;
        for voice in &mut self.voices {
            if voice.is_active() {
                output += voice.next_sample();
            }
        }
        output * self.volume
    }

    /// Fill a mono buffer
    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
        }
    }

    /// Add to a buffer (mixing)
    pub fn add_to_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample += self.next_sample();
        }
    }

    /// Check if any voice is active
    pub fn is_active(&self) -> bool {
        self.voices.iter().any(|v| v.is_active())
    }
}

/// Drum pattern step
#[derive(Debug, Clone, Copy)]
pub struct DrumStep {
    /// Drum type
    pub drum: DrumType,
    /// Velocity (0.0 - 1.0)
    pub velocity: f32,
}

/// Drum pattern (one bar)
pub struct DrumPattern {
    /// Pattern name
    pub name: String,
    /// Steps per bar (typically 16 for 16th notes)
    pub steps: usize,
    /// Pattern data: each step can have multiple drums
    pub data: Vec<Vec<DrumStep>>,
}

impl DrumPattern {
    /// Create an empty pattern
    pub fn new(name: impl Into<String>, steps: usize) -> Self {
        Self {
            name: name.into(),
            steps,
            data: vec![Vec::new(); steps],
        }
    }

    /// Add a drum hit at a step
    pub fn add(&mut self, step: usize, drum: DrumType, velocity: f32) {
        if step < self.steps {
            self.data[step].push(DrumStep { drum, velocity });
        }
    }

    /// Remove all drums at a step
    pub fn clear_step(&mut self, step: usize) {
        if step < self.steps {
            self.data[step].clear();
        }
    }

    /// Get hits at a step
    pub fn get_step(&self, step: usize) -> &[DrumStep] {
        if step < self.steps {
            &self.data[step]
        } else {
            &[]
        }
    }

    /// Create a basic rock beat
    pub fn rock_beat() -> Self {
        let mut pattern = Self::new("Rock Beat", 16);

        // Kick on 1 and 3 (steps 0 and 8)
        pattern.add(0, DrumType::Kick, 0.9);
        pattern.add(8, DrumType::Kick, 0.85);

        // Snare on 2 and 4 (steps 4 and 12)
        pattern.add(4, DrumType::Snare, 0.85);
        pattern.add(12, DrumType::Snare, 0.85);

        // Hi-hat on every 8th note
        for i in (0..16).step_by(2) {
            pattern.add(i, DrumType::ClosedHiHat, 0.6);
        }

        pattern
    }

    /// Create a disco beat
    pub fn disco_beat() -> Self {
        let mut pattern = Self::new("Disco Beat", 16);

        // Four-on-the-floor kick
        for i in (0..16).step_by(4) {
            pattern.add(i, DrumType::Kick, 0.9);
        }

        // Snare on 2 and 4
        pattern.add(4, DrumType::Snare, 0.8);
        pattern.add(12, DrumType::Snare, 0.8);

        // Open hi-hat on off-beats
        for i in (2..16).step_by(4) {
            pattern.add(i, DrumType::OpenHiHat, 0.7);
        }

        // Closed hi-hat on 8th notes
        for i in (0..16).step_by(2) {
            pattern.add(i, DrumType::ClosedHiHat, 0.5);
        }

        pattern
    }

    /// Create a metal double-kick pattern
    pub fn metal_beat() -> Self {
        let mut pattern = Self::new("Metal Beat", 16);

        // Double kick
        for i in 0..16 {
            if i % 2 == 0 || i == 7 || i == 15 {
                pattern.add(i, DrumType::Kick, 0.85);
            }
        }

        // Snare on 2 and 4
        pattern.add(4, DrumType::Snare, 0.9);
        pattern.add(12, DrumType::Snare, 0.9);

        // Ride/crash pattern
        for i in (0..16).step_by(2) {
            pattern.add(i, DrumType::Ride, 0.6);
        }

        pattern
    }

    /// Create a blast beat pattern (32 steps for 32nd note resolution)
    /// Standard blast: snare and kick alternate on every beat, hi-hat on every beat
    pub fn blast_beat() -> Self {
        let mut pattern = Self::new("Blast Beat", 32);

        // Blast beat at 32nd note resolution
        for i in 0..32 {
            // Alternating kick and snare
            if i % 2 == 0 {
                pattern.add(i, DrumType::MetalKick, 0.9);
            } else {
                pattern.add(i, DrumType::Snare, 0.85);
            }

            // Hi-hat on every 8th note (every 4 steps at 32nd resolution)
            if i % 4 == 0 {
                pattern.add(i, DrumType::ClosedHiHat, 0.7);
            }
        }

        pattern
    }

    /// Hammer blast - kick on every beat, snare on off-beats
    pub fn hammer_blast() -> Self {
        let mut pattern = Self::new("Hammer Blast", 32);

        for i in 0..32 {
            // Kick on every 16th note
            if i % 2 == 0 {
                pattern.add(i, DrumType::MetalKick, 0.9);
            }

            // Snare on off-beat 16ths
            if i % 4 == 2 {
                pattern.add(i, DrumType::Snare, 0.88);
            }

            // China on quarter notes
            if i % 8 == 0 {
                pattern.add(i, DrumType::China, 0.75);
            }
        }

        pattern
    }

    /// Gravity blast - one-handed roll alternating kick
    pub fn gravity_blast() -> Self {
        let mut pattern = Self::new("Gravity Blast", 32);

        for i in 0..32 {
            // Snare on every 32nd note (one-handed roll)
            pattern.add(i, DrumType::Snare, if i % 2 == 0 { 0.85 } else { 0.75 });

            // Kick on every 16th note
            if i % 2 == 0 {
                pattern.add(i, DrumType::MetalKick, 0.88);
            }

            // Ride bell on quarter notes
            if i % 8 == 0 {
                pattern.add(i, DrumType::RideBell, 0.7);
            }
        }

        pattern
    }

    /// Technical death metal pattern with ghost notes
    pub fn tech_death_beat() -> Self {
        let mut pattern = Self::new("Tech Death", 32);

        // Complex kick pattern
        let kick_steps = [0, 3, 6, 8, 11, 14, 16, 19, 22, 24, 27, 30];
        for &step in &kick_steps {
            pattern.add(step, DrumType::MetalKick, 0.9);
        }

        // Snare on 2 and 4 (steps 8 and 24 at 32nd resolution)
        pattern.add(8, DrumType::Snare, 0.9);
        pattern.add(24, DrumType::Snare, 0.9);

        // Ghost notes
        for step in [4, 12, 20, 28] {
            pattern.add(step, DrumType::GhostSnare, 0.4);
        }

        // Hi-hat pattern with accents
        for i in (0..32).step_by(2) {
            let vel = if i % 8 == 0 { 0.8 } else { 0.5 };
            pattern.add(i, DrumType::ClosedHiHat, vel);
        }

        // China accent
        pattern.add(0, DrumType::China, 0.75);

        pattern
    }

    /// Double bass 16th note pattern
    pub fn double_bass_16ths() -> Self {
        let mut pattern = Self::new("Double Bass 16ths", 16);

        // Continuous 16th note kicks
        for i in 0..16 {
            pattern.add(i, DrumType::MetalKick, 0.85);
        }

        // Snare on 2 and 4
        pattern.add(4, DrumType::Snare, 0.9);
        pattern.add(12, DrumType::Snare, 0.9);

        // China on beat 1
        pattern.add(0, DrumType::China, 0.75);

        // Ride on 8th notes
        for i in (0..16).step_by(2) {
            pattern.add(i, DrumType::Ride, 0.6);
        }

        pattern
    }

    /// D-beat punk/crust pattern
    pub fn d_beat() -> Self {
        let mut pattern = Self::new("D-Beat", 16);

        // D-beat kick pattern
        pattern.add(0, DrumType::Kick, 0.9);
        pattern.add(4, DrumType::Kick, 0.85);
        pattern.add(6, DrumType::Kick, 0.85);
        pattern.add(8, DrumType::Kick, 0.9);
        pattern.add(12, DrumType::Kick, 0.85);
        pattern.add(14, DrumType::Kick, 0.85);

        // Snare on 2 and 4
        pattern.add(4, DrumType::Snare, 0.9);
        pattern.add(12, DrumType::Snare, 0.9);

        // Crash ride pattern
        for i in (0..16).step_by(2) {
            pattern.add(i, DrumType::Crash, 0.6);
        }

        pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drum_voice() {
        let mut voice = DrumVoice::new(44100);
        voice.trigger(DrumType::Kick, 0.8);

        assert!(voice.is_active());

        // Generate some samples
        let mut buffer = vec![0.0; 1000];
        for sample in &mut buffer {
            *sample = voice.next_sample();
        }

        // Should have output
        let max = buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_drum_machine() {
        let mut machine = DrumMachine::default_machine(44100);

        machine.trigger(DrumType::Kick, 0.8);
        machine.trigger(DrumType::Snare, 0.7);
        machine.trigger(DrumType::ClosedHiHat, 0.6);

        assert!(machine.is_active());

        let mut buffer = vec![0.0; 1000];
        machine.fill_buffer(&mut buffer);

        let max = buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_drum_pattern() {
        let pattern = DrumPattern::rock_beat();

        assert_eq!(pattern.steps, 16);
        assert!(!pattern.get_step(0).is_empty()); // Has kick
        assert!(!pattern.get_step(4).is_empty()); // Has snare
    }

    #[test]
    fn test_midi_mapping() {
        assert_eq!(DrumType::from_midi(36), Some(DrumType::Kick));
        assert_eq!(DrumType::from_midi(38), Some(DrumType::Snare));
        assert_eq!(DrumType::from_midi(42), Some(DrumType::ClosedHiHat));
        assert_eq!(DrumType::Kick.midi_note(), 36);
    }
}
