import { describe, it, expect, beforeEach } from 'vitest';
import { MarkerManager, type Marker } from './marker-manager';
import type { MusicalPosition } from './timeline';

describe('marker-manager.ts - Timeline Markers', () => {
  let manager: MarkerManager;
  let position1: MusicalPosition;
  let position2: MusicalPosition;
  let position3: MusicalPosition;

  beforeEach(() => {
    manager = new MarkerManager();

    position1 = { measure: 0, beat: 0, tick: 0, totalBeats: 0 };
    position2 = { measure: 4, beat: 0, tick: 0, totalBeats: 16 };
    position3 = { measure: 8, beat: 0, tick: 0, totalBeats: 32 };
  });

  describe('MarkerManager construction', () => {
    it('should create empty marker manager', () => {
      expect(manager).toBeInstanceOf(MarkerManager);
      expect(manager.getAll()).toHaveLength(0);
    });
  });

  describe('addMarker', () => {
    it('should add marker and return ID', () => {
      const id = manager.addMarker({
        position: position1,
        name: 'Intro',
        type: 'section',
        color: '#3b82f6',
      });

      expect(id).toBeDefined();
      expect(typeof id).toBe('string');
    });

    it('should generate unique IDs', () => {
      const id1 = manager.addMarker({
        position: position1,
        name: 'Intro',
        type: 'section',
        color: '#3b82f6',
      });

      const id2 = manager.addMarker({
        position: position2,
        name: 'Verse',
        type: 'section',
        color: '#3b82f6',
      });

      expect(id1).not.toBe(id2);
    });

    it('should add section marker', () => {
      const id = manager.addMarker({
        position: position1,
        name: 'Intro',
        type: 'section',
        color: '#3b82f6',
      });

      const marker = manager.getMarker(id);
      expect(marker?.type).toBe('section');
    });

    it('should add rehearsal marker', () => {
      const id = manager.addMarker({
        position: position1,
        name: 'A',
        type: 'rehearsal',
        color: '#10b981',
      });

      const marker = manager.getMarker(id);
      expect(marker?.type).toBe('rehearsal');
    });

    it('should add custom marker', () => {
      const id = manager.addMarker({
        position: position1,
        name: 'Custom',
        type: 'custom',
        color: '#6b7280',
      });

      const marker = manager.getMarker(id);
      expect(marker?.type).toBe('custom');
    });

    it('should store marker properties', () => {
      const id = manager.addMarker({
        position: position1,
        name: 'Test Marker',
        type: 'section',
        color: '#ff0000',
      });

      const marker = manager.getMarker(id);
      expect(marker?.name).toBe('Test Marker');
      expect(marker?.color).toBe('#ff0000');
      expect(marker?.position).toEqual(position1);
    });

    it('should increment marker count', () => {
      manager.addMarker({ position: position1, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: position2, name: 'B', type: 'section', color: '#000' });
      manager.addMarker({ position: position3, name: 'C', type: 'section', color: '#000' });

      expect(manager.getAll()).toHaveLength(3);
    });
  });

  describe('removeMarker', () => {
    it('should remove marker by ID', () => {
      const id = manager.addMarker({
        position: position1,
        name: 'Test',
        type: 'section',
        color: '#000',
      });

      manager.removeMarker(id);
      expect(manager.getMarker(id)).toBeUndefined();
    });

    it('should handle removing non-existent marker', () => {
      manager.removeMarker('invalid-id');
      // Should not throw
    });

    it('should remove correct marker when multiple exist', () => {
      const id1 = manager.addMarker({ position: position1, name: 'A', type: 'section', color: '#000' });
      const id2 = manager.addMarker({ position: position2, name: 'B', type: 'section', color: '#000' });
      const id3 = manager.addMarker({ position: position3, name: 'C', type: 'section', color: '#000' });

      manager.removeMarker(id2);

      expect(manager.getMarker(id1)).toBeDefined();
      expect(manager.getMarker(id2)).toBeUndefined();
      expect(manager.getMarker(id3)).toBeDefined();
    });

    it('should reduce marker count', () => {
      const id = manager.addMarker({ position: position1, name: 'Test', type: 'section', color: '#000' });
      manager.addMarker({ position: position2, name: 'Test2', type: 'section', color: '#000' });

      manager.removeMarker(id);
      expect(manager.getAll()).toHaveLength(1);
    });
  });

  describe('getMarker', () => {
    it('should get marker by ID', () => {
      const id = manager.addMarker({
        position: position1,
        name: 'Test',
        type: 'section',
        color: '#000',
      });

      const marker = manager.getMarker(id);
      expect(marker?.id).toBe(id);
    });

    it('should return undefined for non-existent ID', () => {
      expect(manager.getMarker('invalid-id')).toBeUndefined();
    });

    it('should return correct marker properties', () => {
      const id = manager.addMarker({
        position: position1,
        name: 'Intro',
        type: 'section',
        color: '#3b82f6',
      });

      const marker = manager.getMarker(id);
      expect(marker?.name).toBe('Intro');
      expect(marker?.type).toBe('section');
      expect(marker?.color).toBe('#3b82f6');
    });
  });

  describe('getAll', () => {
    it('should return empty array when no markers', () => {
      expect(manager.getAll()).toHaveLength(0);
    });

    it('should return all markers', () => {
      manager.addMarker({ position: position1, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: position2, name: 'B', type: 'section', color: '#000' });
      manager.addMarker({ position: position3, name: 'C', type: 'section', color: '#000' });

      expect(manager.getAll()).toHaveLength(3);
    });

    it('should return markers sorted by position', () => {
      // Add in random order
      manager.addMarker({ position: position3, name: 'C', type: 'section', color: '#000' });
      manager.addMarker({ position: position1, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: position2, name: 'B', type: 'section', color: '#000' });

      const all = manager.getAll();
      expect(all[0].name).toBe('A');
      expect(all[1].name).toBe('B');
      expect(all[2].name).toBe('C');
    });

    it('should sort by totalBeats', () => {
      const markers = manager.getAll();
      for (let i = 1; i < markers.length; i++) {
        expect(markers[i].position.totalBeats).toBeGreaterThanOrEqual(
          markers[i - 1].position.totalBeats
        );
      }
    });
  });

  describe('getInRange', () => {
    beforeEach(() => {
      manager.addMarker({ position: { ...position1, totalBeats: 0 }, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position2, totalBeats: 16 }, name: 'B', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position3, totalBeats: 32 }, name: 'C', type: 'section', color: '#000' });
    });

    it('should return markers in range', () => {
      const inRange = manager.getInRange(0, 20);
      expect(inRange).toHaveLength(2);
      expect(inRange[0].name).toBe('A');
      expect(inRange[1].name).toBe('B');
    });

    it('should include markers at range boundaries', () => {
      const inRange = manager.getInRange(0, 16);
      expect(inRange).toHaveLength(2);
    });

    it('should return empty array when no markers in range', () => {
      const inRange = manager.getInRange(100, 200);
      expect(inRange).toHaveLength(0);
    });

    it('should handle range containing all markers', () => {
      const inRange = manager.getInRange(0, 100);
      expect(inRange).toHaveLength(3);
    });

    it('should handle range containing no markers', () => {
      const inRange = manager.getInRange(40, 50);
      expect(inRange).toHaveLength(0);
    });
  });

  describe('findNearest', () => {
    beforeEach(() => {
      manager.addMarker({ position: { ...position1, totalBeats: 0 }, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position2, totalBeats: 16 }, name: 'B', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position3, totalBeats: 32 }, name: 'C', type: 'section', color: '#000' });
    });

    it('should find nearest marker', () => {
      const nearest = manager.findNearest({ measure: 2, beat: 0, tick: 0, totalBeats: 8 });
      expect(nearest?.name).toBe('A');
    });

    it('should find nearest marker when exactly at position', () => {
      const nearest = manager.findNearest({ measure: 4, beat: 0, tick: 0, totalBeats: 16 });
      expect(nearest?.name).toBe('B');
    });

    it('should return null when no markers exist', () => {
      const emptyManager = new MarkerManager();
      expect(emptyManager.findNearest(position1)).toBeNull();
    });

    it('should find marker closest to middle position', () => {
      const nearest = manager.findNearest({ measure: 5, beat: 0, tick: 0, totalBeats: 20 });
      expect(nearest?.name).toBe('B'); // 16 is closer than 32
    });

    it('should handle ties (pick first)', () => {
      const nearest = manager.findNearest({ measure: 6, beat: 0, tick: 0, totalBeats: 24 });
      // 24 is 8 beats from both B (16) and C (32)
      expect(nearest?.name).toBe('B');
    });
  });

  describe('getNext', () => {
    beforeEach(() => {
      manager.addMarker({ position: { ...position1, totalBeats: 0 }, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position2, totalBeats: 16 }, name: 'B', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position3, totalBeats: 32 }, name: 'C', type: 'section', color: '#000' });
    });

    it('should return next marker after position', () => {
      const next = manager.getNext({ measure: 2, beat: 0, tick: 0, totalBeats: 8 });
      expect(next?.name).toBe('B');
    });

    it('should return null when at last marker', () => {
      const next = manager.getNext({ measure: 10, beat: 0, tick: 0, totalBeats: 40 });
      expect(next).toBeNull();
    });

    it('should return null when past all markers', () => {
      const next = manager.getNext({ measure: 100, beat: 0, tick: 0, totalBeats: 400 });
      expect(next).toBeNull();
    });

    it('should return first marker when before all', () => {
      const next = manager.getNext({ measure: 0, beat: 0, tick: 0, totalBeats: -1 });
      expect(next?.name).toBe('A');
    });

    it('should skip to next marker even if exactly at one', () => {
      const next = manager.getNext({ measure: 4, beat: 0, tick: 0, totalBeats: 16 });
      expect(next?.name).toBe('C');
    });
  });

  describe('getPrevious', () => {
    beforeEach(() => {
      manager.addMarker({ position: { ...position1, totalBeats: 0 }, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position2, totalBeats: 16 }, name: 'B', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position3, totalBeats: 32 }, name: 'C', type: 'section', color: '#000' });
    });

    it('should return previous marker before position', () => {
      const prev = manager.getPrevious({ measure: 6, beat: 0, tick: 0, totalBeats: 24 });
      expect(prev?.name).toBe('B');
    });

    it('should return null when before all markers', () => {
      const prev = manager.getPrevious({ measure: 0, beat: 0, tick: 0, totalBeats: -1 });
      expect(prev).toBeNull();
    });

    it('should return last marker when after all', () => {
      const prev = manager.getPrevious({ measure: 100, beat: 0, tick: 0, totalBeats: 400 });
      expect(prev?.name).toBe('C');
    });

    it('should skip to previous marker even if exactly at one', () => {
      const prev = manager.getPrevious({ measure: 4, beat: 0, tick: 0, totalBeats: 16 });
      expect(prev?.name).toBe('A');
    });
  });

  describe('clear', () => {
    it('should remove all markers', () => {
      manager.addMarker({ position: position1, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: position2, name: 'B', type: 'section', color: '#000' });

      manager.clear();
      expect(manager.getAll()).toHaveLength(0);
    });

    it('should allow adding markers after clear', () => {
      manager.addMarker({ position: position1, name: 'A', type: 'section', color: '#000' });
      manager.clear();

      const id = manager.addMarker({ position: position1, name: 'B', type: 'section', color: '#000' });
      expect(manager.getMarker(id)).toBeDefined();
    });

    it('should handle clearing empty manager', () => {
      manager.clear();
      expect(manager.getAll()).toHaveLength(0);
    });
  });

  describe('Marker interface', () => {
    it('should define complete marker structure', () => {
      const marker: Marker = {
        id: 'marker-1',
        position: position1,
        name: 'Intro',
        type: 'section',
        color: '#3b82f6',
      };

      expect(marker.id).toBe('marker-1');
      expect(marker.name).toBe('Intro');
      expect(marker.type).toBe('section');
      expect(marker.color).toBe('#3b82f6');
    });

    it('should support all marker types', () => {
      const types: Array<Marker['type']> = ['section', 'rehearsal', 'custom'];

      types.forEach(type => {
        const marker: Marker = {
          id: 'test',
          position: position1,
          name: 'Test',
          type,
          color: '#000',
        };

        expect(marker.type).toBe(type);
      });
    });
  });

  describe('Edge cases', () => {
    it('should handle many markers (1000+)', () => {
      for (let i = 0; i < 1000; i++) {
        manager.addMarker({
          position: { measure: i, beat: 0, tick: 0, totalBeats: i * 4 },
          name: `Marker ${i}`,
          type: 'custom',
          color: '#000',
        });
      }

      expect(manager.getAll()).toHaveLength(1000);
    });

    it('should handle markers at same position', () => {
      manager.addMarker({ position: position1, name: 'A', type: 'section', color: '#000' });
      manager.addMarker({ position: position1, name: 'B', type: 'rehearsal', color: '#000' });

      const all = manager.getAll();
      expect(all).toHaveLength(2);
      expect(all[0].position.totalBeats).toBe(all[1].position.totalBeats);
    });

    it('should handle markers with very large beat values', () => {
      const largePosition = { measure: 10000, beat: 0, tick: 0, totalBeats: 40000 };
      const id = manager.addMarker({
        position: largePosition,
        name: 'Far',
        type: 'custom',
        color: '#000',
      });

      expect(manager.getMarker(id)?.position.totalBeats).toBe(40000);
    });

    it('should handle markers with fractional positions', () => {
      const fractionalPos = { measure: 0, beat: 0, tick: 240, totalBeats: 0.5 };
      const id = manager.addMarker({
        position: fractionalPos,
        name: 'Half',
        type: 'custom',
        color: '#000',
      });

      expect(manager.getMarker(id)?.position.tick).toBe(240);
    });

    it('should handle long marker names', () => {
      const longName = 'A'.repeat(1000);
      const id = manager.addMarker({
        position: position1,
        name: longName,
        type: 'custom',
        color: '#000',
      });

      expect(manager.getMarker(id)?.name).toBe(longName);
    });

    it('should handle special characters in names', () => {
      const specialName = 'Marker!@#$%^&*()_+-=[]{}|;:,.<>?/~`';
      const id = manager.addMarker({
        position: position1,
        name: specialName,
        type: 'custom',
        color: '#000',
      });

      expect(manager.getMarker(id)?.name).toBe(specialName);
    });
  });

  describe('Integration tests', () => {
    it('should support full song structure workflow', () => {
      const intro = manager.addMarker({ position: { ...position1, totalBeats: 0 }, name: 'Intro', type: 'section', color: '#3b82f6' });
      const verse1 = manager.addMarker({ position: { ...position1, totalBeats: 16 }, name: 'Verse 1', type: 'section', color: '#3b82f6' });
      const chorus = manager.addMarker({ position: { ...position1, totalBeats: 32 }, name: 'Chorus', type: 'section', color: '#10b981' });
      const verse2 = manager.addMarker({ position: { ...position1, totalBeats: 48 }, name: 'Verse 2', type: 'section', color: '#3b82f6' });
      const bridge = manager.addMarker({ position: { ...position1, totalBeats: 64 }, name: 'Bridge', type: 'section', color: '#f59e0b' });
      const outro = manager.addMarker({ position: { ...position1, totalBeats: 80 }, name: 'Outro', type: 'section', color: '#ef4444' });

      expect(manager.getAll()).toHaveLength(6);
      expect(manager.getAll()[0].name).toBe('Intro');
      expect(manager.getAll()[5].name).toBe('Outro');
    });

    it('should support rehearsal mark workflow', () => {
      manager.addMarker({ position: { ...position1, totalBeats: 0 }, name: 'A', type: 'rehearsal', color: '#10b981' });
      manager.addMarker({ position: { ...position1, totalBeats: 16 }, name: 'B', type: 'rehearsal', color: '#10b981' });
      manager.addMarker({ position: { ...position1, totalBeats: 32 }, name: 'C', type: 'rehearsal', color: '#10b981' });

      const allRehearsalMarks = manager.getAll().filter(m => m.type === 'rehearsal');
      expect(allRehearsalMarks).toHaveLength(3);
    });

    it('should support navigation workflow', () => {
      manager.addMarker({ position: { ...position1, totalBeats: 0 }, name: 'Start', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position1, totalBeats: 20 }, name: 'Middle', type: 'section', color: '#000' });
      manager.addMarker({ position: { ...position1, totalBeats: 40 }, name: 'End', type: 'section', color: '#000' });

      const currentPos = { measure: 6, beat: 0, tick: 0, totalBeats: 24 };

      const prev = manager.getPrevious(currentPos);
      const next = manager.getNext(currentPos);

      expect(prev?.name).toBe('Middle');
      expect(next?.name).toBe('End');
    });
  });
});
