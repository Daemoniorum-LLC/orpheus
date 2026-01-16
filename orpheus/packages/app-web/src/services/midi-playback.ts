/**
 * MIDI Playback Service
 * Plays back MIDI using Tone.js
 */

import * as Tone from 'tone';
import type { MaestroProject } from '@orpheus/shared-types';
import { scheduleProjectNotes, midiNoteToName, type ScheduledNote } from './note-scheduler';

export type PlaybackState = 'stopped' | 'playing' | 'paused';

export interface PlaybackPosition {
  measure: number;
  beat: number;
  seconds: number;
}

/**
 * MIDI Playback Engine using Tone.js
 */
export class MIDIPlaybackEngine {
  private synths: Map<string, Tone.PolySynth> = new Map();
  private transport: typeof Tone.Transport;
  private project: MaestroProject | null = null;
  private state: PlaybackState = 'stopped';
  private totalDuration: number = 0;
  private beatsPerMeasure: number = 4;
  private beatDuration: number = 0.5;
  private callbacks: {
    onStateChange?: (state: PlaybackState) => void;
    onPositionChange?: (position: PlaybackPosition) => void;
  } = {};

  constructor() {
    this.transport = Tone.getTransport();
    this.transport.loop = false;
  }

  /**
   * Load a project for playback
   */
  async loadProject(project: MaestroProject): Promise<void> {
    console.log('[MIDIPlayback] Loading project:', project.project.metadata.title);

    // Dispose existing synths
    this.dispose();

    this.project = project;

    // Set tempo
    const tempo = project.project.metadata.tempo || 120;
    this.transport.bpm.value = tempo;
    console.log('[MIDIPlayback] Tempo set to', tempo, 'BPM');

    // Set time signature
    const timeSig = project.project.metadata.timeSignature || { numerator: 4, denominator: 4 };
    this.transport.timeSignature = [timeSig.numerator, timeSig.denominator];
    this.beatsPerMeasure = timeSig.numerator;
    this.beatDuration = 60 / tempo;
    console.log('[MIDIPlayback] Time signature:', `${timeSig.numerator}/${timeSig.denominator}`);

    // Create synths for each track
    (project.project.composition.tracks || []).forEach((track: any) => {
      const synth = this.createSynthForTrack(track);
      this.synths.set(track.id, synth);
      console.log(`[MIDIPlayback] Created synth for track: ${track.name} (${track.instrument?.type})`);
    });

    // Schedule notes (simplified - would need full note scheduling)
    this.scheduleNotes();

    console.log('[MIDIPlayback] Project loaded successfully');
  }

  /**
   * Create a synth for a track based on instrument type
   */
  private createSynthForTrack(track: any): Tone.PolySynth {
    const instrumentType = track.instrument?.type || 'guitar';

    let synth: Tone.PolySynth;

    switch (instrumentType) {
      case 'guitar':
      case 'electric-guitar':
        // Guitar-like sound
        synth = new Tone.PolySynth(Tone.Synth, {
          oscillator: { type: 'triangle' },
          envelope: {
            attack: 0.005,
            decay: 0.1,
            sustain: 0.3,
            release: 1,
          },
        });
        break;

      case 'bass':
      case 'electric-bass':
        // Bass sound
        synth = new Tone.PolySynth(Tone.Synth, {
          oscillator: { type: 'sine' },
          envelope: {
            attack: 0.01,
            decay: 0.2,
            sustain: 0.5,
            release: 0.8,
          },
        });
        break;

      case 'piano':
      case 'keyboard':
        // Piano-like sound
        synth = new Tone.PolySynth(Tone.Synth, {
          oscillator: { type: 'sine' },
          envelope: {
            attack: 0.002,
            decay: 0.1,
            sustain: 0.2,
            release: 2,
          },
        });
        break;

      case 'drums':
        // Percussion (simplified - using triangle wave for drum-like sounds)
        synth = new Tone.PolySynth(Tone.Synth, {
          oscillator: { type: 'triangle' },
          envelope: {
            attack: 0.001,
            decay: 0.2,
            sustain: 0,
            release: 0.2,
          },
        });
        break;

      default:
        // Default synth
        synth = new Tone.PolySynth(Tone.Synth);
    }

    // Connect to output
    synth.toDestination();

    // Set volume based on track
    if (track.volume !== undefined) {
      synth.volume.value = Tone.gainToDb(track.volume);
    }

    return synth;
  }

  /**
   * Schedule notes for playback from the project data
   */
  private scheduleNotes(): void {
    if (!this.project) return;

    console.log('[MIDIPlayback] Scheduling notes from project...');

    // Parse and schedule all notes from the project
    const schedule = scheduleProjectNotes(this.project);

    if (schedule.notes.length === 0) {
      console.warn('[MIDIPlayback] No notes found to schedule');
      return;
    }

    // Schedule each note
    schedule.notes.forEach((note: ScheduledNote) => {
      const synth = this.synths.get(note.trackId);
      if (!synth) {
        console.warn(`[MIDIPlayback] No synth found for track ${note.trackId}`);
        return;
      }

      // Convert MIDI note number to Tone.js note name
      const noteName = midiNoteToName(note.midiNote);

      // Adjust velocity (Tone.js uses -Infinity to 0 dB)
      const velocity = note.velocity;

      // Schedule the note
      this.transport.schedule((scheduleTime) => {
        // Apply technique-specific modifications
        this.applyTechniqueModifications(synth, note);

        // Trigger the note with velocity
        synth.triggerAttackRelease(
          noteName,
          note.duration,
          scheduleTime,
          velocity
        );

        // Reset any technique modifications
        this.resetTechniqueModifications(synth);
      }, note.startTime);
    });

    console.log(`[MIDIPlayback] Scheduled ${schedule.notes.length} notes`);
    console.log(`[MIDIPlayback] Total duration: ${schedule.totalDuration.toFixed(2)}s`);

    // Store total duration for position tracking
    this.totalDuration = schedule.totalDuration;

    // Set loop points if needed
    if (schedule.totalDuration > 0) {
      this.transport.loop = false;
      this.transport.loopEnd = schedule.totalDuration;
    }
  }

