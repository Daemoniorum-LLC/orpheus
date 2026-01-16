/**
 * Instrument Library Service - Comprehensive sound library for playback
 * Provides Guitar Pro 8-style RSE (Realistic Sound Engine) instruments
 */

import * as Tone from 'tone';

/**
 * Instrument categories
 */
export type InstrumentCategory =
  | 'guitars'
  | 'basses'
  | 'keyboards'
  | 'strings'
  | 'brass'
  | 'woodwinds'
  | 'percussion'
  | 'synths'
  | 'world'
  | 'misc';

/**
 * Instrument definition
 */
export interface InstrumentDefinition {
  id: string;
  name: string;
  category: InstrumentCategory;
  subCategory?: string;
  midiProgram: number;
  defaultTuning?: number[]; // MIDI note numbers for open strings
  stringCount?: number;
  description?: string;
  tags?: string[];
  synthConfig: SynthConfig;
}

/**
 * Synth configuration for Tone.js
 */
export interface SynthConfig {
  type: 'synth' | 'fm' | 'am' | 'membrane' | 'metal' | 'pluck' | 'noise' | 'sampler';
  oscillator?: {
    type: OscillatorType | string;
    partials?: number[];
    partialCount?: number;
  };
  envelope?: {
    attack: number;
    decay: number;
    sustain: number;
    release: number;
  };
  filter?: {
    type: BiquadFilterType;
    frequency: number;
    Q?: number;
  };
  effects?: EffectConfig[];
  volume?: number;
}

type OscillatorType = 'sine' | 'square' | 'triangle' | 'sawtooth' | 'custom';

/**
 * Effect configuration
 */
export interface EffectConfig {
  type: 'reverb' | 'delay' | 'chorus' | 'distortion' | 'eq' | 'compressor' | 'phaser' | 'tremolo';
  params: Record<string, number | string | boolean>;
}

/**
 * Guitar Standard Tuning
 */
const STANDARD_GUITAR_TUNING = [64, 59, 55, 50, 45, 40]; // E4, B3, G3, D3, A2, E2

/**
 * Bass Standard Tuning
 */
const STANDARD_BASS_TUNING = [43, 38, 33, 28]; // G2, D2, A1, E1

/**
 * Comprehensive instrument library
 */
