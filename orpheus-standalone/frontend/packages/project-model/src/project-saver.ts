/**
 * Saver for writing Maestro project files
 */

import type { MaestroProject } from '@maestro-ai/shared-types';
import { validateProject } from './validators';

export interface SaveOptions {
  pretty?: boolean;
  validate?: boolean;
}

/**
 * Converts a Maestro project to JSON string
 */
export function saveProjectToJSON(
  project: MaestroProject,
  options: SaveOptions = {}
): string {
  const { pretty = true, validate = true } = options;

  // Validate before saving
  if (validate) {
    const validation = validateProject(project);
    if (!validation.valid) {
      throw new Error(`Cannot save invalid project: ${validation.errors.join(', ')}`);
    }
  }

  // Update modified timestamp
  project.project.metadata.modified = new Date().toISOString();

  return JSON.stringify(project, null, pretty ? 2 : 0);
}

/**
 * Saves a Maestro project to a file (Node.js environment)
 */
export async function saveProjectToFile(
  project: MaestroProject,
  filePath: string,
  options: SaveOptions = {}
): Promise<void> {
  const fs = await import('fs/promises');
  const json = saveProjectToJSON(project, options);

  await fs.writeFile(filePath, json, 'utf-8');
}

/**
 * Triggers a browser download of the project file
 */
export function saveProjectToBrowserDownload(
  project: MaestroProject,
  fileName: string,
  options: SaveOptions = {}
): void {
  const json = saveProjectToJSON(project, options);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);

  const link = document.createElement('a');
  link.href = url;
  link.download = fileName.endsWith('.maestro') ? fileName : `${fileName}.maestro`;
  link.click();

  URL.revokeObjectURL(url);
}
