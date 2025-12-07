# Orpheus - Mode Switching UX Design

**Version:** 1.0
**Date:** November 16, 2025
**Status:** Specification

## Overview

The mode switching system is the core navigational interface for Orpheus, enabling seamless transitions between Compose, Record, Mix, Master, and Practice modes. This document defines the UX patterns, transitions, state management, and user flows.

## Design Principles

1. **Zero Context Loss:** Switching modes preserves all state and context
2. **Instant Feedback:** Mode switches feel immediate (<200ms)
3. **Clear Mental Model:** Users always know which mode they're in
4. **Workflow Continuity:** Common actions available across relevant modes
5. **Keyboard-First:** Power users can navigate entirely by keyboard

## Mode Navigation Bar

### Primary Location
Fixed header bar at the top of the application, always visible.

```
┌─────────────────────────────────────────────────────────────────┐
│ [🎵 Orpheus]   Project: "My Song"                    [User] │
├─────────────────────────────────────────────────────────────────┤
│ [Compose] [Record] [Mix] [Master] [Practice]    ⚡AI  💾  ⚙️   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                     [MODE CONTENT AREA]                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Mode Buttons

**States:**
- **Active:** Bold text, accent underline, filled background
- **Inactive:** Normal text, transparent background, hover effect
- **Disabled:** Grayed out (e.g., Master mode when no tracks recorded)
- **Modified:** Orange dot indicator (unsaved changes in that mode)

**Visual Design:**
```
Active:    ▼ Compose ▼  (Accent color #0078D4, bold)
Inactive:  Record       (Gray, hover shows light background)
Modified:  Mix •        (Orange dot indicator)
Disabled:  Master       (Gray 50% opacity, no interaction)
```

### Keyboard Shortcuts
- `Cmd/Ctrl + 1`: Compose Mode
- `Cmd/Ctrl + 2`: Record Mode
- `Cmd/Ctrl + 3`: Mix Mode
- `Cmd/Ctrl + 4`: Master Mode
- `Cmd/Ctrl + 5`: Practice Mode
- `Tab`: Cycle forward through modes
- `Shift + Tab`: Cycle backward through modes

## Mode Transition Animations

### Standard Transition (200ms)
```
1. Fade out current mode (100ms)
2. Crossfade to new mode (100ms)
3. Total: 200ms smooth transition
```

### Quick Switch (100ms)
When using keyboard shortcuts:
```
1. Instant crossfade (100ms)
2. Skip fade-out for speed
```

### First-Time Mode Entry
When entering a mode for the first time in a project:
```
1. Show 3-second tooltip overlay
2. Highlight key UI elements
3. Dismissible with click or Esc
```

## State Management

### Persisted State Per Mode

**Compose Mode:**
- Current measure/beat position
- Zoom level
- Selected tracks/scores
- Tool selection (note entry, chord, etc.)
- Playback state

**Record Mode:**
- Timeline view range (start/end)
- Track heights
- Mixer panel state (open/closed)
- Selected track(s)
- Transport state

**Mix Mode:**
- Mixer layout (vertical/horizontal)
- Channel strip visibility
- EQ/dynamics view state
- Selected track(s)
- Automation view mode

**Master Mode:**
- Mastering chain view
- Reference track loaded
- Metering display settings
- Export settings

**Practice Mode:**
- Current loop section
- Speed trainer setting
- Selected difficulty section
- Metronome state

### Global Shared State

Always synchronized across modes:
- Playback position (timeline cursor)
- Tempo
- Time signature
- Key signature
- Project title and metadata
- Undo/redo stack

### State Transitions

```typescript
interface ModeTransition {
  from: MaestroMode;
  to: MaestroMode;
  preservePlaybackPosition: boolean;
  preserveSelection: boolean;
  syncTimeline: boolean;
}

// Example: Compose → Record
{
  from: 'compose',
  to: 'record',
  preservePlaybackPosition: true,    // Keep timeline position
  preserveSelection: true,            // Keep selected score → track
  syncTimeline: true                  // Sync zoom/view
}
```

## Workflow Patterns

### Pattern 1: Compose → Record Workflow

**Scenario:** User finishes writing guitar tab, wants to record it

**Flow:**
1. User in Compose mode, working on guitar score
2. Clicks "Send to Record" button (or Cmd+Shift+R)
3. System:
   - Creates audio track linked to score
   - Switches to Record mode
   - Selects newly created track
   - Arms track for recording
   - Shows "Ready to record [Score Name]" notification
4. User sees:
   - Record mode with new track
   - Playback position preserved
   - Metronome enabled (for click track)
   - Input monitoring active

**UI Indicators:**
```
[Compose Mode]
  ┌─────────────────────────┐
  │ Guitar Score            │
  │ [Score Editor]          │
  │                         │
  │ [Send to Record →] Cmd+Shift+R
  └─────────────────────────┘

[Transition: 200ms]

[Record Mode]
  ┌─────────────────────────┐
  │ ✓ Track "Guitar" created│ (3s toast)
  │ [Timeline]              │
  │ Track 1: Guitar [🔴 REC]│
  │ [Transport Controls]    │
  └─────────────────────────┘
```

### Pattern 2: Record → Mix Workflow

**Scenario:** User finished recording, wants to mix

**Flow:**
1. User stops recording
2. Automatic save of recorded audio
3. Clicks "Mix" mode button
4. System:
   - Opens mixer panel
   - Selects all recorded tracks
   - Shows basic level balancing view
   - Displays AI mixing suggestion button
5. User sees:
   - Full mixer console
   - All tracks with faders
   - AI assistant ready: "Get mixing suggestions"

### Pattern 3: Mix → Master Workflow

**Scenario:** Mix is complete, ready to master

**Flow:**
1. User clicks "Master" mode
2. System:
   - Bounces mix to master bus (if needed)
   - Analyzes loudness metrics
   - Shows AI mastering recommendations
   - Displays target platform presets
3. User sees:
   - Mastering chain view
   - Loudness meters (LUFS)
   - "One-Click Master" AI button
   - Export options

### Pattern 4: Practice Mode Loop

**Scenario:** User wants to practice a difficult section

**Flow:**
1. From Compose or Record mode, user selects measures
2. Clicks Practice mode or presses Cmd+5
3. System:
   - Enters Practice mode
   - Sets loop points to selection
   - Enables metronome
   - Sets speed to 70% for learning
4. User sees:
   - Looped section highlighted
   - Speed slider (50-150%)
   - "Increase Speed" smart button
   - Practice timer

### Pattern 5: Circular Workflow

**Scenario:** Iterative composition and production

```
Compose → Record → Mix → [Listen] → Compose (refine)
    ↑                                     ↓
    └────────── Practice (learn) ─────────┘
```

## Context Preservation

### Timeline Synchronization

**Shared Timeline:**
- Compose mode: Musical time (measures/beats)
- Record/Mix/Master modes: Absolute time (seconds)
- System converts between formats automatically

**Sync Rules:**
```
When tempo changes:
  - Update all time-based calculations
  - Preserve musical positions (measures stay aligned)
  - Audio regions stretch/compress if elastic audio enabled

When switching Compose → Record:
  - Convert measure position to seconds
  - Preserve zoom level (bars → seconds equivalent)
  - Keep playback position aligned

When switching Record → Compose:
  - Convert seconds to nearest measure/beat
  - Snap to musical grid
  - Maintain context of what's being viewed
```

### Selection Preservation

**Score → Track Mapping:**
```typescript
interface ScoreTrackLink {
  scoreId: string;
  trackId: string;
  autoSync: boolean;  // Keep tempo/key in sync
}

// When switching modes with selection:
const preserveSelection = (mode: MaestroMode) => {
  if (from === 'compose' && to === 'record') {
    const linkedTracks = getTracksLinkedToScore(selectedScoreId);
    selectTracks(linkedTracks);
  }

  if (from === 'record' && to === 'compose') {
    const linkedScores = getScoresLinkedToTrack(selectedTrackId);
    selectScores(linkedScores);
  }
};
```

## Smart Mode Suggestions

### AI-Powered Next Actions

**After Recording:**
```
┌────────────────────────────────────┐
│ Recording complete! ✓              │
│                                    │
│ Next steps:                        │
│ → [Mix this track]                 │
│ → [Record another take]            │
│ → [Edit in Compose]                │
└────────────────────────────────────┘
```

**After Mixing:**
```
┌────────────────────────────────────┐
│ Mix sounds great! 🎚️               │
│                                    │
│ Ready to master?                   │
│ → [Master with AI]                 │
│ → [Export stems]                   │
│ → [Refine mix]                     │
└────────────────────────────────────┘
```

### Contextual Mode Recommendations

**AI analyzes project state:**
- No tracks recorded yet → Highlight "Compose" mode
- Recordings exist but unmixed → Suggest "Mix" mode
- Mix complete but not mastered → Suggest "Master" mode
- Difficult sections detected → Suggest "Practice" mode

## Error Handling & Validation

### Prevent Data Loss

**Unsaved Changes:**
```
┌────────────────────────────────────┐
│ ⚠️  Unsaved changes in Mix mode    │
│                                    │
│ Save before switching to Master?   │
│                                    │
│ [Save & Continue]  [Discard]  [Cancel]
└────────────────────────────────────┘
```

**Auto-save Strategy:**
- Save project state every 30 seconds
- Save before mode switches (automatic)
- Save before AI operations
- Maintain undo/redo history across modes

### Mode Prerequisites

**Master Mode Requirements:**
```
If no recorded tracks:
  - Master mode button disabled
  - Tooltip: "Record audio to enable mastering"

If mix bus empty:
  - Master mode available but shows onboarding
  - "Create a mix first" suggestion
```

**Practice Mode Requirements:**
```
If no scores in Compose:
  - Practice mode disabled
  - Tooltip: "Create a score to practice"
```

## Accessibility

### Screen Reader Support
- Announce mode changes: "Entered Compose mode"
- Describe state: "Compose mode, measure 16, Guitar track"
- Mode buttons have proper ARIA labels

### Keyboard Navigation
- All modes fully keyboard accessible
- Focus management on mode switch
- Keyboard shortcuts displayed in tooltips

### Visual Indicators
- High contrast mode button states
- Color + shape + text (not color alone)
- Clear active mode indicator for colorblind users

## Mobile/Tablet Considerations

### Compact Mode Switcher

For screens < 768px:
```
┌─────────────────────────────┐
│ [☰ Menu] Compose    [User]  │
└─────────────────────────────┘

When menu opened:
┌─────────────────────────┐
│ [×] Orpheus          │
├─────────────────────────┤
│ → Compose     (Active)  │
│   Record                │
│   Mix                   │
│   Master                │
│   Practice              │
└─────────────────────────┘
```

### Gesture Support
- Swipe left/right to switch modes
- Two-finger tap to open mode menu
- Long press for mode options

## Performance Targets

### Mode Switch Speed
- **Target:** <200ms total transition time
- **Critical:** <100ms for keyboard shortcuts
- **Loading:** Show skeleton UI if mode takes >500ms

### Memory Management
- Lazy load mode UI (load on first access)
- Keep 2 most recent modes in memory
- Unload unused modes after 5 minutes
- Preserve state even when UI unloaded

### Background Processing
```
When switching away from mode:
  - Save state immediately
  - Continue background tasks (rendering, analysis)
  - Show progress in mode badge: "Mix • 45%"
```

## Example User Scenarios

### Scenario 1: New User Creates First Song

**Step-by-step:**
1. Opens Orpheus → Lands in Compose mode
2. First-time tooltip: "Start by writing your song here"
3. Creates guitar score, adds chords
4. AI suggests: "Nice progression! Want to record it?"
5. Clicks "Send to Record" → Record mode
6. Records guitar → Auto-save
7. AI suggests: "Mix your track for better sound"
8. Switches to Mix mode
9. Uses AI one-click mixing
10. Switches to Master mode
11. Exports final song

**Total mode switches:** 4 (Compose → Record → Mix → Master)
**AI assistance:** 3 interventions
**Time to finished song:** ~30 minutes

### Scenario 2: Experienced User Iteration

**Workflow:**
1. Compose → Record → Mix → Listen
2. Hears issue in composition
3. Mix → Compose (Cmd+1)
4. Fixes chord progression
5. Compose → Record (Cmd+Shift+R)
6. Re-records section
7. Record → Mix (Cmd+3)
8. Tweaks mix
9. Mix → Master (Cmd+4)
10. Exports

**Total mode switches:** 6
**Keyboard shortcuts used:** 5/6
**Time to iteration:** ~5 minutes

## Technical Implementation

### State Manager Structure

```typescript
interface ModeState {
  currentMode: MaestroMode;
  modeHistory: MaestroMode[];  // For back button
  modeStates: {
    compose: ComposeState;
    record: RecordState;
    mix: MixState;
    master: MasterState;
    practice: PracticeState;
  };
  transitionInProgress: boolean;
  unsavedChanges: Set<MaestroMode>;
}

class ModeManager {
  async switchMode(to: MaestroMode): Promise<void> {
    // 1. Check for unsaved changes
    if (this.hasUnsavedChanges(this.currentMode)) {
      const confirmed = await this.confirmSwitch();
      if (!confirmed) return;
    }

    // 2. Save current mode state
    await this.saveMode(this.currentMode);

    // 3. Prepare new mode
    const newState = await this.loadMode(to);

    // 4. Execute transition
    await this.transitionToMode(to, newState);

    // 5. Update UI
    this.updateModeUI(to);
  }

  preserveContext(from: MaestroMode, to: MaestroMode) {
    // Timeline position
    const position = this.getPlaybackPosition();

    // Selection
    const selection = this.getSelection(from);
    const mappedSelection = this.mapSelection(selection, from, to);

    // Apply to new mode
    this.setPlaybackPosition(position, to);
    this.setSelection(mappedSelection, to);
  }
}
```

### React Component Structure

```typescript
function ModeNavigator() {
  const [currentMode, setCurrentMode] = useState<MaestroMode>('compose');
  const [unsavedModes, setUnsavedModes] = useState<Set<MaestroMode>>(new Set());

  const handleModeSwitch = async (newMode: MaestroMode) => {
    if (unsavedModes.has(currentMode)) {
      const confirmed = await confirmDialog('Save changes?');
      if (!confirmed) return;
    }

    await saveCurrentMode();
    await loadNewMode(newMode);
    setCurrentMode(newMode);
  };

  return (
    <ModeBar>
      {MODES.map(mode => (
        <ModeButton
          key={mode}
          mode={mode}
          active={currentMode === mode}
          hasChanges={unsavedModes.has(mode)}
          onClick={() => handleModeSwitch(mode)}
        />
      ))}
    </ModeBar>
  );
}
```

## Future Enhancements

### Split Screen Mode
```
┌────────────────┬────────────────┐
│   Compose      │    Record      │
│   [Tab Editor] │   [Timeline]   │
│                │                │
└────────────────┴────────────────┘
```

### Custom Workflows
Allow users to create custom mode sequences:
- "Recording Workflow": Compose → Record → Record (loops)
- "Mixing Workflow": Record → Mix → Master
- Save as templates

### Mode Plugins
Third-party developers could add custom modes:
- "Video Sync" mode
- "Live Performance" mode
- "Collaboration" mode

---

**Version History:**
- 1.0 (2025-11-16): Initial specification

**Next Steps:**
1. Implement ModeManager class
2. Build React components
3. User testing with musicians
4. Iterate based on feedback