export const INSTRUMENT_LIBRARY: InstrumentDefinition[] = [
  // ====== GUITARS ======
  {
    id: 'acoustic-steel',
    name: 'Acoustic Steel String',
    category: 'guitars',
    subCategory: 'acoustic',
    midiProgram: 25,
    defaultTuning: STANDARD_GUITAR_TUNING,
    stringCount: 6,
    description: 'Bright steel-string acoustic guitar',
    tags: ['acoustic', 'folk', 'country'],
    synthConfig: {
      type: 'pluck',
      oscillator: { type: 'triangle' },
      envelope: { attack: 0.001, decay: 0.3, sustain: 0.1, release: 1.5 },
      effects: [
        { type: 'reverb', params: { decay: 1.5, wet: 0.2 } },
        { type: 'eq', params: { high: 3, mid: 0, low: -2 } },
      ],
      volume: -6,
    },
  },
  {
    id: 'acoustic-nylon',
    name: 'Classical Nylon String',
    category: 'guitars',
    subCategory: 'acoustic',
    midiProgram: 24,
    defaultTuning: STANDARD_GUITAR_TUNING,
    stringCount: 6,
    description: 'Warm nylon-string classical guitar',
    tags: ['classical', 'flamenco', 'fingerstyle'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sine' },
      envelope: { attack: 0.005, decay: 0.4, sustain: 0.2, release: 2 },
      filter: { type: 'lowpass', frequency: 2000, Q: 1 },
      effects: [
        { type: 'reverb', params: { decay: 2, wet: 0.25 } },
      ],
      volume: -8,
    },
  },
  {
    id: 'acoustic-12string',
    name: '12-String Acoustic',
    category: 'guitars',
    subCategory: 'acoustic',
    midiProgram: 25,
    defaultTuning: STANDARD_GUITAR_TUNING,
    stringCount: 12,
    description: 'Shimmering 12-string acoustic guitar',
    tags: ['acoustic', 'folk', 'jangle'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'triangle', partialCount: 8 },
      envelope: { attack: 0.002, decay: 0.3, sustain: 0.15, release: 2 },
      effects: [
        { type: 'chorus', params: { frequency: 1.5, delayTime: 3.5, depth: 0.7, wet: 0.5 } },
        { type: 'reverb', params: { decay: 2, wet: 0.3 } },
      ],
      volume: -6,
    },
  },
  {
    id: 'electric-clean',
    name: 'Electric Clean',
    category: 'guitars',
    subCategory: 'electric',
    midiProgram: 27,
    defaultTuning: STANDARD_GUITAR_TUNING,
    stringCount: 6,
    description: 'Clean electric guitar tone',
    tags: ['electric', 'clean', 'jazz'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'triangle' },
      envelope: { attack: 0.003, decay: 0.2, sustain: 0.4, release: 1 },
      effects: [
        { type: 'chorus', params: { frequency: 0.5, depth: 0.4, wet: 0.3 } },
        { type: 'reverb', params: { decay: 1.2, wet: 0.15 } },
      ],
      volume: -6,
    },
  },
  {
    id: 'electric-overdrive',
    name: 'Electric Overdrive',
    category: 'guitars',
    subCategory: 'electric',
    midiProgram: 29,
    defaultTuning: STANDARD_GUITAR_TUNING,
    stringCount: 6,
    description: 'Crunchy overdriven electric guitar',
    tags: ['electric', 'rock', 'blues'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sawtooth' },
      envelope: { attack: 0.005, decay: 0.15, sustain: 0.5, release: 0.8 },
      filter: { type: 'lowpass', frequency: 4000, Q: 2 },
      effects: [
        { type: 'distortion', params: { distortion: 0.4, wet: 0.8 } },
        { type: 'eq', params: { high: 2, mid: 4, low: 1 } },
        { type: 'reverb', params: { decay: 1, wet: 0.1 } },
      ],
      volume: -8,
    },
  },
  {
    id: 'electric-distortion',
    name: 'Electric Distortion',
    category: 'guitars',
    subCategory: 'electric',
    midiProgram: 30,
    defaultTuning: STANDARD_GUITAR_TUNING,
    stringCount: 6,
    description: 'Heavy distorted electric guitar',
    tags: ['electric', 'metal', 'hard rock'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'square' },
      envelope: { attack: 0.003, decay: 0.1, sustain: 0.6, release: 0.6 },
      filter: { type: 'lowpass', frequency: 5000, Q: 3 },
      effects: [
        { type: 'distortion', params: { distortion: 0.8, wet: 1 } },
        { type: 'eq', params: { high: 1, mid: 5, low: 3 } },
        { type: 'compressor', params: { threshold: -24, ratio: 6, attack: 0.003, release: 0.25 } },
        { type: 'reverb', params: { decay: 0.8, wet: 0.1 } },
      ],
      volume: -10,
    },
  },
  {
    id: 'electric-jazz',
    name: 'Electric Jazz',
    category: 'guitars',
    subCategory: 'electric',
    midiProgram: 26,
    defaultTuning: STANDARD_GUITAR_TUNING,
    stringCount: 6,
    description: 'Warm hollow-body jazz guitar',
    tags: ['electric', 'jazz', 'smooth'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sine' },
      envelope: { attack: 0.01, decay: 0.3, sustain: 0.5, release: 1.5 },
      filter: { type: 'lowpass', frequency: 1500, Q: 0.7 },
      effects: [
        { type: 'reverb', params: { decay: 2, wet: 0.25 } },
      ],
      volume: -6,
    },
  },

  // ====== BASSES ======
  {
    id: 'bass-electric',
    name: 'Electric Bass',
    category: 'basses',
    subCategory: 'electric',
    midiProgram: 33,
    defaultTuning: STANDARD_BASS_TUNING,
    stringCount: 4,
    description: 'Standard electric bass guitar',
    tags: ['bass', 'rock', 'pop'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sine' },
      envelope: { attack: 0.01, decay: 0.2, sustain: 0.6, release: 0.8 },
      filter: { type: 'lowpass', frequency: 800, Q: 1 },
      effects: [
        { type: 'compressor', params: { threshold: -20, ratio: 4, attack: 0.003, release: 0.15 } },
      ],
      volume: -4,
    },
  },
  {
    id: 'bass-5string',
    name: '5-String Bass',
    category: 'basses',
    subCategory: 'electric',
    midiProgram: 33,
    defaultTuning: [43, 38, 33, 28, 23], // G2, D2, A1, E1, B0
    stringCount: 5,
    description: 'Extended range 5-string bass',
    tags: ['bass', 'metal', 'progressive'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sine' },
      envelope: { attack: 0.008, decay: 0.25, sustain: 0.55, release: 0.9 },
      filter: { type: 'lowpass', frequency: 700, Q: 1.2 },
      volume: -4,
    },
  },
  {
    id: 'bass-slap',
    name: 'Slap Bass',
    category: 'basses',
    subCategory: 'electric',
    midiProgram: 36,
    defaultTuning: STANDARD_BASS_TUNING,
    stringCount: 4,
    description: 'Funky slap bass tone',
    tags: ['bass', 'funk', 'disco'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'triangle' },
      envelope: { attack: 0.001, decay: 0.1, sustain: 0.3, release: 0.4 },
      filter: { type: 'bandpass', frequency: 1200, Q: 2 },
      effects: [
        { type: 'compressor', params: { threshold: -18, ratio: 8, attack: 0.001, release: 0.1 } },
        { type: 'eq', params: { high: 5, mid: 2, low: 3 } },
      ],
      volume: -6,
    },
  },
  {
    id: 'bass-acoustic',
    name: 'Acoustic Bass',
    category: 'basses',
    subCategory: 'acoustic',
    midiProgram: 32,
    defaultTuning: STANDARD_BASS_TUNING,
    stringCount: 4,
    description: 'Upright/acoustic bass sound',
    tags: ['bass', 'jazz', 'acoustic'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sine' },
      envelope: { attack: 0.02, decay: 0.4, sustain: 0.4, release: 1.2 },
      filter: { type: 'lowpass', frequency: 600 },
      effects: [
        { type: 'reverb', params: { decay: 1.8, wet: 0.2 } },
      ],
      volume: -5,
    },
  },
  {
    id: 'bass-synth',
    name: 'Synth Bass',
    category: 'basses',
    subCategory: 'synth',
    midiProgram: 38,
    defaultTuning: STANDARD_BASS_TUNING,
    stringCount: 4,
    description: 'Electronic synth bass',
    tags: ['bass', 'electronic', 'synth'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sawtooth' },
      envelope: { attack: 0.005, decay: 0.2, sustain: 0.5, release: 0.5 },
      filter: { type: 'lowpass', frequency: 1000, Q: 4 },
      effects: [
        { type: 'distortion', params: { distortion: 0.2, wet: 0.4 } },
      ],
      volume: -6,
    },
  },

  // ====== KEYBOARDS ======
  {
    id: 'piano-grand',
    name: 'Grand Piano',
    category: 'keyboards',
    subCategory: 'acoustic',
    midiProgram: 0,
    description: 'Concert grand piano',
    tags: ['piano', 'classical', 'ballad'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'triangle', partialCount: 6 },
      envelope: { attack: 0.002, decay: 0.5, sustain: 0.3, release: 2.5 },
      effects: [
        { type: 'reverb', params: { decay: 2.5, wet: 0.25 } },
      ],
      volume: -6,
    },
  },
  {
    id: 'piano-electric',
    name: 'Electric Piano',
    category: 'keyboards',
    subCategory: 'electric',
    midiProgram: 4,
    description: 'Classic Rhodes-style electric piano',
    tags: ['piano', 'jazz', 'soul'],
    synthConfig: {
      type: 'fm',
      envelope: { attack: 0.001, decay: 0.4, sustain: 0.3, release: 2 },
      effects: [
        { type: 'tremolo', params: { frequency: 4, depth: 0.3 } },
        { type: 'reverb', params: { decay: 2, wet: 0.2 } },
      ],
      volume: -6,
    },
  },
  {
    id: 'organ-hammond',
    name: 'Hammond Organ',
    category: 'keyboards',
    subCategory: 'organ',
    midiProgram: 16,
    description: 'Classic B3 Hammond organ',
    tags: ['organ', 'blues', 'rock'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sine', partials: [1, 0.5, 0.3, 0.15, 0.08] },
      envelope: { attack: 0.005, decay: 0.1, sustain: 0.9, release: 0.3 },
      effects: [
        { type: 'chorus', params: { frequency: 1, depth: 0.5, wet: 0.4 } },
        { type: 'distortion', params: { distortion: 0.15, wet: 0.3 } },
        { type: 'reverb', params: { decay: 1.5, wet: 0.2 } },
      ],
      volume: -6,
    },
  },

  // ====== STRINGS ======
  {
    id: 'strings-ensemble',
    name: 'String Ensemble',
    category: 'strings',
    subCategory: 'ensemble',
    midiProgram: 48,
    description: 'Full orchestral strings',
    tags: ['strings', 'orchestral', 'cinematic'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sawtooth' },
      envelope: { attack: 0.3, decay: 0.5, sustain: 0.7, release: 1.5 },
      filter: { type: 'lowpass', frequency: 3000, Q: 0.5 },
      effects: [
        { type: 'chorus', params: { frequency: 0.3, depth: 0.5, wet: 0.4 } },
        { type: 'reverb', params: { decay: 3, wet: 0.35 } },
      ],
      volume: -8,
    },
  },
  {
    id: 'violin-solo',
    name: 'Solo Violin',
    category: 'strings',
    subCategory: 'solo',
    midiProgram: 40,
    description: 'Expressive solo violin',
    tags: ['violin', 'classical', 'solo'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sawtooth' },
      envelope: { attack: 0.1, decay: 0.2, sustain: 0.8, release: 0.8 },
      filter: { type: 'lowpass', frequency: 4000, Q: 1 },
      effects: [
        { type: 'reverb', params: { decay: 2, wet: 0.25 } },
      ],
      volume: -8,
    },
  },
  {
    id: 'cello-solo',
    name: 'Solo Cello',
    category: 'strings',
    subCategory: 'solo',
    midiProgram: 42,
    description: 'Rich solo cello',
    tags: ['cello', 'classical', 'solo'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sawtooth' },
      envelope: { attack: 0.15, decay: 0.3, sustain: 0.75, release: 1 },
      filter: { type: 'lowpass', frequency: 2000, Q: 0.8 },
      effects: [
        { type: 'reverb', params: { decay: 2.5, wet: 0.3 } },
      ],
      volume: -7,
    },
  },

  // ====== BRASS ======
  {
    id: 'trumpet',
    name: 'Trumpet',
    category: 'brass',
    subCategory: 'solo',
    midiProgram: 56,
    description: 'Bright brass trumpet',
    tags: ['brass', 'jazz', 'orchestral'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'square' },
      envelope: { attack: 0.05, decay: 0.15, sustain: 0.7, release: 0.3 },
      filter: { type: 'lowpass', frequency: 3500, Q: 2 },
      effects: [
        { type: 'reverb', params: { decay: 1.5, wet: 0.2 } },
      ],
      volume: -8,
    },
  },
  {
    id: 'brass-section',
    name: 'Brass Section',
    category: 'brass',
    subCategory: 'ensemble',
    midiProgram: 61,
    description: 'Full brass section',
    tags: ['brass', 'big band', 'orchestral'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'square', partialCount: 4 },
      envelope: { attack: 0.08, decay: 0.2, sustain: 0.65, release: 0.4 },
      filter: { type: 'lowpass', frequency: 3000, Q: 1.5 },
      effects: [
        { type: 'chorus', params: { frequency: 0.5, depth: 0.3, wet: 0.3 } },
        { type: 'reverb', params: { decay: 2, wet: 0.25 } },
      ],
      volume: -8,
    },
  },

  // ====== WOODWINDS ======
  {
    id: 'saxophone',
    name: 'Saxophone',
    category: 'woodwinds',
    subCategory: 'reeds',
    midiProgram: 65,
    description: 'Expressive alto saxophone',
    tags: ['sax', 'jazz', 'blues'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sawtooth' },
      envelope: { attack: 0.05, decay: 0.2, sustain: 0.6, release: 0.5 },
      filter: { type: 'lowpass', frequency: 2500, Q: 2 },
      effects: [
        { type: 'reverb', params: { decay: 1.5, wet: 0.2 } },
      ],
      volume: -8,
    },
  },
  {
    id: 'flute',
    name: 'Flute',
    category: 'woodwinds',
    subCategory: 'flutes',
    midiProgram: 73,
    description: 'Airy orchestral flute',
    tags: ['flute', 'classical', 'ambient'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sine' },
      envelope: { attack: 0.1, decay: 0.2, sustain: 0.7, release: 0.6 },
      effects: [
        { type: 'reverb', params: { decay: 2.5, wet: 0.35 } },
      ],
      volume: -10,
    },
  },

  // ====== PERCUSSION ======
  {
    id: 'drums-kit',
    name: 'Drum Kit',
    category: 'percussion',
    subCategory: 'kit',
    midiProgram: 0, // Channel 10
    description: 'Standard rock drum kit',
    tags: ['drums', 'rock', 'pop'],
    synthConfig: {
      type: 'membrane',
      envelope: { attack: 0.001, decay: 0.3, sustain: 0, release: 0.3 },
      volume: -6,
    },
  },
  {
    id: 'percussion-latin',
    name: 'Latin Percussion',
    category: 'percussion',
    subCategory: 'world',
    midiProgram: 0,
    description: 'Congas, bongos, shakers',
    tags: ['percussion', 'latin', 'world'],
    synthConfig: {
      type: 'membrane',
      envelope: { attack: 0.001, decay: 0.2, sustain: 0, release: 0.2 },
      volume: -8,
    },
  },

  // ====== SYNTHS ======
  {
    id: 'synth-lead',
    name: 'Synth Lead',
    category: 'synths',
    subCategory: 'lead',
    midiProgram: 80,
    description: 'Classic analog lead synth',
    tags: ['synth', 'electronic', 'lead'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sawtooth' },
      envelope: { attack: 0.01, decay: 0.1, sustain: 0.7, release: 0.5 },
      filter: { type: 'lowpass', frequency: 5000, Q: 3 },
      effects: [
        { type: 'delay', params: { delayTime: 0.25, feedback: 0.3, wet: 0.25 } },
        { type: 'reverb', params: { decay: 1.5, wet: 0.2 } },
      ],
      volume: -8,
    },
  },
  {
    id: 'synth-pad',
    name: 'Synth Pad',
    category: 'synths',
    subCategory: 'pad',
    midiProgram: 88,
    description: 'Ambient evolving pad',
    tags: ['synth', 'ambient', 'pad'],
    synthConfig: {
      type: 'synth',
      oscillator: { type: 'sine', partialCount: 8 },
      envelope: { attack: 0.5, decay: 0.5, sustain: 0.8, release: 3 },
      filter: { type: 'lowpass', frequency: 2000, Q: 0.5 },
      effects: [
        { type: 'chorus', params: { frequency: 0.2, depth: 0.8, wet: 0.5 } },
        { type: 'reverb', params: { decay: 4, wet: 0.5 } },
      ],
      volume: -10,
    },
  },

  // ====== WORLD ======
  {
    id: 'sitar',
    name: 'Sitar',
    category: 'world',
    subCategory: 'indian',
    midiProgram: 104,
    description: 'Traditional Indian sitar',
    tags: ['sitar', 'indian', 'world'],
    synthConfig: {
      type: 'pluck',
      oscillator: { type: 'triangle' },
      envelope: { attack: 0.001, decay: 0.5, sustain: 0.1, release: 2 },
      filter: { type: 'bandpass', frequency: 1500, Q: 3 },
      effects: [
        { type: 'reverb', params: { decay: 3, wet: 0.3 } },
      ],
      volume: -8,
    },
  },
  {
    id: 'ukulele',
    name: 'Ukulele',
    category: 'world',
    subCategory: 'plucked',
    midiProgram: 24,
    defaultTuning: [69, 64, 60, 67], // A4, E4, C4, G4
    stringCount: 4,
    description: 'Bright Hawaiian ukulele',
    tags: ['ukulele', 'acoustic', 'tropical'],
    synthConfig: {
      type: 'pluck',
      oscillator: { type: 'triangle' },
      envelope: { attack: 0.001, decay: 0.2, sustain: 0.1, release: 1 },
      filter: { type: 'highpass', frequency: 200 },
      effects: [
        { type: 'reverb', params: { decay: 1, wet: 0.15 } },
      ],
      volume: -6,
    },
  },
  {
    id: 'banjo',
    name: 'Banjo',
    category: 'world',
    subCategory: 'plucked',
    midiProgram: 105,
    defaultTuning: [62, 59, 55, 50, 67], // D4, B3, G3, D3, G4 (5-string)
    stringCount: 5,
    description: 'Bright 5-string banjo',
    tags: ['banjo', 'bluegrass', 'country'],
    synthConfig: {
      type: 'pluck',
      oscillator: { type: 'triangle' },
      envelope: { attack: 0.001, decay: 0.15, sustain: 0.05, release: 0.8 },
      filter: { type: 'highpass', frequency: 400 },
      effects: [
        { type: 'eq', params: { high: 5, mid: 2, low: -3 } },
        { type: 'reverb', params: { decay: 0.8, wet: 0.1 } },
      ],
      volume: -6,
    },
  },
  {
    id: 'mandolin',
    name: 'Mandolin',
    category: 'world',
    subCategory: 'plucked',
    midiProgram: 25,
    defaultTuning: [76, 69, 62, 55], // E5, A4, D4, G3 (pairs)
    stringCount: 8,
    description: 'Bright mandolin with tremolo',
    tags: ['mandolin', 'folk', 'bluegrass'],
    synthConfig: {
      type: 'pluck',
      oscillator: { type: 'triangle' },
      envelope: { attack: 0.001, decay: 0.1, sustain: 0.1, release: 0.6 },
      effects: [
        { type: 'chorus', params: { frequency: 6, depth: 0.4, wet: 0.4 } },
        { type: 'reverb', params: { decay: 1.2, wet: 0.15 } },
      ],
      volume: -6,
    },
  },
];

