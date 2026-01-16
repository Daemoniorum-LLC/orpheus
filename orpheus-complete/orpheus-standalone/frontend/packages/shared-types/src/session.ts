/**
 * Session types for audio recording and MIDI
 */

import type { CurveType } from './common';

export interface SessionData {
  sampleRate: number;
  bitDepth: number;
  bufferSize: number;
  tracks: AudioTrack[];
  buses: Bus[];
  sends: Send[];
  timeline: Timeline;
}

export interface Timeline {
  viewStart: number;
  viewEnd: number;
  loop?: {
    enabled: boolean;
    start: number;
    end: number;
  };
}

export interface AudioTrack {
  id: string;
  name: string;
  type: 'audio' | 'midi' | 'instrument' | 'aux';
  linkedScoreId?: string;
  muted: boolean;
  solo: boolean;
  armed: boolean;
  monitorMode: 'off' | 'input' | 'auto';
  inputSource?: string;
  outputBus: string;
  regions: Region[];
  volume: number;
  pan: number;
  plugins: Plugin[];
  automation: Automation[];
  color?: string;
}

export interface Region {
  id: string;
  name: string;
  startTime: number;
  duration: number;
  audioFile?: AudioFileReference;
  midiData?: MidiData;
  fadeIn?: number;
  fadeOut?: number;
  gain: number;
}

export interface AudioFileReference {
  path: string;
  sampleRate: number;
  channels: number;
  offset: number;
  timeStretch?: number;
  pitchShift?: number;
}

export interface MidiData {
  notes: MidiNote[];
  controlChanges?: MidiCC[];
}

export interface MidiNote {
  startTime: number;
  duration: number;
  pitch: number;
  velocity: number;
  channel: number;
}

export interface MidiCC {
  time: number;
  controller: number;
  value: number;
  channel: number;
}

export interface Bus {
  id: string;
  name: string;
  type: 'master' | 'aux' | 'group';
  volume: number;
  pan: number;
  plugins: Plugin[];
  automation: Automation[];
  outputBus?: string;
}

export interface Send {
  id: string;
  sourceTrackId: string;
  destinationBusId: string;
  amount: number;
  preFader: boolean;
}

export interface Plugin {
  id: string;
  name: string;
  format: 'vst' | 'vst3' | 'au' | 'aax' | 'builtin';
  path?: string;
  enabled: boolean;
  state?: string;
  parameters?: Record<string, number>;
}

export interface Automation {
  parameterId: string;
  points: AutomationPoint[];
  mode: 'latch' | 'touch' | 'write' | 'read';
}

export interface AutomationPoint {
  time: number;
  value: number;
  curve?: CurveType;
}
