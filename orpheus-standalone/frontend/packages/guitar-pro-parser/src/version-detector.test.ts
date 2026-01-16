import { describe, it, expect } from 'vitest';
import { detectVersion, getFileExtension, isGuitarProFile } from './version-detector';

describe('version-detector.ts', () => {
  describe('detectVersion()', () => {
    it('should detect GP7/GP6 from ZIP signature', () => {
      // Create buffer with ZIP signature (PK\x03\x04)
      const buffer = new Uint8Array([0x50, 0x4B, 0x03, 0x04, ...Array(100).fill(0)]).buffer;

      const version = detectVersion(buffer);

      expect(version).toBe('GP7');
    });

    it('should detect GP5 from header', () => {
      const header = 'FICHIER GUITAR PRO v5.00';
      const buffer = new Uint8Array(100);

      for (let i = 0; i < header.length; i++) {
        buffer[i] = header.charCodeAt(i);
      }

      const version = detectVersion(buffer.buffer);

      expect(version).toBe('GP5');
    });

    it('should detect GP4 from header', () => {
      const header = 'FICHIER GUITAR PRO v4.06';
      const buffer = new Uint8Array(100);

      for (let i = 0; i < header.length; i++) {
        buffer[i] = header.charCodeAt(i);
      }

      const version = detectVersion(buffer.buffer);

      expect(version).toBe('GP4');
    });

    it('should detect GP3 from header', () => {
      const header = 'FICHIER GUITAR PRO v3.00';
      const buffer = new Uint8Array(100);

      for (let i = 0; i < header.length; i++) {
        buffer[i] = header.charCodeAt(i);
      }

      const version = detectVersion(buffer.buffer);

      expect(version).toBe('GP3');
    });

    it('should throw error for unrecognized format', () => {
      const buffer = new Uint8Array([0xFF, 0xFF, 0xFF, 0xFF, ...Array(100).fill(0)]).buffer;

      expect(() => detectVersion(buffer)).toThrow('Unrecognized Guitar Pro file format');
    });

    it('should throw error for empty buffer', () => {
      const buffer = new ArrayBuffer(0);

      expect(() => detectVersion(buffer)).toThrow('Unrecognized Guitar Pro file format');
    });

    it('should throw error for random data', () => {
      const buffer = new Uint8Array(100);
      for (let i = 0; i < 100; i++) {
        buffer[i] = Math.floor(Math.random() * 256);
      }

      expect(() => detectVersion(buffer.buffer)).toThrow('Unrecognized Guitar Pro file format');
    });

    it('should detect ZIP even with additional data', () => {
      const buffer = new Uint8Array(200);
      buffer[0] = 0x50; // P
      buffer[1] = 0x4B; // K
      buffer[2] = 0x03;
      buffer[3] = 0x04;

      const version = detectVersion(buffer.buffer);

      expect(version).toBe('GP7');
    });

    it('should handle small buffers (< 31 bytes)', () => {
      const buffer = new Uint8Array(10).buffer;

      expect(() => detectVersion(buffer)).toThrow('Unrecognized Guitar Pro file format');
    });

    it('should detect version from partial header match', () => {
      const buffer = new Uint8Array(100);
      const header = 'FICHIER GUITAR PRO v5 extra data here';

      for (let i = 0; i < header.length && i < buffer.length; i++) {
        buffer[i] = header.charCodeAt(i);
      }

      const version = detectVersion(buffer.buffer);

      expect(version).toBe('GP5');
    });
  });

  describe('getFileExtension()', () => {
    it('should extract .gp extension', () => {
      expect(getFileExtension('song.gp')).toBe('gp');
    });

    it('should extract .gp3 extension', () => {
      expect(getFileExtension('song.gp3')).toBe('gp3');
    });

    it('should extract .gp4 extension', () => {
      expect(getFileExtension('song.gp4')).toBe('gp4');
    });

    it('should extract .gp5 extension', () => {
      expect(getFileExtension('song.gp5')).toBe('gp5');
    });

    it('should extract .gpx extension', () => {
      expect(getFileExtension('song.gpx')).toBe('gpx');
    });

    it('should return lowercase extension', () => {
      expect(getFileExtension('song.GP5')).toBe('gp5');
      expect(getFileExtension('song.GPX')).toBe('gpx');
    });

    it('should handle files with multiple dots', () => {
      expect(getFileExtension('my.song.file.gp5')).toBe('gp5');
    });

    it('should return empty string for files without extension', () => {
      expect(getFileExtension('song')).toBe('');
    });

    it('should handle files with dot at start', () => {
      expect(getFileExtension('.gitignore')).toBe('gitignore');
    });

    it('should handle empty filename', () => {
      expect(getFileExtension('')).toBe('');
    });

    it('should handle filename with only dot', () => {
      expect(getFileExtension('.')).toBe('');
    });

    it('should handle path separators', () => {
      expect(getFileExtension('/path/to/song.gp5')).toBe('gp5');
      expect(getFileExtension('C:\\Users\\Music\\song.gp5')).toBe('gp5');
    });

    it('should handle mixed case extensions', () => {
      expect(getFileExtension('Song.Gp5')).toBe('gp5');
      expect(getFileExtension('SONG.GPX')).toBe('gpx');
    });
  });

  describe('isGuitarProFile()', () => {
    it('should return true for .gp extension', () => {
      expect(isGuitarProFile('song.gp')).toBe(true);
    });

    it('should return true for .gp3 extension', () => {
      expect(isGuitarProFile('song.gp3')).toBe(true);
    });

    it('should return true for .gp4 extension', () => {
      expect(isGuitarProFile('song.gp4')).toBe(true);
    });

    it('should return true for .gp5 extension', () => {
      expect(isGuitarProFile('song.gp5')).toBe(true);
    });

    it('should return true for .gpx extension', () => {
      expect(isGuitarProFile('song.gpx')).toBe(true);
    });

    it('should return false for .txt extension', () => {
      expect(isGuitarProFile('song.txt')).toBe(false);
    });

    it('should return false for .pdf extension', () => {
      expect(isGuitarProFile('song.pdf')).toBe(false);
    });

    it('should return false for .mid extension', () => {
      expect(isGuitarProFile('song.mid')).toBe(false);
    });

    it('should return false for no extension', () => {
      expect(isGuitarProFile('song')).toBe(false);
    });

    it('should be case-insensitive', () => {
      expect(isGuitarProFile('song.GP5')).toBe(true);
      expect(isGuitarProFile('song.GPX')).toBe(true);
      expect(isGuitarProFile('song.Gp')).toBe(true);
    });

    it('should handle files with full paths', () => {
      expect(isGuitarProFile('/path/to/my/song.gp5')).toBe(true);
      expect(isGuitarProFile('C:\\Music\\song.gpx')).toBe(true);
    });

    it('should return false for empty filename', () => {
      expect(isGuitarProFile('')).toBe(false);
    });

    it('should handle files with multiple dots', () => {
      expect(isGuitarProFile('my.favorite.song.gp5')).toBe(true);
    });

    it('should return false for similar but wrong extensions', () => {
      expect(isGuitarProFile('song.gp2')).toBe(false);
      expect(isGuitarProFile('song.gp6')).toBe(false);
      expect(isGuitarProFile('song.gp7')).toBe(false);
      expect(isGuitarProFile('song.guitar')).toBe(false);
    });

    it('should handle filenames with spaces', () => {
      expect(isGuitarProFile('my song.gp5')).toBe(true);
      expect(isGuitarProFile('another song.gpx')).toBe(true);
    });

    it('should handle unicode characters in filename', () => {
      expect(isGuitarProFile('música.gp5')).toBe(true);
      expect(isGuitarProFile('歌曲.gpx')).toBe(true);
    });
  });

  describe('integration tests', () => {
    it('should work together to validate Guitar Pro files', () => {
      const filename = 'my-song.gp5';

      expect(isGuitarProFile(filename)).toBe(true);
      expect(getFileExtension(filename)).toBe('gp5');
    });

    it('should detect GP5 file from buffer and filename', () => {
      const header = 'FICHIER GUITAR PRO v5.00';
      const buffer = new Uint8Array(100);

      for (let i = 0; i < header.length; i++) {
        buffer[i] = header.charCodeAt(i);
      }

      const filename = 'song.gp5';

      expect(isGuitarProFile(filename)).toBe(true);
      expect(getFileExtension(filename)).toBe('gp5');
      expect(detectVersion(buffer.buffer)).toBe('GP5');
    });

    it('should detect GP7 file from buffer and filename', () => {
      const buffer = new Uint8Array([0x50, 0x4B, 0x03, 0x04, ...Array(100).fill(0)]).buffer;
      const filename = 'song.gp';

      expect(isGuitarProFile(filename)).toBe(true);
      expect(getFileExtension(filename)).toBe('gp');
      expect(detectVersion(buffer)).toBe('GP7');
    });

    it('should reject invalid files both by filename and buffer', () => {
      const invalidBuffer = new Uint8Array([0xFF, 0xFF, 0xFF, 0xFF]).buffer;
      const invalidFilename = 'song.txt';

      expect(isGuitarProFile(invalidFilename)).toBe(false);
      expect(() => detectVersion(invalidBuffer)).toThrow();
    });
  });
});
