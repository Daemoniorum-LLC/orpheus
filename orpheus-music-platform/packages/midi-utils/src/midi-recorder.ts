/**
 * MIDI recording from input devices
 */

import type { MIDITrack, MIDIEvent } from './midi-file';

export interface MIDIRecorderOptions {
  quantize?: boolean;
  quantizeValue?: number; // 16th, 8th, quarter note, etc.
}

export class MIDIRecorder {
  private recording: boolean = false;
  private events: MIDIEvent[] = [];
  private startTime: number = 0;
  private midiInput: any = null;
  private options: MIDIRecorderOptions;

  constructor(options: MIDIRecorderOptions = {}) {
    this.options = {
      quantize: false,
      quantizeValue: 16, // 16th notes
      ...options,
    };
  }

  /**
   * Connects to a MIDI input device
   */
  async connectMIDIInput(inputIndex: number = 0): Promise<void> {
    if (!navigator.requestMIDIAccess) {
      throw new Error('Web MIDI API not supported');
    }

    const midiAccess = await navigator.requestMIDIAccess();
    const inputs: MIDIInput[] = [];
    midiAccess.inputs.forEach((input) => inputs.push(input));

    if (inputs.length === 0) {
      throw new Error('No MIDI input devices found');
    }

    this.midiInput = inputs[inputIndex];
    this.midiInput.onmidimessage = this.handleMIDIMessage.bind(this);
  }

  /**
   * Starts recording MIDI input
   */
  start(): void {
    this.recording = true;
    this.events = [];
    this.startTime = performance.now();
  }

  /**
   * Stops recording and returns the recorded track
   */
  stop(): MIDITrack {
    this.recording = false;

    if (this.options.quantize) {
      this.quantizeEvents();
    }

    return { events: this.events };
  }

  /**
   * Clears recorded events
   */
  clear(): void {
    this.events = [];
  }

  private handleMIDIMessage(message: any): void {
    if (!this.recording) return;

    const [status, data1, data2] = message.data;
    const deltaTime = performance.now() - this.startTime;

    const eventType = status >> 4;
    const channel = status & 0x0F;

    let event: MIDIEvent | null = null;

    switch (eventType) {
      case 0x9: // Note On
        event = {
          type: data2 > 0 ? 'noteOn' : 'noteOff',
          deltaTime,
          channel,
          note: data1,
          velocity: data2,
        };
        break;

      case 0x8: // Note Off
        event = {
          type: 'noteOff',
          deltaTime,
          channel,
          note: data1,
          velocity: data2,
        };
        break;

      case 0xB: // Control Change
        event = {
          type: 'controlChange',
          deltaTime,
          channel,
          controller: data1,
          value: data2,
        };
        break;

      case 0xC: // Program Change
        event = {
          type: 'programChange',
          deltaTime,
          channel,
          program: data1,
        };
        break;

      case 0xE: // Pitch Bend
        const value = ((data2 << 7) | data1) - 8192;
        event = {
          type: 'pitchBend',
          deltaTime,
          channel,
          value,
        };
        break;
    }

    if (event) {
      this.events.push(event);
    }
  }

  private quantizeEvents(): void {
    // Simple quantization: snap to nearest grid division
    // In a full implementation, this would be more sophisticated
    const gridSize = this.options.quantizeValue!;

    this.events = this.events.map(event => ({
      ...event,
      deltaTime: Math.round(event.deltaTime / gridSize) * gridSize,
    }));
  }
}
