/**
 * Project Templates Service
 * Pre-configured project templates for quick start
 */

import type { MaestroProject } from '@maestro-ai/shared-types';

export interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  genre: string;
  tempo: number;
  key: string;
  timeSignature: { numerator: number; denominator: number };
  trackCount: number;
  trackNames: string[];
}

/**
 * Available project templates
 */
export const PROJECT_TEMPLATES: ProjectTemplate[] = [
  {
    id: 'blank',
    name: 'Blank Project',
    description: 'Start with a single guitar track',
    genre: 'General',
    tempo: 120,
    key: 'C',
    timeSignature: { numerator: 4, denominator: 4 },
    trackCount: 1,
    trackNames: ['Guitar 1'],
  },
  {
    id: 'rock-band',
    name: 'Rock Band',
    description: '2 guitars, bass, and drums',
    genre: 'Rock',
    tempo: 140,
    key: 'E',
    timeSignature: { numerator: 4, denominator: 4 },
    trackCount: 4,
    trackNames: ['Lead Guitar', 'Rhythm Guitar', 'Bass', 'Drums'],
  },
  {
    id: 'blues',
    name: 'Blues Trio',
    description: 'Guitar, bass, and drums for blues',
    genre: 'Blues',
    tempo: 120,
    key: 'A',
    timeSignature: { numerator: 4, denominator: 4 },
    trackCount: 3,
    trackNames: ['Guitar', 'Bass', 'Drums'],
  },
  {
    id: 'jazz',
    name: 'Jazz Quartet',
    description: 'Piano, bass, drums, and guitar',
    genre: 'Jazz',
    tempo: 160,
    key: 'Bb',
    timeSignature: { numerator: 4, denominator: 4 },
    trackCount: 4,
    trackNames: ['Piano', 'Guitar', 'Bass', 'Drums'],
  },
  {
    id: 'acoustic-duo',
    name: 'Acoustic Duo',
    description: 'Two acoustic guitars',
    genre: 'Folk',
    tempo: 90,
    key: 'G',
    timeSignature: { numerator: 4, denominator: 4 },
    trackCount: 2,
    trackNames: ['Guitar 1', 'Guitar 2'],
  },
  {
    id: 'metal',
    name: 'Metal Band',
    description: '2 guitars, bass, drums - Drop D tuning',
    genre: 'Metal',
    tempo: 180,
    key: 'D',
    timeSignature: { numerator: 4, denominator: 4 },
    trackCount: 4,
    trackNames: ['Lead Guitar', 'Rhythm Guitar', 'Bass', 'Drums'],
  },
];

/**
 * Create a new project from a template
 */
export function createProjectFromTemplate(
  template: ProjectTemplate,
  title: string = 'Untitled Project',
  artist?: string
): MaestroProject {
  const now = new Date().toISOString();

  // Create simple measures (4 empty bars)
  const measures = Array.from({ length: 4 }, (_, i) => ({
    number: i + 1,
    timeSignature: template.timeSignature,
    voices: [
      {
        voiceIndex: 0,
        beats: [
          {
            startTime: 0,
            duration: 4.0, // Whole note
            notes: [],
            rest: true,
          },
        ],
      },
    ],
  }));

  // Create tracks
  const tracks = template.trackNames.map((name, index) => ({
    id: `track-${index + 1}`,
    name,
    voiceCount: 1,
    measures: JSON.parse(JSON.stringify(measures)), // Deep copy
  }));

  const project: MaestroProject = {
    formatVersion: '1.0.0',
    project: {
      metadata: {
        id: `project-${Date.now()}`,
        title,
        artist: artist || '',
        album: '',
        genre: template.genre,
        tempo: template.tempo,
        key: template.key,
        timeSignature: template.timeSignature,
        created: now,
        modified: now,
      },
      composition: {
        scores: [],
        tempoMap: [{ measureNumber: 1, tempo: template.tempo }],
        markers: [],
        tracks,
        measures,
      },
      session: {} as any,
      mixing: {} as any,
      mastering: {} as any,
      practice: {} as any,
      aiHistory: {} as any,
      collaboration: {} as any,
    },
  };

  return project;
}
