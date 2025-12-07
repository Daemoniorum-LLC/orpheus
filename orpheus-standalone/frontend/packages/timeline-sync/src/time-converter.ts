/**
 * Time Converter - Converts between musical time and absolute time
 */

import type { TimeSignature } from '@maestro-ai/shared-types';
import type { MusicalPosition } from './timeline';
import { TempoMap } from './tempo-map';

/**
 * TimeConverter handles the conversion between musical positions (measures/beats)
 * and absolute time (seconds), taking into account tempo changes and time signatures
 */
export class TimeConverter {
  private tempoMap: TempoMap;
  private timeSignature: TimeSignature;
  private cache: Map<string, number> = new Map();

  constructor(tempoMap: TempoMap, timeSignature: TimeSignature) {
    this.tempoMap = tempoMap;
    this.timeSignature = timeSignature;
  }

  /**
   * Converts musical position to seconds
   */
  musicalToSeconds(position: MusicalPosition): number {
    const cacheKey = `${position.measure}:${position.beat}:${position.tick}`;
    const cached = this.cache.get(cacheKey);
    if (cached !== undefined) return cached;

    const totalBeats = position.totalBeats;
    const seconds = this.tempoMap.beatsToSeconds(0, totalBeats);

    this.cache.set(cacheKey, seconds);
    return seconds;
  }

  /**
   * Converts seconds to musical position
   */
  secondsToMusical(seconds: number): MusicalPosition {
    const totalBeats = this.tempoMap.secondsToBeats(seconds);
    return this.beatsToMusical(totalBeats);
  }

  /**
   * Converts total beats to musical position (measure/beat/tick)
   */
  beatsToMusical(totalBeats: number): MusicalPosition {
    const beatsPerMeasure = this.timeSignature.numerator;
    const measure = Math.floor(totalBeats / beatsPerMeasure);
    const beatInMeasure = totalBeats % beatsPerMeasure;
    const beat = Math.floor(beatInMeasure);
    const tick = Math.round((beatInMeasure - beat) * 480); // 480 ticks per beat

    return {
      measure,
      beat,
      tick,
      totalBeats,
    };
  }

  /**
   * Converts musical position to total beats
   */
  musicalToBeats(position: MusicalPosition): number {
    const beatsPerMeasure = this.timeSignature.numerator;
    return position.measure * beatsPerMeasure + position.beat + position.tick / 480;
  }

  /**
   * Sets the time signature
   */
  setTimeSignature(timeSignature: TimeSignature): void {
    this.timeSignature = timeSignature;
    this.invalidateCache();
  }

  /**
   * Invalidates the conversion cache (call when tempo changes)
   */
  invalidateCache(): void {
    this.cache.clear();
  }

  /**
   * Calculates the duration between two musical positions
   */
  getDuration(start: MusicalPosition, end: MusicalPosition): number {
    const startSeconds = this.musicalToSeconds(start);
    const endSeconds = this.musicalToSeconds(end);
    return endSeconds - startSeconds;
  }

  /**
   * Adds a duration (in beats) to a musical position
   */
  addBeats(position: MusicalPosition, beats: number): MusicalPosition {
    const totalBeats = position.totalBeats + beats;
    return this.beatsToMusical(totalBeats);
  }

  /**
   * Adds a duration (in seconds) to a musical position
   */
  addSeconds(position: MusicalPosition, seconds: number): MusicalPosition {
    const currentSeconds = this.musicalToSeconds(position);
    const newSeconds = currentSeconds + seconds;
    return this.secondsToMusical(newSeconds);
  }
}
