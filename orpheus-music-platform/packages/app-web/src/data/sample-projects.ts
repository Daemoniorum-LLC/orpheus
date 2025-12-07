/**
 * Sample Projects - Pre-made projects for immediate exploration
 * These provide users with examples to learn from and play with
 */

import type { MaestroProject } from '@orpheus/shared-types';

// Helper to create a unique ID
const createId = () => Math.random().toString(36).substring(2, 11);

// Helper to create base project structure
function createBaseProject(
  title: string,
  artist: string,
  tempo: number,
  key: string
): MaestroProject {
  const now = new Date().toISOString();
  return {
    formatVersion: '1.0',
    project: {
      metadata: {
        id: createId(),
        title,
        artist,
        tempo,
        timeSignature: { numerator: 4, denominator: 4 },
        key,
        created: now,
        modified: now,
        tags: ['sample'],
      },
      composition: {
        scores: [],
        tempoMap: [{ measureNumber: 1, tempo }],
        markers: [],
        tracks: [],
      },
      session: {
        audioTracks: [],
        midiTracks: [],
        regions: [],
      },
      mixing: {
        channels: [],
        buses: [],
        masterBus: {
          id: 'master',
          name: 'Master',
          volume: 0,
          pan: 0,
          mute: false,
          solo: false,
          effects: [],
        },
      },
      mastering: {
        chain: [],
        targetLoudness: -14,
        targetPlatform: 'spotify',
        exportSettings: {
          format: 'wav',
          sampleRate: 44100,
          bitDepth: 16,
        },
      },
      practice: {
        loops: [],
        speedTrainer: {
          currentTempo: tempo,
          targetTempo: tempo,
          increment: 5,
        },
      },
      aiHistory: {
        conversations: [],
      },
      collaboration: {
        owner: 'local',
        collaborators: [],
        comments: [],
        history: [],
      },
    },
  };
}

/**
 * Sample 1: Blues Shuffle Riff
 * Classic 12-bar blues pattern in E
 */
export const bluesShuffleRiff: MaestroProject = (() => {
  const project = createBaseProject('Blues Shuffle in E', 'Orpheus Demo', 120, 'E');

  project.project.composition.tracks = [
    {
      id: 'blues-guitar',
      name: 'Blues Guitar',
      voiceCount: 1,
      measures: [
        // Measure 1-4: E chord shuffle
        ...Array.from({ length: 4 }, (_, i) => ({
          number: i + 1,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.8 }, { string: 5, fret: 2, velocity: 0.7 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.6 }, { string: 5, fret: 4, velocity: 0.6 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.8 }, { string: 5, fret: 2, velocity: 0.7 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.6 }, { string: 5, fret: 4, velocity: 0.6 }] },
            ],
          }],
        })),
        // Measure 5-6: A chord shuffle
        ...Array.from({ length: 2 }, (_, i) => ({
          number: i + 5,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.8 }, { string: 4, fret: 2, velocity: 0.7 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.6 }, { string: 4, fret: 4, velocity: 0.6 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.8 }, { string: 4, fret: 2, velocity: 0.7 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.6 }, { string: 4, fret: 4, velocity: 0.6 }] },
            ],
          }],
        })),
        // Measure 7-8: Back to E
        ...Array.from({ length: 2 }, (_, i) => ({
          number: i + 7,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.8 }, { string: 5, fret: 2, velocity: 0.7 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.6 }, { string: 5, fret: 4, velocity: 0.6 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.8 }, { string: 5, fret: 2, velocity: 0.7 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.6 }, { string: 5, fret: 4, velocity: 0.6 }] },
            ],
          }],
        })),
        // Measure 9: B7 (turnaround)
        {
          number: 9,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 5, fret: 2, velocity: 0.8 }, { string: 4, fret: 1, velocity: 0.7 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 5, fret: 2, velocity: 0.6 }, { string: 4, fret: 4, velocity: 0.6 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 5, fret: 2, velocity: 0.8 }, { string: 4, fret: 1, velocity: 0.7 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 5, fret: 2, velocity: 0.6 }, { string: 4, fret: 4, velocity: 0.6 }] },
            ],
          }],
        },
        // Measure 10: A
        {
          number: 10,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.8 }, { string: 4, fret: 2, velocity: 0.7 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.6 }, { string: 4, fret: 4, velocity: 0.6 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.8 }, { string: 4, fret: 2, velocity: 0.7 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.6 }, { string: 4, fret: 4, velocity: 0.6 }] },
            ],
          }],
        },
        // Measure 11-12: E with turnaround
        {
          number: 11,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.8 }, { string: 5, fret: 2, velocity: 0.7 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.6 }, { string: 5, fret: 4, velocity: 0.6 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.8 }, { string: 5, fret: 2, velocity: 0.7 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.6 }, { string: 5, fret: 4, velocity: 0.6 }] },
            ],
          }],
        },
        {
          number: 12,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.9 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 6, fret: 3, velocity: 0.7 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 6, fret: 4, velocity: 0.8 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 5, fret: 2, velocity: 0.9, techniques: { slide: { type: 'shiftSlide', targetFret: 4 } } }] },
            ],
          }],
        },
      ],
    },
  ];

  project.project.composition.markers = [
    { id: 'm1', name: 'Intro', measureNumber: 1, type: 'section', color: '#4a90d9' },
    { id: 'm2', name: 'IV Chord', measureNumber: 5, type: 'section', color: '#d9a74a' },
    { id: 'm3', name: 'Turnaround', measureNumber: 9, type: 'section', color: '#d94a4a' },
  ];

  return project;
})();

