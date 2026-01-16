/**
 * Mixing types for mix-specific settings and AI suggestions
 */

import type { Plugin } from './session';

export interface MixingData {
  referenceTrack?: ReferenceTrack;
  aiSuggestions?: AIMixingSuggestions;
  snapshots: MixSnapshot[];
}

export interface ReferenceTrack {
  path: string;
  volume: number;
}

export interface AIMixingSuggestions {
  timestamp: string;
  genre?: string;
  suggestions: MixSuggestion[];
}

export interface MixSuggestion {
  trackId: string;
  type: 'eq' | 'compression' | 'reverb' | 'delay' | 'panning' | 'volume';
  description: string;
  parameters?: Record<string, any>;
  applied: boolean;
}

export interface MixSnapshot {
  id: string;
  name: string;
  timestamp: string;
  trackStates: Record<string, TrackState>;
  busStates: Record<string, BusState>;
}

export interface TrackState {
  volume: number;
  pan: number;
  muted: boolean;
  solo: boolean;
  plugins: Plugin[];
}

export interface BusState {
  volume: number;
  pan: number;
  plugins: Plugin[];
}
