# @maestro-ai/guitar-pro-parser

Guitar Pro file format parser for Orpheus. Imports Guitar Pro files (.gp, .gpx, .gp5) and converts them to Maestro's native `.maestro` format.

## Features

- **GP7 Support** ✅ - Full support for Guitar Pro 7 format (.gp)
- **GP6 Support** ✅ - Full support for Guitar Pro 6 format (.gpx)
- **GP5 Support** 🚧 - Planned (simplified implementation)
- **Automatic Version Detection** - Detects GP3/GP4/GP5/GP6/GP7 automatically
- **Complete Conversion** - Preserves tracks, measures, notes, techniques, and metadata
- **Type-Safe** - Written in TypeScript with comprehensive type definitions

## Installation

```bash
npm install @maestro-ai/guitar-pro-parser
```

## Usage

### Basic Import

```typescript
import { parseGuitarProFile } from '@maestro-ai/guitar-pro-parser';

// Parse a Guitar Pro file
const fileBuffer = await fetch('/path/to/song.gp').then(r => r.arrayBuffer());
const result = await parseGuitarProFile(fileBuffer);

console.log(`Loaded: ${result.project.project.metadata.title}`);
console.log(`Version: ${result.version}`);
console.log(`Tracks: ${result.project.project.composition.tracks.length}`);
console.log(`Measures: ${result.project.project.composition.measures.length}`);
```

### Browser File Upload

```typescript
import { parseGuitarProFileObject } from '@maestro-ai/guitar-pro-parser';

// Handle file input
const fileInput = document.querySelector('input[type="file"]');

fileInput.addEventListener('change', async (event) => {
  const file = event.target.files[0];

  if (file) {
    const result = await parseGuitarProFileObject(file);
    console.log('Imported project:', result.project);
  }
});
```

### With Options

```typescript
import { parseGuitarProFile } from '@maestro-ai/guitar-pro-parser';

const result = await parseGuitarProFile(fileBuffer, {
  skipValidation: false,      // Validate converted project
  includeRawData: true,        // Include raw Guitar Pro data
});

// Access raw Guitar Pro data
if (result.rawData) {
  console.log('Original GP tracks:', result.rawData.tracks);
}

// Check for warnings
if (result.warnings) {
  result.warnings.forEach(warning => console.warn(warning));
}
```

### Version Detection

```typescript
import { detectVersion, isGuitarProFile } from '@maestro-ai/guitar-pro-parser';

// Check if file is a Guitar Pro file
if (isGuitarProFile('song.gp')) {
  console.log('Valid Guitar Pro file');
}

// Detect version
const fileBuffer = await fetch('/path/to/song.gp').then(r => r.arrayBuffer());
const version = detectVersion(fileBuffer);
console.log(`Guitar Pro version: ${version}`);
```

### Advanced Usage

```typescript
import { GuitarProParser } from '@maestro-ai/guitar-pro-parser';

const parser = new GuitarProParser();

// Parse multiple files
const files = ['song1.gp', 'song2.gpx', 'song3.gp5'];

for (const filename of files) {
  try {
    const buffer = await fetch(filename).then(r => r.arrayBuffer());
    const result = await parser.parse(buffer);

    console.log(`✓ ${filename}: ${result.project.project.metadata.title}`);
  } catch (error) {
    console.error(`✗ ${filename}: ${error.message}`);
  }
}
```

## API Reference

### `parseGuitarProFile(fileBuffer, options?)`

Parses a Guitar Pro file from an ArrayBuffer.

**Parameters:**
- `fileBuffer: ArrayBuffer` - The Guitar Pro file data
- `options?: ParseOptions` - Optional parsing options

**Returns:** `Promise<ParseResult>`

### `parseGuitarProFileObject(file, options?)`

Parses a Guitar Pro file from a File object (browser).

**Parameters:**
- `file: File` - The File object from a file input
- `options?: ParseOptions` - Optional parsing options

**Returns:** `Promise<ParseResult>`

### `ParseOptions`

