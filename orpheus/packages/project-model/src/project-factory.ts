/**
 * Factory for creating new Maestro projects
 */

import type { MaestroProject, ProjectMetadata } from '@orpheus/shared-types';
import { generateId } from './utils';

export interface CreateProjectOptions {
  title: string;
  artist?: string;
  tempo?: number;
  key?: string;
  timeSignature?: { numerator: number; denominator: number };
}

/**
 * Creates a new empty Maestro project with default values
 */
export function createProject(options: CreateProjectOptions): MaestroProject {
  const now = new Date().toISOString();

  const metadata: ProjectMetadata = {
    id: generateId(),
    title: options.title,
    artist: options.artist,
    tempo: options.tempo || 120,
    key: options.key || 'C',
    timeSignature: options.timeSignature || { numerator: 4, denominator: 4 },
    created: now,
    modified: now,
  };

  return {
    formatVersion: '1.0',
    project: {
      metadata,
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
        buses: [
          {
            id: 'master',
            name: 'Master',
            type: 'master',
            volume: 0,
            pan: 0,
            plugins: [],
            automation: [],
          },
        ],
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
}

/**
 * Creates a project with example data for testing/demo
 */
export function createExampleProject(): MaestroProject {
  const project = createProject({
    title: 'Example Song',
    artist: 'Maestro AI Demo',
    tempo: 120,
    key: 'C',
  });

  // Add example score
  project.project.composition.scores.push({
    id: generateId(),
    title: 'Guitar',
    instrument: 'electric-guitar',
    tuning: ['E', 'A', 'D', 'G', 'B', 'E'],
    tracks: [
      {
        id: generateId(),
        name: 'Guitar Track',
        voiceCount: 1,
        measures: [],
      },
    ],
  });

  // Add song structure
  project.project.composition.songStructure = {
    sections: [
      { name: 'Intro', startMeasure: 0, endMeasure: 4, color: '#3b82f6' },
      { name: 'Verse 1', startMeasure: 4, endMeasure: 12, color: '#10b981' },
      { name: 'Chorus', startMeasure: 12, endMeasure: 20, color: '#f59e0b' },
    ],
  };

  return project;
}
