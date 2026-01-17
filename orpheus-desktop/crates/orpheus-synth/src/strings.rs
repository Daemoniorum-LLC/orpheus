//! Bowed string synthesizer using physical modeling
//!
//! Implements realistic bowed string instruments (violin, viola, cello, bass)
//! using digital waveguide synthesis with bow-string friction modeling.

use std::f32::consts::PI;

/// String instrument type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StringInstrument {
    Violin,
    Viola,
    Cello,
    DoubleBass,
}

impl Default for StringInstrument {
    fn default() -> Self {
        Self::Violin
    }
}

impl StringInstrument {
    /// Get the open string pitches (MIDI notes) for this instrument
    pub fn open_strings(&self) -> &[u8] {
        match self {
            // G3, D4, A4, E5
            StringInstrument::Violin => &[55, 62, 69, 76],
            // C3, G3, D4, A4
            StringInstrument::Viola => &[48, 55, 62, 69],
            // C2, G2, D3, A3
            StringInstrument::Cello => &[36, 43, 50, 57],
            // E1, A1, D2, G2 (orchestra tuning)
            StringInstrument::DoubleBass => &[28, 33, 38, 43],
        }
    }

    /// Get characteristic body resonance frequencies
    fn body_resonances(&self) -> &[(f32, f32, f32)] {
        // (frequency, Q, gain)
        match self {
            StringInstrument::Violin => &[
                (280.0, 8.0, 0.8),   // Main air resonance
                (460.0, 12.0, 0.6),  // Main wood resonance
                (550.0, 15.0, 0.4),  //
                (700.0, 10.0, 0.3),
                (1200.0, 8.0, 0.25),
                (3000.0, 6.0, 0.15), // Brilliance
            ],
            StringInstrument::Viola => &[
                (220.0, 8.0, 0.8),
                (380.0, 12.0, 0.6),
                (480.0, 15.0, 0.4),
                (600.0, 10.0, 0.3),
                (1000.0, 8.0, 0.25),
                (2500.0, 6.0, 0.15),
            ],
            StringInstrument::Cello => &[
                (100.0, 6.0, 0.9),
                (180.0, 10.0, 0.7),
                (280.0, 12.0, 0.5),
                (400.0, 10.0, 0.35),
                (700.0, 8.0, 0.25),
                (1800.0, 6.0, 0.15),
            ],
            StringInstrument::DoubleBass => &[
                (60.0, 5.0, 0.95),
                (100.0, 8.0, 0.8),
                (180.0, 10.0, 0.6),
                (280.0, 10.0, 0.4),
                (500.0, 8.0, 0.25),
                (1200.0, 6.0, 0.1),
            ],
        }
    }
}

/// Articulation type for string playing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StringArticulation {
    /// Normal sustained bowing (arco)
    Arco,
    /// Smooth connected notes
    Legato,
    /// Short, separated notes
    Staccato,
    /// Very short, bounced bow
    Spiccato,
    /// Plucked (like guitar)
    Pizzicato,
    /// Rapid bow tremolo
    Tremolo,
    /// Sul ponticello (near bridge, glassy)
    SulPonticello,
    /// Sul tasto (over fingerboard, soft)
    SulTasto,
    /// Harmonic
    Harmonic,
}

impl Default for StringArticulation {
    fn default() -> Self {
        Self::Arco
    }
}

