# Fluent UI to @persona-framework/ui Migration Guide

This document provides patterns and examples for migrating the remaining 34 orpheus components from Fluent UI to @persona-framework/ui (Radix UI + Tailwind CSS).

## Completed Migrations

The following files have been successfully migrated:

1. `/src/contexts/ThemeContext.tsx` - Theme provider using UIThemeProvider
2. `/src/App.tsx` - Main app with Tailwind CSS classes
3. `/src/components/Toolbar.tsx` - Toolbar with all UI components migrated
4. `/src/components/ConfirmDialog.tsx` - Dialog component migrated
5. `/src/components/MessageDialog.tsx` - Message dialog migrated
6. `/src/components/AIAssistant.tsx` - AI assistant panel migrated
7. `/src/components/AIQuickActions.tsx` - Quick actions component migrated

## Migration Patterns

### 1. Import Replacements

#### Component Imports
```typescript
// BEFORE (Fluent UI)
import {
  Button,
  Dialog,
  DialogSurface,
  DialogBody,
  DialogTitle,
  DialogContent,
  DialogActions,
  Input,
  Spinner,
  makeStyles,
  shorthands,
  tokens,
} from '@fluentui/react-components';

// AFTER (@persona-framework/ui)
import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  Input,
  Spinner,
} from '@persona-framework/ui';
```

#### Icon Imports
```typescript
// BEFORE (Fluent UI)
import {
  Save24Regular,
  Delete24Regular,
  Play24Regular,
  Pause24Regular,
  Warning24Regular,
  Info24Regular,
  CheckmarkCircle24Regular,
  ErrorCircle24Regular,
} from '@fluentui/react-icons';

// AFTER (lucide-react)
import {
  Save,
  Trash2,
  Play,
  Pause,
  AlertTriangle,
  Info,
  CheckCircle2,
  XCircle,
} from 'lucide-react';
```

### 2. Component API Changes

#### Button
```typescript
// BEFORE
<Button appearance="primary">Save</Button>
<Button appearance="subtle">Cancel</Button>
<Button appearance="secondary">Reset</Button>

// AFTER
<Button variant="default">Save</Button>
<Button variant="ghost">Cancel</Button>
<Button variant="outline">Reset</Button>
```

#### Dialog
```typescript
// BEFORE
<Dialog open={open} onOpenChange={(_, data) => data.open || onClose()}>
  <DialogSurface>
    <DialogBody>
      <DialogTitle>Title</DialogTitle>
      <DialogContent>Content</DialogContent>
      <DialogActions>
        <Button>OK</Button>
      </DialogActions>
    </DialogBody>
  </DialogSurface>
</Dialog>

// AFTER
<Dialog open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Title</DialogTitle>
      <DialogDescription>Content</DialogDescription>
    </DialogHeader>
    <DialogFooter>
      <Button>OK</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
```

#### Tooltip
```typescript
// BEFORE
<Tooltip content="Save file" relationship="label">
  <ToolbarButton icon={<Save24Regular />}>
    Save
  </ToolbarButton>
</Tooltip>

// AFTER
<Tooltip>
  <TooltipTrigger asChild>
    <Button variant="ghost" size="sm">
      <Save className="h-4 w-4 mr-2" />
      Save
    </Button>
  </TooltipTrigger>
  <TooltipContent>Save file</TooltipContent>
</Tooltip>
```

### 3. makeStyles to Tailwind CSS

#### Basic Styling
```typescript
// BEFORE
const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground2,
  },
  title: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground1,
  },
});

// Usage
<div className={styles.container}>
  <div className={styles.title}>Title</div>
</div>

// AFTER
<div className="flex flex-col gap-4 p-4 bg-secondary">
  <div className="text-base font-semibold text-foreground">Title</div>
</div>
```

#### Color Mappings
```
Fluent UI Token                    → Tailwind Class
────────────────────────────────────────────────────
colorNeutralBackground1            → bg-background
colorNeutralBackground2            → bg-secondary
colorNeutralBackground3            → bg-muted
colorNeutralForeground1            → text-foreground
colorNeutralForeground2            → text-muted-foreground
colorNeutralForeground3            → text-muted-foreground
colorNeutralStroke1                → border-border
colorBrandBackground               → bg-primary
colorBrandForeground1              → text-primary
colorPaletteRedForeground1         → text-red-500
colorPaletteGreenForeground1       → text-green-500
colorPaletteYellowForeground1      → text-yellow-500
colorPaletteBlueForeground1        → text-blue-500
```

