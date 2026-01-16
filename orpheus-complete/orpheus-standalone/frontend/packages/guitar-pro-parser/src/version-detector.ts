/**
 * Guitar Pro version detection
 */

import type { GuitarProVersion } from './types';

/**
 * Detects Guitar Pro file version from file buffer
 */
export function detectVersion(fileBuffer: ArrayBuffer): GuitarProVersion {
  const view = new DataView(fileBuffer);

  // Check for ZIP signature (GP7/GP6)
  if (isZipFile(view)) {
    // GP7 and GP6 are both ZIP files containing XML
    // We'll determine the actual version when parsing the XML
    return 'GP7';  // Default to GP7, will be refined during parsing
  }

  // Check for binary GP5/GP4/GP3 signature
  const header = readString(view, 0, 31);

  if (header.includes('FICHIER GUITAR PRO v5')) {
    return 'GP5';
  }

  if (header.includes('FICHIER GUITAR PRO v4')) {
    return 'GP4';
  }

  if (header.includes('FICHIER GUITAR PRO v3')) {
    return 'GP3';
  }

  throw new Error('Unrecognized Guitar Pro file format');
}

/**
 * Checks if file is a ZIP archive (GP7/GP6)
 */
function isZipFile(view: DataView): boolean {
  if (view.byteLength < 4) return false;

  // ZIP files start with PK\x03\x04
  const sig1 = view.getUint8(0);
  const sig2 = view.getUint8(1);
  const sig3 = view.getUint8(2);
  const sig4 = view.getUint8(3);

  return sig1 === 0x50 && sig2 === 0x4B && sig3 === 0x03 && sig4 === 0x04;
}

/**
 * Reads a string from DataView
 */
function readString(view: DataView, offset: number, length: number): string {
  const bytes: number[] = [];

  for (let i = 0; i < length && offset + i < view.byteLength; i++) {
    bytes.push(view.getUint8(offset + i));
  }

  return String.fromCharCode(...bytes);
}

/**
 * Gets file extension from filename
 */
export function getFileExtension(filename: string): string {
  const parts = filename.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

/**
 * Checks if file extension is a known Guitar Pro format
 */
export function isGuitarProFile(filename: string): boolean {
  const ext = getFileExtension(filename);
  return ['gp', 'gp3', 'gp4', 'gp5', 'gpx'].includes(ext);
}
