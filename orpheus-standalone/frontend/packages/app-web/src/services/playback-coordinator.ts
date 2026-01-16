/**
 * Playback Coordinator
 * Synchronizes Timeline, MIDI Playback, and alphaTab cursor
 */

import { Timeline, type MusicalPosition, type AbsolutePosition } from '@maestro-ai/timeline-sync';
import type { MaestroProject } from '@maestro-ai/shared-types';
import type { AlphaTabApi } from '@coderline/alphatab';
import { getPlaybackEngine } from './midi-playback';

export interface PlaybackState {
  isPlaying: boolean;
  position: {
    musical: MusicalPosition;
    absolute: AbsolutePosition;
  };
  tempo: number;
  timeSignature: { numerator: number; denominator: number };
}

/**
 * Singleton coordinator that manages all playback systems
 */
class PlaybackCoordinator {
  private timeline: Timeline | null = null;
  private midiEngine = getPlaybackEngine();
  private alphaTabApi: AlphaTabApi | null = null;
  private animationFrameId: number | null = null;
  private stateCallbacks: Array<(state: PlaybackState) => void> = [];
  private initialized = false;

  /**
   * Initialize with a project
   */
  async initialize(project: MaestroProject): Promise<void> {
    console.log('[PlaybackCoordinator] Initializing with project');

    const metadata = project.project.metadata;
    const timeSignature = metadata.timeSignature || { numerator: 4, denominator: 4 };
    const tempo = metadata.tempo || 120;

    // Create unified timeline
    this.timeline = new Timeline({
      initialTempo: tempo,
      timeSignature,
      sampleRate: 48000,
    });

    // Load project into MIDI engine
    await this.midiEngine.loadProject(project);

    // Subscribe to timeline position changes
    this.timeline.onPositionChange((position) => {
      this.syncPositionToEngines(position);
      this.notifyStateChange();
    });

    this.initialized = true;
    console.log('[PlaybackCoordinator] Initialization complete');
  }

  /**
   * Register alphaTab API for cursor sync
   */
  setAlphaTabApi(api: AlphaTabApi | null): void {
    this.alphaTabApi = api;
    console.log('[PlaybackCoordinator] alphaTab API registered');
  }

  /**
   * Start playback
   */
  async play(): Promise<void> {
    if (!this.timeline || !this.initialized) {
      console.warn('[PlaybackCoordinator] Not initialized');
      return;
    }

    console.log('[PlaybackCoordinator] Starting playback');

    // Start timeline
    this.timeline.play();

    // Start MIDI playback
    await this.midiEngine.play();

    // Start alphaTab playback
    if (this.alphaTabApi) {
      this.alphaTabApi.play();
    }

    // Start animation loop for position sync
    this.startAnimationLoop();

    this.notifyStateChange();
  }

  /**
   * Pause playback
   */
  pause(): void {
    if (!this.timeline) return;

    console.log('[PlaybackCoordinator] Pausing playback');

    // Pause timeline
    this.timeline.pause();

    // Pause MIDI
    this.midiEngine.pause();

    // Pause alphaTab
    if (this.alphaTabApi) {
      this.alphaTabApi.pause();
    }

    // Stop animation loop
    this.stopAnimationLoop();

    this.notifyStateChange();
  }

  /**
   * Stop playback
   */
  stop(): void {
    if (!this.timeline) return;

    console.log('[PlaybackCoordinator] Stopping playback');

    // Stop timeline
    this.timeline.stop();

    // Stop MIDI
    this.midiEngine.stop();

    // Stop alphaTab
    if (this.alphaTabApi) {
      this.alphaTabApi.stop();
    }

    // Stop animation loop
    this.stopAnimationLoop();

    this.notifyStateChange();
  }

  /**
   * Seek to a position
   */
  seekTo(position: MusicalPosition | AbsolutePosition): void {
    if (!this.timeline) return;

    this.timeline.setPosition(position);

    // Sync to all engines
    this.syncPositionToEngines(this.timeline.getPosition());
  }

  /**
   * Get current playback state
   */
  getState(): PlaybackState {
    if (!this.timeline) {
      return {
        isPlaying: false,
        position: {
          musical: { measure: 0, beat: 0, tick: 0, totalBeats: 0 },
          absolute: { seconds: 0, samples: 0, frames: 0 },
        },
        tempo: 120,
        timeSignature: { numerator: 4, denominator: 4 },
      };
    }

    return {
      isPlaying: this.timeline.isPlaying(),
      position: this.timeline.getPosition(),
      tempo: this.timeline.getCurrentTempo(),
      timeSignature: this.timeline.getTimeSignature(),
    };
  }

  /**
   * Subscribe to state changes
   */
  onStateChange(callback: (state: PlaybackState) => void): () => void {
    this.stateCallbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = this.stateCallbacks.indexOf(callback);
      if (index !== -1) {
        this.stateCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Set loop range
   */
  setLoopRange(start: MusicalPosition, end: MusicalPosition): void {
    if (!this.timeline) return;
    this.timeline.setLoopRange(start, end);
  }

  /**
   * Enable/disable looping
   */
  setLoopEnabled(enabled: boolean): void {
    if (!this.timeline) return;
    this.timeline.setLoopEnabled(enabled);
  }

  /**
   * Set tempo
   */
  setTempo(bpm: number): void {
    if (!this.timeline) return;
    this.timeline.setGlobalTempo(bpm);
    this.notifyStateChange();
  }

  // ========== Private Methods ==========

  /**
   * Sync position to all playback engines
   */
  private syncPositionToEngines(position: { musical: MusicalPosition; absolute: AbsolutePosition }): void {
    // Sync alphaTab cursor
    if (this.alphaTabApi) {
      // alphaTab uses tick position (milliseconds)
      const tickPosition = position.absolute.seconds * 1000;
      this.alphaTabApi.tickPosition = tickPosition;
    }

    // MIDI engine updates itself via Tone.Transport
  }

  /**
   * Animation loop for position updates
   */
  private startAnimationLoop(): void {
    if (this.animationFrameId !== null) return;

    const updatePosition = () => {
      if (!this.timeline || !this.timeline.isPlaying()) {
        this.stopAnimationLoop();
        return;
      }

      // Timeline updates happen via Playhead which uses performance.now()
      // We just need to notify state changes
      this.notifyStateChange();

      this.animationFrameId = requestAnimationFrame(updatePosition);
    };

    this.animationFrameId = requestAnimationFrame(updatePosition);
  }

  /**
   * Stop animation loop
   */
  private stopAnimationLoop(): void {
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
  }

  /**
   * Notify all state change callbacks
   */
  private notifyStateChange(): void {
    const state = this.getState();
    for (const callback of this.stateCallbacks) {
      callback(state);
    }
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    this.stop();
    this.timeline = null;
    this.alphaTabApi = null;
    this.stateCallbacks = [];
    this.initialized = false;
  }
}

// Singleton instance
let coordinatorInstance: PlaybackCoordinator | null = null;

/**
 * Get the global playback coordinator instance
 */
export function getPlaybackCoordinator(): PlaybackCoordinator {
  if (!coordinatorInstance) {
    coordinatorInstance = new PlaybackCoordinator();
  }
  return coordinatorInstance;
}

/**
 * Reset the coordinator (for testing or reinitialize)
 */
export function resetPlaybackCoordinator(): void {
  if (coordinatorInstance) {
    coordinatorInstance.destroy();
    coordinatorInstance = null;
  }
}
