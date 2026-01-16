/**
 * MIDI playback using Web MIDI API or virtual instruments
 */

import type { MIDIFile, MIDIEvent } from './midi-file';

export interface MIDIPlayerOptions {
  tempo?: number;        // BPM
  loop?: boolean;
  onNoteOn?: (note: number, velocity: number, channel: number) => void;
  onNoteOff?: (note: number, channel: number) => void;
  onEnd?: () => void;
}

export class MIDIPlayer {
  private file: MIDIFile | null = null;
  private options: MIDIPlayerOptions;
  private playing: boolean = false;
  private paused: boolean = false;
  private currentTick: number = 0;
  private startTime: number = 0;
  private midiOutput: any = null; // WebMIDI Output

  constructor(options: MIDIPlayerOptions = {}) {
    this.options = {
      tempo: 120,
      loop: false,
      ...options,
    };
  }

  /**
   * Loads a MIDI file for playback
   */
  loadFile(file: MIDIFile): void {
    this.file = file;
    this.currentTick = 0;
  }

  /**
   * Starts playback
   */
  async play(): Promise<void> {
    if (!this.file) {
      throw new Error('No MIDI file loaded');
    }

    if (this.paused) {
      this.paused = false;
      this.playing = true;
      this.resume();
      return;
    }

    this.playing = true;
    this.startTime = performance.now();
    this.scheduleEvents();
  }

  /**
   * Pauses playback
   */
  pause(): void {
    this.playing = false;
    this.paused = true;
  }

  /**
   * Stops playback and resets position
   */
  stop(): void {
    this.playing = false;
    this.paused = false;
    this.currentTick = 0;
    this.allNotesOff();
  }

  /**
   * Seeks to a specific position (in ticks)
   */
  seek(tick: number): void {
    this.currentTick = tick;
    if (this.playing) {
      this.startTime = performance.now();
      this.scheduleEvents();
    }
  }

  /**
   * Gets the current playback position (in seconds)
   */
  getCurrentTime(): number {
    if (!this.file) return 0;
    const ticksPerSecond = (this.options.tempo! / 60) * this.file.header.division;
    return this.currentTick / ticksPerSecond;
  }

  /**
   * Sets the playback tempo
   */
  setTempo(bpm: number): void {
    this.options.tempo = bpm;
  }

  private resume(): void {
    this.startTime = performance.now();
    this.scheduleEvents();
  }

  private scheduleEvents(): void {
    if (!this.file || !this.playing) return;

    const track = this.file.tracks[0]; // Simplified: play first track only
    const division = this.file.header.division;
    const ticksPerSecond = (this.options.tempo! / 60) * division;

    let tick = this.currentTick;

    for (const event of track.events) {
      tick += event.deltaTime;

      const timeMs = (tick / ticksPerSecond) * 1000;
      const delay = timeMs - (performance.now() - this.startTime);

      if (delay > 0) {
        setTimeout(() => this.handleEvent(event), delay);
      }
    }

    // Schedule end
    const totalTime = (tick / ticksPerSecond) * 1000;
    setTimeout(() => {
      if (this.options.loop) {
        this.currentTick = 0;
        this.play();
      } else {
        this.stop();
        this.options.onEnd?.();
      }
    }, totalTime);
  }

  private handleEvent(event: MIDIEvent): void {
    switch (event.type) {
      case 'noteOn':
        this.options.onNoteOn?.(event.note, event.velocity, event.channel);
        if (this.midiOutput) {
          this.midiOutput.send([0x90 | event.channel, event.note, event.velocity]);
        }
        break;

      case 'noteOff':
        this.options.onNoteOff?.(event.note, event.channel);
        if (this.midiOutput) {
          this.midiOutput.send([0x80 | event.channel, event.note, 0]);
        }
        break;

      // Handle other event types as needed
    }
  }

  private allNotesOff(): void {
    // Send all notes off message
    for (let channel = 0; channel < 16; channel++) {
      if (this.midiOutput) {
        this.midiOutput.send([0xB0 | channel, 123, 0]); // All notes off
      }
    }
  }

  /**
   * Connects to a Web MIDI output device
   */
  async connectMIDIOutput(outputIndex: number = 0): Promise<void> {
    if (!navigator.requestMIDIAccess) {
      throw new Error('Web MIDI API not supported');
    }

    const midiAccess = await navigator.requestMIDIAccess();
    const outputs: MIDIOutput[] = [];
    midiAccess.outputs.forEach((output) => outputs.push(output));

    if (outputs.length === 0) {
      throw new Error('No MIDI output devices found');
    }

    this.midiOutput = outputs[outputIndex];
  }
}
