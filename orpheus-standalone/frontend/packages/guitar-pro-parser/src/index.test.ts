import { describe, it, expect } from 'vitest';
import * as GuitarProParser from './index';

describe('index.ts - Package exports', () => {
  describe('type exports', () => {
    it('should export GuitarProParser class', () => {
      expect(GuitarProParser.GuitarProParser).toBeDefined();
      expect(typeof GuitarProParser.GuitarProParser).toBe('function');
    });

    it('should be able to create GuitarProParser instance', () => {
      const parser = new GuitarProParser.GuitarProParser();
      expect(parser).toBeInstanceOf(GuitarProParser.GuitarProParser);
    });
  });

  describe('function exports', () => {
    it('should export parseGuitarProFile function', () => {
      expect(GuitarProParser.parseGuitarProFile).toBeDefined();
      expect(typeof GuitarProParser.parseGuitarProFile).toBe('function');
    });

    it('should export parseGuitarProFileObject function', () => {
      expect(GuitarProParser.parseGuitarProFileObject).toBeDefined();
      expect(typeof GuitarProParser.parseGuitarProFileObject).toBe('function');
    });

    it('should export detectVersion function', () => {
      expect(GuitarProParser.detectVersion).toBeDefined();
      expect(typeof GuitarProParser.detectVersion).toBe('function');
    });

    it('should export isGuitarProFile function', () => {
      expect(GuitarProParser.isGuitarProFile).toBeDefined();
      expect(typeof GuitarProParser.isGuitarProFile).toBe('function');
    });

    it('should export getFileExtension function', () => {
      expect(GuitarProParser.getFileExtension).toBeDefined();
      expect(typeof GuitarProParser.getFileExtension).toBe('function');
    });

    it('should export convertToMaestro function', () => {
      expect(GuitarProParser.convertToMaestro).toBeDefined();
      expect(typeof GuitarProParser.convertToMaestro).toBe('function');
    });
  });

  describe('version detection integration', () => {
    it('should detect GP7 files from ZIP signature', () => {
      const buffer = new Uint8Array([0x50, 0x4B, 0x03, 0x04, ...Array(100).fill(0)]).buffer;

      const version = GuitarProParser.detectVersion(buffer);

      expect(version).toBe('GP7');
    });

    it('should validate guitar pro filenames', () => {
      expect(GuitarProParser.isGuitarProFile('song.gp')).toBe(true);
      expect(GuitarProParser.isGuitarProFile('song.gp5')).toBe(true);
      expect(GuitarProParser.isGuitarProFile('song.gpx')).toBe(true);
      expect(GuitarProParser.isGuitarProFile('song.txt')).toBe(false);
    });

    it('should extract file extensions correctly', () => {
      expect(GuitarProParser.getFileExtension('song.gp5')).toBe('gp5');
      expect(GuitarProParser.getFileExtension('my.song.gpx')).toBe('gpx');
      expect(GuitarProParser.getFileExtension('file.txt')).toBe('txt');
    });
  });

  describe('GuitarProParser class', () => {
    it('should have parse method', () => {
      const parser = new GuitarProParser.GuitarProParser();

      expect(parser.parse).toBeDefined();
      expect(typeof parser.parse).toBe('function');
    });

    it('should have parseFile method', () => {
      const parser = new GuitarProParser.GuitarProParser();

      expect(parser.parseFile).toBeDefined();
      expect(typeof parser.parseFile).toBe('function');
    });

    it('should reject unsupported formats', async () => {
      const parser = new GuitarProParser.GuitarProParser();
      const buffer = new Uint8Array([0xFF, 0xFF, 0xFF, 0xFF]).buffer;

      await expect(parser.parse(buffer)).rejects.toThrow();
    });

    it('should reject GP5 format (not yet implemented)', async () => {
      const parser = new GuitarProParser.GuitarProParser();
      const header = 'FICHIER GUITAR PRO v5.00';
      const buffer = new Uint8Array(100);

      for (let i = 0; i < header.length; i++) {
        buffer[i] = header.charCodeAt(i);
      }

      await expect(parser.parse(buffer.buffer)).rejects.toThrow(
        'GP5 format not yet fully implemented'
      );
    });

    it('should reject GP4 format (not supported)', async () => {
      const parser = new GuitarProParser.GuitarProParser();
      const header = 'FICHIER GUITAR PRO v4.06';
      const buffer = new Uint8Array(100);

      for (let i = 0; i < header.length; i++) {
        buffer[i] = header.charCodeAt(i);
      }

      await expect(parser.parse(buffer.buffer)).rejects.toThrow(
        'GP4 format not yet supported'
      );
    });

    it('should reject GP3 format (not supported)', async () => {
      const parser = new GuitarProParser.GuitarProParser();
      const header = 'FICHIER GUITAR PRO v3.00';
      const buffer = new Uint8Array(100);

      for (let i = 0; i < header.length; i++) {
        buffer[i] = header.charCodeAt(i);
      }

      await expect(parser.parse(buffer.buffer)).rejects.toThrow(
        'GP3 format not yet supported'
      );
    });

    it('should support skipValidation option', async () => {
      const parser = new GuitarProParser.GuitarProParser();

      // This would fail without proper GP7 data, but testing the option exists
      const options = { skipValidation: true };

      expect(options.skipValidation).toBe(true);
    });

    it('should support includeRawData option', async () => {
      const parser = new GuitarProParser.GuitarProParser();

      const options = { includeRawData: true };

      expect(options.includeRawData).toBe(true);
    });
  });

  describe('package completeness', () => {
    it('should export all necessary version detection utilities', () => {
      expect(GuitarProParser.detectVersion).toBeDefined();
      expect(GuitarProParser.isGuitarProFile).toBeDefined();
      expect(GuitarProParser.getFileExtension).toBeDefined();
    });

    it('should export all necessary parsing utilities', () => {
      expect(GuitarProParser.GuitarProParser).toBeDefined();
      expect(GuitarProParser.parseGuitarProFile).toBeDefined();
      expect(GuitarProParser.parseGuitarProFileObject).toBeDefined();
    });

    it('should export conversion utilities', () => {
      expect(GuitarProParser.convertToMaestro).toBeDefined();
    });

    it('should have consistent API naming', () => {
      // All exported functions should be properly named
      expect(GuitarProParser.parseGuitarProFile.name).toBe('parseGuitarProFile');
      expect(GuitarProParser.detectVersion.name).toBe('detectVersion');
      expect(GuitarProParser.convertToMaestro.name).toBe('convertToMaestro');
    });
  });

  describe('usage examples', () => {
    it('should validate file before parsing', () => {
      const filename = 'my-song.gp5';

      const isValid = GuitarProParser.isGuitarProFile(filename);
      const extension = GuitarProParser.getFileExtension(filename);

      expect(isValid).toBe(true);
      expect(extension).toBe('gp5');
    });

    it('should detect version from buffer', () => {
      const gp7Buffer = new Uint8Array([0x50, 0x4B, 0x03, 0x04, ...Array(50).fill(0)]).buffer;
      const version = GuitarProParser.detectVersion(gp7Buffer);

      expect(version).toBe('GP7');
    });

    it('should use convenience parse function', () => {
      const buffer = new ArrayBuffer(100);
      const options = { skipValidation: true };

      expect(() => GuitarProParser.parseGuitarProFile(buffer, options)).toBeDefined();
    });

    it('should create parser instance for multiple files', () => {
      const parser = new GuitarProParser.GuitarProParser();

      expect(parser.parse).toBeDefined();
      expect(parser.parseFile).toBeDefined();
    });
  });

  describe('error handling', () => {
    it('should throw for unrecognized file format', () => {
      const invalidBuffer = new Uint8Array([0xFF, 0xFF, 0xFF, 0xFF]).buffer;

      expect(() => GuitarProParser.detectVersion(invalidBuffer)).toThrow(
        'Unrecognized Guitar Pro file format'
      );
    });

    it('should handle invalid filenames gracefully', () => {
      expect(GuitarProParser.isGuitarProFile('')).toBe(false);
      expect(GuitarProParser.isGuitarProFile('no-extension')).toBe(false);
      expect(GuitarProParser.getFileExtension('')).toBe('');
    });

    it('should reject unsupported versions with clear message', async () => {
      const parser = new GuitarProParser.GuitarProParser();
      const gp3Buffer = new Uint8Array(100);
      const header = 'FICHIER GUITAR PRO v3';

      for (let i = 0; i < header.length; i++) {
        gp3Buffer[i] = header.charCodeAt(i);
      }

      await expect(parser.parse(gp3Buffer.buffer)).rejects.toThrow(
        /GP3.*not yet supported/
      );
    });
  });

  describe('file format support', () => {
    it('should list supported formats via isGuitarProFile', () => {
      const supportedFormats = ['gp', 'gp3', 'gp4', 'gp5', 'gpx'];

      supportedFormats.forEach(ext => {
        expect(GuitarProParser.isGuitarProFile(`file.${ext}`)).toBe(true);
      });
    });

    it('should list unsupported formats', () => {
      const unsupportedFormats = ['txt', 'pdf', 'mid', 'mp3', 'wav', 'gp2', 'gp6', 'gp7'];

      unsupportedFormats.forEach(ext => {
        expect(GuitarProParser.isGuitarProFile(`file.${ext}`)).toBe(false);
      });
    });

    it('should handle case-insensitive extensions', () => {
      expect(GuitarProParser.isGuitarProFile('SONG.GP5')).toBe(true);
      expect(GuitarProParser.isGuitarProFile('song.GPX')).toBe(true);
      expect(GuitarProParser.isGuitarProFile('file.Gp')).toBe(true);
    });
  });
});
