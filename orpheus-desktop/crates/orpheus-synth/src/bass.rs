//! Bass synthesizer using subtractive synthesis
//!
//! Features:
//! - Multiple oscillator waveforms (saw, square, triangle, sine)
//! - Sub-oscillator for deep bass
//! - Resonant low-pass filter
//! - Filter envelope modulation
//! - Multiple bass presets

use crate::midi_to_freq;

/// Oscillator waveform type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
    /// Pulse with variable width
    Pulse(f32),
}

/// Bass synth preset
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BassPreset {
    /// Classic analog-style synth bass
    SynthBass,
    /// Sub bass - pure low end
    SubBass,
    /// Funk bass - bright and punchy
    FunkBass,
    /// 808-style sub bass
    Bass808,
    /// Wobble bass (dubstep style)
    WobbleBass,
    /// Moog-style bass
    MoogBass,
    /// Reese bass (detuned saws)
    ReeseBass,
}

/// Filter type for bass
#[derive(Debug, Clone, Copy)]
pub struct LowPassFilter {
    /// Cutoff frequency (Hz)
    cutoff: f32,
    /// Resonance (0.0 - 1.0)
    resonance: f32,
    /// Filter state variables
    low: f32,
    band: f32,
    high: f32,
}

impl Default for LowPassFilter {
    fn default() -> Self {
        Self {
            cutoff: 500.0,
            resonance: 0.3,
            low: 0.0,
            band: 0.0,
            high: 0.0,
        }
    }
}

impl LowPassFilter {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff,
            resonance: resonance.clamp(0.0, 0.99),
            ..Default::default()
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff.clamp(20.0, 20000.0);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(0.0, 0.99);
    }

    /// Process a sample (state variable filter)
    pub fn process(&mut self, input: f32, sample_rate: f32) -> f32 {
        // Calculate filter coefficients
        let f = 2.0 * (std::f32::consts::PI * self.cutoff / sample_rate).sin();
        let q = 1.0 - self.resonance;

        // State variable filter
        self.low += f * self.band;
        self.high = input - self.low - q * self.band;
        self.band += f * self.high;

        // Return lowpass output
        self.low
    }

    pub fn reset(&mut self) {
        self.low = 0.0;
        self.band = 0.0;
        self.high = 0.0;
    }
}

/// ADSR envelope
#[derive(Debug, Clone, Copy)]
pub struct BassEnvelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for BassEnvelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.1,
        }
    }
}

/// Single bass voice
pub struct BassVoice {
    sample_rate: u32,
    note: u8,
    frequency: f32,
    velocity: f32,

    // Oscillators
    osc1_waveform: Waveform,
    osc1_phase: f32,
    osc1_detune: f32, // cents

    osc2_waveform: Waveform,
    osc2_phase: f32,
    osc2_detune: f32,
    osc2_mix: f32, // 0.0 = osc1 only, 1.0 = equal mix

    // Sub oscillator (one octave down)
    sub_osc_enabled: bool,
    sub_osc_phase: f32,
    sub_osc_mix: f32,

    // Filter
    filter: LowPassFilter,
    filter_envelope: BassEnvelope,
    filter_env_amount: f32, // How much envelope affects cutoff

    // Amp envelope
    amp_envelope: BassEnvelope,

    // Envelope states
    amp_env_level: f32,
    filter_env_level: f32,
    env_stage: u8, // 0=attack, 1=decay, 2=sustain, 3=release, 4=off
    stage_time: f32,

    active: bool,
}

impl BassVoice {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            note: 0,
            frequency: 110.0,
            velocity: 0.8,

            osc1_waveform: Waveform::Saw,
            osc1_phase: 0.0,
            osc1_detune: 0.0,

            osc2_waveform: Waveform::Saw,
            osc2_phase: 0.0,
            osc2_detune: -10.0, // Slightly detuned for fatness
            osc2_mix: 0.5,