/**
 * Sample 2: Acoustic Folk Progression
 * G - C - D pattern with fingerpicking
 */
export const acousticFolkProgression: MaestroProject = (() => {
  const project = createBaseProject('Folk Fingerpicking', 'Orpheus Demo', 100, 'G');

  project.project.composition.tracks = [
    {
      id: 'acoustic-guitar',
      name: 'Acoustic Guitar',
      voiceCount: 1,
      measures: [
        // G chord - fingerpicking pattern
        {
          number: 1,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.25, notes: [{ string: 6, fret: 3, velocity: 0.8 }] },
              { startTime: 0.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 2, fret: 0, velocity: 0.6 }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.7 }] },
              { startTime: 1.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1.5, duration: 0.25, notes: [{ string: 2, fret: 0, velocity: 0.6 }] },
              { startTime: 1.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
            ],
          }],
        },
        // G chord - continued
        {
          number: 2,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.25, notes: [{ string: 6, fret: 3, velocity: 0.8 }] },
              { startTime: 0.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 2, fret: 0, velocity: 0.6 }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.7 }] },
              { startTime: 1.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1.5, duration: 0.25, notes: [{ string: 2, fret: 0, velocity: 0.6 }] },
              { startTime: 1.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
            ],
          }],
        },
        // C chord
        {
          number: 3,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.25, notes: [{ string: 5, fret: 3, velocity: 0.8 }] },
              { startTime: 0.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 2, fret: 1, velocity: 0.6 }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1, duration: 0.25, notes: [{ string: 4, fret: 2, velocity: 0.7 }] },
              { startTime: 1.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1.5, duration: 0.25, notes: [{ string: 2, fret: 1, velocity: 0.6 }] },
              { startTime: 1.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
            ],
          }],
        },
        // C chord - continued
        {
          number: 4,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.25, notes: [{ string: 5, fret: 3, velocity: 0.8 }] },
              { startTime: 0.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 2, fret: 1, velocity: 0.6 }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1, duration: 0.25, notes: [{ string: 4, fret: 2, velocity: 0.7 }] },
              { startTime: 1.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1.5, duration: 0.25, notes: [{ string: 2, fret: 1, velocity: 0.6 }] },
              { startTime: 1.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
            ],
          }],
        },
        // D chord
        {
          number: 5,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.8 }] },
              { startTime: 0.25, duration: 0.25, notes: [{ string: 3, fret: 2, velocity: 0.6 }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 2, fret: 3, velocity: 0.6 }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 3, fret: 2, velocity: 0.6 }] },
              { startTime: 1, duration: 0.25, notes: [{ string: 1, fret: 2, velocity: 0.7 }] },
              { startTime: 1.25, duration: 0.25, notes: [{ string: 3, fret: 2, velocity: 0.6 }] },
              { startTime: 1.5, duration: 0.25, notes: [{ string: 2, fret: 3, velocity: 0.6 }] },
              { startTime: 1.75, duration: 0.25, notes: [{ string: 3, fret: 2, velocity: 0.6 }] },
            ],
          }],
        },
        // D chord - continued
        {
          number: 6,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.8 }] },
              { startTime: 0.25, duration: 0.25, notes: [{ string: 3, fret: 2, velocity: 0.6 }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 2, fret: 3, velocity: 0.6 }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 3, fret: 2, velocity: 0.6 }] },
              { startTime: 1, duration: 0.25, notes: [{ string: 1, fret: 2, velocity: 0.7 }] },
              { startTime: 1.25, duration: 0.25, notes: [{ string: 3, fret: 2, velocity: 0.6 }] },
              { startTime: 1.5, duration: 0.25, notes: [{ string: 2, fret: 3, velocity: 0.6 }] },
              { startTime: 1.75, duration: 0.25, notes: [{ string: 3, fret: 2, velocity: 0.6 }] },
            ],
          }],
        },
        // G chord - resolution
        {
          number: 7,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.25, notes: [{ string: 6, fret: 3, velocity: 0.9 }] },
              { startTime: 0.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 2, fret: 0, velocity: 0.6 }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.7 }] },
              { startTime: 1.25, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
              { startTime: 1.5, duration: 0.25, notes: [{ string: 2, fret: 0, velocity: 0.6 }] },
              { startTime: 1.75, duration: 0.25, notes: [{ string: 3, fret: 0, velocity: 0.6 }] },
            ],
          }],
        },
        // G chord - final
        {
          number: 8,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 2, notes: [
                { string: 6, fret: 3, velocity: 0.9, techniques: { letRing: true } },
                { string: 5, fret: 2, velocity: 0.8, techniques: { letRing: true } },
                { string: 4, fret: 0, velocity: 0.8, techniques: { letRing: true } },
                { string: 3, fret: 0, velocity: 0.8, techniques: { letRing: true } },
                { string: 2, fret: 0, velocity: 0.8, techniques: { letRing: true } },
                { string: 1, fret: 3, velocity: 0.7, techniques: { letRing: true } },
              ]},
            ],
          }],
        },
      ],
    },
  ];

  project.project.composition.markers = [
    { id: 'm1', name: 'G (I)', measureNumber: 1, type: 'section', color: '#4ad94a' },
    { id: 'm2', name: 'C (IV)', measureNumber: 3, type: 'section', color: '#4a90d9' },
    { id: 'm3', name: 'D (V)', measureNumber: 5, type: 'section', color: '#d9a74a' },
    { id: 'm4', name: 'Resolution', measureNumber: 7, type: 'section', color: '#4ad94a' },
  ];

  return project;
})();

