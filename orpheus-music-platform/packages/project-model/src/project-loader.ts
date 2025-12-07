/**
 * Loader for reading Maestro project files
 */

import type { MaestroProject } from '@orpheus/shared-types';
import { validateProject } from './validators';

/**
 * Loads a Maestro project from JSON string
 */
export function loadProjectFromJSON(json: string): MaestroProject {
  try {
    const project = JSON.parse(json) as MaestroProject;

    // Validate the loaded project
    const validation = validateProject(project);
    if (!validation.valid) {
      throw new Error(`Invalid project file: ${validation.errors.join(', ')}`);
    }

    return project;
  } catch (error) {
    if (error instanceof SyntaxError) {
      throw new Error('Invalid JSON format');
    }
    throw error;
  }
}

/**
 * Loads a Maestro project from a file (Node.js environment)
 */
export async function loadProjectFromFile(filePath: string): Promise<MaestroProject> {
  const fs = await import('fs/promises');

  try {
    const content = await fs.readFile(filePath, 'utf-8');
    return loadProjectFromJSON(content);
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      throw new Error(`Project file not found: ${filePath}`);
    }
    throw error;
  }
}

/**
 * Loads a Maestro project from browser File object
 */
export function loadProjectFromBrowserFile(file: File): Promise<MaestroProject> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = (e) => {
      try {
        const json = e.target?.result as string;
        const project = loadProjectFromJSON(json);
        resolve(project);
      } catch (error) {
        reject(error);
      }
    };

    reader.onerror = () => {
      reject(new Error('Failed to read file'));
    };

    reader.readAsText(file);
  });
}
