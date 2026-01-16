import { describe, it, expect } from 'vitest';
import { loadProjectFromJSON } from './project-loader';
import { createProject } from './project-factory';
import { saveProjectToJSON } from './project-saver';

describe('project-loader.ts - Project Loading', () => {
  describe('loadProjectFromJSON', () => {
    it('should load valid project JSON', () => {
      const original = createProject({ title: 'Test' });
      const json = saveProjectToJSON(original, { pretty: false });

      const loaded = loadProjectFromJSON(json);
      expect(loaded.project.metadata.title).toBe('Test');
    });

    it('should throw on invalid JSON', () => {
      expect(() => loadProjectFromJSON('invalid json')).toThrow('Invalid JSON format');
    });

    it('should throw on invalid project structure', () => {
      const invalidProject = JSON.stringify({ invalid: 'structure' });
      expect(() => loadProjectFromJSON(invalidProject)).toThrow('Invalid project file');
    });

    it('should validate loaded project', () => {
      const project = createProject({ title: 'Test' });
      delete project.project.metadata.tempo;
      const json = JSON.stringify(project);

      expect(() => loadProjectFromJSON(json)).toThrow();
    });

    it('should load project with all sections', () => {
      const original = createProject({ title: 'Complete' });
      const json = saveProjectToJSON(original, { pretty: false });

      const loaded = loadProjectFromJSON(json);
      expect(loaded.project.composition).toBeDefined();
      expect(loaded.project.session).toBeDefined();
      expect(loaded.project.mixing).toBeDefined();
      expect(loaded.project.mastering).toBeDefined();
      expect(loaded.project.practice).toBeDefined();
      expect(loaded.project.aiHistory).toBeDefined();
      expect(loaded.project.collaboration).toBeDefined();
    });

    it('should preserve project metadata', () => {
      const original = createProject({ title: 'Test', artist: 'Artist', tempo: 140, key: 'Gm' });
      const json = saveProjectToJSON(original, { pretty: false });

      const loaded = loadProjectFromJSON(json);
      expect(loaded.project.metadata.title).toBe('Test');
      expect(loaded.project.metadata.artist).toBe('Artist');
      expect(loaded.project.metadata.tempo).toBe(140);
      expect(loaded.project.metadata.key).toBe('Gm');
    });

    it('should handle pretty formatted JSON', () => {
      const original = createProject({ title: 'Test' });
      const json = saveProjectToJSON(original, { pretty: true });

      const loaded = loadProjectFromJSON(json);
      expect(loaded.project.metadata.title).toBe('Test');
    });

    it('should handle compact JSON', () => {
      const original = createProject({ title: 'Test' });
      const json = saveProjectToJSON(original, { pretty: false });

      const loaded = loadProjectFromJSON(json);
      expect(loaded.project.metadata.title).toBe('Test');
    });
  });
});