/// Biquad filter for body resonance
#[derive(Debug, Clone)]
struct ResonanceFilter {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl ResonanceFilter {
    fn new(freq: f32, q: f32, gain: f32, sample_rate: f32) -> Self {
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let a = 10.0_f32.powf(gain / 20.0);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
                   - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    fn reset(&mut self) {
        self.x1 = 0.0; self.x2 = 0.0;
        self.y1 = 0.0; self.y2 = 0.0;
    }
}

/// Single bowed string voice
#[derive(Debug, Clone)]
pub struct BowedString {
    /// Sample rate
    sample_rate: f32,
    /// Delay line (string waveguide)
    delay_line: Vec<f32>,
    /// Delay line position
    delay_pos: usize,
    /// Current period (fractional for pitch accuracy)
    period: f64,
    /// Bow velocity (affects amplitude and tone)
    bow_velocity: f32,
    /// Bow pressure (affects grip and brightness)
    bow_pressure: f32,
    /// Bow position (0.0 = bridge, 1.0 = fingerboard)
    bow_position: f32,
    /// String damping
    damping: f32,
    /// Feedback coefficient
    feedback: f32,
    /// Previous output for lowpass
    prev_output: f32,
    /// Bow-string state (for stick-slip)
    bow_state: f32,
    /// Is currently bowing
    is_bowing: bool,
    /// Current amplitude
    amplitude: f32,
    /// Target amplitude (for attack/release)
    target_amplitude: f32,
    /// Amplitude slew rate
    amp_slew: f32,
    /// Vibrato depth (semitones)
    vibrato_depth: f32,
    /// Vibrato rate (Hz)
    vibrato_rate: f32,
    /// Vibrato phase
    vibrato_phase: f64,
    /// Current articulation
    articulation: StringArticulation,
    /// Tremolo phase (for tremolo articulation)
    tremolo_phase: f64,
    /// Tremolo rate
    tremolo_rate: f32,
}

impl BowedString {
    /// Create a new bowed string
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            delay_line: vec![0.0; 4096],
            delay_pos: 0,
            period: 100.0,
            bow_velocity: 0.0,
            bow_pressure: 0.5,
            bow_position: 0.12, // Default: slightly away from bridge
            damping: 0.3,
            feedback: 0.995,
            prev_output: 0.0,
            bow_state: 0.0,
            is_bowing: false,
            amplitude: 0.0,
            target_amplitude: 0.0,
            amp_slew: 0.001,
            vibrato_depth: 0.0,
            vibrato_rate: 5.5,
            vibrato_phase: 0.0,
            articulation: StringArticulation::Arco,
            tremolo_phase: 0.0,
            tremolo_rate: 8.0,
        }
    }

    /// Start bowing at a frequency
    pub fn bow(&mut self, frequency: f32, velocity: f32) {
        self.period = self.sample_rate as f64 / frequency as f64;
        self.bow_velocity = velocity.clamp(0.0, 1.0);
        self.is_bowing = true;

        // Set target amplitude based on velocity and articulation
        self.target_amplitude = match self.articulation {
            StringArticulation::Staccato | StringArticulation::Spiccato => velocity * 0.9,
            StringArticulation::Pizzicato => velocity * 1.2,
            StringArticulation::SulTasto => velocity * 0.6,
            _ => velocity * 0.8,
        };

        // Set attack speed based on articulation
        self.amp_slew = match self.articulation {
            StringArticulation::Legato => 0.0005,
            StringArticulation::Staccato | StringArticulation::Spiccato => 0.01,
            StringArticulation::Pizzicato => 0.05,
            _ => 0.002,
        };

        // Initialize delay line for new note
        if self.amplitude < 0.01 {
            self.delay_line.fill(0.0);
            self.bow_state = 0.0;
        }
    }

    /// Stop bowing (release)
    pub fn release(&mut self) {
        self.is_bowing = false;
        self.target_amplitude = 0.0;

        // Set release speed based on articulation
        self.amp_slew = match self.articulation {
            StringArticulation::Staccato | StringArticulation::Spiccato => 0.02,
            StringArticulation::Pizzicato => 0.0005, // Let it ring
            _ => 0.001,
        };
    }

    /// Set bow pressure (0.0 - 1.0)
    pub fn set_bow_pressure(&mut self, pressure: f32) {
        self.bow_pressure = pressure.clamp(0.1, 1.0);
    }

    /// Set bow position (0.0 = sul ponticello, 1.0 = sul tasto)
    pub fn set_bow_position(&mut self, position: f32) {
        self.bow_position = position.clamp(0.05, 0.5);
    }

    /// Set articulation
    pub fn set_articulation(&mut self, articulation: StringArticulation) {
        self.articulation = articulation;

        // Adjust parameters based on articulation
        match articulation {
            StringArticulation::SulPonticello => {
                self.bow_position = 0.05;
                self.damping = 0.15;
            }
            StringArticulation::SulTasto => {
                self.bow_position = 0.4;
                self.damping = 0.5;
            }
            StringArticulation::Pizzicato => {
                self.feedback = 0.992;
                self.damping = 0.4;
            }
            StringArticulation::Tremolo => {
                self.tremolo_rate = 12.0;
            }
            _ => {
                self.bow_position = 0.12;
                self.damping = 0.3;
                self.feedback = 0.995;
            }
        }
    }

