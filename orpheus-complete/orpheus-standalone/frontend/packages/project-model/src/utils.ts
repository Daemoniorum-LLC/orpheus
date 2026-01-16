/**
 * Utility functions for project model operations
 */

import type { MaestroProject } from '@maestro-ai/shared-types';

/**
 * Generates a unique ID for project elements
 */
export function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Deep clones a project
 */
export function cloneProject(project: MaestroProject): MaestroProject {
  return JSON.parse(JSON.stringify(project));
}

/**
 * Calculates the duration of a project in seconds based on the longest track
 */
export function calculateProjectDuration(project: MaestroProject): number {
  let maxDuration = 0;

  for (const track of project.project.session.tracks) {
    for (const region of track.regions) {
      const endTime = region.startTime + region.duration;
      if (endTime > maxDuration) {
        maxDuration = endTime;
      }
    }
  }

  return maxDuration;
}

/**
 * Converts musical time (measures/beats) to absolute time (seconds)
 */
export function musicalTimeToSeconds(
  measure: number,
  beat: number,
  tempo: number,
  timeSignature: { numerator: number; denominator: number }
): number {
  const beatsPerMeasure = timeSignature.numerator;
  const totalBeats = measure * beatsPerMeasure + beat;
  const secondsPerBeat = 60 / tempo;

  return totalBeats * secondsPerBeat;
}

/**
 * Converts absolute time (seconds) to musical time (measures/beats)
 */
export function secondsToMusicalTime(
  seconds: number,
  tempo: number,
  timeSignature: { numerator: number; denominator: number }
): { measure: number; beat: number } {
  const secondsPerBeat = 60 / tempo;
  const totalBeats = seconds / secondsPerBeat;
  const beatsPerMeasure = timeSignature.numerator;

  const measure = Math.floor(totalBeats / beatsPerMeasure);
  const beat = totalBeats % beatsPerMeasure;

  return { measure, beat };
}

/**
 * Merges two projects (useful for collaboration)
 */
export function mergeProjects(
  base: MaestroProject,
  incoming: MaestroProject
): MaestroProject {
  const merged = cloneProject(base);

  // Merge tracks (add new tracks from incoming)
  const existingTrackIds = new Set(merged.project.session.tracks.map(t => t.id));
  for (const track of incoming.project.session.tracks) {
    if (!existingTrackIds.has(track.id)) {
      merged.project.session.tracks.push(track);
    }
  }

  // Merge scores
  const existingScoreIds = new Set(merged.project.composition.scores.map(s => s.id));
  for (const score of incoming.project.composition.scores) {
    if (!existingScoreIds.has(score.id)) {
      merged.project.composition.scores.push(score);
    }
  }

  // Update modified timestamp
  merged.project.metadata.modified = new Date().toISOString();

  return merged;
}

/**
 * Gets all tracks linked to a specific score
 */
export function getTracksLinkedToScore(
  project: MaestroProject,
  scoreId: string
): typeof project.project.session.tracks {
  return project.project.session.tracks.filter(
    track => track.linkedScoreId === scoreId
  );
}

/**
 * Gets the master bus
 */
export function getMasterBus(project: MaestroProject) {
  return project.project.session.buses.find(bus => bus.type === 'master');
}