```typescript
interface ParseOptions {
  /** Skip validation (faster but less safe) */
  skipValidation?: boolean;
  /** Include raw Guitar Pro data in result */
  includeRawData?: boolean;
}
```

### `ParseResult`

```typescript
interface ParseResult {
  /** Converted Maestro project */
  project: MaestroProject;
  /** Detected Guitar Pro version */
  version: GuitarProVersion;
  /** Raw Guitar Pro data (if includeRawData is true) */
  rawData?: GuitarProData;
  /** Parse warnings (non-fatal issues) */
  warnings?: string[];
}
```

### Utility Functions

#### `detectVersion(fileBuffer: ArrayBuffer): GuitarProVersion`

Detects the Guitar Pro file version from a file buffer.

**Returns:** `'GP3' | 'GP4' | 'GP5' | 'GP6' | 'GP7'`

#### `isGuitarProFile(filename: string): boolean`

Checks if a filename has a Guitar Pro extension.

**Returns:** `boolean`

#### `getFileExtension(filename: string): string`

Gets the file extension from a filename.

**Returns:** Extension in lowercase (e.g., `'gp'`, `'gpx'`, `'gp5'`)

## Supported Features

### ✅ Fully Supported (GP7/GP6)

- **Metadata**: Title, artist, album, copyright, instructions
- **Tracks**: Multiple instruments with tuning, effects, colors
- **Time Signatures**: Any time signature (4/4, 3/4, 7/8, etc.)
- **Key Signatures**: All major and minor keys
- **Tempo**: Static and changing tempo
- **Measures**: Section markers, repeats, alternate endings
- **Notes**: Fret/string notation, duration, velocity, dynamics
- **Techniques**:
  - Hammer-on / Pull-off
  - Slides (shift, legato, in, out)
  - Bends with multiple points
  - Vibrato (slight, wide)
  - Harmonics (natural, artificial, tap, pinch, semi)
  - Palm mute
  - Let ring
  - Tapping
  - Slap / Pop
  - Dead notes
  - Ghost notes
  - Accents / Staccato
- **Chord Diagrams**: Full chord support with fingerings
- **Lyrics**: Song lyrics with timing
- **Annotations**: Text annotations on beats

### 🚧 Partial Support

- **GP5 Format**: Simplified parsing (some advanced features may be lost)
- **Triplets/Tuplets**: Basic support (may need refinement)

### ❌ Not Supported

- **GP3/GP4 Formats**: Legacy formats (use GP5+ for import)
- **MIDI Effects**: Some advanced MIDI effects may not translate
- **Visual Styling**: Page layout, fonts, colors (layout-specific)

## Conversion Details

### Track Mapping

Guitar Pro tracks are converted to Maestro instrument tracks with:
- Name, color, and instrument type preserved
- Tuning converted from MIDI note numbers to note names
- Volume and pan normalized to 0-1 range
- Effects (chorus, reverb, phaser, tremolo) preserved

### Note Techniques

All Guitar Pro note techniques are mapped to Maestro's technique system:

| Guitar Pro | Maestro |
|------------|---------|
| Hammer-On | `hammer-on` |
| Pull-Off | `pull-off` |
| Slide | `slide` (with direction) |
| Bend | `bend` (with bend points) |
| Vibrato | `vibrato` (with intensity) |
| Harmonic | `harmonic` (with type) |
| Palm Mute | `palm-mute` |
| Let Ring | `let-ring` |
| Tap | `tap` |
| Slap/Pop | `slap` / `pop` |
| Dead Note | `muted` |
| Ghost Note | `ghost` |
| Accent | `accent` |
| Staccato | `staccato` |

### Time Conversion

- **Time Signatures**: Converted to string format (`"4/4"`, `"6/8"`, etc.)
- **Note Durations**: Converted to fractional format (`"1/4"`, `"1/8"`, etc.)
- **Tempo**: Preserved as BPM (beats per minute)

### Key Signatures

Guitar Pro's accidental count system is converted to Maestro's key names:

