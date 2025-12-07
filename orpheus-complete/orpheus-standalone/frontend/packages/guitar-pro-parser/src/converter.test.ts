import { describe, it, expect } from 'vitest';
import { convertToMaestro } from './converter';
import type { GuitarProData } from './types';

describe('converter.ts', () => {
  const createMinimalGPData = (): GuitarProData => ({
    version: 'GP7',
    score: {
      title: 'Test Song',
      artist: 'Test Artist',
    },
    tracks: [],
    masterBars: [],
    bars: [],
    voices: [],
    beats: [],
    notes: [],
    rhythms: [],
  });

  describe('convertToMaestro()', () => {
    it('should convert minimal GP data to Maestro project', () => {
      const gpData = createMinimalGPData();

      const result = convertToMaestro(gpData);

      expect(result).toBeDefined();
      expect(result.project).toBeDefined();
      expect(result.project.metadata).toBeDefined();
    });

    it('should convert score metadata', () => {
      const gpData = createMinimalGPData();
      gpData.score.title = 'My Guitar Solo';
      gpData.score.artist = 'John Doe';
      gpData.score.album = 'Greatest Hits';

      const result = convertToMaestro(gpData);

      expect(result.project.metadata.title).toBe('My Guitar Solo');
      expect(result.project.metadata.artist).toBe('John Doe');
      expect(result.project.metadata.album).toBe('Greatest Hits');
    });

    it('should convert tempo from first master bar', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [{
        index: 0,
        tempo: 140,
      }];

      const result = convertToMaestro(gpData);

      expect(result.project.metadata.tempo).toBe(140);
    });

    it('should use default tempo when no master bars', () => {
      const gpData = createMinimalGPData();

      const result = convertToMaestro(gpData);

      expect(result.project.metadata.tempo).toBe(120);
    });

    it('should convert key signature from first master bar', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [{
        index: 0,
        key: { accidentalCount: 2, mode: 'Major' }, // D major
      }];

      const result = convertToMaestro(gpData);

      expect(result.project.metadata.key).toBe('D');
    });

    it('should convert minor key signature', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [{
        index: 0,
        key: { accidentalCount: 0, mode: 'Minor' }, // A minor
      }];

      const result = convertToMaestro(gpData);

      expect(result.project.metadata.key).toBe('A');
    });

    it('should convert flat key signatures', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [{
        index: 0,
        key: { accidentalCount: -2, mode: 'Major' }, // Bb major
      }];

      const result = convertToMaestro(gpData);

      expect(result.project.metadata.key).toBe('Bb');
    });

    it('should convert time signature', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [{
        index: 0,
        time: { numerator: 7, denominator: 8 },
      }];

      const result = convertToMaestro(gpData);

      expect(result.project.metadata.timeSignature).toEqual({ numerator: 7, denominator: 8 });
    });

    it('should use default time signature when not specified', () => {
      const gpData = createMinimalGPData();

      const result = convertToMaestro(gpData);

      expect(result.project.metadata.timeSignature).toEqual({ numerator: 4, denominator: 4 });
    });

    it('should convert guitar track', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Electric Guitar',
        color: { r: 255, g: 0, b: 0 },
        instrument: {
          type: 'guitar',
          strings: [
            { number: 1, tuning: 64 }, // E
            { number: 2, tuning: 59 }, // B
            { number: 3, tuning: 55 }, // G
            { number: 4, tuning: 50 }, // D
            { number: 5, tuning: 45 }, // A
            { number: 6, tuning: 40 }, // E
          ],
        },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
        tuning: [64, 59, 55, 50, 45, 40],
        capo: 0,
      }];

      const result = convertToMaestro(gpData);

      expect(result.project.composition.tracks).toBeDefined();
      expect(result.project.composition.tracks!.length).toBe(1);

      const track = result.project.composition.tracks![0];
      expect(track.name).toBe('Electric Guitar');
      expect(track.instrument.type).toBe('guitar');
    });

    it('should convert track color to hex', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Test',
        color: { r: 255, g: 128, b: 64 },
        instrument: { type: 'guitar', strings: [] },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.color).toBe('#ff8040');
    });

    it('should normalize track volume (0-15 to 0-1)', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Test',
        color: { r: 255, g: 0, b: 0 },
        instrument: { type: 'guitar', strings: [] },
        channel: 0,
        volume: 15, // Max volume
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.volume).toBe(1.0); // 15/15 = 1.0
    });

    it('should convert track balance to pan (-1 to 1)', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Test',
        color: { r: 255, g: 0, b: 0 },
        instrument: { type: 'guitar', strings: [] },
        channel: 0,
        volume: 15,
        balance: 16, // Far right
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.pan).toBe(1.0); // (16-8)/8 = 1.0
    });

    it('should convert MIDI tuning to note names', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Drop D Guitar',
        color: { r: 255, g: 0, b: 0 },
        instrument: {
          type: 'guitar',
          strings: Array.from({ length: 6 }, (_, i) => ({ number: i + 1, tuning: 40 + i * 5 })),
        },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
        tuning: [62, 59, 55, 50, 45, 40], // Drop D tuning
        capo: 0,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.instrument.tuning).toBeDefined();
      expect(track.instrument.tuning!.length).toBe(6);
    });

    it('should convert measures from master bars', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [
        { index: 0, tempo: 120 },
        { index: 1, tempo: 140 },
        { index: 2 },
      ];

      const result = convertToMaestro(gpData);

      expect(result.project.composition.measures).toBeDefined();
      expect(result.project.composition.measures!.length).toBe(3);
    });

    it('should convert measure with section marker', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [{
        index: 0,
        section: {
          text: 'Verse 1',
          color: { r: 255, g: 200, b: 100 },
        },
      }];

      const result = convertToMaestro(gpData);

      const measure = result.project.composition.measures![0];
      expect(measure.marker).toBeDefined();
      expect(measure.marker!.text).toBe('Verse 1');
      expect(measure.marker!.color).toBeDefined();
    });

    it('should convert measure with repeat markers', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [{
        index: 0,
        repeat: {
          start: true,
          end: true,
          count: 3,
        },
      }];

      const result = convertToMaestro(gpData);

      const measure = result.project.composition.measures![0];
      expect(measure.repeat).toBeDefined();
      expect(measure.repeat!.start).toBe(true);
      expect(measure.repeat!.end).toBe(true);
      expect(measure.repeat!.count).toBe(3);
    });

    it('should convert measure with alternate endings', () => {
      const gpData = createMinimalGPData();
      gpData.masterBars = [{
        index: 0,
        alternateEndings: [1, 2, 3],
      }];

      const result = convertToMaestro(gpData);

      const measure = result.project.composition.measures![0];
      expect(measure.alternateEndings).toEqual([1, 2, 3]);
    });

    it('should handle missing composition arrays', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Test',
        color: { r: 255, g: 0, b: 0 },
        instrument: { type: 'guitar', strings: [] },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
      }];

      const result = convertToMaestro(gpData);

      expect(result.project.composition.tracks).toBeDefined();
      expect(Array.isArray(result.project.composition.tracks)).toBe(true);
    });

    it('should convert bass track', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Bass',
        color: { r: 0, g: 255, b: 0 },
        instrument: { type: 'bass', strings: [] },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.instrument.type).toBe('bass');
    });

    it('should convert drums track', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Drums',
        color: { r: 0, g: 0, b: 255 },
        instrument: { type: 'drums', strings: [] },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.instrument.type).toBe('drums');
    });

    it('should convert multiple tracks', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [
        {
          id: 1,
          name: 'Guitar',
          color: { r: 255, g: 0, b: 0 },
          instrument: { type: 'guitar', strings: [] },
          channel: 0,
          volume: 15,
          balance: 8,
          chorus: 0,
          reverb: 0,
          phaser: 0,
          tremolo: 0,
        },
        {
          id: 2,
          name: 'Bass',
          color: { r: 0, g: 255, b: 0 },
          instrument: { type: 'bass', strings: [] },
          channel: 1,
          volume: 15,
          balance: 8,
          chorus: 0,
          reverb: 0,
          phaser: 0,
          tremolo: 0,
        },
      ];

      const result = convertToMaestro(gpData);

      expect(result.project.composition.tracks!.length).toBe(2);
      expect(result.project.composition.tracks![0].name).toBe('Guitar');
      expect(result.project.composition.tracks![1].name).toBe('Bass');
    });

    it('should include track effects', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Guitar',
        color: { r: 255, g: 0, b: 0 },
        instrument: { type: 'guitar', strings: [] },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 5,
        reverb: 7,
        phaser: 3,
        tremolo: 2,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.effects).toBeDefined();
      expect(track.effects.chorus).toBe(5);
      expect(track.effects.reverb).toBe(7);
      expect(track.effects.phaser).toBe(3);
      expect(track.effects.tremolo).toBe(2);
    });

    it('should set default track properties', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Guitar',
        color: { r: 255, g: 0, b: 0 },
        instrument: { type: 'guitar', strings: [] },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.muted).toBe(false);
      expect(track.solo).toBe(false);
      expect(track.type).toBe('instrument');
    });

    it('should handle capo setting', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Guitar',
        color: { r: 255, g: 0, b: 0 },
        instrument: { type: 'guitar', strings: [] },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
        capo: 2,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.instrument.capo).toBe(2);
    });

    it('should default capo to 0 when not specified', () => {
      const gpData = createMinimalGPData();
      gpData.tracks = [{
        id: 1,
        name: 'Guitar',
        color: { r: 255, g: 0, b: 0 },
        instrument: { type: 'guitar', strings: [] },
        channel: 0,
        volume: 15,
        balance: 8,
        chorus: 0,
        reverb: 0,
        phaser: 0,
        tremolo: 0,
      }];

      const result = convertToMaestro(gpData);

      const track = result.project.composition.tracks![0];
      expect(track.instrument.capo).toBe(0);
    });

    it('should convert all sharp key signatures correctly', () => {
      const sharpKeys = [
        { accidentals: 0, expected: 'C' },
        { accidentals: 1, expected: 'G' },
        { accidentals: 2, expected: 'D' },
        { accidentals: 3, expected: 'A' },
        { accidentals: 4, expected: 'E' },
        { accidentals: 5, expected: 'B' },
        { accidentals: 6, expected: 'F#' },
        { accidentals: 7, expected: 'C#' },
      ];

      sharpKeys.forEach(({ accidentals, expected }) => {
        const gpData = createMinimalGPData();
        gpData.masterBars = [{
          index: 0,
          key: { accidentalCount: accidentals, mode: 'Major' },
        }];

        const result = convertToMaestro(gpData);
        expect(result.project.metadata.key).toBe(expected);
      });
    });

    it('should convert all flat key signatures correctly', () => {
      const flatKeys = [
        { accidentals: -1, expected: 'F' },
        { accidentals: -2, expected: 'Bb' },
        { accidentals: -3, expected: 'Eb' },
        { accidentals: -4, expected: 'Ab' },
        { accidentals: -5, expected: 'Db' },
        { accidentals: -6, expected: 'Gb' },
        { accidentals: -7, expected: 'Cb' },
      ];

      flatKeys.forEach(({ accidentals, expected }) => {
        const gpData = createMinimalGPData();
        gpData.masterBars = [{
          index: 0,
          key: { accidentalCount: accidentals, mode: 'Major' },
        }];

        const result = convertToMaestro(gpData);
        expect(result.project.metadata.key).toBe(expected);
      });
    });
  });
});
