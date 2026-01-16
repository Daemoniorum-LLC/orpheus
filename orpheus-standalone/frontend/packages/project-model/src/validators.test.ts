import { describe, it, expect } from 'vitest';
import { validateProject, validateScore, validateTrack, type ValidationResult } from './validators';
import { createProject } from './project-factory';

describe('validators.ts - Project Validation', () => {
  describe('validateProject', () => {
    it('should validate valid project', () => {
      const project = createProject({ title: 'Test' });
      const result = validateProject(project);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should reject null project', () => {
      const result = validateProject(null);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Project must be an object');
    });

    it('should reject missing formatVersion', () => {
      const project = createProject({ title: 'Test' });
      delete (project as any).formatVersion;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Missing formatVersion');
    });

    it('should reject invalid formatVersion type', () => {
      const project = createProject({ title: 'Test' });
      (project as any).formatVersion = 123;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('formatVersion must be a string');
    });

    it('should reject missing project data', () => {
      const result = validateProject({ formatVersion: '1.0' });
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Missing project data');
    });

    it('should reject missing metadata', () => {
      const project = createProject({ title: 'Test' });
      delete project.project.metadata;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Missing project metadata');
    });

    it('should reject invalid project id', () => {
      const project = createProject({ title: 'Test' });
      (project.project.metadata as any).id = 123;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid or missing project id');
    });

    it('should reject missing title', () => {
      const project = createProject({ title: 'Test' });
      delete project.project.metadata.title;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid or missing project title');
    });

    it('should reject invalid tempo', () => {
      const project = createProject({ title: 'Test' });
      project.project.metadata.tempo = 0;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid tempo (must be positive number)');
    });

    it('should reject negative tempo', () => {
      const project = createProject({ title: 'Test' });
      project.project.metadata.tempo = -120;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
    });

    it('should validate sample rate', () => {
      const project = createProject({ title: 'Test' });
      project.project.session.sampleRate = 0;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid sampleRate');
    });

    it('should validate bit depth', () => {
      const project = createProject({ title: 'Test' });
      project.project.session.bitDepth = 8;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid bitDepth (must be 16, 24, or 32)');
    });

    it('should accept valid bit depths', () => {
      [16, 24, 32].forEach(bitDepth => {
        const project = createProject({ title: 'Test' });
        project.project.session.bitDepth = bitDepth;
        const result = validateProject(project);
        expect(result.valid).toBe(true);
      });
    });

    it('should require master bus', () => {
      const project = createProject({ title: 'Test' });
      project.project.session.buses = [];
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('session.buses must include a master bus');
    });

    it('should reject missing required sections', () => {
      const project = createProject({ title: 'Test' });
      delete project.project.composition;
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Missing composition section');
    });

    it('should validate ISO 8601 timestamps', () => {
      const project = createProject({ title: 'Test' });
      project.project.metadata.created = 'invalid-date';
      const result = validateProject(project);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Invalid created timestamp');
    });
  });

  describe('validateScore', () => {
    it('should validate valid score', () => {
      const score = {
        id: 'score-1',
        title: 'Guitar',
        instrument: 'electric-guitar',
        tracks: [],
      };
      const result = validateScore(score);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should reject score without id', () => {
      const score = {
        title: 'Guitar',
        instrument: 'electric-guitar',
        tracks: [],
      };
      const result = validateScore(score);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Score must have a valid id');
    });

    it('should reject score without title', () => {
      const score = {
        id: 'score-1',
        instrument: 'electric-guitar',
        tracks: [],
      };
      const result = validateScore(score);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Score must have a title');
    });

    it('should reject score without instrument', () => {
      const score = {
        id: 'score-1',
        title: 'Guitar',
        tracks: [],
      };
      const result = validateScore(score);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Score must have an instrument type');
    });

    it('should reject score without tracks array', () => {
      const score = {
        id: 'score-1',
        title: 'Guitar',
        instrument: 'electric-guitar',
      };
      const result = validateScore(score);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Score must have tracks array');
    });
  });

  describe('validateTrack', () => {
    it('should validate valid track', () => {
      const track = {
        id: 'track-1',
        name: 'Guitar Track',
        type: 'instrument',
        regions: [],
        plugins: [],
      };
      const result = validateTrack(track);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should reject track without id', () => {
      const track = {
        name: 'Guitar Track',
        type: 'instrument',
        regions: [],
        plugins: [],
      };
      const result = validateTrack(track);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Track must have a valid id');
    });

    it('should reject track without name', () => {
      const track = {
        id: 'track-1',
        type: 'instrument',
        regions: [],
        plugins: [],
      };
      const result = validateTrack(track);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Track must have a name');
    });

    it('should validate track types', () => {
      ['audio', 'midi', 'instrument', 'aux'].forEach(type => {
        const track = {
          id: 'track-1',
          name: 'Track',
          type,
          regions: [],
          plugins: [],
        };
        const result = validateTrack(track);
        expect(result.valid).toBe(true);
      });
    });

    it('should reject invalid track type', () => {
      const track = {
        id: 'track-1',
        name: 'Track',
        type: 'invalid',
        regions: [],
        plugins: [],
      };
      const result = validateTrack(track);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Track must have a valid type');
    });

    it('should reject track without regions array', () => {
      const track = {
        id: 'track-1',
        name: 'Track',
        type: 'audio',
        plugins: [],
      };
      const result = validateTrack(track);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Track must have regions array');
    });

    it('should reject track without plugins array', () => {
      const track = {
        id: 'track-1',
        name: 'Track',
        type: 'audio',
        regions: [],
      };
      const result = validateTrack(track);
      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Track must have plugins array');
    });
  });

  describe('ValidationResult interface', () => {
    it('should define validation result structure', () => {
      const result: ValidationResult = {
        valid: true,
        errors: [],
      };

      expect(result.valid).toBe(true);
      expect(result.errors).toEqual([]);
    });

    it('should support multiple errors', () => {
      const result: ValidationResult = {
        valid: false,
        errors: ['Error 1', 'Error 2', 'Error 3'],
      };

      expect(result.valid).toBe(false);
      expect(result.errors).toHaveLength(3);
    });
  });
});
