/**
 * Unified Timeline - The core synchronization system
 */

import type { TimeSignature } from '@orpheus/shared-types';
import { TempoMap } from './tempo-map';
import { TimeConverter } from './time-converter';
import { MarkerManager } from './marker-manager';
import { Playhead } from './playhead';

export interface TimelineConfig {
  initialTempo: number;
  timeSignature: TimeSignature;
  sampleRate: number;
}

export interface MusicalPosition {
  measure: number;
  beat: number;
  tick: number;        // Sub-beat precision (480 ticks per beat)
  totalBeats: number;  // Absolute beat count from start
}

export interface AbsolutePosition {
  seconds: number;
  samples: number;
  frames: number;      // For video sync (30fps, 60fps, etc.)
}

/**
 * The Timeline is the central synchronization point for all time-based operations
 * It maintains the relationship between musical time and absolute time
 */
export class Timeline {
  private tempoMap: TempoMap;
  private converter: TimeConverter;
  private markers: MarkerManager;
  private playhead: Playhead;
  private sampleRate: number;
  private timeSignature: TimeSignature;

  constructor(config: TimelineConfig) {
    this.sampleRate = config.sampleRate;
    this.timeSignature = config.timeSignature;

    this.tempoMap = new TempoMap(config.initialTempo);
    this.converter = new TimeConverter(this.tempoMap, config.timeSignature);
    this.markers = new MarkerManager();
    this.playhead = new Playhead();
  }

  // ========== Position Conversion ==========

  /**
   * Converts musical position (measure/beat) to absolute time (seconds)
   */
  musicalToAbsolute(position: MusicalPosition): AbsolutePosition {
    const seconds = this.converter.musicalToSeconds(position);
    const samples = Math.round(seconds * this.sampleRate);
    const frames = Math.round(seconds * 30); // Assuming 30fps

    return { seconds, samples, frames };
  }

  /**
   * Converts absolute time (seconds) to musical position (measure/beat)
   */
  absoluteToMusical(position: AbsolutePosition): MusicalPosition {
    return this.converter.secondsToMusical(position.seconds);
  }

  /**
   * Snaps a position to the nearest grid division
   */
  snapToGrid(
    position: MusicalPosition,
    gridSize: number // 1 = quarter, 0.5 = eighth, 0.25 = sixteenth
  ): MusicalPosition {
    const totalBeats = position.totalBeats;
    const snappedBeats = Math.round(totalBeats / gridSize) * gridSize;

    return this.converter.beatsToMusical(snappedBeats);
  }

  // ========== Playhead Control ==========

  /**
   * Sets the playhead position
   */
  setPosition(position: MusicalPosition | AbsolutePosition): void {
    if ('measure' in position) {
      const absolute = this.musicalToAbsolute(position);
      this.playhead.setPosition(absolute.seconds);
    } else {
      this.playhead.setPosition(position.seconds);
    }
  }

  /**
   * Gets the current playhead position in both formats
   */
  getPosition(): { musical: MusicalPosition; absolute: AbsolutePosition } {
    const seconds = this.playhead.getPosition();
    const absolute: AbsolutePosition = {
      seconds,
      samples: Math.round(seconds * this.sampleRate),
      frames: Math.round(seconds * 30),
    };
    const musical = this.absoluteToMusical(absolute);

    return { musical, absolute };
  }

  /**
   * Starts playback
   */
  play(): void {
    this.playhead.play();
  }

  /**
   * Pauses playback
   */
  pause(): void {
    this.playhead.pause();
  }

  /**
   * Stops playback and returns to start
   */
  stop(): void {
    this.playhead.stop();
    this.setPosition({ measure: 0, beat: 0, tick: 0, totalBeats: 0 });
  }

  /**
   * Checks if currently playing
   */
  isPlaying(): boolean {
    return this.playhead.isPlaying();
  }

  // ========== Tempo Management ==========

  /**
   * Sets the tempo at a specific position
   */
  setTempoAt(position: MusicalPosition, bpm: number): void {
    this.tempoMap.setTempoAt(position.totalBeats, bpm);
    this.converter.invalidateCache(); // Recalculate conversions
  }

  /**
   * Gets the tempo at a specific position
   */
  getTempoAt(position: MusicalPosition): number {
    return this.tempoMap.getTempoAt(position.totalBeats);
  }

  /**
   * Sets a global tempo (removes all tempo changes)
   */
  setGlobalTempo(bpm: number): void {
    this.tempoMap.setGlobalTempo(bpm);
    this.converter.invalidateCache();
  }

