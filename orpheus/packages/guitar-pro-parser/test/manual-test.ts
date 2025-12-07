/**
 * Manual test script for Guitar Pro Parser
 * Run with: npx ts-node test/manual-test.ts
 */

import * as fs from 'fs';
import * as path from 'path';
import { GuitarProParser, detectVersion } from '../src';

const TEST_FILES_DIR = path.resolve(__dirname, '../../../../docs/Bands');

async function testFile(filePath: string): Promise<void> {
  const fileName = path.basename(filePath);
  console.log(`\n${'='.repeat(60)}`);
  console.log(`Testing: ${fileName}`);
  console.log('='.repeat(60));

  try {
    if (!fs.existsSync(filePath)) {
      console.log(`  File not found: ${filePath}`);
      return;
    }

    const buffer = fs.readFileSync(filePath);
    const arrayBuffer = buffer.buffer.slice(buffer.byteOffset, buffer.byteOffset + buffer.byteLength);

    // Detect version
    const version = detectVersion(arrayBuffer);
    console.log(`  Detected version: ${version}`);

    // Parse file
    const parser = new GuitarProParser();
    const result = await parser.parse(arrayBuffer);

    console.log(`  Parse successful!`);
    console.log(`  - Title: "${result.project.project.metadata.title}"`);
    console.log(`  - Artist: "${result.project.project.metadata.artist}"`);
    console.log(`  - Album: "${result.project.project.metadata.album || 'N/A'}"`);
    console.log(`  - Tempo: ${result.project.project.metadata.tempo} BPM`);
    console.log(`  - Key: ${result.project.project.metadata.key}`);
    console.log(`  - Time: ${result.project.project.metadata.timeSignature.numerator}/${result.project.project.metadata.timeSignature.denominator}`);

    const tracks = result.project.project.composition.tracks || [];
    console.log(`  - Tracks (${tracks.length}):`);
    tracks.forEach((track: any, i: number) => {
      console.log(`      ${i + 1}. ${track.name} (${track.instrument?.type || 'unknown'})`);
    });

    const measures = result.project.project.composition.measures || [];
    console.log(`  - Measures: ${measures.length}`);

    if (result.warnings && result.warnings.length > 0) {
      console.log(`  - Warnings:`);
      result.warnings.forEach(w => console.log(`      * ${w}`));
    }

  } catch (error: any) {
    console.log(`  FAILED: ${error.message}`);
    console.error(error.stack);
  }
}

async function main(): Promise<void> {
  console.log('Guitar Pro Parser Test Suite');
  console.log('============================\n');

  // GP5 files
  const gp5Files = [
    path.join(TEST_FILES_DIR, 'Gojira/Gojira - Magma.gp5'),
    path.join(TEST_FILES_DIR, 'Gojira/Gojira - Pray.gp5'),
    path.join(TEST_FILES_DIR, 'Gojira/Gojira - Stranded (ver 4 by Playtodie).gp5'),
    path.join(TEST_FILES_DIR, 'Necrophagist - Foul Body Autopsy.gp5'),
  ];

  // GP7 files
  const gp7Files = [
    path.join(TEST_FILES_DIR, 'Cradle of Filth/Cradle Of Filth - Honey And Sulphur (guitar pro)(2).gp'),
  ];

  console.log('Testing GP5 files...');
  for (const file of gp5Files) {
    await testFile(file);
  }

  console.log('\n\nTesting GP7 files...');
  for (const file of gp7Files) {
    await testFile(file);
  }

  console.log('\n\nAll tests completed!');
}

main().catch(console.error);
