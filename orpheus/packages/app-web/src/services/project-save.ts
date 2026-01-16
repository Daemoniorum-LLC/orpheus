/**
 * Project Save Service
 * Handles saving and auto-save for Maestro projects
 */

import type { MaestroProject } from '@orpheus/shared-types';
import { saveProjectToBrowserDownload } from '@orpheus/project-model';
import { showSuccess, showError } from './toast';

/**
 * Save a project to browser download
 */
export function saveProject(project: MaestroProject): void {
  if (!project) {
    showError('No project to save', 3000);
    return;
  }

  try {
    const title = project.project.metadata.title || 'Untitled';
    const sanitizedTitle = title.replace(/[^a-z0-9]/gi, '_').toLowerCase();
    const filename = `${sanitizedTitle}.maestro`;

    console.log('[ProjectSave] Saving project:', filename);
    saveProjectToBrowserDownload(project, filename);
    showSuccess(`Saved project: ${filename}`, 3000);
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    showError(`Failed to save project: ${message}`, 6000);
    console.error('[ProjectSave] Save error:', error);
  }
}

/**
 * Auto-save to localStorage for recovery
 */
const AUTOSAVE_KEY = 'orpheus-autosave';
const AUTOSAVE_TIMESTAMP_KEY = 'orpheus-autosave-timestamp';

/**
 * Save project to localStorage for auto-recovery
 */
export function autoSaveToLocalStorage(project: MaestroProject): void {
  try {
    const json = JSON.stringify(project);
    localStorage.setItem(AUTOSAVE_KEY, json);
    localStorage.setItem(AUTOSAVE_TIMESTAMP_KEY, new Date().toISOString());
    console.log('[AutoSave] Saved to localStorage');
  } catch (error) {
    console.error('[AutoSave] Failed to save to localStorage:', error);
  }
}

/**
 * Load auto-saved project from localStorage
 */
export function loadAutoSave(): { project: MaestroProject; timestamp: string } | null {
  try {
    const json = localStorage.getItem(AUTOSAVE_KEY);
    const timestamp = localStorage.getItem(AUTOSAVE_TIMESTAMP_KEY);

    if (!json || !timestamp) {
      return null;
    }

    const project = JSON.parse(json) as MaestroProject;
    console.log('[AutoSave] Loaded from localStorage (saved:', timestamp, ')');
    return { project, timestamp };
  } catch (error) {
    console.error('[AutoSave] Failed to load from localStorage:', error);
    return null;
  }
}

/**
 * Clear auto-save from localStorage
 */
export function clearAutoSave(): void {
  localStorage.removeItem(AUTOSAVE_KEY);
  localStorage.removeItem(AUTOSAVE_TIMESTAMP_KEY);
  console.log('[AutoSave] Cleared from localStorage');
}

/**
 * Auto-save manager
 */
class AutoSaveManager {
  private intervalId: NodeJS.Timeout | null = null;
  private project: MaestroProject | null = null;
  private isEnabled = false;
  private intervalMs = 30000; // 30 seconds default

  /**
   * Start auto-save to localStorage
   */
  start(project: MaestroProject, intervalMs: number = 30000): void {
    this.stop(); // Clear any existing interval

    this.project = project;
    this.intervalMs = intervalMs;
    this.isEnabled = true;

    // Save immediately
    autoSaveToLocalStorage(project);

    // Set up interval
    this.intervalId = setInterval(() => {
      if (this.isEnabled && this.project) {
        console.log('[AutoSave] Auto-saving to localStorage...');
        autoSaveToLocalStorage(this.project);
      }
    }, this.intervalMs);

    console.log(`[AutoSave] Started (interval: ${intervalMs}ms)`);
  }

  /**
   * Update the project reference for auto-save
   */
  updateProject(project: MaestroProject): void {
    this.project = project;
  }

  /**
   * Stop auto-save
   */
  stop(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
    this.isEnabled = false;
    this.project = null;
    console.log('[AutoSave] Stopped');
  }

  /**
   * Check if auto-save is enabled
   */
  enabled(): boolean {
    return this.isEnabled;
  }

  /**
   * Toggle auto-save
   */
  toggle(project: MaestroProject): void {
    if (this.isEnabled) {
      this.stop();
    } else {
      this.start(project);
    }
  }
}

// Singleton instance
let autoSaveManager: AutoSaveManager | null = null;

/**
 * Get the auto-save manager instance
 */
export function getAutoSaveManager(): AutoSaveManager {
  if (!autoSaveManager) {
    autoSaveManager = new AutoSaveManager();
  }
  return autoSaveManager;
}