    /// Set vibrato parameters
    pub fn set_vibrato(&mut self, depth_semitones: f32, rate_hz: f32) {
        self.vibrato_depth = depth_semitones.clamp(0.0, 1.0);
        self.vibrato_rate = rate_hz.clamp(3.0, 8.0);
    }

    /// Enable default string vibrato
    pub fn enable_vibrato(&mut self) {
        self.vibrato_depth = 0.25;
        self.vibrato_rate = 5.5;
    }

    /// Disable vibrato
    pub fn disable_vibrato(&mut self) {
        self.vibrato_depth = 0.0;
    }

    /// Bend pitch by semitones
    pub fn bend(&mut self, semitones: f32) {
        let ratio = 2.0_f64.powf(semitones as f64 / 12.0);
        let current_freq = self.sample_rate as f64 / self.period;
        let new_freq = current_freq * ratio;
        self.period = self.sample_rate as f64 / new_freq;
    }

    /// Check if string is active
    pub fn is_active(&self) -> bool {
        self.amplitude > 0.001 || self.is_bowing
    }

    /// Bow-string friction model (simplified Friedlander)
    fn bow_friction(&mut self, string_velocity: f32) -> f32 {
        let relative_velocity = self.bow_velocity - string_velocity;

        // Stick-slip friction curve
        let friction = if relative_velocity.abs() < 0.01 {
            // Sticking regime
            relative_velocity * self.bow_pressure * 50.0
        } else {
            // Slipping regime (hyperbolic friction)
            let sign = relative_velocity.signum();
            let slip_force = self.bow_pressure * 0.3 / (1.0 + relative_velocity.abs() * 2.0);
            sign * slip_force
        };

        // Add some noise for rosin texture
        let noise = (self.bow_state * 12.345).sin() * 0.02 * self.bow_pressure;
        self.bow_state += 0.1;

        friction + noise
    }

    /// Generate next sample
    pub fn next_sample(&mut self) -> f32 {
        // Update amplitude (attack/release envelope)
        let amp_diff = self.target_amplitude - self.amplitude;
        self.amplitude += amp_diff * self.amp_slew * 100.0;

        if !self.is_active() {
            return 0.0;
        }

        // Apply vibrato
        let effective_period = if self.vibrato_depth > 0.0 {
            self.vibrato_phase += self.vibrato_rate as f64 / self.sample_rate as f64;
            if self.vibrato_phase >= 1.0 {
                self.vibrato_phase -= 1.0;
            }
            let vibrato_mod = (self.vibrato_phase * std::f64::consts::TAU).sin();
            let depth_ratio = 2.0_f64.powf(self.vibrato_depth as f64 * vibrato_mod / 12.0);
            self.period * depth_ratio
        } else {
            self.period
        };

        // Tremolo modulation (for tremolo articulation)
        let bow_mod = if self.articulation == StringArticulation::Tremolo {
            self.tremolo_phase += self.tremolo_rate as f64 / self.sample_rate as f64;
            if self.tremolo_phase >= 1.0 {
                self.tremolo_phase -= 1.0;
            }
            0.5 + 0.5 * (self.tremolo_phase * std::f64::consts::TAU).sin() as f32
        } else {
            1.0
        };

        let delay_len = self.delay_line.len();

        // Read from delay line with interpolation
        let read_offset = effective_period;
        let read_pos = (self.delay_pos as f64 + delay_len as f64 - read_offset)
            .rem_euclid(delay_len as f64);
        let read_int = read_pos as usize;
        let read_frac = read_pos - read_pos.floor();

        let sample_a = self.delay_line[read_int % delay_len];
        let sample_b = self.delay_line[(read_int + 1) % delay_len];
        let delayed = sample_a * (1.0 - read_frac as f32) + sample_b * read_frac as f32;

        // Bow excitation (only when bowing)
        let excitation = if self.is_bowing && self.articulation != StringArticulation::Pizzicato {
            let bow_force = self.bow_friction(delayed) * bow_mod;
            // Bow position affects harmonic content
            let pos_filter = 1.0 - self.bow_position * 0.5;
            bow_force * pos_filter * self.amplitude
        } else if self.articulation == StringArticulation::Pizzicato && self.amplitude > 0.5 {
            // Pizzicato: impulse excitation
            let impulse = self.amplitude * 0.5;
            self.amplitude *= 0.95; // Quick decay of excitation
            impulse
        } else {
            0.0
        };

        // String waveguide with damping
        let filtered = self.damping * self.prev_output + (1.0 - self.damping) * delayed;
        self.prev_output = filtered;

        // Write back with feedback and excitation
        let output = filtered * self.feedback + excitation;
        self.delay_line[self.delay_pos] = output.clamp(-1.0, 1.0);

        // Advance position
        self.delay_pos = (self.delay_pos + 1) % delay_len;

        output * self.amplitude
    }

