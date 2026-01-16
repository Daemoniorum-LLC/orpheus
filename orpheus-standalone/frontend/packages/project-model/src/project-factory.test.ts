import { describe, it, expect } from 'vitest';
import { createProject, createExampleProject, type CreateProjectOptions } from './project-factory';

describe('project-factory.ts - Project Creation', () => {
  describe('createProject', () => {
    it('should create project with title', () => {
      const project = createProject({ title: 'My Song' });
      expect(project.project.metadata.title).toBe('My Song');
    });

    it('should create project with default tempo', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.metadata.tempo).toBe(120);
    });

    it('should create project with custom tempo', () => {
      const project = createProject({ title: 'Test', tempo: 140 });
      expect(project.project.metadata.tempo).toBe(140);
    });

    it('should create project with default key', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.metadata.key).toBe('C');
    });

    it('should create project with custom key', () => {
      const project = createProject({ title: 'Test', key: 'Gm' });
      expect(project.project.metadata.key).toBe('Gm');
    });

    it('should create project with default time signature', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.metadata.timeSignature).toEqual({ numerator: 4, denominator: 4 });
    });

    it('should create project with custom time signature', () => {
      const project = createProject({ title: 'Test', timeSignature: { numerator: 3, denominator: 4 } });
      expect(project.project.metadata.timeSignature).toEqual({ numerator: 3, denominator: 4 });
    });

    it('should generate unique project ID', () => {
      const project1 = createProject({ title: 'Test1' });
      const project2 = createProject({ title: 'Test2' });
      expect(project1.project.metadata.id).not.toBe(project2.project.metadata.id);
    });

    it('should set created timestamp', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.metadata.created).toBeDefined();
      expect(new Date(project.project.metadata.created)).toBeInstanceOf(Date);
    });

    it('should set modified timestamp', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.metadata.modified).toBeDefined();
      expect(new Date(project.project.metadata.modified)).toBeInstanceOf(Date);
    });

    it('should create project with all required sections', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.composition).toBeDefined();
      expect(project.project.session).toBeDefined();
      expect(project.project.mixing).toBeDefined();
      expect(project.project.mastering).toBeDefined();
      expect(project.project.practice).toBeDefined();
      expect(project.project.aiHistory).toBeDefined();
      expect(project.project.collaboration).toBeDefined();
    });

    it('should initialize empty composition arrays', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.composition.scores).toEqual([]);
      expect(project.project.composition.tempoMap).toEqual([]);
      expect(project.project.composition.markers).toEqual([]);
    });

    it('should create master bus', () => {
      const project = createProject({ title: 'Test' });
      const masterBus = project.project.session.buses.find(bus => bus.type === 'master');
      expect(masterBus).toBeDefined();
      expect(masterBus?.name).toBe('Master');
    });

    it('should set format version', () => {
      const project = createProject({ title: 'Test' });
      expect(project.formatVersion).toBe('1.0');
    });

    it('should set default sample rate', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.session.sampleRate).toBe(48000);
    });

    it('should set default bit depth', () => {
      const project = createProject({ title: 'Test' });
      expect(project.project.session.bitDepth).toBe(24);
    });

    it('should include artist when provided', () => {
      const project = createProject({ title: 'Test', artist: 'Test Artist' });
      expect(project.project.metadata.artist).toBe('Test Artist');
    });

    it('should handle various tempos', () => {
      [60, 90, 120, 140, 180, 200].forEach(tempo => {
        const project = createProject({ title: 'Test', tempo });
        expect(project.project.metadata.tempo).toBe(tempo);
      });
    });

    it('should handle various keys', () => {
      ['C', 'Gm', 'F#', 'Bbm'].forEach(key => {
        const project = createProject({ title: 'Test', key });
        expect(project.project.metadata.key).toBe(key);
      });
    });
  });

  describe('createExampleProject', () => {
    it('should create example project', () => {
      const project = createExampleProject();
      expect(project).toBeDefined();
      expect(project.project.metadata.title).toBe('Example Song');
    });

    it('should have example artist', () => {
      const project = createExampleProject();
      expect(project.project.metadata.artist).toBe('Maestro AI Demo');
    });

    it('should include example score', () => {
      const project = createExampleProject();
      expect(project.project.composition.scores).toHaveLength(1);
      expect(project.project.composition.scores[0].title).toBe('Guitar');
    });

    it('should include song structure', () => {
      const project = createExampleProject();
      expect(project.project.composition.songStructure).toBeDefined();
      expect(project.project.composition.songStructure?.sections).toHaveLength(3);
    });

    it('should have guitar tuning', () => {
      const project = createExampleProject();
      const guitar = project.project.composition.scores[0];
      expect(guitar.tuning).toEqual(['E', 'A', 'D', 'G', 'B', 'E']);
    });

    it('should have valid song sections', () => {
      const project = createExampleProject();
      const sections = project.project.composition.songStructure?.sections || [];
      expect(sections[0].name).toBe('Intro');
      expect(sections[1].name).toBe('Verse 1');
      expect(sections[2].name).toBe('Chorus');
    });

    it('should have section colors', () => {
      const project = createExampleProject();
      const sections = project.project.composition.songStructure?.sections || [];
      sections.forEach(section => {
        expect(section.color).toMatch(/^#[0-9a-f]{6}$/i);
      });
    });
  });

  describe('CreateProjectOptions interface', () => {
    it('should accept all valid options', () => {
      const options: CreateProjectOptions = {
        title: 'Test',
        artist: 'Artist',
        tempo: 140,
        key: 'Gm',
        timeSignature: { numerator: 3, denominator: 4 },
      };

      const project = createProject(options);
      expect(project.project.metadata.title).toBe('Test');
      expect(project.project.metadata.artist).toBe('Artist');
      expect(project.project.metadata.tempo).toBe(140);
      expect(project.project.metadata.key).toBe('Gm');
      expect(project.project.metadata.timeSignature).toEqual({ numerator: 3, denominator: 4 });
    });

    it('should accept minimal options', () => {
      const options: CreateProjectOptions = {
        title: 'Test',
      };

      const project = createProject(options);
      expect(project.project.metadata.title).toBe('Test');
    });
  });
});