/**
 * Get instrument by ID
 */
export function getInstrument(id: string): InstrumentDefinition | undefined {
  return INSTRUMENT_LIBRARY.find((i) => i.id === id);
}

/**
 * Get instruments by category
 */
export function getInstrumentsByCategory(category: InstrumentCategory): InstrumentDefinition[] {
  return INSTRUMENT_LIBRARY.filter((i) => i.category === category);
}

/**
 * Search instruments by name or tags
 */
export function searchInstruments(query: string): InstrumentDefinition[] {
  const lowerQuery = query.toLowerCase();
  return INSTRUMENT_LIBRARY.filter(
    (i) =>
      i.name.toLowerCase().includes(lowerQuery) ||
      i.description?.toLowerCase().includes(lowerQuery) ||
      i.tags?.some((t) => t.toLowerCase().includes(lowerQuery))
  );
}

/**
 * Get all categories with instrument counts
 */
export function getCategories(): { category: InstrumentCategory; count: number; name: string }[] {
  const categoryNames: Record<InstrumentCategory, string> = {
    guitars: 'Guitars',
    basses: 'Basses',
    keyboards: 'Keyboards',
    strings: 'Strings',
    brass: 'Brass',
    woodwinds: 'Woodwinds',
    percussion: 'Percussion',
    synths: 'Synthesizers',
    world: 'World Instruments',
    misc: 'Miscellaneous',
  };

  const counts = new Map<InstrumentCategory, number>();
  INSTRUMENT_LIBRARY.forEach((i) => {
    counts.set(i.category, (counts.get(i.category) || 0) + 1);
  });

  return Array.from(counts.entries()).map(([category, count]) => ({
    category,
    count,
    name: categoryNames[category],
  }));
}

