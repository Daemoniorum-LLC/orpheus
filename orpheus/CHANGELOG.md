# Changelog

All notable changes to the Orpheus project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - Wave 5 (November 16, 2025)

#### File Import System
- **Toast Notification Service** (`toast.ts`)
  - Observer pattern-based notification system
  - Success, error, warning, and info toast types
  - Auto-dismiss with configurable duration
  - Manual dismiss capability
  - Slide-in animations

- **ToastContainer Component** (`ToastContainer.tsx`)
  - Fixed position top-right display
  - Color-coded borders (green/red/yellow/blue)
  - Icon indicators for each toast type
  - Smooth slide-in/fade-out animations
  - Manual dismiss button

- **Drag and Drop System** (`DragDropZone.tsx`)
  - Global drag-and-drop file import
  - Full-screen overlay with drop zone
  - Drag counter for nested element handling
  - Supports .gp3-7, .maestro, and .maestro.json files
  - Same import logic as file picker

- **File Import Integration** (Updated `Toolbar.tsx`)
  - File picker dialog with accept filters
  - Loading states and progress indication
  - Success/error toast notifications
  - Automatic project loading on import
  - Warning display for import issues

#### Save and Auto-Save System
- **Project Save Service** (`project-save.ts`)
  - Manual save: Downloads .maestro file to browser
  - Auto-save: Saves to localStorage every 30 seconds
  - Auto-recovery: Offers recovery on app startup (< 1 hour old)
  - Timestamp tracking for recovery prompts
  - `AutoSaveManager` class for managing intervals

- **Save Integration** (Updated `App.tsx`)
  - Auto-recovery prompt on mount
  - Automatic auto-save start/stop based on project state
  - Project-aware auto-save updates
  - User confirmation dialog for recovery

- **Keyboard Shortcuts** (Updated `useKeyboardShortcuts.ts`)
  - `Ctrl+S` - Manual save (downloads file)
  - Input field detection (skips shortcuts during text entry)
  - Integrated with `saveProject()` service

#### Undo/Redo System
- **History Management** (Updated `app-store.ts`)
  - 50-state history buffer (circular)
  - `undo()` - Navigate backward in history
  - `redo()` - Navigate forward in history
  - `canUndo()` / `canRedo()` - Check availability
  - `updateProject()` - Add to history (for edits)
  - `setProject()` - Reset history (for loads)
  - `projectModified` flag tracking

- **Keyboard Shortcuts**
  - `Ctrl+Z` - Undo last change
  - `Ctrl+Shift+Z` - Redo last undone change
  - Input field detection (disabled during text entry)

- **UI Controls** (Updated `Toolbar.tsx`)
  - Undo button with icon (ArrowUndo24Regular)
  - Redo button with icon (ArrowRedo24Regular)
  - Disabled states when unavailable
  - Tooltips with keyboard shortcut hints

#### New Project Creation
- **Project Templates Service** (`project-templates.ts`)
  - 6 professional templates:
    - **Blank Project**: C major, 120 BPM, 1 guitar track
    - **Rock Band**: E major, 140 BPM, 4 tracks (lead/rhythm guitar, bass, drums)
    - **Blues Trio**: A major, 120 BPM, 3 tracks (guitar, bass, drums)
    - **Jazz Quartet**: Bb major, 160 BPM, 4 tracks (piano, guitar, bass, drums)
    - **Acoustic Duo**: G major, 90 BPM, 2 acoustic guitars
    - **Metal Band**: D major (Drop D), 180 BPM, 4 tracks (lead/rhythm guitar, bass, drums)
  - `createProjectFromTemplate()` function
  - Generates valid MaestroProject with 4 empty measures
  - Configurable title and artist

- **New Project Dialog** (`NewProjectDialog.tsx`)
  - Template selection grid with cards
  - Project title and artist input fields
  - Template details display (tempo, key, tracks)
  - Selected template highlighting
  - Success toast on creation
  - Form reset after creation

- **Compose Mode Integration** (Updated `ComposeMode.tsx`)
  - "New Project" button in empty state
  - Dialog state management
  - Icon integration (DocumentAdd24Regular)

#### Documentation Updates
- **USER_GUIDE.md** - Comprehensive updates:
  - Updated "Getting Started" with New Project and drag-and-drop
  - Added "Auto-Save and Recovery" section
  - Expanded "Compose Mode" with template workflow
  - Updated "Keyboard Shortcuts" with Ctrl+Z, Ctrl+Shift+Z, Ctrl+S
  - Added troubleshooting for drag-and-drop, save/recovery, and undo/redo

