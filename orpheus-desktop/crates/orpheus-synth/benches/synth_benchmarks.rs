//! Performance benchmarks for Orpheus synth engines
//!
//! Run with: cargo bench -p orpheus-synth

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use orpheus_synth::{
    KarplusStrong, GuitarSynth, GuitarConfig,
    PianoSynth,
    SAMPLE_RATE_48000,
};

const BUFFER_SIZES: [usize; 4] = [64, 128, 256, 512];
const SAMPLE_RATE: u32 = SAMPLE_RATE_48000;

fn bench_karplus_strong(c: &mut Criterion) {
    let mut group = c.benchmark_group("karplus_strong");

    for size in BUFFER_SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("fill_buffer", size),
            &size,
            |b, &size| {
                let mut ks = KarplusStrong::new(SAMPLE_RATE);
                ks.pluck(440.0);
                let mut buffer = vec![0.0f32; size];

                b.iter(|| {
                    ks.fill_buffer(black_box(&mut buffer));
                    black_box(&buffer);
                });
            },
        );
    }

    // Benchmark 1 second of audio generation
    group.throughput(Throughput::Elements(SAMPLE_RATE as u64));
    group.bench_function("one_second", |b| {
        let mut ks = KarplusStrong::new(SAMPLE_RATE);
        ks.pluck(440.0);
        let mut buffer = vec![0.0f32; SAMPLE_RATE as usize];

        b.iter(|| {
            ks.pluck(440.0); // Re-pluck each iteration for consistent state
            ks.fill_buffer(black_box(&mut buffer));
            black_box(&buffer);
        });
    });

    group.finish();
}

fn bench_guitar_synth(c: &mut Criterion) {
    let mut group = c.benchmark_group("guitar_synth");

    // Single string pluck - stereo interleaved buffer
    group.throughput(Throughput::Elements(SAMPLE_RATE as u64));
    group.bench_function("single_string_1s", |b| {
        let mut guitar = GuitarSynth::new(GuitarConfig::default());
        let mut stereo = vec![0.0f32; (SAMPLE_RATE as usize) * 2]; // Interleaved L/R

        b.iter(|| {
            guitar.pluck(1, 5, 0.8);
            guitar.fill_buffer_stereo(black_box(&mut stereo));
            black_box(&stereo);
        });
    });

    // Chord (6 strings)
    group.bench_function("chord_1s", |b| {
        let mut guitar = GuitarSynth::new(GuitarConfig::default());
        let mut stereo = vec![0.0f32; (SAMPLE_RATE as usize) * 2];

        b.iter(|| {
            // E major chord
            guitar.pluck(1, 0, 0.8); // E
            guitar.pluck(2, 2, 0.8); // B
            guitar.pluck(3, 2, 0.8); // E
            guitar.pluck(4, 1, 0.8); // G#
            guitar.pluck(5, 0, 0.8); // B
            guitar.pluck(6, 0, 0.8); // E
            guitar.fill_buffer_stereo(black_box(&mut stereo));
            black_box(&stereo);
        });
    });

    group.finish();
}

fn bench_piano(c: &mut Criterion) {
    let mut group = c.benchmark_group("piano_synth");
    const POLYPHONY: usize = 16;

    group.throughput(Throughput::Elements(SAMPLE_RATE as u64));
    group.bench_function("single_note_1s", |b| {
        let mut piano = PianoSynth::new(SAMPLE_RATE, POLYPHONY);
        let mut buffer = vec![0.0f32; SAMPLE_RATE as usize];

        b.iter(|| {
            piano.note_on(60, 0.8); // Middle C with velocity 0.8
            piano.fill_buffer(black_box(&mut buffer));
            piano.note_off(60);
            black_box(&buffer);
        });
    });

    group.bench_function("chord_1s", |b| {
        let mut piano = PianoSynth::new(SAMPLE_RATE, POLYPHONY);
        let mut buffer = vec![0.0f32; SAMPLE_RATE as usize];

        b.iter(|| {
            // C major chord
            piano.note_on(60, 0.8); // C
            piano.note_on(64, 0.8); // E
            piano.note_on(67, 0.8); // G
            piano.fill_buffer(black_box(&mut buffer));
            piano.note_off(60);
            piano.note_off(64);
            piano.note_off(67);
            black_box(&buffer);
        });
    });

    group.finish();
}

fn bench_next_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("next_sample");

    // Benchmark individual sample generation (the hot path)
    group.throughput(Throughput::Elements(1));
    group.bench_function("karplus_strong", |b| {
        let mut ks = KarplusStrong::new(SAMPLE_RATE);
        ks.pluck(440.0);

        b.iter(|| {
            black_box(ks.next_sample());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_karplus_strong,
    bench_guitar_synth,
    bench_piano,
    bench_next_sample,
);

criterion_main!(benches);
