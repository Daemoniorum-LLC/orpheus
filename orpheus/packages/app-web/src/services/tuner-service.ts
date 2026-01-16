/**
 * Tuner Service - Real-time pitch detection for instrument tuning
 */

import { FrequencyAnalyzer } from '@orpheus/audio-analysis';

export interface TunerNote {
  name: string;
  octave: number;
  frequency: number;
  cents: number; // Deviation from perfect pitch (-50 to +50)
  inTune: boolean; // Within tolerance
}

export interface TunerState {
  isActive: boolean;
  currentNote: TunerNote | null;
  inputLevel: number;
  referencePitch: number; // A4 frequency (default 440 Hz)
}

export interface TuningPreset {
  name: string;
  notes: string[]; // e.g., ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'] for standard
}

// Standard tuning presets
export const TUNING_PRESETS: TuningPreset[] = [
  { name: 'Standard (EADGBE)', notes: ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'] },
  { name: 'Drop D (DADGBE)', notes: ['D2', 'A2', 'D3', 'G3', 'B3', 'E4'] },
  { name: 'Half Step Down (Eb)', notes: ['Eb2', 'Ab2', 'Db3', 'Gb3', 'Bb3', 'Eb4'] },
  { name: 'Full Step Down (D)', notes: ['D2', 'G2', 'C3', 'F3', 'A3', 'D4'] },
  { name: 'Drop C (CGCFAD)', notes: ['C2', 'G2', 'C3', 'F3', 'A3', 'D4'] },
  { name: 'Open D (DADF#AD)', notes: ['D2', 'A2', 'D3', 'F#3', 'A3', 'D4'] },
  { name: 'Open G (DGDGBD)', notes: ['D2', 'G2', 'D3', 'G3', 'B3', 'D4'] },
  { name: 'DADGAD', notes: ['D2', 'A2', 'D3', 'G3', 'A3', 'D4'] },
  { name: 'Bass Standard (EADG)', notes: ['E1', 'A1', 'D2', 'G2'] },
  { name: 'Bass 5-String (BEADG)', notes: ['B0', 'E1', 'A1', 'D2', 'G2'] },
];

// Note names and frequencies
const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
const FLAT_NAMES = ['C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab', 'A', 'Bb', 'B'];

/**
 * Convert frequency to note info
 */
export function frequencyToNote(frequency: number, referencePitch: number = 440): TunerNote | null {
  if (frequency <= 0 || !isFinite(frequency)) {
    return null;
  }

  // Calculate semitones from A4
  const semitones = 12 * Math.log2(frequency / referencePitch);
  const roundedSemitones = Math.round(semitones);
  const cents = Math.round((semitones - roundedSemitones) * 100);

  // Calculate note index (A is index 9)
  const noteIndex = ((roundedSemitones % 12) + 12 + 9) % 12;
  const octave = Math.floor((roundedSemitones + 9) / 12) + 4;

  // Calculate perfect frequency for this note
  const perfectFrequency = referencePitch * Math.pow(2, roundedSemitones / 12);

  return {
    name: NOTE_NAMES[noteIndex],
    octave,
    frequency: perfectFrequency,
    cents,
    inTune: Math.abs(cents) <= 5, // Within 5 cents is considered in tune
  };
}

/**
 * Parse note string (e.g., 'E4', 'Ab3') to frequency
 */
export function noteToFrequency(noteString: string, referencePitch: number = 440): number {
  const match = noteString.match(/^([A-G])(#|b)?(\d)$/);
  if (!match) return 0;

  const [, noteLetter, accidental, octaveStr] = match;
  const octave = parseInt(octaveStr, 10);

  // Find note index
  let noteIndex = NOTE_NAMES.indexOf(noteLetter);
  if (noteIndex === -1) return 0;

  if (accidental === '#') noteIndex += 1;
  else if (accidental === 'b') noteIndex -= 1;

  noteIndex = ((noteIndex % 12) + 12) % 12;

  // Calculate semitones from A4
  const semitonesFromA4 = (octave - 4) * 12 + (noteIndex - 9);

  return referencePitch * Math.pow(2, semitonesFromA4 / 12);
}

/**
 * Tuner class for real-time pitch detection
 */
export class TunerService {
  private analyzer: FrequencyAnalyzer;
  private audioContext: AudioContext | null = null;
  private mediaStream: MediaStream | null = null;
  private analyserNode: AnalyserNode | null = null;
  private scriptProcessor: ScriptProcessorNode | null = null;
  private isActive = false;
  private referencePitch = 440;
  private callbacks: Array<(state: TunerState) => void> = [];
  private animationFrame: number | null = null;

  constructor() {
    this.analyzer = new FrequencyAnalyzer({
      fftSize: 4096,
      sampleRate: 48000,
      windowFunction: 'hann',
      smoothing: 0.5,
    });
  }

  /**
   * Start the tuner
   */
  async start(): Promise<void> {
    if (this.isActive) return;

    try {
      // Create audio context
      this.audioContext = new AudioContext({ sampleRate: 48000 });

      // Request microphone access
      this.mediaStream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: false,
          noiseSuppression: false,
          autoGainControl: false,
          sampleRate: 48000,
        },
      });

      // Create analyser node
      this.analyserNode = this.audioContext.createAnalyser();
      this.analyserNode.fftSize = 4096;
      this.analyserNode.smoothingTimeConstant = 0.5;

      // Connect microphone to analyser
      const source = this.audioContext.createMediaStreamSource(this.mediaStream);
      source.connect(this.analyserNode);

      this.isActive = true;
      this.startAnalysis();

      console.log('[TunerService] Started');
    } catch (error) {
      console.error('[TunerService] Failed to start:', error);
      throw error;
    }
  }

  /**
   * Stop the tuner
   */
  stop(): void {
    if (!this.isActive) return;

    // Stop animation frame
    if (this.animationFrame) {
      cancelAnimationFrame(this.animationFrame);
      this.animationFrame = null;
    }

    // Stop media stream
    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach((track) => track.stop());
      this.mediaStream = null;
    }

    // Close audio context
    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }

    this.analyserNode = null;
    this.isActive = false;

    // Notify listeners
    this.notifyListeners({
      isActive: false,
      currentNote: null,
      inputLevel: 0,
      referencePitch: this.referencePitch,
    });

    console.log('[TunerService] Stopped');
  }

  /**
   * Set reference pitch (A4 frequency)
   */
  setReferencePitch(frequency: number): void {
    if (frequency >= 400 && frequency <= 480) {
      this.referencePitch = frequency;
    }
  }

  /**
   * Get reference pitch
   */
  getReferencePitch(): number {
    return this.referencePitch;
  }

  /**
   * Subscribe to state changes
   */
  subscribe(callback: (state: TunerState) => void): () => void {
    this.callbacks.push(callback);
    return () => {
      this.callbacks = this.callbacks.filter((cb) => cb !== callback);
    };
  }

  /**
   * Check if tuner is active
   */
  getIsActive(): boolean {
    return this.isActive;
  }

  /**
   * Start pitch analysis loop
   */
  private startAnalysis(): void {
    if (!this.analyserNode || !this.audioContext) return;

    const bufferLength = this.analyserNode.fftSize;
    const dataArray = new Float32Array(bufferLength);
    const frequencyData = new Uint8Array(this.analyserNode.frequencyBinCount);

    const analyze = () => {
      if (!this.isActive || !this.analyserNode) return;

      // Get time domain data for pitch detection
      this.analyserNode.getFloatTimeDomainData(dataArray);

      // Get frequency data for level metering
      this.analyserNode.getByteFrequencyData(frequencyData);

      // Calculate input level (RMS)
      let sum = 0;
      for (let i = 0; i < dataArray.length; i++) {
        sum += dataArray[i] * dataArray[i];
      }
      const rms = Math.sqrt(sum / dataArray.length);
      const inputLevel = Math.min(1, rms * 5); // Scale for visibility

      // Detect pitch
      let currentNote: TunerNote | null = null;

      if (inputLevel > 0.01) {
        // Only detect if there's sufficient signal
        const frequency = this.analyzer.detectPitch(dataArray);

        if (frequency && frequency > 20 && frequency < 5000) {
          currentNote = frequencyToNote(frequency, this.referencePitch);
        }
      }

      // Notify listeners
      this.notifyListeners({
        isActive: this.isActive,
        currentNote,
        inputLevel,
        referencePitch: this.referencePitch,
      });

      // Continue loop
      this.animationFrame = requestAnimationFrame(analyze);
    };

    analyze();
  }

  /**
   * Notify all listeners of state change
   */
  private notifyListeners(state: TunerState): void {
    this.callbacks.forEach((cb) => cb(state));
  }
}

// Singleton instance
let tunerInstance: TunerService | null = null;

export function getTunerService(): TunerService {
  if (!tunerInstance) {
    tunerInstance = new TunerService();
  }
  return tunerInstance;
}