/**
 * Sample 3: Rock Power Chords
 * Classic rock power chord progression
 */
export const rockPowerChords: MaestroProject = (() => {
  const project = createBaseProject('Power Chord Rock', 'Orpheus Demo', 140, 'A');

  project.project.composition.tracks = [
    {
      id: 'electric-guitar',
      name: 'Electric Guitar',
      voiceCount: 1,
      measures: [
        // A5 power chord
        {
          number: 1,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.9, techniques: { palmMute: true } }, { string: 4, fret: 2, velocity: 0.9, techniques: { palmMute: true } }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 5, fret: 0, velocity: 0.7, techniques: { palmMute: true } }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 5, fret: 0, velocity: 0.7, techniques: { palmMute: true } }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.9 }, { string: 4, fret: 2, velocity: 0.9 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.8 }, { string: 4, fret: 2, velocity: 0.8 }] },
            ],
          }],
        },
        // A5 continued
        {
          number: 2,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.9, techniques: { palmMute: true } }, { string: 4, fret: 2, velocity: 0.9, techniques: { palmMute: true } }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 5, fret: 0, velocity: 0.7, techniques: { palmMute: true } }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 5, fret: 0, velocity: 0.7, techniques: { palmMute: true } }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.9 }, { string: 4, fret: 2, velocity: 0.9 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.8 }, { string: 4, fret: 2, velocity: 0.8 }] },
            ],
          }],
        },
        // D5 power chord
        {
          number: 3,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 4, fret: 0, velocity: 0.9, techniques: { palmMute: true } }, { string: 3, fret: 2, velocity: 0.9, techniques: { palmMute: true } }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.7, techniques: { palmMute: true } }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.7, techniques: { palmMute: true } }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 4, fret: 0, velocity: 0.9 }, { string: 3, fret: 2, velocity: 0.9 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 4, fret: 0, velocity: 0.8 }, { string: 3, fret: 2, velocity: 0.8 }] },
            ],
          }],
        },
        // D5 continued
        {
          number: 4,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 4, fret: 0, velocity: 0.9, techniques: { palmMute: true } }, { string: 3, fret: 2, velocity: 0.9, techniques: { palmMute: true } }] },
              { startTime: 0.5, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.7, techniques: { palmMute: true } }] },
              { startTime: 0.75, duration: 0.25, notes: [{ string: 4, fret: 0, velocity: 0.7, techniques: { palmMute: true } }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 4, fret: 0, velocity: 0.9 }, { string: 3, fret: 2, velocity: 0.9 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 4, fret: 0, velocity: 0.8 }, { string: 3, fret: 2, velocity: 0.8 }] },
            ],
          }],
        },
        // E5 power chord
        {
          number: 5,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.9 }, { string: 5, fret: 2, velocity: 0.9 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.9 }, { string: 5, fret: 2, velocity: 0.9 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.9 }, { string: 5, fret: 2, velocity: 0.9 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 6, fret: 0, velocity: 0.9 }, { string: 5, fret: 2, velocity: 0.9 }] },
            ],
          }],
        },
        // D5 to A5 transition
        {
          number: 6,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 0.5, notes: [{ string: 4, fret: 0, velocity: 0.9 }, { string: 3, fret: 2, velocity: 0.9 }] },
              { startTime: 0.5, duration: 0.5, notes: [{ string: 4, fret: 0, velocity: 0.9 }, { string: 3, fret: 2, velocity: 0.9 }] },
              { startTime: 1, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.9 }, { string: 4, fret: 2, velocity: 0.9 }] },
              { startTime: 1.5, duration: 0.5, notes: [{ string: 5, fret: 0, velocity: 0.9 }, { string: 4, fret: 2, velocity: 0.9 }] },
            ],
          }],
        },
        // A5 with slides
        {
          number: 7,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 1, notes: [
                { string: 5, fret: 0, velocity: 0.9 },
                { string: 4, fret: 2, velocity: 0.9 },
              ]},
              { startTime: 1, duration: 0.5, notes: [
                { string: 5, fret: 2, velocity: 0.8, techniques: { slide: { type: 'shiftSlide', targetFret: 5 } } },
                { string: 4, fret: 4, velocity: 0.8, techniques: { slide: { type: 'shiftSlide', targetFret: 7 } } },
              ]},
              { startTime: 1.5, duration: 0.5, notes: [
                { string: 5, fret: 5, velocity: 0.9 },
                { string: 4, fret: 7, velocity: 0.9 },
              ]},
            ],
          }],
        },
        // Final hit
        {
          number: 8,
          timeSignature: { numerator: 4, denominator: 4 },
          voices: [{
            voiceIndex: 0,
            beats: [
              { startTime: 0, duration: 2, notes: [
                { string: 5, fret: 0, velocity: 1.0 },
                { string: 4, fret: 2, velocity: 1.0 },
                { string: 3, fret: 2, velocity: 0.9 },
              ]},
            ],
          }],
        },
      ],
    },
  ];

  project.project.composition.markers = [
    { id: 'm1', name: 'Verse (A5)', measureNumber: 1, type: 'section', color: '#d94a4a' },
    { id: 'm2', name: 'Pre-Chorus (D5)', measureNumber: 3, type: 'section', color: '#d9a74a' },
    { id: 'm3', name: 'Chorus (E5)', measureNumber: 5, type: 'section', color: '#4a90d9' },
    { id: 'm4', name: 'Outro', measureNumber: 7, type: 'section', color: '#9b4ad9' },
  ];

  return project;
})();

/**
 * All sample projects
 */
export const sampleProjects = [
  {
    id: 'blues-shuffle',
    project: bluesShuffleRiff,
    title: 'Blues Shuffle in E',
    description: 'Classic 12-bar blues with shuffle rhythm',
    difficulty: 'Beginner',
    genre: 'Blues',
    duration: '~30 sec',
    icon: '🎸',
  },
  {
    id: 'folk-fingerpicking',
    project: acousticFolkProgression,
    title: 'Folk Fingerpicking',
    description: 'G-C-D progression with Travis picking pattern',
    difficulty: 'Intermediate',
    genre: 'Folk',
    duration: '~20 sec',
    icon: '🪕',
  },
  {
    id: 'rock-power-chords',
    project: rockPowerChords,
    title: 'Power Chord Rock',
    description: 'Energetic power chord progression with palm muting',
    difficulty: 'Beginner',
    genre: 'Rock',
    duration: '~25 sec',
    icon: '🤘',
  },
] as const;

export type SampleProjectId = typeof sampleProjects[number]['id'];
