/**
 * Mastering types for final mastering settings
 */

import type { Plugin } from './session';

export interface MasteringData {
  masteringChain: Plugin[];
  aiMastering?: AIMasteringSettings;
  loudness: LoudnessMetrics;
  exports: ExportFormat[];
}

export interface AIMasteringSettings {
  enabled: boolean;
  genre?: string;
  targetLoudness: number;
  targetPlatform?: 'spotify' | 'apple-music' | 'youtube' | 'cd' | 'custom';
  settings: Record<string, any>;
}

export interface LoudnessMetrics {
  integrated: number;
  shortTerm: number;
  momentary: number;
  truePeak: number;
  range: number;
}

export interface ExportFormat {
  name: string;
  format: 'wav' | 'mp3' | 'aac' | 'flac' | 'ogg';
  sampleRate: number;
  bitDepth?: number;
  bitrate?: number;
  path: string;
}
