/**
 * @orpheus/guitar-pro-parser
 *
 * Guitar Pro file format parser for Maestro AI
 * Supports GP7, GP6, and GP5 formats
 */

import type { MaestroProject } from '@orpheus/shared-types';
import { parseGP7 } from './gp7-parser';
import { parseGP5 } from './gp5-parser';
import { convertToMaestro } from './converter';
import { detectVersion, isGuitarProFile, getFileExtension } from './version-detector';
import type { GuitarProData, GuitarProVersion } from './types';

export type { GuitarProData, GuitarProVersion };
export { detectVersion, isGuitarProFile, getFileExtension };
export { convertToMaestro };

/**
 * Options for Guitar Pro file parsing
 */
export interface ParseOptions {
  /** Skip validation (faster but less safe) */
  skipValidation?: boolean;
  /** Include raw Guitar Pro data in result */
  includeRawData?: boolean;
}

/**
 * Result of Guitar Pro file parsing
 */
export interface ParseResult {
  /** Converted Maestro project */
  project: MaestroProject;
  /** Detected Guitar Pro version */
  version: GuitarProVersion;
  /** Raw Guitar Pro data (if includeRawData is true) */
  rawData?: GuitarProData;
  /** Parse warnings (non-fatal issues) */
  warnings?: string[];
}

/**
 * Main parser class for Guitar Pro files
 */
export class GuitarProParser {
  /**
   * Parses a Guitar Pro file from ArrayBuffer
   */
  async parse(fileBuffer: ArrayBuffer, options: ParseOptions = {}): Promise<ParseResult> {
    const warnings: string[] = [];

    // Detect version
    const version = detectVersion(fileBuffer);

    // Parse based on version
    let gpData: GuitarProData;

    switch (version) {
      case 'GP7':
      case 'GP6':
        gpData = await parseGP7(fileBuffer);
        break;

      case 'GP5':
        // GP5 binary format is complex with many version variations
        // For now, recommend converting to GP7 for best results
        try {
          gpData = await parseGP5(fileBuffer);
          warnings.push('GP5 format support is experimental - some features may not be fully preserved');
        } catch (parseError: any) {
          throw new Error(
            `GP5 parsing failed: ${parseError.message}. ` +
            `For best results, please open this file in Guitar Pro 7+ and re-save it as .gp format.`
          );
        }
        break;

      case 'GP4':
      case 'GP3':
        throw new Error(`${version} format not yet supported. Please use GP7/GP6 format.`);

      default:
        throw new Error(`Unsupported Guitar Pro version: ${version}`);
    }

    // Convert to Maestro format
    const project = convertToMaestro(gpData);

    // Validation
    if (!options.skipValidation) {
      this.validate(project, warnings);
    }

    return {
      project,
      version,
      rawData: options.includeRawData ? gpData : undefined,
      warnings: warnings.length > 0 ? warnings : undefined,
    };
  }

  /**
   * Parses a Guitar Pro file from File object (browser)
   */
  async parseFile(file: File, options: ParseOptions = {}): Promise<ParseResult> {
    const buffer = await file.arrayBuffer();
    return this.parse(buffer, options);
  }

  /**
   * Validates converted project
   */
  private validate(project: MaestroProject, warnings: string[]): void {
    // Check for empty tracks
    if ((project.project.composition.tracks || []).length === 0) {
      warnings.push('No tracks found in Guitar Pro file');
    }

    // Check for empty measures
    if ((project.project.composition.measures || []).length === 0) {
      warnings.push('No measures found in Guitar Pro file');
    }

    // Check for invalid tempo
    if (!project.project.metadata.tempo || project.project.metadata.tempo < 20 || project.project.metadata.tempo > 400) {
      warnings.push('Unusual tempo detected - please verify');
    }

    // Check for invalid key signature
    const validKeys = ['C', 'C#', 'Db', 'D', 'D#', 'Eb', 'E', 'F', 'F#', 'Gb', 'G', 'G#', 'Ab', 'A', 'A#', 'Bb', 'B'];
    if (!validKeys.includes(project.project.metadata.key)) {
      warnings.push(`Unusual key signature detected: ${project.project.metadata.key}`);
    }
  }
}

/**
 * Convenience function to parse a Guitar Pro file
 */
export async function parseGuitarProFile(
  fileBuffer: ArrayBuffer,
  options: ParseOptions = {}
): Promise<ParseResult> {
  const parser = new GuitarProParser();
  return parser.parse(fileBuffer, options);
}

/**
 * Convenience function to parse a Guitar Pro File object (browser)
 */
export async function parseGuitarProFileObject(
  file: File,
  options: ParseOptions = {}
): Promise<ParseResult> {
  const parser = new GuitarProParser();
  return parser.parseFile(file, options);
}

// Re-export types
export type * from './types';
