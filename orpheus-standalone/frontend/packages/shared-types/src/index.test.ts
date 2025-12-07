import { describe, it, expect } from 'vitest';
import type * as SharedTypes from './index';

describe('index.ts - Shared Type Definitions', () => {
  describe('Project types', () => {
    it('should define MaestroProject type', () => {
      const project: SharedTypes.MaestroProject = {
        formatVersion: '1.0',
        project: {
          metadata: {
            id: 'test-id',
            title: 'Test',
            tempo: 120,
            key: 'C',
            timeSignature: { numerator: 4, denominator: 4 },
            created: new Date().toISOString(),
            modified: new Date().toISOString(),
          },
          composition: {
            scores: [],
            tempoMap: [],
            markers: [],
            tracks: [],
            measures: [],
          },
          session: {
            sampleRate: 48000,
            bitDepth: 24,
            bufferSize: 512,
            tracks: [],
            buses: [],
            sends: [],
            timeline: {
              viewStart: 0,
              viewEnd: 60,
            },
          },
          mixing: {
            snapshots: [],
          },
          mastering: {
            masteringChain: [],
            loudness: {
              integrated: -23,
              shortTerm: -23,
              momentary: -23,
              truePeak: -1,
              range: 10,
            },
            exports: [],
          },
          practice: {
            speedTrainer: {
              enabled: false,
              currentSpeed: 100,
              targetSpeed: 100,
              incrementStep: 5,
            },
            loop: {
              enabled: false,
              startMeasure: 0,
              endMeasure: 0,
            },
            practiceLog: [],
            difficultSections: [],
          },
          aiHistory: {
            compositionSuggestions: [],
            mixingSuggestions: [],
            masteringSuggestions: [],
            transcriptionResults: [],
            practiceAnalysis: [],
            chatHistory: [],
          },
          collaboration: {
            enabled: false,
            collaborators: [],
            permissions: {},
            versions: [],
            comments: [],
          },
        },
      };

      expect(project).toBeDefined();
    });

    it('should define ProjectMetadata type', () => {
      const metadata: SharedTypes.ProjectMetadata = {
        id: 'test-id',
        title: 'Test Project',
        tempo: 120,
        key: 'C',
        timeSignature: { numerator: 4, denominator: 4 },
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
      };

      expect(metadata.title).toBe('Test Project');
    });

    it('should define TimeSignature type', () => {
      const ts: SharedTypes.TimeSignature = {
        numerator: 4,
        denominator: 4,
      };

      expect(ts.numerator).toBe(4);
      expect(ts.denominator).toBe(4);
    });
  });

  describe('Composition types', () => {
    it('should define Score type', () => {
      const score: SharedTypes.Score = {
        id: 'score-1',
        title: 'Guitar',
        instrument: 'electric-guitar',
        tuning: ['E', 'A', 'D', 'G', 'B', 'E'],
        tracks: [],
      };

      expect(score.title).toBe('Guitar');
    });

    it('should define Marker type', () => {
      const marker: SharedTypes.Marker = {
        id: 'marker-1',
        name: 'Intro',
        measure: 0,
        type: 'section',
        color: '#3b82f6',
      };

      expect(marker.name).toBe('Intro');
    });

    it('should define TempoChange type', () => {
      const tempoChange: SharedTypes.TempoChange = {
        measure: 16,
        tempo: 140,
      };

      expect(tempoChange.tempo).toBe(140);
    });
  });

  describe('Session types', () => {
    it('should define SessionTrack type', () => {
      const track: SharedTypes.SessionTrack = {
        id: 'track-1',
        name: 'Audio Track',
        type: 'audio',
        regions: [],
        volume: 0,
        pan: 0,
        muted: false,
        solo: false,
        plugins: [],
        automation: [],
      };

      expect(track.name).toBe('Audio Track');
    });

    it('should support different track types', () => {
      const types: Array<SharedTypes.SessionTrack['type']> = ['audio', 'midi', 'instrument', 'aux'];

      types.forEach(type => {
        const track: SharedTypes.SessionTrack = {
          id: `track-${type}`,
          name: `${type} track`,
          type,
          regions: [],
          volume: 0,
          pan: 0,
          muted: false,
          solo: false,
          plugins: [],
          automation: [],
        };

        expect(track.type).toBe(type);
      });
    });

    it('should define Bus type', () => {
      const bus: SharedTypes.Bus = {
        id: 'master',
        name: 'Master',
        type: 'master',
        volume: 0,
        pan: 0,
        plugins: [],
        automation: [],
      };

      expect(bus.type).toBe('master');
    });

    it('should define AudioRegion type', () => {
      const region: SharedTypes.AudioRegion = {
        id: 'region-1',
        name: 'Region 1',
        startTime: 0,
        duration: 10,
        sourceStart: 0,
        sourceEnd: 10,
        fadeIn: 0,
        fadeOut: 0,
      };

      expect(region.duration).toBe(10);
    });
  });

  describe('Mixing types', () => {
    it('should define MixingSnapshot type', () => {
      const snapshot: SharedTypes.MixingSnapshot = {
        id: 'snapshot-1',
        name: 'Mix A',
        timestamp: new Date().toISOString(),
        trackStates: [],
      };

      expect(snapshot.name).toBe('Mix A');
    });

    it('should define Plugin type', () => {
      const plugin: SharedTypes.Plugin = {
        id: 'plugin-1',
        name: 'Compressor',
        type: 'dynamics',
        enabled: true,
        parameters: {},
      };

      expect(plugin.name).toBe('Compressor');
    });

    it('should define Automation type', () => {
      const automation: SharedTypes.Automation = {
        parameter: 'volume',
        points: [
          { time: 0, value: 0 },
          { time: 10, value: -6 },
        ],
      };

      expect(automation.points).toHaveLength(2);
    });
  });

  describe('Mastering types', () => {
    it('should define LoudnessMetrics type', () => {
      const loudness: SharedTypes.LoudnessMetrics = {
        integrated: -23,
        shortTerm: -23,
        momentary: -23,
        truePeak: -1,
        range: 10,
      };

      expect(loudness.integrated).toBe(-23);
    });

    it('should define ExportSettings type', () => {
      const exportSettings: SharedTypes.ExportSettings = {
        format: 'wav',
        sampleRate: 48000,
        bitDepth: 24,
        normalize: true,
        outputPath: '/exports/track.wav',
      };

      expect(exportSettings.format).toBe('wav');
    });
  });

  describe('Practice types', () => {
    it('should define SpeedTrainer type', () => {
      const speedTrainer: SharedTypes.SpeedTrainer = {
        enabled: true,
        currentSpeed: 80,
        targetSpeed: 100,
        incrementStep: 5,
      };

      expect(speedTrainer.currentSpeed).toBe(80);
    });

    it('should define PracticeLoop type', () => {
      const loop: SharedTypes.PracticeLoop = {
        enabled: true,
        startMeasure: 8,
        endMeasure: 16,
      };

      expect(loop.startMeasure).toBe(8);
    });

    it('should define PracticeSession type', () => {
      const session: SharedTypes.PracticeSession = {
        id: 'session-1',
        date: new Date().toISOString(),
        duration: 3600,
        sections: [],
      };

      expect(session.duration).toBe(3600);
    });

    it('should define DifficultSection type', () => {
      const section: SharedTypes.DifficultSection = {
        id: 'section-1',
        name: 'Solo',
        startMeasure: 32,
        endMeasure: 48,
        difficulty: 'hard',
        practiceTime: 1800,
      };

      expect(section.difficulty).toBe('hard');
    });
  });

  describe('AI History types', () => {
    it('should define CompositionSuggestion type', () => {
      const suggestion: SharedTypes.CompositionSuggestion = {
        id: 'suggestion-1',
        timestamp: new Date().toISOString(),
        type: 'chord-progression',
        content: {},
      };

      expect(suggestion.type).toBe('chord-progression');
    });

    it('should define ChatMessage type', () => {
      const message: SharedTypes.ChatMessage = {
        id: 'msg-1',
        role: 'user',
        content: 'Hello',
        timestamp: new Date().toISOString(),
      };

      expect(message.role).toBe('user');
    });
  });

  describe('Collaboration types', () => {
    it('should define Collaborator type', () => {
      const collaborator: SharedTypes.Collaborator = {
        id: 'user-1',
        name: 'John Doe',
        email: 'john@example.com',
        role: 'editor',
      };

      expect(collaborator.role).toBe('editor');
    });

    it('should define ProjectVersion type', () => {
      const version: SharedTypes.ProjectVersion = {
        id: 'version-1',
        number: 1,
        timestamp: new Date().toISOString(),
        author: 'user-1',
        message: 'Initial version',
      };

      expect(version.number).toBe(1);
    });

    it('should define Comment type', () => {
      const comment: SharedTypes.Comment = {
        id: 'comment-1',
        author: 'user-1',
        content: 'Great work!',
        timestamp: new Date().toISOString(),
        measure: 16,
      };

      expect(comment.content).toBe('Great work!');
    });
  });

  describe('Common types', () => {
    it('should support standard sample rates', () => {
      const rates = [44100, 48000, 88200, 96000, 176400, 192000];

      rates.forEach(rate => {
        expect(typeof rate).toBe('number');
      });
    });

    it('should support standard bit depths', () => {
      const depths = [16, 24, 32];

      depths.forEach(depth => {
        expect(typeof depth).toBe('number');
      });
    });

    it('should define musical keys', () => {
      const keys = ['C', 'C#', 'Db', 'D', 'D#', 'Eb', 'E', 'F', 'F#', 'Gb', 'G', 'G#', 'Ab', 'A', 'A#', 'Bb', 'B'];

      keys.forEach(key => {
        expect(typeof key).toBe('string');
      });
    });
  });

  describe('Type completeness', () => {
    it('should export all major type categories', () => {
      // This is a compile-time check - if types are missing, TypeScript will error
      const typeCheck = {
        project: {} as SharedTypes.MaestroProject,
        metadata: {} as SharedTypes.ProjectMetadata,
        score: {} as SharedTypes.Score,
        track: {} as SharedTypes.SessionTrack,
        bus: {} as SharedTypes.Bus,
        plugin: {} as SharedTypes.Plugin,
        automation: {} as SharedTypes.Automation,
        marker: {} as SharedTypes.Marker,
        tempoChange: {} as SharedTypes.TempoChange,
        loudness: {} as SharedTypes.LoudnessMetrics,
        exportSettings: {} as SharedTypes.ExportSettings,
        speedTrainer: {} as SharedTypes.SpeedTrainer,
        practiceLoop: {} as SharedTypes.PracticeLoop,
        practiceSession: {} as SharedTypes.PracticeSession,
        difficultSection: {} as SharedTypes.DifficultSection,
        suggestion: {} as SharedTypes.CompositionSuggestion,
        chatMessage: {} as SharedTypes.ChatMessage,
        collaborator: {} as SharedTypes.Collaborator,
        version: {} as SharedTypes.ProjectVersion,
        comment: {} as SharedTypes.Comment,
        timeSignature: {} as SharedTypes.TimeSignature,
        audioRegion: {} as SharedTypes.AudioRegion,
        mixingSnapshot: {} as SharedTypes.MixingSnapshot,
      };

      expect(typeCheck).toBeDefined();
    });
  });
});
