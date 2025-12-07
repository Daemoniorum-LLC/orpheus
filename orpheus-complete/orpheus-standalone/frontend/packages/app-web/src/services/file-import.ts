/**
 * File Import Service
 * Handles importing Guitar Pro files and Maestro projects
 */

import { parseGuitarProFile, isGuitarProFile } from '@maestro-ai/guitar-pro-parser';
import type { MaestroProject } from '@maestro-ai/shared-types';

export interface ImportResult {
  success: boolean;
  project?: MaestroProject;
  error?: string;
  warnings?: string[];
  fileName: string;
  fileSize: number;
  importTime: number; // in milliseconds
  rawFileBuffer?: ArrayBuffer; // Original file buffer for alphaTab
}

export interface ImportOptions {
  onProgress?: (progress: number, message: string) => void;
  onCancel?: () => void;
}

/**
 * Import a file (Guitar Pro or Maestro project)
 */
export async function importFile(file: File, options?: ImportOptions): Promise<ImportResult> {
  const startTime = performance.now();
  const fileName = file.name;
  const fileSize = file.size;
  const { onProgress } = options || {};

  try {
    onProgress?.(0, 'Starting import...');

    // Check file type
    const extension = fileName.split('.').pop()?.toLowerCase() || '';
    onProgress?.(10, 'Validating file format...');

    // Maestro project file
    if (extension === 'maestro') {
      return await importMaestroProject(file, startTime, fileName, fileSize, onProgress);
    }

    // Guitar Pro file
    if (isGuitarProFile(fileName)) {
      return await importGuitarProFile(file, startTime, fileName, fileSize, onProgress);
    }

    // Unsupported format
    return {
      success: false,
      error: `Unsupported file format: .${extension}`,
      fileName,
      fileSize,
      importTime: performance.now() - startTime,
    };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error occurred',
      fileName,
      fileSize,
      importTime: performance.now() - startTime,
    };
  }
}

/**
 * Import a Maestro project file
 */
async function importMaestroProject(
  file: File,
  startTime: number,
  fileName: string,
  fileSize: number,
  onProgress?: (progress: number, message: string) => void
): Promise<ImportResult> {
  try {
    onProgress?.(20, 'Reading Maestro project file...');
    const text = await file.text();

    onProgress?.(60, 'Parsing project data...');
    const project = JSON.parse(text) as MaestroProject;

    onProgress?.(80, 'Validating project structure...');
    // Basic validation
    if (!project.formatVersion || !project.project) {
      throw new Error('Invalid Maestro project file');
    }

    onProgress?.(100, 'Import complete!');
    return {
      success: true,
      project,
      fileName,
      fileSize,
      importTime: performance.now() - startTime,
    };
  } catch (error) {
    throw new Error(
      `Failed to parse Maestro project: ${
        error instanceof Error ? error.message : 'Invalid JSON'
      }`
    );
  }
}

/**
 * Import a Guitar Pro file
 */
async function importGuitarProFile(
  file: File,
  startTime: number,
  fileName: string,
  fileSize: number,
  onProgress?: (progress: number, message: string) => void
): Promise<ImportResult> {
  try {
    console.log(`[FileImport] Importing Guitar Pro file: ${fileName} (${(fileSize / 1024).toFixed(2)} KB)`);

    onProgress?.(20, 'Reading Guitar Pro file...');
    // Read file as ArrayBuffer
    const arrayBuffer = await file.arrayBuffer();

    onProgress?.(50, 'Parsing Guitar Pro format...');
    // Parse with guitar-pro-parser
    const result = await parseGuitarProFile(arrayBuffer, {
      skipValidation: false,
      includeRawData: false,
    });

    onProgress?.(90, 'Processing tablature data...');
    const importTime = performance.now() - startTime;

    console.log(`[FileImport] Import successful in ${importTime.toFixed(2)}ms`);
    console.log(`[FileImport] Version: ${result.version}`);
    console.log(`[FileImport] Title: ${result.project.project.metadata.title}`);
    console.log(`[FileImport] Tracks: ${(result.project.project.composition.tracks || []).length}`);
    console.log(`[FileImport] Measures: ${(result.project.project.composition.measures || []).length}`);

    onProgress?.(100, 'Import complete!');
    return {
      success: true,
      project: result.project,
      warnings: result.warnings,
      fileName,
      fileSize,
      importTime,
      rawFileBuffer: arrayBuffer, // Store for alphaTab rendering
    };
  } catch (error) {
    throw new Error(
      `Failed to import Guitar Pro file: ${
        error instanceof Error ? error.message : 'Parse error'
      }`
    );
  }
}

/**
 * Validate file before import
 */
export function validateFile(file: File): { valid: boolean; error?: string } {
  // Check file size (max 50MB)
  const MAX_SIZE = 50 * 1024 * 1024; // 50MB
  if (file.size > MAX_SIZE) {
    return {
      valid: false,
      error: `File too large (${(file.size / 1024 / 1024).toFixed(2)}MB). Maximum size is 50MB.`,
    };
  }

  // Check file extension
  const extension = file.name.split('.').pop()?.toLowerCase() || '';
  const validExtensions = ['maestro', 'gp', 'gpx', 'gp5', 'gp4', 'gp3'];

  if (!validExtensions.includes(extension)) {
    return {
      valid: false,
      error: `Unsupported file type: .${extension}. Supported: ${validExtensions.join(', ')}`,
    };
  }

  return { valid: true };
}

/**
 * Get file type from filename
 */
export function getFileType(fileName: string): 'maestro' | 'guitar-pro' | 'unknown' {
  const extension = fileName.split('.').pop()?.toLowerCase() || '';

  if (extension === 'maestro') {
    return 'maestro';
  }

  if (isGuitarProFile(fileName)) {
    return 'guitar-pro';
  }

  return 'unknown';
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  } else if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(2)} KB`;
  } else {
    return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
  }
}

/**
 * Create a file input dialog
 */
export function createFileInputDialog(
  accept: string[] = ['.maestro', '.gp', '.gpx', '.gp5', '.gp4', '.gp3'],
  onSelect: (file: File) => void
): void {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = accept.join(',');

  input.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (file) {
      onSelect(file);
    }
  };

  input.click();
}
