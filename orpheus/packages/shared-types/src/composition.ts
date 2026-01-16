/**
 * Composition types for tablature, notation, and scores
 */

import type { TimeSignature, InstrumentType } from './common';

export interface CompositionData {
  scores: Score[];
  tempoMap: TempoChange[];
  markers: Marker[];
  songStructure?: SongStructure;
  // Simplified structure for basic use cases (e.g., Guitar Pro import)
  tracks?: NotationTrack[];
  measures?: Measure[];
}

export interface Score {
  id: string;
  title: string;
  instrument: InstrumentType;
  tuning?: string[];
  capo?: number;
  tracks: NotationTrack[];
  guitarProVersion?: string;
  guitarProData?: any;
}

export interface NotationTrack {
  id: string;
  name: string;
  voiceCount: number;
  measures: Measure[];
}

export interface Measure {
  number: number;
  timeSignature?: TimeSignature;
  voices: Voice[];
}

export interface Voice {
  voiceIndex: number;
  beats: Beat[];
}

export interface Beat {
  startTime: number;
  duration: number;
  notes: Note[];
  rest?: boolean;
  tuplet?: TupletInfo;
}

/**
 * Tuplet information supporting nested tuplets (tuplets within tuplets)
 * Example: A triplet containing another triplet would have parent/children
 */
export interface TupletInfo {
  /** Number of notes in this tuplet (e.g., 3 for triplet) */
  actual: number;
  /** Number of normal notes this tuplet replaces (e.g., 2 for triplet) */
  normal: number;
  /** Tuplet bracket type */
  bracket?: 'start' | 'stop' | 'continue' | 'none';
  /** Whether to show the tuplet number */
  showNumber?: boolean;
  /** Whether to show the tuplet bracket */
  showBracket?: boolean;
  /** Nested tuplets (for tuplets within tuplets) */
  nested?: TupletInfo;
  /** Depth level for nested tuplets (0 = root, 1 = first level nested, etc.) */
  depth?: number;
  /** Unique ID for tuplet grouping */
  groupId?: string;
}

export interface Note {
  string: number;
  fret: number;
  velocity?: number;
  techniques?: NoteTechniques;
}

export interface NoteTechniques {
  bend?: BendTechnique;
  slide?: SlideTechnique;
  hammer?: boolean;
  pull?: boolean;
  harmonic?: HarmonicType;
  palmMute?: boolean;
  letRing?: boolean;
  staccato?: boolean;
  vibrato?: boolean;
  trill?: { fret: number };
}

export interface BendTechnique {
  type: 'bend' | 'prebend' | 'release';
  value: number;
  points?: BendPoint[];
}

export interface BendPoint {
  position: number;
  value: number;
}

export interface SlideTechnique {
  type: 'slideInFromAbove' | 'slideInFromBelow' | 'slideOutUp' | 'slideOutDown' | 'shiftSlide' | 'legatoSlide';
  targetFret?: number;
}

export type HarmonicType = 'natural' | 'artificial' | 'pinch' | 'tap';

export interface TempoChange {
  measureNumber: number;
  tempo: number;
}

export interface Marker {
  id: string;
  name: string;
  measureNumber: number;
  type: 'section' | 'rehearsal' | 'custom';
  color?: string;
}

export interface SongStructure {
  sections: Section[];
}

export interface Section {
  name: string;
  startMeasure: number;
  endMeasure: number;
  color?: string;
}