- **README.md** - Status updates:
  - Version bumped to 0.3.0
  - Status updated to 85% complete
  - New "Web Application (app-web)" section
  - Wave 5 achievements documented
  - Updated cumulative stats (~28,800+ lines)
  - Updated "What's Operational" checklist

- **CHANGELOG.md** - Created comprehensive changelog

### Technical Details

#### New Files Created (8 total)
1. `maestro/packages/app-web/src/services/toast.ts` (110 lines)
2. `maestro/packages/app-web/src/components/ToastContainer.tsx` (120 lines)
3. `maestro/packages/app-web/src/components/DragDropZone.tsx` (169 lines)
4. `maestro/packages/app-web/src/services/project-save.ts` (162 lines)
5. `maestro/packages/app-web/src/services/project-templates.ts` (171 lines)
6. `maestro/packages/app-web/src/components/NewProjectDialog.tsx` (184 lines)
7. `maestro/USER_GUIDE.md` (updated - 740+ lines)
8. `maestro/CHANGELOG.md` (this file)

#### Modified Files (5 total)
1. `maestro/packages/app-web/src/store/app-store.ts` - Undo/redo system
2. `maestro/packages/app-web/src/hooks/useKeyboardShortcuts.ts` - New shortcuts
3. `maestro/packages/app-web/src/components/Toolbar.tsx` - Undo/redo buttons, save integration
4. `maestro/packages/app-web/src/App.tsx` - Toast container, drag-drop, auto-save/recovery
5. `maestro/packages/app-web/src/modes/ComposeMode.tsx` - New Project dialog

#### Key Patterns Used
- **Observer Pattern**: Toast notification subscription system
- **Singleton Pattern**: Toast manager instance
- **History Pattern**: Undo/redo with circular buffer
- **Template Pattern**: Project creation from templates
- **State Management**: Zustand for global app state
- **React Hooks**: useState, useEffect for lifecycle management

#### Browser API Usage
- **LocalStorage**: Auto-save persistence
- **File API**: FileReader for file import
- **Drag and Drop API**: Native drag/drop events
- **Keyboard Events**: Global keyboard shortcuts

### Build and Testing
- All TypeScript compilation successful (0 errors)
- Vite build completed successfully
- dist/ folder updated with compiled assets
- No runtime errors in browser console

### Git History (8 commits)
1. feat: Add toast notification system and file import feedback
2. feat: Implement project save and auto-save system
3. feat: Add undo/redo system with 50-state history
4. feat: Implement New Project creation with templates
5. build: Update dist with new features
6. docs: Update USER_GUIDE.md with all new features
7. docs: Update README.md with Wave 5 achievements
8. docs: Add comprehensive CHANGELOG.md

---

## [0.2.0] - 2025-11-16 (Waves 3-4)

### Added - Wave 4
- Guitar Pro parser (GP7/GP6 complete XML parsing)
- Guitar Pro to Maestro format conversion
- Complete tablature import with all techniques preserved
- Browser File API support
- Validation with warnings

### Added - Wave 3
- Timeline synchronization system (musical ↔ absolute time)
- Professional audio analysis suite (LUFS, RMS, peak, frequency, phase)
- Guitar Pro file format specification
- Three new AI personas (Composer, Production Tutor, Session Assistant)

---

## [0.1.0] - 2025-11-15 (Waves 1-2)

### Added - Initial Foundation
- Complete `.maestro` file format specification
- 8 TypeScript packages (shared-types, project-model, music-theory, midi-utils, etc.)
- Music theory library (100+ chords, 20+ scales)
- MIDI utilities (player, recorder, converter)
- 7 AI personas (Music Theory Tutor, Guitar Coach, Mixing/Mastering Engineers, etc.)
- Mode switching UX design
- API contracts specification
- Audio engine specification (JUCE C++)

---

## Future Releases

### [0.4.0] - Planned (alphaTab Integration)
- alphaTab library integration
- Tablature rendering in browser
- MIDI playback with Tone.js
- Basic notation editing

### [0.5.0] - Planned (Recording Mode)
- Audio recording with Web Audio API
- Multi-track recording
- Real-time waveform visualization
- Audio export

### [1.0.0] - Planned (Full Release)
- All modes fully functional
- VST plugin hosting
- Real-time collaboration
- Cloud storage integration
- Desktop application (Electron)
- Mobile app (React Native)

---

**Branch:** `claude/unified-music-production-suite-01V1TvASUqur8Dyuj1ia5pTn`
**Ready for:** Merge Request
**Total Lines Added (Wave 5):** ~1,100+ lines of production code
**Total Files Changed:** 13 files (8 new, 5 modified)