  /**
   * Gets the current tempo at playhead
   */
  getCurrentTempo(): number {
    const position = this.getPosition();
    return this.getTempoAt(position.musical);
  }

  // ========== Time Signature ==========

  /**
   * Sets the time signature
   */
  setTimeSignature(timeSignature: TimeSignature): void {
    this.timeSignature = timeSignature;
    this.converter.setTimeSignature(timeSignature);
  }

  /**
   * Gets the current time signature
   */
  getTimeSignature(): TimeSignature {
    return this.timeSignature;
  }

  // ========== Markers ==========

  /**
   * Adds a marker at a position
   */
  addMarker(position: MusicalPosition, name: string, type: 'section' | 'rehearsal' | 'custom' = 'custom'): string {
    return this.markers.addMarker({
      position,
      name,
      type,
      color: type === 'section' ? '#3b82f6' : type === 'rehearsal' ? '#10b981' : '#6b7280',
    });
  }

  /**
   * Removes a marker
   */
  removeMarker(id: string): void {
    this.markers.removeMarker(id);
  }

  /**
   * Gets all markers
   */
  getMarkers(): Array<{ id: string; position: MusicalPosition; name: string; type: string; color: string }> {
    return this.markers.getAll();
  }

  /**
   * Gets markers in a range
   */
  getMarkersInRange(start: MusicalPosition, end: MusicalPosition): Array<any> {
    return this.markers.getInRange(start.totalBeats, end.totalBeats);
  }

  // ========== Loop Management ==========

  private loopEnabled: boolean = false;
  private loopStart: MusicalPosition | null = null;
  private loopEnd: MusicalPosition | null = null;

  /**
   * Sets the loop range
   */
  setLoopRange(start: MusicalPosition, end: MusicalPosition): void {
    this.loopStart = start;
    this.loopEnd = end;
  }

  /**
   * Enables or disables looping
   */
  setLoopEnabled(enabled: boolean): void {
    this.loopEnabled = enabled;
  }

  /**
   * Checks if looping is enabled
   */
  isLoopEnabled(): boolean {
    return this.loopEnabled;
  }

  /**
   * Gets the loop range
   */
  getLoopRange(): { start: MusicalPosition; end: MusicalPosition } | null {
    if (!this.loopStart || !this.loopEnd) return null;
    return { start: this.loopStart, end: this.loopEnd };
  }

  /**
   * Updates playback (called from audio callback or animation frame)
   * Handles loop wraparound
   */
  update(deltaSeconds: number): void {
    if (!this.isPlaying()) return;

    const currentPos = this.playhead.getPosition();
    const newPos = currentPos + deltaSeconds;

    if (this.loopEnabled && this.loopStart && this.loopEnd) {
      const startAbs = this.musicalToAbsolute(this.loopStart);
      const endAbs = this.musicalToAbsolute(this.loopEnd);

      if (newPos >= endAbs.seconds) {
        // Loop back to start
        this.playhead.setPosition(startAbs.seconds);
      } else {
        this.playhead.setPosition(newPos);
      }
    } else {
      this.playhead.setPosition(newPos);
    }
  }

  // ========== Utility Methods ==========

  /**
   * Calculates the total duration in both musical and absolute time
   */
  getDuration(endPosition: MusicalPosition): { musical: number; absolute: number } {
    const absolute = this.musicalToAbsolute(endPosition);
    return {
      musical: endPosition.totalBeats,
      absolute: absolute.seconds,
    };
  }

  /**
   * Formats a position for display
   */
  formatPosition(position: MusicalPosition, format: 'measures' | 'time' = 'measures'): string {
    if (format === 'measures') {
      return `${position.measure + 1}:${position.beat + 1}:${Math.floor(position.tick / 48)}`;
    } else {
      const absolute = this.musicalToAbsolute(position);
      const minutes = Math.floor(absolute.seconds / 60);
      const seconds = Math.floor(absolute.seconds % 60);
      const ms = Math.floor((absolute.seconds % 1) * 100);
      return `${minutes}:${seconds.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`;
    }
  }

  /**
   * Subscribe to playhead position updates
   */
  onPositionChange(callback: (position: { musical: MusicalPosition; absolute: AbsolutePosition }) => void): () => void {
    return this.playhead.onPositionChange(() => {
      callback(this.getPosition());
    });
  }
}