/**
 * Create Tone.js synth from instrument definition
 */
export function createSynthFromInstrument(instrument: InstrumentDefinition): Tone.PolySynth | Tone.Synth {
  const config = instrument.synthConfig;
  let synth: Tone.PolySynth | Tone.Synth;

  switch (config.type) {
    case 'fm':
      synth = new Tone.PolySynth(Tone.FMSynth);
      break;
    case 'am':
      synth = new Tone.PolySynth(Tone.AMSynth);
      break;
    case 'membrane':
      synth = new Tone.PolySynth(Tone.MembraneSynth);
      break;
    case 'metal':
      synth = new Tone.PolySynth(Tone.MetalSynth);
      break;
    case 'pluck':
      synth = new Tone.PolySynth(Tone.PluckSynth);
      break;
    case 'synth':
    default:
      synth = new Tone.PolySynth(Tone.Synth, {
        oscillator: config.oscillator as Tone.OmniOscillatorOptions,
        envelope: config.envelope,
      });
  }

  // Apply effects chain
  let currentNode: Tone.ToneAudioNode = synth;

  if (config.effects) {
    config.effects.forEach((effectConfig) => {
      const effect = createEffect(effectConfig);
      if (effect) {
        currentNode.connect(effect);
        currentNode = effect;
      }
    });
  }

  // Connect to destination
  currentNode.toDestination();

  // Set volume
  if (config.volume !== undefined) {
    synth.volume.value = config.volume;
  }

  return synth;
}

