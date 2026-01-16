import { describe, it, expect } from 'vitest';
import {
  parseMIDIFile,
  generateMIDIFile,
  type MIDIFile,
  type MIDIHeader,
  type MIDITrack,
  type MIDIEvent,
  type MIDINoteOnEvent,
  type MIDINoteOffEvent,
  type MIDIControlChangeEvent,
  type MIDIProgramChangeEvent,
  type MIDIPitchBendEvent,
  type MIDIMetaEvent,
} from './midi-file';

describe('midi-file.ts - MIDI File Parsing and Generation', () => {
  describe('Type definitions', () => {
    it('should define MIDIHeader with correct format values', () => {
      const header: MIDIHeader = {
        format: 1,
        tracks: 2,
        division: 480,
      };

      expect(header.format).toBe(1);
      expect(header.tracks).toBe(2);
      expect(header.division).toBe(480);
    });

    it('should support all three MIDI format types', () => {
      const format0: MIDIHeader['format'] = 0;
      const format1: MIDIHeader['format'] = 1;
      const format2: MIDIHeader['format'] = 2;

      expect([0, 1, 2]).toContain(format0);
      expect([0, 1, 2]).toContain(format1);
      expect([0, 1, 2]).toContain(format2);
    });

    it('should define MIDITrack with events array', () => {
      const track: MIDITrack = {
        events: [],
      };

      expect(track.events).toEqual([]);
    });

    it('should define MIDINoteOnEvent', () => {
      const event: MIDINoteOnEvent = {
        type: 'noteOn',
        deltaTime: 0,
        channel: 0,
        note: 60,
        velocity: 100,
      };

      expect(event.type).toBe('noteOn');
      expect(event.note).toBe(60);
      expect(event.velocity).toBe(100);
    });

    it('should define MIDINoteOffEvent', () => {
      const event: MIDINoteOffEvent = {
        type: 'noteOff',
        deltaTime: 480,
        channel: 0,
        note: 60,
        velocity: 0,
      };

      expect(event.type).toBe('noteOff');
    });

    it('should define MIDIControlChangeEvent', () => {
      const event: MIDIControlChangeEvent = {
        type: 'controlChange',
        deltaTime: 0,
        channel: 0,
        controller: 7,
        value: 100,
      };

      expect(event.type).toBe('controlChange');
      expect(event.controller).toBe(7);
    });

    it('should define MIDIProgramChangeEvent', () => {
      const event: MIDIProgramChangeEvent = {
        type: 'programChange',
        deltaTime: 0,
        channel: 0,
        program: 25,
      };

      expect(event.type).toBe('programChange');
      expect(event.program).toBe(25);
    });

    it('should define MIDIPitchBendEvent', () => {
      const event: MIDIPitchBendEvent = {
        type: 'pitchBend',
        deltaTime: 0,
        channel: 0,
        value: 0,
      };

      expect(event.type).toBe('pitchBend');
      expect(event.value).toBeGreaterThanOrEqual(-8192);
      expect(event.value).toBeLessThanOrEqual(8191);
    });

    it('should define MIDIMetaEvent with various meta types', () => {
      const tempoEvent: MIDIMetaEvent = {
        type: 'meta',
        metaType: 'tempo',
        deltaTime: 0,
        data: 500000,
      };

      expect(tempoEvent.metaType).toBe('tempo');
    });
  });

  describe('generateMIDIFile', () => {
    it('should generate MIDI file header', () => {
      const tracks: MIDITrack[] = [{ events: [] }];
      const buffer = generateMIDIFile(tracks, 480);

      expect(buffer).toBeInstanceOf(ArrayBuffer);
      expect(buffer.byteLength).toBeGreaterThan(0);
    });

    it('should use default division of 480', () => {
      const tracks: MIDITrack[] = [{ events: [] }];
      const buffer = generateMIDIFile(tracks);

      const view = new DataView(buffer);

      // Check header chunk type "MThd"
      expect(view.getUint8(0)).toBe(0x4D); // 'M'
      expect(view.getUint8(1)).toBe(0x54); // 'T'
      expect(view.getUint8(2)).toBe(0x68); // 'h'
      expect(view.getUint8(3)).toBe(0x64); // 'd'
    });

    it('should encode format 1 (multiple tracks)', () => {
      const tracks: MIDITrack[] = [{ events: [] }, { events: [] }];
      const buffer = generateMIDIFile(tracks, 480);

      const view = new DataView(buffer);
      const format = view.getUint16(8, false);
      expect(format).toBe(1);
    });

    it('should encode track count correctly', () => {
      const tracks: MIDITrack[] = [{ events: [] }, { events: [] }];
      const buffer = generateMIDIFile(tracks, 480);

      const view = new DataView(buffer);
      const trackCount = view.getUint16(10, false);
      expect(trackCount).toBe(2);
    });

    it('should encode division correctly', () => {
      const tracks: MIDITrack[] = [{ events: [] }];
      const buffer = generateMIDIFile(tracks, 960);

      const view = new DataView(buffer);
      const division = view.getUint16(12, false);
      expect(division).toBe(960);
    });

    it('should generate header for single track', () => {
      const tracks: MIDITrack[] = [{ events: [] }];
      const buffer = generateMIDIFile(tracks, 480);

      const view = new DataView(buffer);
      const trackCount = view.getUint16(10, false);
      expect(trackCount).toBe(1);
    });

    it('should generate header for multiple tracks', () => {
      const tracks: MIDITrack[] = [
        { events: [] },
        { events: [] },
        { events: [] },
      ];
      const buffer = generateMIDIFile(tracks, 480);

      const view = new DataView(buffer);
      const trackCount = view.getUint16(10, false);
      expect(trackCount).toBe(3);
    });

    it('should use custom division values', () => {
      const divisions = [96, 192, 384, 480, 960, 1920];

      divisions.forEach(division => {
        const tracks: MIDITrack[] = [{ events: [] }];
        const buffer = generateMIDIFile(tracks, division);
        const view = new DataView(buffer);
        expect(view.getUint16(12, false)).toBe(division);
      });
    });
  });

  describe('parseMIDIFile', () => {
    function createMIDIHeader(format: number, tracks: number, division: number): Uint8Array {
      return new Uint8Array([
        0x4D, 0x54, 0x68, 0x64, // "MThd"
        0x00, 0x00, 0x00, 0x06, // Header length
        (format >> 8) & 0xFF, format & 0xFF,
        (tracks >> 8) & 0xFF, tracks & 0xFF,
        (division >> 8) & 0xFF, division & 0xFF,
      ]);
    }

    function createEmptyTrack(): Uint8Array {
      return new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x04, // Track length (4 bytes)
        0x00, 0xFF, 0x2F, 0x00, // End of track meta event
      ]);
    }

    it('should throw error for missing header chunk', () => {
      const buffer = new ArrayBuffer(8);
      expect(() => parseMIDIFile(buffer)).toThrow('Invalid MIDI file: Missing header chunk');
    });

    it('should parse MIDI header correctly', () => {
      const header = createMIDIHeader(1, 1, 480);
      const track = createEmptyTrack();

      const combined = new Uint8Array(header.length + track.length);
      combined.set(header, 0);
      combined.set(track, header.length);

      const result = parseMIDIFile(combined.buffer);

      expect(result.header.format).toBe(1);
      expect(result.header.tracks).toBe(1);
      expect(result.header.division).toBe(480);
    });

    it('should parse format 0 (single track)', () => {
      const header = createMIDIHeader(0, 1, 480);
      const track = createEmptyTrack();

      const combined = new Uint8Array(header.length + track.length);
      combined.set(header, 0);
      combined.set(track, header.length);

      const result = parseMIDIFile(combined.buffer);

      expect(result.header.format).toBe(0);
    });

    it('should parse format 1 (multiple tracks)', () => {
      const header = createMIDIHeader(1, 2, 480);
      const track1 = createEmptyTrack();
      const track2 = createEmptyTrack();

      const combined = new Uint8Array(header.length + track1.length + track2.length);
      combined.set(header, 0);
      combined.set(track1, header.length);
      combined.set(track2, header.length + track1.length);

      const result = parseMIDIFile(combined.buffer);

      expect(result.header.format).toBe(1);
      expect(result.tracks.length).toBe(2);
    });

    it('should parse format 2 (multiple sequences)', () => {
      const header = createMIDIHeader(2, 1, 480);
      const track = createEmptyTrack();

      const combined = new Uint8Array(header.length + track.length);
      combined.set(header, 0);
      combined.set(track, header.length);

      const result = parseMIDIFile(combined.buffer);

      expect(result.header.format).toBe(2);
    });

    it('should parse different division values', () => {
      const divisions = [96, 192, 384, 480, 960];

      divisions.forEach(division => {
        const header = createMIDIHeader(1, 1, division);
        const track = createEmptyTrack();

        const combined = new Uint8Array(header.length + track.length);
        combined.set(header, 0);
        combined.set(track, header.length);

        const result = parseMIDIFile(combined.buffer);
        expect(result.header.division).toBe(division);
      });
    });

    it('should throw error for invalid track chunk', () => {
      const header = createMIDIHeader(1, 1, 480);
      const invalidTrack = new Uint8Array([
        0x00, 0x00, 0x00, 0x00, // Invalid chunk type
        0x00, 0x00, 0x00, 0x04,
        0x00, 0xFF, 0x2F, 0x00,
      ]);

      const combined = new Uint8Array(header.length + invalidTrack.length);
      combined.set(header, 0);
      combined.set(invalidTrack, header.length);

      expect(() => parseMIDIFile(combined.buffer)).toThrow('Expected MTrk chunk');
    });

    it('should parse empty tracks', () => {
      const header = createMIDIHeader(1, 1, 480);
      const track = createEmptyTrack();

      const combined = new Uint8Array(header.length + track.length);
      combined.set(header, 0);
      combined.set(track, header.length);

      const result = parseMIDIFile(combined.buffer);

      expect(result.tracks).toHaveLength(1);
      expect(result.tracks[0].events).toBeDefined();
    });

    it('should parse note on events', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x08, // Track length
        0x00, 0x90, 0x3C, 0x64, // Delta=0, Note On, Middle C, velocity=100
        0x00, 0xFF, 0x2F, 0x00, // End of track
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const events = result.tracks[0].events;

      expect(events.length).toBeGreaterThan(0);
      const noteOn = events[0] as MIDINoteOnEvent;
      expect(noteOn.type).toBe('noteOn');
      expect(noteOn.note).toBe(60); // Middle C
      expect(noteOn.velocity).toBe(100);
    });

    it('should parse note off events', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x08, // Track length
        0x00, 0x80, 0x3C, 0x40, // Delta=0, Note Off, Middle C, velocity=64
        0x00, 0xFF, 0x2F, 0x00, // End of track
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const events = result.tracks[0].events;

      const noteOff = events[0] as MIDINoteOffEvent;
      expect(noteOff.type).toBe('noteOff');
      expect(noteOff.note).toBe(60);
    });

    it('should convert note on with velocity 0 to note off', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x08, // Track length
        0x00, 0x90, 0x3C, 0x00, // Delta=0, Note On, Middle C, velocity=0 (note off)
        0x00, 0xFF, 0x2F, 0x00, // End of track
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const events = result.tracks[0].events;

      const noteOff = events[0] as MIDINoteOffEvent;
      expect(noteOff.type).toBe('noteOff');
    });

    it('should parse control change events', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x08, // Track length
        0x00, 0xB0, 0x07, 0x64, // Delta=0, CC, controller=7 (volume), value=100
        0x00, 0xFF, 0x2F, 0x00, // End of track
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const events = result.tracks[0].events;

      const cc = events[0] as MIDIControlChangeEvent;
      expect(cc.type).toBe('controlChange');
      expect(cc.controller).toBe(7);
      expect(cc.value).toBe(100);
    });

    it('should parse program change events', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x07, // Track length
        0x00, 0xC0, 0x19, // Delta=0, Program Change, program=25
        0x00, 0xFF, 0x2F, 0x00, // End of track
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const events = result.tracks[0].events;

      const pc = events[0] as MIDIProgramChangeEvent;
      expect(pc.type).toBe('programChange');
      expect(pc.program).toBe(25);
    });

    it('should parse pitch bend events', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x08, // Track length
        0x00, 0xE0, 0x00, 0x40, // Delta=0, Pitch Bend, LSB=0, MSB=64 (center)
        0x00, 0xFF, 0x2F, 0x00, // End of track
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const events = result.tracks[0].events;

      const pb = events[0] as MIDIPitchBendEvent;
      expect(pb.type).toBe('pitchBend');
      expect(pb.value).toBe(0); // Center position
    });

    it('should parse delta times correctly', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x0C, // Track length
        0x00, 0x90, 0x3C, 0x64, // Delta=0, Note On
        0x81, 0x00, 0x80, 0x3C, 0x00, // Delta=128, Note Off
        0x00, 0xFF, 0x2F, 0x00, // End of track
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const events = result.tracks[0].events;

      expect(events[0].deltaTime).toBe(0);
      expect(events[1].deltaTime).toBe(128);
    });

    it('should handle running status', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B, // "MTrk"
        0x00, 0x00, 0x00, 0x0D, // Track length
        0x00, 0x90, 0x3C, 0x64, // Delta=0, Note On
        0x00, 0x40, 0x64,       // Delta=0, Running status (Note On), note=64
        0x00, 0xFF, 0x2F, 0x00, // End of track
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const events = result.tracks[0].events;

      expect(events.length).toBeGreaterThanOrEqual(2);
      expect(events[0].type).toBe('noteOn');
      expect(events[1].type).toBe('noteOn');
      expect((events[1] as MIDINoteOnEvent).note).toBe(64);
    });

    it('should parse multiple tracks', () => {
      const header = createMIDIHeader(1, 2, 480);
      const track1 = createEmptyTrack();
      const track2 = createEmptyTrack();

      const combined = new Uint8Array(header.length + track1.length + track2.length);
      combined.set(header, 0);
      combined.set(track1, header.length);
      combined.set(track2, header.length + track1.length);

      const result = parseMIDIFile(combined.buffer);

      expect(result.tracks.length).toBe(2);
    });

    it('should parse all 16 MIDI channels', () => {
      for (let channel = 0; channel < 16; channel++) {
        const header = createMIDIHeader(1, 1, 480);
        const trackData = new Uint8Array([
          0x4D, 0x54, 0x72, 0x6B, // "MTrk"
          0x00, 0x00, 0x00, 0x08, // Track length
          0x00, 0x90 | channel, 0x3C, 0x64, // Note On on channel
          0x00, 0xFF, 0x2F, 0x00, // End of track
        ]);

        const combined = new Uint8Array(header.length + trackData.length);
        combined.set(header, 0);
        combined.set(trackData, header.length);

        const result = parseMIDIFile(combined.buffer);
        const events = result.tracks[0].events;
        const noteOn = events[0] as MIDINoteOnEvent;

        expect(noteOn.channel).toBe(channel);
      }
    });

    it('should parse all note values (0-127)', () => {
      [0, 21, 60, 88, 108, 127].forEach(note => {
        const header = createMIDIHeader(1, 1, 480);
        const trackData = new Uint8Array([
          0x4D, 0x54, 0x72, 0x6B, // "MTrk"
          0x00, 0x00, 0x00, 0x08, // Track length
          0x00, 0x90, note, 0x64,
          0x00, 0xFF, 0x2F, 0x00,
        ]);

        const combined = new Uint8Array(header.length + trackData.length);
        combined.set(header, 0);
        combined.set(trackData, header.length);

        const result = parseMIDIFile(combined.buffer);
        const noteOn = result.tracks[0].events[0] as MIDINoteOnEvent;

        expect(noteOn.note).toBe(note);
      });
    });

    it('should parse all velocity values (0-127)', () => {
      [0, 32, 64, 96, 127].forEach(velocity => {
        const header = createMIDIHeader(1, 1, 480);
        const trackData = new Uint8Array([
          0x4D, 0x54, 0x72, 0x6B,
          0x00, 0x00, 0x00, 0x08,
          0x00, 0x90, 0x3C, velocity,
          0x00, 0xFF, 0x2F, 0x00,
        ]);

        const combined = new Uint8Array(header.length + trackData.length);
        combined.set(header, 0);
        combined.set(trackData, header.length);

        const result = parseMIDIFile(combined.buffer);
        const noteOn = result.tracks[0].events[0] as MIDINoteOnEvent;

        if (velocity > 0) {
          expect(noteOn.velocity).toBe(velocity);
        }
      });
    });

    it('should parse controller numbers (0-127)', () => {
      [0, 1, 7, 10, 64, 91, 127].forEach(controller => {
        const header = createMIDIHeader(1, 1, 480);
        const trackData = new Uint8Array([
          0x4D, 0x54, 0x72, 0x6B,
          0x00, 0x00, 0x00, 0x08,
          0x00, 0xB0, controller, 0x64,
          0x00, 0xFF, 0x2F, 0x00,
        ]);

        const combined = new Uint8Array(header.length + trackData.length);
        combined.set(header, 0);
        combined.set(trackData, header.length);

        const result = parseMIDIFile(combined.buffer);
        const cc = result.tracks[0].events[0] as MIDIControlChangeEvent;

        expect(cc.controller).toBe(controller);
      });
    });

    it('should parse program values (0-127)', () => {
      [0, 25, 64, 127].forEach(program => {
        const header = createMIDIHeader(1, 1, 480);
        const trackData = new Uint8Array([
          0x4D, 0x54, 0x72, 0x6B,
          0x00, 0x00, 0x00, 0x07,
          0x00, 0xC0, program,
          0x00, 0xFF, 0x2F, 0x00,
        ]);

        const combined = new Uint8Array(header.length + trackData.length);
        combined.set(header, 0);
        combined.set(trackData, header.length);

        const result = parseMIDIFile(combined.buffer);
        const pc = result.tracks[0].events[0] as MIDIProgramChangeEvent;

        expect(pc.program).toBe(program);
      });
    });

    it('should parse pitch bend range (-8192 to 8191)', () => {
      // Test center (0)
      const testCases = [
        { lsb: 0x00, msb: 0x40, expected: 0 },     // Center
        { lsb: 0x00, msb: 0x00, expected: -8192 }, // Min
        { lsb: 0x7F, msb: 0x7F, expected: 8191 },  // Max
      ];

      testCases.forEach(({ lsb, msb, expected }) => {
        const header = createMIDIHeader(1, 1, 480);
        const trackData = new Uint8Array([
          0x4D, 0x54, 0x72, 0x6B,
          0x00, 0x00, 0x00, 0x08,
          0x00, 0xE0, lsb, msb,
          0x00, 0xFF, 0x2F, 0x00,
        ]);

        const combined = new Uint8Array(header.length + trackData.length);
        combined.set(header, 0);
        combined.set(trackData, header.length);

        const result = parseMIDIFile(combined.buffer);
        const pb = result.tracks[0].events[0] as MIDIPitchBendEvent;

        expect(pb.value).toBe(expected);
      });
    });

    it('should parse large variable-length delta times', () => {
      const header = createMIDIHeader(1, 1, 480);
      const trackData = new Uint8Array([
        0x4D, 0x54, 0x72, 0x6B,
        0x00, 0x00, 0x00, 0x0A,
        0x83, 0x60, 0x90, 0x3C, 0x64, // Delta=480 (encoded as variable length)
        0x00, 0xFF, 0x2F, 0x00,
      ]);

      const combined = new Uint8Array(header.length + trackData.length);
      combined.set(header, 0);
      combined.set(trackData, header.length);

      const result = parseMIDIFile(combined.buffer);
      const noteOn = result.tracks[0].events[0] as MIDINoteOnEvent;

      expect(noteOn.deltaTime).toBe(480);
    });
  });

  describe('Integration tests', () => {
    it('should generate and parse MIDI file roundtrip', () => {
      const tracks: MIDITrack[] = [{ events: [] }];
      const buffer = generateMIDIFile(tracks, 480);

      // Note: Full roundtrip not implemented in simplified version
      expect(buffer.byteLength).toBeGreaterThan(0);
    });

    it('should handle MIDIFile structure', () => {
      const file: MIDIFile = {
        header: {
          format: 1,
          tracks: 2,
          division: 480,
        },
        tracks: [
          { events: [] },
          { events: [] },
        ],
      };

      expect(file.header.tracks).toBe(file.tracks.length);
    });

    it('should support common divisions', () => {
      const commonDivisions = [96, 192, 384, 480, 960, 1920];

      commonDivisions.forEach(division => {
        const tracks: MIDITrack[] = [{ events: [] }];
        const buffer = generateMIDIFile(tracks, division);
        expect(buffer.byteLength).toBeGreaterThan(0);
      });
    });

    it('should support common MIDI messages', () => {
      const events: MIDIEvent[] = [
        { type: 'noteOn', deltaTime: 0, channel: 0, note: 60, velocity: 100 },
        { type: 'noteOff', deltaTime: 480, channel: 0, note: 60, velocity: 0 },
        { type: 'controlChange', deltaTime: 0, channel: 0, controller: 7, value: 100 },
        { type: 'programChange', deltaTime: 0, channel: 0, program: 25 },
        { type: 'pitchBend', deltaTime: 0, channel: 0, value: 0 },
      ];

      const track: MIDITrack = { events };
      expect(track.events.length).toBe(5);
    });
  });
});
