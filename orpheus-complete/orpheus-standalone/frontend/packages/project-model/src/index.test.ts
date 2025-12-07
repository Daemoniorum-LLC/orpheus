import { describe, it, expect } from 'vitest';
import * as ProjectModel from './index';

describe('index.ts - Package Exports', () => {
  describe('Factory exports', () => {
    it('should export createProject function', () => {
      expect(ProjectModel.createProject).toBeDefined();
      expect(typeof ProjectModel.createProject).toBe('function');
    });

    it('should export createExampleProject function', () => {
      expect(ProjectModel.createExampleProject).toBeDefined();
      expect(typeof ProjectModel.createExampleProject).toBe('function');
    });

    it('should create project using exported function', () => {
      const project = ProjectModel.createProject({ title: 'Test' });
      expect(project.project.metadata.title).toBe('Test');
    });
  });

  describe('Loader exports', () => {
    it('should export loadProjectFromJSON function', () => {
      expect(ProjectModel.loadProjectFromJSON).toBeDefined();
      expect(typeof ProjectModel.loadProjectFromJSON).toBe('function');
    });
  });

  describe('Saver exports', () => {
    it('should export saveProjectToJSON function', () => {
      expect(ProjectModel.saveProjectToJSON).toBeDefined();
      expect(typeof ProjectModel.saveProjectToJSON).toBe('function');
    });
  });

  describe('Validator exports', () => {
    it('should export validateProject function', () => {
      expect(ProjectModel.validateProject).toBeDefined();
      expect(typeof ProjectModel.validateProject).toBe('function');
    });

    it('should export validateScore function', () => {
      expect(ProjectModel.validateScore).toBeDefined();
      expect(typeof ProjectModel.validateScore).toBe('function');
    });

    it('should export validateTrack function', () => {
      expect(ProjectModel.validateTrack).toBeDefined();
      expect(typeof ProjectModel.validateTrack).toBe('function');
    });
  });

  describe('Utils exports', () => {
    it('should export generateId function', () => {
      expect(ProjectModel.generateId).toBeDefined();
      expect(typeof ProjectModel.generateId).toBe('function');
    });

    it('should export cloneProject function', () => {
      expect(ProjectModel.cloneProject).toBeDefined();
      expect(typeof ProjectModel.cloneProject).toBe('function');
    });

    it('should export calculateProjectDuration function', () => {
      expect(ProjectModel.calculateProjectDuration).toBeDefined();
      expect(typeof ProjectModel.calculateProjectDuration).toBe('function');
    });

    it('should export musicalTimeToSeconds function', () => {
      expect(ProjectModel.musicalTimeToSeconds).toBeDefined();
      expect(typeof ProjectModel.musicalTimeToSeconds).toBe('function');
    });

    it('should export secondsToMusicalTime function', () => {
      expect(ProjectModel.secondsToMusicalTime).toBeDefined();
      expect(typeof ProjectModel.secondsToMusicalTime).toBe('function');
    });

    it('should export mergeProjects function', () => {
      expect(ProjectModel.mergeProjects).toBeDefined();
      expect(typeof ProjectModel.mergeProjects).toBe('function');
    });

    it('should export getTracksLinkedToScore function', () => {
      expect(ProjectModel.getTracksLinkedToScore).toBeDefined();
      expect(typeof ProjectModel.getTracksLinkedToScore).toBe('function');
    });

    it('should export getMasterBus function', () => {
      expect(ProjectModel.getMasterBus).toBeDefined();
      expect(typeof ProjectModel.getMasterBus).toBe('function');
    });
  });

  describe('Integration', () => {
    it('should support complete workflow', () => {
      // Create
      const project = ProjectModel.createProject({ title: 'Test Workflow' });

      // Validate
      const validation = ProjectModel.validateProject(project);
      expect(validation.valid).toBe(true);

      // Save
      const json = ProjectModel.saveProjectToJSON(project, { pretty: false });

      // Load
      const loaded = ProjectModel.loadProjectFromJSON(json);
      expect(loaded.project.metadata.title).toBe('Test Workflow');
    });

    it('should provide all necessary functions', () => {
      expect(ProjectModel.createProject).toBeDefined();
      expect(ProjectModel.createExampleProject).toBeDefined();
      expect(ProjectModel.loadProjectFromJSON).toBeDefined();
      expect(ProjectModel.saveProjectToJSON).toBeDefined();
      expect(ProjectModel.validateProject).toBeDefined();
      expect(ProjectModel.validateScore).toBeDefined();
      expect(ProjectModel.validateTrack).toBeDefined();
      expect(ProjectModel.generateId).toBeDefined();
      expect(ProjectModel.cloneProject).toBeDefined();
      expect(ProjectModel.calculateProjectDuration).toBeDefined();
      expect(ProjectModel.musicalTimeToSeconds).toBeDefined();
      expect(ProjectModel.secondsToMusicalTime).toBeDefined();
      expect(ProjectModel.mergeProjects).toBeDefined();
      expect(ProjectModel.getTracksLinkedToScore).toBeDefined();
      expect(ProjectModel.getMasterBus).toBeDefined();
    });
  });
});