    /// Reset string state
    pub fn reset(&mut self) {
        self.delay_line.fill(0.0);
        self.amplitude = 0.0;
        self.target_amplitude = 0.0;
        self.is_bowing = false;
        self.prev_output = 0.0;
        self.bow_state = 0.0;
    }
}

/// Body resonator for instrument character
#[derive(Debug, Clone)]
pub struct InstrumentBody {
    /// Resonance filters
    resonances: Vec<ResonanceFilter>,
    /// Instrument type
    instrument: StringInstrument,
    /// Output mix (dry/wet)
    mix: f32,
}

impl InstrumentBody {
    /// Create a new instrument body resonator
    pub fn new(instrument: StringInstrument, sample_rate: f32) -> Self {
        let resonances = instrument.body_resonances()
            .iter()
            .map(|&(freq, q, gain)| ResonanceFilter::new(freq, q, gain * 6.0, sample_rate))
            .collect();

        Self {
            resonances,
            instrument,
            mix: 0.7,
        }
    }

    /// Process a sample through the body
    pub fn process(&mut self, input: f32) -> f32 {
        let num_resonances = self.resonances.len() as f32;
        let mut output = input * (1.0 - self.mix);
        for resonance in &mut self.resonances {
            output += resonance.process(input) * self.mix / num_resonances;
        }
        output
    }

    /// Reset all filters
    pub fn reset(&mut self) {
        for r in &mut self.resonances {
            r.reset();
        }
    }
}

/// Complete string instrument synthesizer
pub struct StringSynth {
    /// Sample rate
    sample_rate: f32,
    /// Individual strings (typically 4 for orchestral strings)
    strings: Vec<BowedString>,
    /// Body resonator
    body: InstrumentBody,
    /// Instrument type
    instrument: StringInstrument,
    /// Master volume
    volume: f32,
    /// Current articulation (applied to all strings)
    articulation: StringArticulation,
}

impl StringSynth {
    /// Create a new string instrument synthesizer
    pub fn new(instrument: StringInstrument, sample_rate: u32) -> Self {
        let sr = sample_rate as f32;
        let num_strings = instrument.open_strings().len();

        let mut strings = Vec::with_capacity(num_strings);
        for _ in 0..num_strings {
            strings.push(BowedString::new(sr));
        }

        Self {
            sample_rate: sr,
            strings,
            body: InstrumentBody::new(instrument, sr),
            instrument,
            volume: 0.8,
            articulation: StringArticulation::Arco,
        }
    }

    /// Create a violin
    pub fn violin(sample_rate: u32) -> Self {
        Self::new(StringInstrument::Violin, sample_rate)
    }

    /// Create a viola
    pub fn viola(sample_rate: u32) -> Self {
        Self::new(StringInstrument::Viola, sample_rate)
    }

    /// Create a cello
    pub fn cello(sample_rate: u32) -> Self {
        Self::new(StringInstrument::Cello, sample_rate)
    }

    /// Create a double bass
    pub fn double_bass(sample_rate: u32) -> Self {
        Self::new(StringInstrument::DoubleBass, sample_rate)
    }

    /// Play a note (MIDI note number, velocity 0-1)
    pub fn note_on(&mut self, midi_note: u8, velocity: f32) {
        let frequency = 440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0);

        // Find best string for this note
        let open_strings = self.instrument.open_strings();
        let mut best_string = 0;
        let mut best_diff = 127i32;