/**
 * Create Tone.js effect from config
 */
function createEffect(config: EffectConfig): Tone.ToneAudioNode | null {
  const { type, params } = config;

  switch (type) {
    case 'reverb':
      return new Tone.Reverb({
        decay: params.decay as number,
        wet: params.wet as number,
      });
    case 'delay':
      return new Tone.FeedbackDelay({
        delayTime: params.delayTime as number,
        feedback: params.feedback as number,
        wet: params.wet as number,
      });
    case 'chorus':
      return new Tone.Chorus({
        frequency: params.frequency as number,
        delayTime: params.delayTime as number,
        depth: params.depth as number,
        wet: params.wet as number,
      });
    case 'distortion':
      return new Tone.Distortion({
        distortion: params.distortion as number,
        wet: params.wet as number,
      });
    case 'phaser':
      return new Tone.Phaser({
        frequency: params.frequency as number,
        octaves: params.octaves as number,
        wet: params.wet as number,
      });
    case 'tremolo':
      return new Tone.Tremolo({
        frequency: params.frequency as number,
        depth: params.depth as number,
      }).start();
    case 'compressor':
      return new Tone.Compressor({
        threshold: params.threshold as number,
        ratio: params.ratio as number,
        attack: params.attack as number,
        release: params.release as number,
      });
    case 'eq':
      return new Tone.EQ3({
        high: params.high as number,
        mid: params.mid as number,
        low: params.low as number,
      });
    default:
      return null;
  }
}

/**
 * Get default instrument for a track type
 */
export function getDefaultInstrument(trackType: string): InstrumentDefinition {
  const typeMap: Record<string, string> = {
    guitar: 'electric-clean',
    'acoustic-guitar': 'acoustic-steel',
    'electric-guitar': 'electric-clean',
    bass: 'bass-electric',
    'electric-bass': 'bass-electric',
    piano: 'piano-grand',
    keyboard: 'piano-electric',
    drums: 'drums-kit',
    strings: 'strings-ensemble',
    default: 'acoustic-steel',
  };

  const instrumentId = typeMap[trackType.toLowerCase()] || typeMap.default;
  return getInstrument(instrumentId) || INSTRUMENT_LIBRARY[0];
}
