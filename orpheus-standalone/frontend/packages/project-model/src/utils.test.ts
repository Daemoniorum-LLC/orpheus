import { describe, it, expect } from 'vitest';
import {
  generateId,
  cloneProject,
  calculateProjectDuration,
  musicalTimeToSeconds,
  secondsToMusicalTime,
  mergeProjects,
  getTracksLinkedToScore,
  getMasterBus,
} from './utils';
import { createProject } from './project-factory';

describe('utils.ts - Project Utilities', () => {
  describe('generateId', () => {
    it('should generate unique IDs', () => {
      const id1 = generateId();
      const id2 = generateId();
      expect(id1).not.toBe(id2);
    });

    it('should generate string IDs', () => {
      const id = generateId();
      expect(typeof id).toBe('string');
    });

    it('should generate non-empty IDs', () => {
      const id = generateId();
      expect(id.length).toBeGreaterThan(0);
    });

    it('should generate many unique IDs', () => {
      const ids = new Set();
      for (let i = 0; i < 1000; i++) {
        ids.add(generateId());
      }
      expect(ids.size).toBe(1000);
    });
  });

  describe('cloneProject', () => {
    it('should create deep clone', () => {
      const original = createProject({ title: 'Original' });
      const clone = cloneProject(original);

      expect(clone).toEqual(original);
      expect(clone).not.toBe(original);
    });

    it('should not share references', () => {
      const original = createProject({ title: 'Original' });
      const clone = cloneProject(original);

      clone.project.metadata.title = 'Modified';
      expect(original.project.metadata.title).toBe('Original');
    });

    it('should clone all nested structures', () => {
      const original = createProject({ title: 'Test' });
      const clone = cloneProject(original);

      expect(clone.project.session).not.toBe(original.project.session);
      expect(clone.project.composition).not.toBe(original.project.composition);
      expect(clone.project.metadata).not.toBe(original.project.metadata);
    });

    it('should preserve all data', () => {
      const original = createProject({ title: 'Test', tempo: 140, key: 'Gm' });
      const clone = cloneProject(original);

      expect(clone.project.metadata.title).toBe('Test');
      expect(clone.project.metadata.tempo).toBe(140);
      expect(clone.project.metadata.key).toBe('Gm');
    });
  });

  describe('calculateProjectDuration', () => {
    it('should return 0 for empty project', () => {
      const project = createProject({ title: 'Test' });
      expect(calculateProjectDuration(project)).toBe(0);
    });

    it('should calculate duration from regions', () => {
      const project = createProject({ title: 'Test' });
      project.project.session.tracks = [{
        id: 'track-1',
        name: 'Track',
        type: 'audio',
        regions: [
          { id: 'r1', name: 'Region 1', startTime: 0, duration: 10, sourceStart: 0, sourceEnd: 10, fadeIn: 0, fadeOut: 0 },
        ],
        volume: 0,
        pan: 0,
        muted: false,
        solo: false,
        plugins: [],
        automation: [],
      }];

      expect(calculateProjectDuration(project)).toBe(10);
    });

    it('should find longest region', () => {
      const project = createProject({ title: 'Test' });
      project.project.session.tracks = [
        {
          id: 'track-1',
          name: 'Track 1',
          type: 'audio',
          regions: [
            { id: 'r1', name: 'R1', startTime: 0, duration: 10, sourceStart: 0, sourceEnd: 10, fadeIn: 0, fadeOut: 0 },
          ],
          volume: 0,
          pan: 0,
          muted: false,
          solo: false,
          plugins: [],
          automation: [],
        },
        {
          id: 'track-2',
          name: 'Track 2',
          type: 'audio',
          regions: [
            { id: 'r2', name: 'R2', startTime: 5, duration: 20, sourceStart: 0, sourceEnd: 20, fadeIn: 0, fadeOut: 0 },
          ],
          volume: 0,
          pan: 0,
          muted: false,
          solo: false,
          plugins: [],
          automation: [],
        },
      ];

      expect(calculateProjectDuration(project)).toBe(25); // 5 + 20
    });
  });

  describe('musicalTimeToSeconds', () => {
    it('should convert measure 0 beat 0 to 0 seconds', () => {
      const seconds = musicalTimeToSeconds(0, 0, 120, { numerator: 4, denominator: 4 });
      expect(seconds).toBe(0);
    });

    it('should convert 1 measure to seconds', () => {
      const seconds = musicalTimeToSeconds(1, 0, 120, { numerator: 4, denominator: 4 });
      expect(seconds).toBeCloseTo(2, 2); // 4 beats at 120 BPM = 2 seconds
    });

    it('should convert beats to seconds', () => {
      const seconds = musicalTimeToSeconds(0, 4, 120, { numerator: 4, denominator: 4 });
      expect(seconds).toBeCloseTo(2, 2);
    });

    it('should handle different tempos', () => {
      const seconds60 = musicalTimeToSeconds(1, 0, 60, { numerator: 4, denominator: 4 });
      const seconds120 = musicalTimeToSeconds(1, 0, 120, { numerator: 4, denominator: 4 });

      expect(seconds60).toBeCloseTo(4, 2);
      expect(seconds120).toBeCloseTo(2, 2);
    });

    it('should handle different time signatures', () => {
      const seconds44 = musicalTimeToSeconds(1, 0, 120, { numerator: 4, denominator: 4 });
      const seconds34 = musicalTimeToSeconds(1, 0, 120, { numerator: 3, denominator: 4 });

      expect(seconds44).toBeCloseTo(2, 2);
      expect(seconds34).toBeCloseTo(1.5, 2);
    });
  });

  describe('secondsToMusicalTime', () => {
    it('should convert 0 seconds to measure 0 beat 0', () => {
      const time = secondsToMusicalTime(0, 120, { numerator: 4, denominator: 4 });
      expect(time.measure).toBe(0);
      expect(time.beat).toBe(0);
    });

    it('should convert 2 seconds to 1 measure', () => {
      const time = secondsToMusicalTime(2, 120, { numerator: 4, denominator: 4 });
      expect(time.measure).toBe(1);
      expect(time.beat).toBeCloseTo(0, 1);
    });

    it('should roundtrip with musicalTimeToSeconds', () => {
      const measure = 2;
      const beat = 3;
      const tempo = 120;
      const timeSignature = { numerator: 4, denominator: 4 };

      const seconds = musicalTimeToSeconds(measure, beat, tempo, timeSignature);
      const result = secondsToMusicalTime(seconds, tempo, timeSignature);

      expect(result.measure).toBe(measure);
      expect(result.beat).toBeCloseTo(beat, 1);
    });

    it('should handle different tempos', () => {
      const time60 = secondsToMusicalTime(4, 60, { numerator: 4, denominator: 4 });
      const time120 = secondsToMusicalTime(2, 120, { numerator: 4, denominator: 4 });

      expect(time60.measure).toBe(1);
      expect(time120.measure).toBe(1);
    });
  });

  describe('mergeProjects', () => {
    it('should merge track from incoming project', () => {
      const base = createProject({ title: 'Base' });
      const incoming = createProject({ title: 'Incoming' });

      incoming.project.session.tracks = [{
        id: 'new-track',
        name: 'New Track',
        type: 'audio',
        regions: [],
        volume: 0,
        pan: 0,
        muted: false,
        solo: false,
        plugins: [],
        automation: [],
      }];

      const merged = mergeProjects(base, incoming);
      expect(merged.project.session.tracks).toHaveLength(1);
      expect(merged.project.session.tracks[0].id).toBe('new-track');
    });

    it('should not modify base project', () => {
      const base = createProject({ title: 'Base' });
      const incoming = createProject({ title: 'Incoming' });

      mergeProjects(base, incoming);
      expect(base.project.session.tracks).toHaveLength(0);
    });

    it('should skip duplicate track IDs', () => {
      const base = createProject({ title: 'Base' });
      base.project.session.tracks = [{
        id: 'track-1',
        name: 'Track 1',
        type: 'audio',
        regions: [],
        volume: 0,
        pan: 0,
        muted: false,
        solo: false,
        plugins: [],
        automation: [],
      }];

      const incoming = createProject({ title: 'Incoming' });
      incoming.project.session.tracks = [{
        id: 'track-1',
        name: 'Different Name',
        type: 'audio',
        regions: [],
        volume: 0,
        pan: 0,
        muted: false,
        solo: false,
        plugins: [],
        automation: [],
      }];

      const merged = mergeProjects(base, incoming);
      expect(merged.project.session.tracks).toHaveLength(1);
      expect(merged.project.session.tracks[0].name).toBe('Track 1');
    });

    it('should update modified timestamp', () => {
      const base = createProject({ title: 'Base' });
      const incoming = createProject({ title: 'Incoming' });

      const merged = mergeProjects(base, incoming);
      expect(merged.project.metadata.modified).not.toBe(base.project.metadata.modified);
    });

    it('should merge scores', () => {
      const base = createProject({ title: 'Base' });
      const incoming = createProject({ title: 'Incoming' });

      incoming.project.composition.scores = [{
        id: 'score-1',
        title: 'Guitar',
        instrument: 'electric-guitar',
        tuning: ['E', 'A', 'D', 'G', 'B', 'E'],
        tracks: [],
      }];

      const merged = mergeProjects(base, incoming);
      expect(merged.project.composition.scores).toHaveLength(1);
    });
  });

  describe('getTracksLinkedToScore', () => {
    it('should return linked tracks', () => {
      const project = createProject({ title: 'Test' });
      project.project.session.tracks = [
        { id: 'track-1', name: 'Track 1', type: 'instrument', linkedScoreId: 'score-1', regions: [], volume: 0, pan: 0, muted: false, solo: false, plugins: [], automation: [] },
        { id: 'track-2', name: 'Track 2', type: 'instrument', linkedScoreId: 'score-2', regions: [], volume: 0, pan: 0, muted: false, solo: false, plugins: [], automation: [] },
      ];

      const linked = getTracksLinkedToScore(project, 'score-1');
      expect(linked).toHaveLength(1);
      expect(linked[0].id).toBe('track-1');
    });

    it('should return empty array when no tracks linked', () => {
      const project = createProject({ title: 'Test' });
      const linked = getTracksLinkedToScore(project, 'score-1');
      expect(linked).toHaveLength(0);
    });
  });

  describe('getMasterBus', () => {
    it('should return master bus', () => {
      const project = createProject({ title: 'Test' });
      const masterBus = getMasterBus(project);

      expect(masterBus).toBeDefined();
      expect(masterBus?.type).toBe('master');
    });

    it('should return undefined if no master bus', () => {
      const project = createProject({ title: 'Test' });
      project.project.session.buses = [];

      const masterBus = getMasterBus(project);
      expect(masterBus).toBeUndefined();
    });
  });
});
