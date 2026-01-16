/**
 * Marker Manager - Manages timeline markers and sections
 */

import type { MusicalPosition } from './timeline';

export interface Marker {
  id: string;
  position: MusicalPosition;
  name: string;
  type: 'section' | 'rehearsal' | 'custom';
  color: string;
}

/**
 * MarkerManager handles all timeline markers (sections, rehearsal marks, custom markers)
 */
export class MarkerManager {
  private markers: Map<string, Marker> = new Map();
  private nextId: number = 1;

  /**
   * Adds a marker
   */
  addMarker(marker: Omit<Marker, 'id'>): string {
    const id = `marker-${this.nextId++}`;
    this.markers.set(id, { ...marker, id });
    return id;
  }

  /**
   * Removes a marker
   */
  removeMarker(id: string): void {
    this.markers.delete(id);
  }

  /**
   * Gets a marker by ID
   */
  getMarker(id: string): Marker | undefined {
    return this.markers.get(id);
  }

  /**
   * Gets all markers
   */
  getAll(): Marker[] {
    return Array.from(this.markers.values()).sort(
      (a, b) => a.position.totalBeats - b.position.totalBeats
    );
  }

  /**
   * Gets markers in a beat range
   */
  getInRange(startBeat: number, endBeat: number): Marker[] {
    return this.getAll().filter(
      m => m.position.totalBeats >= startBeat && m.position.totalBeats <= endBeat
    );
  }

  /**
   * Finds the nearest marker to a position
   */
  findNearest(position: MusicalPosition): Marker | null {
    const all = this.getAll();
    if (all.length === 0) return null;

    let nearest = all[0];
    let minDistance = Math.abs(all[0].position.totalBeats - position.totalBeats);

    for (const marker of all) {
      const distance = Math.abs(marker.position.totalBeats - position.totalBeats);
      if (distance < minDistance) {
        minDistance = distance;
        nearest = marker;
      }
    }

    return nearest;
  }

  /**
   * Gets the next marker after a position
   */
  getNext(position: MusicalPosition): Marker | null {
    const all = this.getAll();
    for (const marker of all) {
      if (marker.position.totalBeats > position.totalBeats) {
        return marker;
      }
    }
    return null;
  }

  /**
   * Gets the previous marker before a position
   */
  getPrevious(position: MusicalPosition): Marker | null {
    const all = this.getAll();
    for (let i = all.length - 1; i >= 0; i--) {
      if (all[i].position.totalBeats < position.totalBeats) {
        return all[i];
      }
    }
    return null;
  }

  /**
   * Clears all markers
   */
  clear(): void {
    this.markers.clear();
  }
}