#### Spacing Mappings
```
Fluent UI                → Tailwind
──────────────────────────────────
padding('4px')           → p-1
padding('8px')           → p-2
padding('12px')          → p-3
padding('16px')          → p-4
padding('20px')          → p-5
padding('24px')          → p-6
gap('4px')               → gap-1
gap('8px')               → gap-2
gap('16px')              → gap-4
margin('8px')            → m-2
margin('16px')           → m-4
borderRadius('4px')      → rounded
borderRadius('8px')      → rounded-lg
borderRadius('12px')     → rounded-xl
```

### 4. Common Icon Mappings

```
Fluent UI                    → Lucide React
────────────────────────────────────────────
FolderOpen24Regular          → FolderOpen
Save24Regular                → Save
ArrowDownload24Regular       → Download
Play24Regular                → Play
Pause24Regular               → Pause
Stop24Regular                → Square
Delete24Regular              → Trash2
Dismiss24Regular             → X
Send24Regular                → Send
Warning24Regular             → AlertTriangle
Info24Regular                → Info
CheckmarkCircle24Regular     → CheckCircle2
ErrorCircle24Regular         → XCircle
ArrowUndo24Regular           → Undo
ArrowRedo24Regular           → Redo
QuestionCircle24Regular      → HelpCircle
Settings24Regular            → Settings
Search24Regular              → Search
Add24Regular                 → Plus
BotRegular                   → Bot
Lightbulb24Regular           → Lightbulb
MusicNote224Regular          → Music2
BookInformation24Regular     → BookOpen
SlideText24Regular           → ListChecks
```

### 5. Complete Migration Example

Here's a complete example of migrating a simple component:

```typescript
// BEFORE
import {
  makeStyles,
  shorthands,
  tokens,
  Button,
  Dialog,
  DialogSurface,
  DialogBody,
  DialogTitle,
  DialogContent,
} from '@fluentui/react-components';
import { Save24Regular, Delete24Regular } from '@fluentui/react-icons';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    ...shorthands.gap('8px'),
    ...shorthands.padding('16px'),
  },
  button: {
    backgroundColor: tokens.colorBrandBackground,
  },
});

export function MyComponent() {
  const styles = useStyles();

  return (
    <div className={styles.container}>
      <Button appearance="primary" icon={<Save24Regular />}>
        Save
      </Button>
      <Button appearance="subtle" icon={<Delete24Regular />}>
        Delete
      </Button>
    </div>
  );
}

// AFTER
import { Button } from '@persona-framework/ui';
import { Save, Trash2 } from 'lucide-react';

export function MyComponent() {
  return (
    <div className="flex gap-2 p-4">
      <Button variant="default">
        <Save className="h-4 w-4 mr-2" />
        Save
      </Button>
      <Button variant="ghost">
        <Trash2 className="h-4 w-4 mr-2" />
        Delete
      </Button>
    </div>
  );
}
```

## Remaining Files to Migrate (34)

### Components (27)
- ChannelStrip.tsx
- ChannelStripWithProcessors.tsx
- ChordLibraryDialog.tsx
- CommandPalette.tsx
- Compressor.tsx
- DragDropZone.tsx
- EffectsRack.tsx
- ErrorBoundary.tsx
- ExportDialog.tsx
- Fretboard.tsx
- InputLevelMeter.tsx
- KeyboardShortcutsDialog.tsx
- LoadingOverlay.tsx
- LUFSMeter.tsx
- MasteringChain.tsx
- Metronome.tsx
- ModeSelector.tsx
- NewProjectDialog.tsx
- OnboardingDialog.tsx
- ParametricEQ.tsx
- Sidebar.tsx
- SkeletonLoader.tsx
- SpeedTrainer.tsx
- StatusBar.tsx
- TabEditor.tsx
- TechniquePicker.tsx
- ToastContainer.tsx
- WaveformVisualizer.tsx

### Modes (6)
- ComposeMode.tsx
- DistributeMode.tsx
- MasterMode.tsx
- MixMode.tsx
- PracticeMode.tsx
- RecordMode.tsx

## Migration Checklist

For each file:
1. [ ] Replace Fluent UI component imports with @persona-framework/ui imports
2. [ ] Replace Fluent UI icon imports with lucide-react imports
3. [ ] Remove makeStyles, tokens, shorthands imports
4. [ ] Convert makeStyles classes to Tailwind CSS classes
5. [ ] Update Button appearance props to variant props
6. [ ] Update Dialog structure (DialogSurface → DialogContent, etc.)
7. [ ] Update Tooltip pattern (use TooltipTrigger/TooltipContent)
8. [ ] Test the component for visual and functional correctness

## Testing

After migration, verify:
- All UI components render correctly
- Theme switching works properly
- Dialogs open/close correctly
- Tooltips display properly
- Buttons have correct variants
- Icons display correctly
- Spacing and layout match original design
- Responsive behavior is maintained