            sub_osc_enabled: true,
            sub_osc_phase: 0.0,
            sub_osc_mix: 0.3,

            filter: LowPassFilter::new(800.0, 0.3),
            filter_envelope: BassEnvelope {
                attack: 0.001,
                decay: 0.15,
                sustain: 0.2,
                release: 0.1,
            },
            filter_env_amount: 2000.0,

            amp_envelope: BassEnvelope::default(),
            amp_env_level: 0.0,
            filter_env_level: 0.0,
            env_stage: 4,
            stage_time: 0.0,

            active: false,
        }
    }

    /// Apply a preset
    pub fn apply_preset(&mut self, preset: BassPreset) {
        match preset {
            BassPreset::SynthBass => {
                self.osc1_waveform = Waveform::Saw;
                self.osc2_waveform = Waveform::Saw;
                self.osc2_detune = -10.0;
                self.osc2_mix = 0.5;
                self.sub_osc_enabled = true;
                self.sub_osc_mix = 0.3;
                self.filter.set_cutoff(800.0);
                self.filter.set_resonance(0.3);
                self.filter_env_amount = 2000.0;
            }
            BassPreset::SubBass => {
                self.osc1_waveform = Waveform::Sine;
                self.osc2_mix = 0.0;
                self.sub_osc_enabled = true;
                self.sub_osc_mix = 0.6;
                self.filter.set_cutoff(200.0);
                self.filter.set_resonance(0.0);
                self.filter_env_amount = 0.0;
            }
            BassPreset::FunkBass => {
                self.osc1_waveform = Waveform::Square;
                self.osc2_waveform = Waveform::Saw;
                self.osc2_detune = 0.0;
                self.osc2_mix = 0.3;
                self.sub_osc_enabled = false;
                self.filter.set_cutoff(1500.0);
                self.filter.set_resonance(0.5);
                self.filter_env_amount = 3000.0;
                self.filter_envelope.decay = 0.08;
            }
            BassPreset::Bass808 => {
                self.osc1_waveform = Waveform::Sine;
                self.osc2_mix = 0.0;
                self.sub_osc_enabled = true;
                self.sub_osc_mix = 0.8;
                self.filter.set_cutoff(150.0);
                self.filter.set_resonance(0.0);
                self.filter_env_amount = 100.0;
                self.amp_envelope.decay = 1.0;
                self.amp_envelope.sustain = 0.0;
                self.amp_envelope.release = 0.3;
            }
            BassPreset::WobbleBass => {
                self.osc1_waveform = Waveform::Saw;
                self.osc2_waveform = Waveform::Square;
                self.osc2_detune = 5.0;
                self.osc2_mix = 0.4;
                self.sub_osc_enabled = true;
                self.sub_osc_mix = 0.4;
                self.filter.set_cutoff(600.0);
                self.filter.set_resonance(0.6);
                self.filter_env_amount = 4000.0;
            }
            BassPreset::MoogBass => {
                self.osc1_waveform = Waveform::Saw;
                self.osc2_waveform = Waveform::Square;
                self.osc2_detune = -12.0; // One semitone down
                self.osc2_mix = 0.4;
                self.sub_osc_enabled = false;
                self.filter.set_cutoff(400.0);
                self.filter.set_resonance(0.5);
                self.filter_env_amount = 2500.0;
                self.filter_envelope.decay = 0.2;
            }
            BassPreset::ReeseBass => {
                self.osc1_waveform = Waveform::Saw;
                self.osc2_waveform = Waveform::Saw;
                self.osc2_detune = 15.0; // Wider detune for movement
                self.osc2_mix = 0.5;
                self.sub_osc_enabled = true;
                self.sub_osc_mix = 0.3;
                self.filter.set_cutoff(1000.0);
                self.filter.set_resonance(0.2);
                self.filter_env_amount = 1500.0;
            }
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        self.note = note;
        self.frequency = midi_to_freq(note);
        self.velocity = velocity.clamp(0.0, 1.0);

        // Reset phases
        self.osc1_phase = 0.0;
        self.osc2_phase = 0.0;
        self.sub_osc_phase = 0.0;

        // Reset filter
        self.filter.reset();

        // Start envelopes
        self.env_stage = 0; // Attack
        self.stage_time = 0.0;
        self.amp_env_level = 0.0;
        self.filter_env_level = 0.0;

        self.active = true;
    }

    pub fn note_off(&mut self) {
        if self.env_stage < 3 {
            self.env_stage = 3; // Release
            self.stage_time = 0.0;
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn current_note(&self) -> u8 {
        self.note
    }

    pub fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let dt = 1.0 / self.sample_rate as f32;

        // Update envelopes
        self.update_envelopes(dt);

        if !self.active {
            return 0.0;
        }

        // Generate oscillators
        let osc1 = self.generate_osc(1, self.frequency);

        let osc2 = if self.osc2_mix > 0.0 {
            self.generate_osc(2, self.frequency)
        } else {
            0.0
        };

        // Sub oscillator (one octave down)
        let sub = if self.sub_osc_enabled {
            let sub_freq = self.frequency * 0.5;
            self.sub_osc_phase += sub_freq / self.sample_rate as f32;
            if self.sub_osc_phase >= 1.0 {
                self.sub_osc_phase -= 1.0;
            }
            (self.sub_osc_phase * std::f32::consts::TAU).sin()
        } else {
            0.0
        };

        // Mix oscillators
        let osc1_amt = 1.0 - self.osc2_mix;
        let osc2_amt = self.osc2_mix;
        let mut sample = osc1 * osc1_amt + osc2 * osc2_amt + sub * self.sub_osc_mix;

        // Apply filter with envelope modulation
        let filter_cutoff = self.filter.cutoff + self.filter_env_level * self.filter_env_amount;
        let mut temp_filter = self.filter;
        temp_filter.set_cutoff(filter_cutoff);
        sample = temp_filter.process(sample, self.sample_rate as f32);
        self.filter = temp_filter;

        // Apply amplitude envelope
        sample *= self.amp_env_level * self.velocity;

        // Soft clip
        (sample * 1.5).tanh()
    }

    fn generate_osc(&mut self, osc_idx: u8, freq: f32) -> f32 {
        let (waveform, phase, detune) = match osc_idx {
            1 => (self.osc1_waveform, &mut self.osc1_phase, self.osc1_detune),
            2 => (self.osc2_waveform, &mut self.osc2_phase, self.osc2_detune),
            _ => return 0.0,
        };

        // Apply detune (cents to frequency ratio)
        let detune_ratio = 2.0_f32.powf(detune / 1200.0);
        let actual_freq = freq * detune_ratio;

        // Update phase
        *phase += actual_freq / self.sample_rate as f32;
        if *phase >= 1.0 {
            *phase -= 1.0;
        }

        Self::oscillator_sample(waveform, *phase)
    }

    fn oscillator_sample(waveform: Waveform, phase: f32) -> f32 {
        match waveform {
            Waveform::Sine => (phase * std::f32::consts::TAU).sin(),
            Waveform::Saw => 2.0 * phase - 1.0,
            Waveform::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Triangle => {
                if phase < 0.5 {
                    4.0 * phase - 1.0
                } else {
                    3.0 - 4.0 * phase
                }
            }
            Waveform::Pulse(width) => {
                if phase < width {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }

    fn update_envelopes(&mut self, dt: f32) {
        self.stage_time += dt;

        match self.env_stage {
            0 => {
                // Attack
                self.amp_env_level = self.stage_time / self.amp_envelope.attack;
                self.filter_env_level = self.stage_time / self.filter_envelope.attack;

                if self.amp_env_level >= 1.0 {
                    self.amp_env_level = 1.0;
                    self.filter_env_level = 1.0;
                    self.env_stage = 1;
                    self.stage_time = 0.0;
                }
            }
            1 => {
                // Decay
                let amp_progress = self.stage_time / self.amp_envelope.decay;
                let filter_progress = self.stage_time / self.filter_envelope.decay;

                self.amp_env_level = 1.0 - (1.0 - self.amp_envelope.sustain) * amp_progress.min(1.0);
                self.filter_env_level = 1.0 - (1.0 - self.filter_envelope.sustain) * filter_progress.min(1.0);

                if amp_progress >= 1.0 && filter_progress >= 1.0 {
                    self.env_stage = 2;
                }
            }
            2 => {
                // Sustain (hold)
                self.amp_env_level = self.amp_envelope.sustain;
                self.filter_env_level = self.filter_envelope.sustain;
            }
            3 => {
                // Release - exponential decay
                self.amp_env_level *= 1.0 - (dt / self.amp_envelope.release);
                self.filter_env_level *= 1.0 - (dt / self.filter_envelope.release);

                if self.amp_env_level < 0.001 {
                    self.active = false;
                    self.env_stage = 4;
                }
            }
            _ => {
                self.active = false;
            }
        }
    }
}

/// Polyphonic bass synthesizer
pub struct BassSynth {
    voices: Vec<BassVoice>,
    sample_rate: u32,
    master_volume: f32,
    current_preset: BassPreset,
}

impl BassSynth {
    /// Create a new bass synth
    pub fn new(sample_rate: u32, polyphony: usize) -> Self {
        let mut voices: Vec<BassVoice> = (0..polyphony)
            .map(|_| BassVoice::new(sample_rate))
            .collect();

        // Apply default preset
        for voice in &mut voices {
            voice.apply_preset(BassPreset::SynthBass);
        }

        Self {
            voices,
            sample_rate,
            master_volume: 0.8,
            current_preset: BassPreset::SynthBass,
        }
    }

    /// Create with standard 4-voice polyphony (bass usually monophonic)
    pub fn standard(sample_rate: u32) -> Self {
        Self::new(sample_rate, 4)
    }

    /// Set preset
    pub fn set_preset(&mut self, preset: BassPreset) {
        self.current_preset = preset;
        for voice in &mut self.voices {
            voice.apply_preset(preset);
        }
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let voice_idx = self.find_voice(note);
        self.voices[voice_idx].apply_preset(self.current_preset);
        self.voices[voice_idx].note_on(note, velocity);
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.is_active() && voice.current_note() == note {
                voice.note_off();
                break;
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            voice.note_off();
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut sample = 0.0;
        for voice in &mut self.voices {
            sample += voice.next_sample();
        }
        sample * self.master_volume
    }

    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
        }
    }

    fn find_voice(&self, note: u8) -> usize {
        // Mono-style: reuse same note
        for (i, voice) in self.voices.iter().enumerate() {
            if voice.is_active() && voice.current_note() == note {
                return i;
            }
        }

        // Find inactive voice
        for (i, voice) in self.voices.iter().enumerate() {
            if !voice.is_active() {
                return i;
            }
        }

        // Steal quietest
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bass_voice() {
        let mut voice = BassVoice::new(44100);
        voice.note_on(36, 0.8); // Low C

        assert!(voice.is_active());

        let mut samples = vec![0.0; 4410];
        for sample in &mut samples {
            *sample = voice.next_sample();
        }

        let max = samples.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0);
    }

    #[test]
    fn test_bass_presets() {
        let mut bass = BassSynth::standard(44100);

        // Test each preset
        for preset in [
            BassPreset::SynthBass,
            BassPreset::SubBass,
            BassPreset::FunkBass,
            BassPreset::Bass808,
        ] {
            bass.set_preset(preset);
            bass.note_on(36, 0.8);

            let mut buffer = vec![0.0; 1000];
            bass.fill_buffer(&mut buffer);

            let max = buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
            assert!(max > 0.0, "Preset {:?} produced no sound", preset);

            bass.all_notes_off();
        }
    }
}