| Accidentals | Major Key | Minor Key |
|-------------|-----------|-----------|
| -7 | Cb | Ab |
| -6 | Gb | Eb |
| -5 | Db | Bb |
| -4 | Ab | F |
| -3 | Eb | C |
| -2 | Bb | G |
| -1 | F | D |
| 0 | C | A |
| +1 | G | E |
| +2 | D | B |
| +3 | A | F# |
| +4 | E | C# |
| +5 | B | G# |
| +6 | F# | D# |
| +7 | C# | A# |

## Error Handling

The parser provides detailed error messages for common issues:

```typescript
try {
  const result = await parseGuitarProFile(fileBuffer);
  console.log('Success!', result.project);
} catch (error) {
  if (error.message.includes('Unrecognized Guitar Pro file format')) {
    console.error('Not a valid Guitar Pro file');
  } else if (error.message.includes('not yet supported')) {
    console.error('This Guitar Pro version is not yet supported');
  } else {
    console.error('Parse error:', error.message);
  }
}
```

## Warnings

Non-fatal issues are reported as warnings:

```typescript
const result = await parseGuitarProFile(fileBuffer);

if (result.warnings) {
  console.warn('Parse warnings:');
  result.warnings.forEach(warning => console.warn(`  - ${warning}`));
}
```

Common warnings:
- No tracks found in file
- No measures found in file
- Unusual tempo detected
- Unusual key signature detected
- GP5 format simplifications applied

## Performance

### File Size Support

- **Small files (<1MB)**: < 100ms parse time
- **Medium files (1-5MB)**: < 500ms parse time
- **Large files (5-20MB)**: < 2s parse time

### Memory Usage

- GP7/GP6 files are loaded entirely into memory (ZIP extraction)
- Typical memory usage: 2-3x file size during parsing
- Memory is freed after parsing completes

### Optimization Tips

- Use `skipValidation: true` for faster parsing (if validation not needed)
- Omit `includeRawData: true` to reduce memory usage
- Parse files sequentially to avoid memory spikes

## Examples

### Complete Import Workflow

```typescript
import { parseGuitarProFile } from '@maestro-ai/guitar-pro-parser';
import { saveProjectToFile } from '@maestro-ai/project-model';

async function importGuitarProSong(gpFilePath: string, outputPath: string) {
  // 1. Load Guitar Pro file
  const fileBuffer = await fetch(gpFilePath).then(r => r.arrayBuffer());

  // 2. Parse Guitar Pro file
  const result = await parseGuitarProFile(fileBuffer, {
    skipValidation: false,
    includeRawData: false,
  });

  // 3. Log import details
  console.log(`✓ Imported: ${result.project.project.metadata.title}`);
  console.log(`  Version: ${result.version}`);
  console.log(`  Tracks: ${result.project.project.composition.tracks.length}`);
  console.log(`  Measures: ${result.project.project.composition.measures.length}`);

  if (result.warnings) {
    console.warn('Warnings:');
    result.warnings.forEach(w => console.warn(`  - ${w}`));
  }

  // 4. Save as Maestro project
  await saveProjectToFile(result.project, outputPath, {
    pretty: true,
    validate: true,
  });

  console.log(`✓ Saved to: ${outputPath}`);

  return result.project;
}

// Usage
await importGuitarProSong('/path/to/song.gp', '/path/to/song.maestro');
```

## Testing

```bash
npm test
```

## Future Enhancements

- [ ] Full GP5 binary format support
- [ ] GP4 and GP3 legacy format support
- [ ] Advanced MIDI effects translation
- [ ] Batch import utility
- [ ] Guitar Pro export (Maestro → GP7)

## References

- [Guitar Pro File Format Documentation](https://github.com/CoderLine/alphaTab/wiki/Guitar-Pro)
- [alphaTab GP Parser](https://github.com/CoderLine/alphaTab) - Reference implementation
- [TuxGuitar](https://github.com/helge17/tuxguitar) - Open-source Guitar Pro compatible editor

## License

MIT

## Author

Orpheus Team
