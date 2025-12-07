/*!
 * Raw FFI bindings to C++ audio engine
 */

use std::os::raw::c_char;

#[repr(C)]
pub struct AudioBufferFFI {
    pub data: *mut f32,
    pub num_channels: i32,
    pub num_samples: i32,
    pub sample_rate: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EQBand {
    pub frequency: f32,
    pub gain: f32,
    pub q: f32,
}

impl EQBand {
    pub fn new(frequency: f32, gain: f32, q: f32) -> Self {
        Self { frequency, gain, q }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompressorParams {
    pub threshold: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub knee: f32,
    pub makeup_gain: f32,
}

impl CompressorParams {
    pub fn new(
        threshold: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        knee: f32,
        makeup_gain: f32,
    ) -> Self {
        Self {
            threshold,
            ratio,
            attack_ms,
            release_ms,
            knee,
            makeup_gain,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ReverbParams {
    pub room_size: f32,
    pub decay: f32,
    pub pre_delay: f32,
    pub wet_dry: f32,
}

impl ReverbParams {
    pub fn new(room_size: f32, decay: f32, pre_delay: f32, wet_dry: f32) -> Self {
        Self {
            room_size,
            decay,
            pre_delay,
            wet_dry,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DelayParams {
    pub time_ms: f32,
    pub feedback: f32,
    pub wet_dry: f32,
}

impl DelayParams {
    pub fn new(time_ms: f32, feedback: f32, wet_dry: f32) -> Self {
        Self {
            time_ms,
            feedback,
            wet_dry,
        }
    }
}

extern "C" {
    pub fn maestro_audio_engine_new(sample_rate: i32, buffer_size: i32) -> *mut libc::c_void;
    pub fn maestro_audio_engine_delete(engine: *mut libc::c_void);
    pub fn maestro_audio_engine_process(
        engine: *mut libc::c_void,
        input: *const AudioBufferFFI,
        output: *mut AudioBufferFFI,
    );

    pub fn maestro_audio_engine_set_eq_band(
        engine: *mut libc::c_void,
        band: i32,
        frequency: f32,
        gain: f32,
        q: f32,
    );
    pub fn maestro_audio_engine_set_eq_enabled(engine: *mut libc::c_void, enabled: bool);

    pub fn maestro_audio_engine_set_compressor(
        engine: *mut libc::c_void,
        threshold: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        knee: f32,
        makeup_gain: f32,
    );
    pub fn maestro_audio_engine_set_compressor_enabled(engine: *mut libc::c_void, enabled: bool);

    pub fn maestro_audio_engine_set_reverb(
        engine: *mut libc::c_void,
        room_size: f32,
        decay: f32,
        pre_delay: f32,
        wet_dry: f32,
    );
    pub fn maestro_audio_engine_set_reverb_enabled(engine: *mut libc::c_void, enabled: bool);

    pub fn maestro_audio_engine_set_delay(
        engine: *mut libc::c_void,
        time_ms: f32,
        feedback: f32,
        wet_dry: f32,
    );
    pub fn maestro_audio_engine_set_delay_enabled(engine: *mut libc::c_void, enabled: bool);

    pub fn maestro_audio_engine_get_latency(engine: *mut libc::c_void) -> i32;
    pub fn maestro_audio_engine_load_plugin(
        engine: *mut libc::c_void,
        plugin_path: *const c_char,
    ) -> bool;
    pub fn maestro_audio_engine_reset(engine: *mut libc::c_void);
}