        for (i, &open) in open_strings.iter().enumerate() {
            if midi_note >= open {
                let diff = (midi_note as i32 - open as i32).abs();
                if diff < best_diff {
                    best_diff = diff;
                    best_string = i;
                }
            }
        }

        // Apply articulation and bow the string
        self.strings[best_string].set_articulation(self.articulation);
        self.strings[best_string].bow(frequency, velocity);
    }

    /// Stop a note
    pub fn note_off(&mut self, midi_note: u8) {
        let frequency = 440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0);

        // Find which string is playing this note
        for string in &mut self.strings {
            let string_freq = self.sample_rate as f64 / string.period;
            if (string_freq - frequency as f64).abs() < 5.0 {
                string.release();
            }
        }
    }

    /// Stop all notes
    pub fn all_notes_off(&mut self) {
        for string in &mut self.strings {
            string.release();
        }
    }

    /// Set articulation for all strings
    pub fn set_articulation(&mut self, articulation: StringArticulation) {
        self.articulation = articulation;
        for string in &mut self.strings {
            string.set_articulation(articulation);
        }
    }

    /// Enable vibrato on all strings
    pub fn enable_vibrato(&mut self, depth: f32, rate: f32) {
        for string in &mut self.strings {
            string.set_vibrato(depth, rate);
        }
    }

    /// Disable vibrato
    pub fn disable_vibrato(&mut self) {
        for string in &mut self.strings {
            string.disable_vibrato();
        }
    }

    /// Set master volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Generate next mono sample
    pub fn next_sample(&mut self) -> f32 {
        let mut output = 0.0;
        for string in &mut self.strings {
            output += string.next_sample();
        }

        // Apply body resonance
        let with_body = self.body.process(output);

        with_body * self.volume
    }

    /// Generate next stereo sample
    pub fn next_sample_stereo(&mut self) -> (f32, f32) {
        let mono = self.next_sample();
        // Slight stereo spread
        (mono * 1.05, mono * 0.95)
    }

    /// Check if any string is active
    pub fn is_active(&self) -> bool {
        self.strings.iter().any(|s| s.is_active())
    }

    /// Reset all strings
    pub fn reset(&mut self) {
        for string in &mut self.strings {
            string.reset();
        }
        self.body.reset();
    }
}

/// String section (ensemble of instruments)
pub struct StringSection {
    /// Individual instruments in the section
    instruments: Vec<StringSynth>,
    /// Detune amounts for each instrument (in cents)
    detune: Vec<f32>,
    /// Pan positions for each instrument
    pan: Vec<f32>,
    /// Section volume
    volume: f32,
}

impl StringSection {
    /// Create a violin section (e.g., 4 violins)
    pub fn violins(count: usize, sample_rate: u32) -> Self {
        Self::new_section(StringInstrument::Violin, count, sample_rate)
    }

    /// Create a viola section
    pub fn violas(count: usize, sample_rate: u32) -> Self {
        Self::new_section(StringInstrument::Viola, count, sample_rate)
    }

    /// Create a cello section
    pub fn cellos(count: usize, sample_rate: u32) -> Self {
        Self::new_section(StringInstrument::Cello, count, sample_rate)
    }

    /// Create a bass section
    pub fn basses(count: usize, sample_rate: u32) -> Self {
        Self::new_section(StringInstrument::DoubleBass, count, sample_rate)
    }

    /// Create a section of instruments
    fn new_section(instrument: StringInstrument, count: usize, sample_rate: u32) -> Self {
        let mut instruments = Vec::with_capacity(count);
        let mut detune = Vec::with_capacity(count);
        let mut pan = Vec::with_capacity(count);

        for i in 0..count {
            instruments.push(StringSynth::new(instrument, sample_rate));

            // Spread detune: +/- 8 cents max
            let t = if count > 1 {
                i as f32 / (count - 1) as f32
            } else {
                0.5
            };
            detune.push((t - 0.5) * 16.0);

            // Spread pan: -0.5 to +0.5
            pan.push((t - 0.5) * 1.0);
        }

        Self {
            instruments,
            detune,
            pan,
            volume: 0.7,
        }
    }

