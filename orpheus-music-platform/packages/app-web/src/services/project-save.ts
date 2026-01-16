/**
 * Project Save Service
 * Handles saving and auto-save for Maestro projects
 */

import type { MaestroProject } from '@orpheus/shared-types';
import { saveProjectToBrowserDownload } from '@orpheus/project-model';
import { showSuccess, showError, showWarning } from './toast';
import { logger } from '../utils/logger';
import { safeStorage } from '../utils/storage';

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

    logger.project.info('Saving project:', filename);
    saveProjectToBrowserDownload(project, filename);
    showSuccess(`Saved project: ${filename}`, 3000);
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    showError(`Failed to save project: ${message}`, 6000);
    logger.project.error('Save error:', error);
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
// Track if we've already warned about storage issues this session
let hasWarnedAboutStorage = false;

export function autoSaveToLocalStorage(project: MaestroProject): boolean {
  try {
    const json = JSON.stringify(project);
    safeStorage.setItem(AUTOSAVE_KEY, json);
    safeStorage.setItem(AUTOSAVE_TIMESTAMP_KEY, new Date().toISOString());
    logger.autoSave.info('Saved to localStorage');
    hasWarnedAboutStorage = false; // Reset warning flag on success
    return true;
  } catch (error) {
    logger.autoSave.error('Failed to save to localStorage:', error);

    // Only warn once per session to avoid spamming
    if (!hasWarnedAboutStorage) {
      hasWarnedAboutStorage = true;
      const isQuotaError = error instanceof DOMException &&
        (error.name === 'QuotaExceededError' || error.name === 'NS_ERROR_DOM_QUOTA_REACHED');

      if (isQuotaError) {
        showWarning('Auto-save failed: Storage is full. Please save your work manually.', 8000);
      } else {
        showWarning('Auto-save failed. Please save your work manually.', 6000);
      }
    }
    return false;
  }
}

/**
 * Load auto-saved project from localStorage
 */
export function loadAutoSave(): { project: MaestroProject; timestamp: string } | null {
  try {
    const json = safeStorage.getItem(AUTOSAVE_KEY);
    const timestamp = safeStorage.getItem(AUTOSAVE_TIMESTAMP_KEY);

    if (!json || !timestamp) {
      return null;
    }

    const project = JSON.parse(json) as MaestroProject;
    logger.autoSave.info('Loaded from localStorage (saved:', timestamp, ')');
    return { project, timestamp };
  } catch (error) {
    logger.autoSave.error('Failed to load from localStorage:', error);
    return null;
  }
}

/**
 * Clear auto-save from localStorage
 */
export function clearAutoSave(): void {
  safeStorage.removeItem(AUTOSAVE_KEY);
  safeStorage.removeItem(AUTOSAVE_TIMESTAMP_KEY);
  logger.autoSave.info('Cleared from localStorage');
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
        logger.autoSave.debug('Auto-saving to localStorage...');
        autoSaveToLocalStorage(this.project);
      }
    }, this.intervalMs);

    logger.autoSave.info(`Started (interval: ${intervalMs}ms)`);
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
    logger.autoSave.info('Stopped');
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
