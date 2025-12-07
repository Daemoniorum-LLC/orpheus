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
  tuplet?: {
    actual: number;
    normal: number;
  };
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
