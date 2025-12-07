/**
 * Common types used across the Maestro AI project
 */

export interface TimeSignature {
  numerator: number;
  denominator: number;
}

export type InstrumentType =
  | 'electric-guitar'
  | 'acoustic-guitar'
  | 'bass-guitar'
  | 'drums'
  | 'piano'
  | 'vocals'
  | 'synth'
  | 'strings'
  | 'brass'
  | 'woodwinds';

export type MaestroMode = 'compose' | 'record' | 'mix' | 'master' | 'practice';

export type CurveType = 'linear' | 'exponential' | 'logarithmic' | 'bezier';
