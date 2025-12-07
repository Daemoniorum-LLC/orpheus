/**
 * Tempo Map - Manages tempo changes over time
 */

export interface TempoChange {
  beat: number;      // Beat position where tempo changes
  bpm: number;       // New tempo in BPM
}

/**
 * TempoMap maintains a list of tempo changes throughout the project
 * This allows for tempo automation and gradual tempo changes
 */
export class TempoMap {
  private changes: TempoChange[] = [];

  constructor(initialTempo: number = 120) {
    this.changes = [{ beat: 0, bpm: initialTempo }];
  }

  /**
   * Sets the tempo at a specific beat
   */
  setTempoAt(beat: number, bpm: number): void {
    // Remove existing change at this beat
    this.changes = this.changes.filter(c => c.beat !== beat);

    // Add new change
    this.changes.push({ beat, bpm });

    // Sort by beat position
    this.changes.sort((a, b) => a.beat - b.beat);
  }

  /**
   * Gets the tempo at a specific beat
   */
  getTempoAt(beat: number): number {
    // Find the most recent tempo change before or at this beat
    for (let i = this.changes.length - 1; i >= 0; i--) {
      if (this.changes[i].beat <= beat) {
        return this.changes[i].bpm;
      }
    }

    // Should never happen if we have an initial tempo
    return 120;
  }

  /**
   * Sets a global tempo (removes all tempo changes)
   */
  setGlobalTempo(bpm: number): void {
    this.changes = [{ beat: 0, bpm }];
  }

  /**
   * Gets all tempo changes
   */
  getAllChanges(): TempoChange[] {
    return [...this.changes];
  }

  /**
   * Removes a tempo change at a specific beat
   */
  removeTempoAt(beat: number): void {
    if (beat === 0) {
      throw new Error('Cannot remove initial tempo');
    }

    this.changes = this.changes.filter(c => c.beat !== beat);
  }

  /**
   * Calculates the time (in seconds) for a given beat range
   * accounting for tempo changes
   */
  beatsToSeconds(fromBeat: number, toBeat: number): number {
    if (fromBeat === toBeat) return 0;

    let totalSeconds = 0;
    let currentBeat = fromBeat;

    // Find all tempo changes in this range
    const relevantChanges = this.changes.filter(
      c => c.beat > fromBeat && c.beat <= toBeat
    );

    // If no changes in range, use constant tempo
    if (relevantChanges.length === 0) {
      const tempo = this.getTempoAt(fromBeat);
      const beats = toBeat - fromBeat;
      return (beats / tempo) * 60;
    }

    // Calculate time for each tempo section
    for (const change of relevantChanges) {
      const tempo = this.getTempoAt(currentBeat);
      const beats = change.beat - currentBeat;
      totalSeconds += (beats / tempo) * 60;
      currentBeat = change.beat;
    }

    // Add time for final section
    const finalTempo = this.getTempoAt(currentBeat);
    const finalBeats = toBeat - currentBeat;
    totalSeconds += (finalBeats / finalTempo) * 60;

    return totalSeconds;
  }

  /**
   * Calculates the beat position for a given time (in seconds)
   * accounting for tempo changes
   */
  secondsToBeats(seconds: number): number {
    let remainingSeconds = seconds;
    let currentBeat = 0;

    for (let i = 0; i < this.changes.length - 1; i++) {
      const change = this.changes[i];
      const nextChange = this.changes[i + 1];
      const tempo = change.bpm;

      // Time available in this tempo section
      const beatsInSection = nextChange.beat - change.beat;
      const secondsInSection = (beatsInSection / tempo) * 60;

      if (remainingSeconds <= secondsInSection) {
        // Target is in this section
        const beatsIntoSection = (remainingSeconds / 60) * tempo;
        return change.beat + beatsIntoSection;
      }

      remainingSeconds -= secondsInSection;
      currentBeat = nextChange.beat;
    }

    // Remainder is in the final tempo section
    const finalTempo = this.changes[this.changes.length - 1].bpm;
    const finalBeats = (remainingSeconds / 60) * finalTempo;
    return currentBeat + finalBeats;
  }
}
