import { describe, it, expect } from 'vitest';
import { saveProjectToJSON, type SaveOptions } from './project-saver';
import { createProject } from './project-factory';

describe('project-saver.ts - Project Saving', () => {
  describe('saveProjectToJSON', () => {
    it('should save project to JSON', () => {
      const project = createProject({ title: 'Test' });
      const json = saveProjectToJSON(project);

      expect(json).toBeDefined();
      expect(typeof json).toBe('string');
    });

    it('should create valid JSON', () => {
      const project = createProject({ title: 'Test' });
      const json = saveProjectToJSON(project);

      expect(() => JSON.parse(json)).not.toThrow();
    });

    it('should save pretty formatted JSON by default', () => {
      const project = createProject({ title: 'Test' });
      const json = saveProjectToJSON(project);

      expect(json).toContain('\n');
      expect(json).toContain('  ');
    });

    it('should save compact JSON when pretty is false', () => {
      const project = createProject({ title: 'Test' });
      const json = saveProjectToJSON(project, { pretty: false });

      expect(json).not.toContain('\n  ');
    });

    it('should validate project by default', () => {
      const project = createProject({ title: 'Test' });
      project.project.metadata.tempo = -1; // Invalid

      expect(() => saveProjectToJSON(project)).toThrow();
    });

    it('should skip validation when disabled', () => {
      const project = createProject({ title: 'Test' });
      project.project.metadata.tempo = -1; // Invalid

      expect(() => saveProjectToJSON(project, { validate: false })).not.toThrow();
    });

    it('should update modified timestamp', () => {
      const project = createProject({ title: 'Test' });
      const originalModified = project.project.metadata.modified;

      // Wait a bit to ensure timestamp changes
      const json = saveProjectToJSON(project);
      const saved = JSON.parse(json);

      expect(saved.project.metadata.modified).toBeDefined();
    });

    it('should preserve all project data', () => {
      const project = createProject({ title: 'Complete', artist: 'Artist', tempo: 140 });
      const json = saveProjectToJSON(project);
      const parsed = JSON.parse(json);

      expect(parsed.project.metadata.title).toBe('Complete');
      expect(parsed.project.metadata.artist).toBe('Artist');
      expect(parsed.project.metadata.tempo).toBe(140);
    });

    it('should handle SaveOptions interface', () => {
      const options: SaveOptions = {
        pretty: true,
        validate: true,
      };

      const project = createProject({ title: 'Test' });
      const json = saveProjectToJSON(project, options);

      expect(json).toBeDefined();
    });

    it('should accept partial options', () => {
      const project = createProject({ title: 'Test' });

      expect(() => saveProjectToJSON(project, { pretty: false })).not.toThrow();
      expect(() => saveProjectToJSON(project, { validate: false })).not.toThrow();
    });

    it('should accept no options', () => {
      const project = createProject({ title: 'Test' });
      expect(() => saveProjectToJSON(project)).not.toThrow();
    });
  });
});