    /// Play a note on all instruments in the section
    pub fn note_on(&mut self, midi_note: u8, velocity: f32) {
        for (i, instrument) in self.instruments.iter_mut().enumerate() {
            // Apply detune
            let detune_semitones = self.detune[i] / 100.0;
            let detuned_note = midi_note as f32 + detune_semitones;
            let frequency = 440.0 * 2.0_f32.powf((detuned_note - 69.0) / 12.0);

            // Slight velocity variation for realism
            let vel_variation = 1.0 + (i as f32 * 0.1).sin() * 0.1;
            let vel = (velocity * vel_variation).clamp(0.0, 1.0);

            // Find and bow the appropriate string
            let open_strings = instrument.instrument.open_strings();
            let mut best_string = 0;
            let mut best_diff = 127i32;

            for (j, &open) in open_strings.iter().enumerate() {
                if midi_note >= open {
                    let diff = (midi_note as i32 - open as i32).abs();
                    if diff < best_diff {
                        best_diff = diff;
                        best_string = j;
                    }
                }
            }

            instrument.strings[best_string].bow(frequency, vel);
        }
    }

    /// Stop a note on all instruments
    pub fn note_off(&mut self, midi_note: u8) {
        for instrument in &mut self.instruments {
            instrument.note_off(midi_note);
        }
    }

    /// Set articulation for all instruments
    pub fn set_articulation(&mut self, articulation: StringArticulation) {
        for instrument in &mut self.instruments {
            instrument.set_articulation(articulation);
        }
    }

    /// Enable vibrato on all instruments
    pub fn enable_vibrato(&mut self, depth: f32, rate: f32) {
        for instrument in &mut self.instruments {
            instrument.enable_vibrato(depth, rate);
        }
    }

    /// Generate next stereo sample
    pub fn next_sample_stereo(&mut self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for (i, instrument) in self.instruments.iter_mut().enumerate() {
            let mono = instrument.next_sample();
            let pan = self.pan[i];

            // Equal power panning
            let pan_angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
            left += mono * pan_angle.cos();
            right += mono * pan_angle.sin();
        }

        let scale = self.volume / (self.instruments.len() as f32).sqrt();
        (left * scale, right * scale)
    }

    /// Check if any instrument is active
    pub fn is_active(&self) -> bool {
        self.instruments.iter().any(|i| i.is_active())
    }

    /// Stop all notes
    pub fn all_notes_off(&mut self) {
        for instrument in &mut self.instruments {
            instrument.all_notes_off();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bowed_string() {
        let mut string = BowedString::new(44100.0);
        string.bow(440.0, 0.8);

        let mut samples = vec![0.0; 4410];
        for sample in &mut samples {
            *sample = string.next_sample();
        }

        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0, "Bowed string should produce output");
    }

    #[test]
    fn test_string_synth() {
        let mut violin = StringSynth::violin(44100);
        violin.note_on(69, 0.8); // A4

        let mut samples = vec![0.0; 4410];
        for sample in &mut samples {
            *sample = violin.next_sample();
        }

        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0, "Violin should produce output");
    }

    #[test]
    fn test_articulations() {
        let mut cello = StringSynth::cello(44100);

        for articulation in [
            StringArticulation::Arco,
            StringArticulation::Legato,
            StringArticulation::Staccato,
            StringArticulation::Pizzicato,
            StringArticulation::Tremolo,
        ] {
            cello.set_articulation(articulation);
            cello.note_on(48, 0.7); // C3

            // Generate some samples
            for _ in 0..1000 {
                cello.next_sample();
            }

            cello.all_notes_off();
            cello.reset();
        }
    }

    #[test]
    fn test_string_section() {
        let mut violins = StringSection::violins(4, 44100);
        violins.note_on(69, 0.7); // A4

        let mut left_sum = 0.0;
        let mut right_sum = 0.0;

        for _ in 0..4410 {
            let (l, r) = violins.next_sample_stereo();
            left_sum += l.abs();
            right_sum += r.abs();
        }

        assert!(left_sum > 0.0 && right_sum > 0.0, "Section should produce stereo output");
    }

    #[test]
    fn test_instruments() {
        // Test all instrument types
        for instrument in [
            StringInstrument::Violin,
            StringInstrument::Viola,
            StringInstrument::Cello,
            StringInstrument::DoubleBass,
        ] {
            let synth = StringSynth::new(instrument, 44100);
            assert!(!synth.strings.is_empty());
        }
    }
}
