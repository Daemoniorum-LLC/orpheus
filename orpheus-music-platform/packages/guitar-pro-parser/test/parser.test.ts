/**
 * Guitar Pro Parser Tests
 */

import * as fs from 'fs';
import * as path from 'path';
import { GuitarProParser, detectVersion, isGuitarProFile } from '../src';

// Test files directory
const TEST_FILES_DIR = path.resolve(__dirname, '../../../../docs/Bands');

describe('GuitarProParser', () => {
  const parser = new GuitarProParser();

  describe('Version Detection', () => {
    it('should detect GP5 files', async () => {
      const gp5File = path.join(TEST_FILES_DIR, 'Gojira/Gojira - Magma.gp5');
      if (fs.existsSync(gp5File)) {
        const buffer = fs.readFileSync(gp5File);
        const version = detectVersion(buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength));
        expect(version).toBe('GP5');
      }
    });

    it('should detect GP7 files', async () => {
      const gp7File = path.join(TEST_FILES_DIR, 'Cradle of Filth/Cradle Of Filth - Honey And Sulphur (guitar pro)(2).gp');
      if (fs.existsSync(gp7File)) {
        const buffer = fs.readFileSync(gp7File);
        const version = detectVersion(buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength));
        expect(version).toBe('GP7');
      }
    });
  });

  describe('File Extension Detection', () => {
    it('should recognize Guitar Pro extensions', () => {
      expect(isGuitarProFile('song.gp')).toBe(true);
      expect(isGuitarProFile('song.gp5')).toBe(true);
      expect(isGuitarProFile('song.gp4')).toBe(true);
      expect(isGuitarProFile('song.gpx')).toBe(true);
      expect(isGuitarProFile('song.mp3')).toBe(false);
      expect(isGuitarProFile('song.txt')).toBe(false);
    });
  });

  describe('GP5 Parsing', () => {
    const gp5Files = [
      'Gojira/Gojira - Magma.gp5',
      'Gojira/Gojira - Pray.gp5',
      'Gojira/Gojira - Stranded (ver 4 by Playtodie).gp5',
      'Necrophagist - Foul Body Autopsy.gp5',
    ];

    gp5Files.forEach(file => {
      it(`should parse ${path.basename(file)}`, async () => {
        const filePath = path.join(TEST_FILES_DIR, file);
        if (!fs.existsSync(filePath)) {
          console.log(`Skipping test: ${file} not found`);
          return;
        }

        const buffer = fs.readFileSync(filePath);
        const arrayBuffer = buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength);

        const result = await parser.parse(arrayBuffer);

        expect(result.version).toBe('GP5');
        expect(result.project).toBeDefined();
        expect(result.project.project.metadata.title).toBeDefined();
        expect(result.project.project.composition.tracks?.length).toBeGreaterThan(0);

        console.log(`Parsed ${file}:`);
        console.log(`  Title: ${result.project.project.metadata.title}`);
        console.log(`  Artist: ${result.project.project.metadata.artist}`);
        console.log(`  Tracks: ${result.project.project.composition.tracks?.length}`);
        console.log(`  Measures: ${result.project.project.composition.measures?.length}`);
        if (result.warnings?.length) {
          console.log(`  Warnings: ${result.warnings.join(', ')}`);
        }
      });
    });
  });

  describe('GP7 Parsing', () => {
    it('should parse Cradle Of Filth - Honey And Sulphur', async () => {
      const filePath = path.join(TEST_FILES_DIR, 'Cradle of Filth/Cradle Of Filth - Honey And Sulphur (guitar pro)(2).gp');
      if (!fs.existsSync(filePath)) {
        console.log('Skipping test: GP7 file not found');
        return;
      }

      const buffer = fs.readFileSync(filePath);
      const arrayBuffer = buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength);

      const result = await parser.parse(arrayBuffer);

      expect(result.version).toMatch(/GP[67]/);
      expect(result.project).toBeDefined();
      expect(result.project.project.metadata.title).toBeDefined();
      expect(result.project.project.composition.tracks?.length).toBeGreaterThan(0);

      console.log(`Parsed GP7 file:`);
      console.log(`  Title: ${result.project.project.metadata.title}`);
      console.log(`  Artist: ${result.project.project.metadata.artist}`);
      console.log(`  Tracks: ${result.project.project.composition.tracks?.length}`);
      console.log(`  Measures: ${result.project.project.composition.measures?.length}`);
    });
  });
});
