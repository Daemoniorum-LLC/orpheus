import { describe, it, expect } from 'vitest';
import * as TimelineSync from './index';

describe('index.ts - Package Exports', () => {
  describe('Timeline exports', () => {
    it('should export Timeline class', () => {
      expect(TimelineSync.Timeline).toBeDefined();
      expect(typeof TimelineSync.Timeline).toBe('function');
    });

    it('should be able to create Timeline instance', () => {
      const timeline = new TimelineSync.Timeline({
        initialTempo: 120,
        timeSignature: { numerator: 4, denominator: 4 },
        sampleRate: 48000,
      });

      expect(timeline).toBeInstanceOf(TimelineSync.Timeline);
    });
  });

  describe('TempoMap exports', () => {
    it('should export TempoMap class', () => {
      expect(TimelineSync.TempoMap).toBeDefined();
      expect(typeof TimelineSync.TempoMap).toBe('function');
    });

    it('should be able to create TempoMap instance', () => {
      const tempoMap = new TimelineSync.TempoMap(120);
      expect(tempoMap).toBeInstanceOf(TimelineSync.TempoMap);
    });
  });

  describe('TimeConverter exports', () => {
    it('should export TimeConverter class', () => {
      expect(TimelineSync.TimeConverter).toBeDefined();
      expect(typeof TimelineSync.TimeConverter).toBe('function');
    });

    it('should be able to create TimeConverter instance', () => {
      const tempoMap = new TimelineSync.TempoMap(120);
      const converter = new TimelineSync.TimeConverter(tempoMap, { numerator: 4, denominator: 4 });
      expect(converter).toBeInstanceOf(TimelineSync.TimeConverter);
    });
  });

  describe('MarkerManager exports', () => {
    it('should export MarkerManager class', () => {
      expect(TimelineSync.MarkerManager).toBeDefined();
      expect(typeof TimelineSync.MarkerManager).toBe('function');
    });

    it('should be able to create MarkerManager instance', () => {
      const manager = new TimelineSync.MarkerManager();
      expect(manager).toBeInstanceOf(TimelineSync.MarkerManager);
    });
  });

  describe('Playhead exports', () => {
    it('should export Playhead class', () => {
      expect(TimelineSync.Playhead).toBeDefined();
      expect(typeof TimelineSync.Playhead).toBe('function');
    });

    it('should be able to create Playhead instance', () => {
      const playhead = new TimelineSync.Playhead();
      expect(playhead).toBeInstanceOf(TimelineSync.Playhead);
    });
  });

  describe('Integration', () => {
    it('should have all core components', () => {
      expect(TimelineSync.Timeline).toBeDefined();
      expect(TimelineSync.TempoMap).toBeDefined();
      expect(TimelineSync.TimeConverter).toBeDefined();
      expect(TimelineSync.MarkerManager).toBeDefined();
      expect(TimelineSync.Playhead).toBeDefined();
    });

    it('should support creating complete timeline system', () => {
      const timeline = new TimelineSync.Timeline({
        initialTempo: 135,
        timeSignature: { numerator: 4, denominator: 4 },
        sampleRate: 48000,
      });

      const tempoMap = new TimelineSync.TempoMap(140);
      const markerManager = new TimelineSync.MarkerManager();
      const playhead = new TimelineSync.Playhead();

      expect(timeline).toBeDefined();
      expect(tempoMap).toBeDefined();
      expect(markerManager).toBeDefined();
      expect(playhead).toBeDefined();
    });

    it('should provide complete synchronization system', () => {
      // Timeline is the main integration point
      const timeline = new TimelineSync.Timeline({
        initialTempo: 120,
        timeSignature: { numerator: 4, denominator: 4 },
        sampleRate: 48000,
      });

      // Can also use individual components
      const standalone = {
        tempoMap: new TimelineSync.TempoMap(120),
        markerManager: new TimelineSync.MarkerManager(),
        playhead: new TimelineSync.Playhead(),
      };

      expect(standalone.tempoMap).toBeInstanceOf(TimelineSync.TempoMap);
      expect(standalone.markerManager).toBeInstanceOf(TimelineSync.MarkerManager);
      expect(standalone.playhead).toBeInstanceOf(TimelineSync.Playhead);
    });
  });

  describe('Usage examples', () => {
    it('should create timeline with custom configuration', () => {
      const timeline = new TimelineSync.Timeline({
        initialTempo: 140,
        timeSignature: { numerator: 3, denominator: 4 },
        sampleRate: 44100,
      });

      expect(timeline).toBeInstanceOf(TimelineSync.Timeline);
    });

    it('should create tempo map with tempo changes', () => {
      const tempoMap = new TimelineSync.TempoMap(120);
      tempoMap.setTempoAt(16, 140);
      tempoMap.setTempoAt(32, 160);

      expect(tempoMap.getTempoAt(0)).toBe(120);
      expect(tempoMap.getTempoAt(16)).toBe(140);
      expect(tempoMap.getTempoAt(32)).toBe(160);
    });

    it('should create time converter for conversions', () => {
      const tempoMap = new TimelineSync.TempoMap(120);
      const converter = new TimelineSync.TimeConverter(tempoMap, { numerator: 4, denominator: 4 });

      const position = converter.beatsToMusical(4);
      expect(position.measure).toBe(1);
    });

    it('should create marker manager for song structure', () => {
      const manager = new TimelineSync.MarkerManager();

      manager.addMarker({
        position: { measure: 0, beat: 0, tick: 0, totalBeats: 0 },
        name: 'Intro',
        type: 'section',
        color: '#3b82f6',
      });

      expect(manager.getAll()).toHaveLength(1);
    });

    it('should create playhead for playback control', () => {
      const playhead = new TimelineSync.Playhead();

      playhead.play();
      expect(playhead.isPlaying()).toBe(true);

      playhead.pause();
      expect(playhead.isPlaying()).toBe(false);
    });
  });
});