  /**
   * Apply playing technique modifications to synth
   */
  private applyTechniqueModifications(synth: Tone.PolySynth, note: ScheduledNote): void {
    if (!note.techniques || note.techniques.length === 0) return;

    note.techniques.forEach(technique => {
      switch (technique.toLowerCase()) {
        case 'palm-mute':
        case 'muted':
          // Dampen the sound
          synth.volume.value = -12; // Reduce volume
          break;

        case 'staccato':
          // Note duration already shortened in schedule
          break;

        case 'accent':
          // Increase volume
          synth.volume.value = 3;
          break;

        case 'ghost':
          // Reduce volume significantly
          synth.volume.value = -18;
          break;

        case 'slide':
        case 'hammer-on':
        case 'pull-off':
          // Enable portamento for legato techniques
          // Note: PolySynth doesn't support portamento directly
          // Would need custom synth implementation for true legato
          break;
      }
    });
  }

  /**
   * Reset synth to default state after technique application
   */
  private resetTechniqueModifications(synth: Tone.PolySynth): void {
    // Reset volume to default
    const defaultVolume = -6; // Moderate volume
    synth.volume.rampTo(defaultVolume, 0.01);
  }

  /**
   * Start playback
   */
  async play(): Promise<void> {
    if (!this.project) {
      console.warn('[MIDIPlayback] No project loaded');
      return;
    }

    // Ensure audio context is started
    await Tone.start();
    console.log('[MIDIPlayback] Audio context started');

    // Resume if paused
    if (this.state === 'paused') {
      this.transport.start();
    } else {
      // Start from beginning
      this.transport.start('+0.1'); // Small delay for smoother start
    }

    this.state = 'playing';
    this.callbacks.onStateChange?.('playing');

    console.log('[MIDIPlayback] Playback started');

    // Monitor position
    this.startPositionMonitoring();
  }

  /**
   * Pause playback
   */
  pause(): void {
    this.transport.pause();
    this.state = 'paused';
    this.callbacks.onStateChange?.('paused');
    console.log('[MIDIPlayback] Playback paused');
  }

  /**
   * Stop playback
   */
  stop(): void {
    this.transport.stop();
    this.transport.position = 0;
    this.state = 'stopped';
    this.callbacks.onStateChange?.('stopped');
    console.log('[MIDIPlayback] Playback stopped');
  }

  /**
   * Seek to a specific position
   */
  seekTo(seconds: number): void {
    this.transport.seconds = seconds;
    console.log(`[MIDIPlayback] Seeked to ${seconds.toFixed(2)}s`);
  }

  /**
   * Set playback speed (0.5 - 2.0)
   */
  setSpeed(speed: number): void {
    const clampedSpeed = Math.max(0.5, Math.min(2.0, speed));
    this.transport.bpm.value = (this.project?.project.metadata.tempo || 120) * clampedSpeed;
    console.log(`[MIDIPlayback] Speed set to ${(clampedSpeed * 100).toFixed(0)}%`);
  }

  /**
   * Get current state
   */
  getState(): PlaybackState {
    return this.state;
  }

  /**
   * Get current position
   */
  getPosition(): PlaybackPosition {
    const seconds = this.transport.seconds;

    // Calculate measure and beat based on actual tempo and time signature
    const measureDuration = this.beatsPerMeasure * this.beatDuration;
    const measure = Math.floor(seconds / measureDuration);
    const secondsIntoMeasure = seconds % measureDuration;
    const beat = Math.floor(secondsIntoMeasure / this.beatDuration);

    return { measure, beat, seconds };
  }

  /**
   * Set callbacks for state and position changes
   */
  setCallbacks(callbacks: {
    onStateChange?: (state: PlaybackState) => void;
    onPositionChange?: (position: PlaybackPosition) => void;
  }): void {
    this.callbacks = callbacks;
  }

  /**
   * Monitor playback position
   */
  private startPositionMonitoring(): void {
    const updateInterval = setInterval(() => {
      if (this.state !== 'playing') {
        clearInterval(updateInterval);
        return;
      }

      const position = this.getPosition();
      this.callbacks.onPositionChange?.(position);

      // Check if playback has ended (reached total duration)
      if (this.totalDuration > 0 && position.seconds >= this.totalDuration) {
        this.stop();
        clearInterval(updateInterval);
        return;
      }

      // Check if transport has stopped
      if (this.transport.state !== 'started') {
        this.stop();
        clearInterval(updateInterval);
      }
    }, 100); // Update every 100ms
  }

  /**
   * Dispose all resources
   */
  dispose(): void {
    console.log('[MIDIPlayback] Disposing playback engine');

    this.stop();

    // Dispose all synths
    this.synths.forEach((synth) => {
      synth.dispose();
    });
    this.synths.clear();

    // Cancel all scheduled events
    this.transport.cancel();

    this.project = null;
  }
}

/**
 * Global singleton playback engine
 */
let playbackEngineInstance: MIDIPlaybackEngine | null = null;

/**
 * Get the global playback engine instance
 */
export function getPlaybackEngine(): MIDIPlaybackEngine {
  if (!playbackEngineInstance) {
    playbackEngineInstance = new MIDIPlaybackEngine();
  }
  return playbackEngineInstance;
}

