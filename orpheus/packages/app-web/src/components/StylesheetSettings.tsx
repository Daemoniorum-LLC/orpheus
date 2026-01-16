/**
 * Stylesheet Settings Component - Notation styling and layout customization UI
 */

import { useState, useEffect, useCallback } from 'react';
import {
  makeStyles,
  shorthands,
  tokens,
  Button,
  Dropdown,
  Option,
  Label,
  Slider,
  Switch,
  Input,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogContent,
  DialogActions,
  Tab,
  TabList,
  Tooltip,
  Accordion,
  AccordionHeader,
  AccordionItem,
  AccordionPanel,
  Field,
  Divider,
} from '@fluentui/react-components';
import {
  PaintBrush24Regular,
  TextFont24Regular,
  LayoutColumnTwoSplitRight24Regular,
  Eye24Regular,
  Add24Regular,
  Delete24Regular,
  Copy24Regular,
  ArrowDownload24Regular,
  ArrowUpload24Regular,
  Checkmark24Regular,
  Dismiss24Regular,
} from '@fluentui/react-icons';
import {
  getStylesheetService,
  NotationStylesheet,
  NotationColors,
  NotationLayout,
  NotationDisplay,
  NotationFont,
} from '../services/stylesheet-service';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
    minWidth: '400px',
  },
  presetRow: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  presetDropdown: {
    flex: 1,
  },
  tabs: {
    marginTop: '16px',
  },
  tabContent: {
    ...shorthands.padding('16px', '0'),
    maxHeight: '400px',
    overflowY: 'auto',
  },
  settingRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  settingLabel: {
    fontSize: '14px',
  },
  settingControl: {
    minWidth: '150px',
  },
  colorSwatch: {
    width: '24px',
    height: '24px',
    ...shorthands.borderRadius('4px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    cursor: 'pointer',
  },
  colorInput: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  sliderWithValue: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    minWidth: '150px',
  },
  sliderValue: {
    fontFamily: 'monospace',
    fontSize: '12px',
    minWidth: '40px',
    textAlign: 'right',
  },
  fontRow: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('4px'),
    marginBottom: '8px',
  },
  fontName: {
    fontSize: '12px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground2,
  },
  fontControls: {
    display: 'flex',
    ...shorthands.gap('8px'),
    flexWrap: 'wrap',
  },
  actions: {
    display: 'flex',
    ...shorthands.gap('8px'),
    justifyContent: 'flex-end',
    marginTop: '16px',
    paddingTop: '16px',
    ...shorthands.borderTop('1px', 'solid', tokens.colorNeutralStroke2),
  },
  builtInBadge: {
    fontSize: '10px',
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.padding('2px', '6px'),
    ...shorthands.borderRadius('4px'),
    marginLeft: '8px',
  },
  previewArea: {
    ...shorthands.padding('16px'),
    backgroundColor: 'var(--notation-background, #1E1E1E)',
    color: 'var(--notation-primary, #E0E0E0)',
    ...shorthands.borderRadius('8px'),
    minHeight: '100px',
    fontFamily: 'var(--notation-font-tab, monospace)',
    fontSize: 'var(--notation-font-tab-size, 12px)',
    textAlign: 'center',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    ...shorthands.gap('8px'),
  },
  previewTitle: {
    fontFamily: 'var(--notation-font-title, Georgia)',
    fontSize: 'var(--notation-font-title-size, 24px)',
    fontWeight: 600,
  },
  previewTab: {
    fontFamily: 'var(--notation-font-tab, monospace)',
    fontSize: 'var(--notation-font-tab-size, 12px)',
    letterSpacing: '0.5em',
    color: 'var(--notation-secondary, #9E9E9E)',
  },
});

type TabType = 'presets' | 'colors' | 'fonts' | 'layout' | 'display';

interface StylesheetSettingsProps {
  onClose?: () => void;
  compact?: boolean;
}

export function StylesheetSettings({ onClose, compact = false }: StylesheetSettingsProps) {
  const styles = useStyles();
  const stylesheetService = getStylesheetService();

  const [stylesheets, setStylesheets] = useState<NotationStylesheet[]>([]);
  const [activeStylesheet, setActiveStylesheet] = useState<NotationStylesheet>(
    stylesheetService.getActiveStylesheet()
  );
  const [editedStylesheet, setEditedStylesheet] = useState<NotationStylesheet>(activeStylesheet);
  const [selectedTab, setSelectedTab] = useState<TabType>('presets');
  const [hasChanges, setHasChanges] = useState(false);
  const [newStylesheetName, setNewStylesheetName] = useState('');
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  // Load stylesheets
  useEffect(() => {
    setStylesheets(stylesheetService.getAllStylesheets());
    const unsubscribe = stylesheetService.subscribe((stylesheet) => {
      setActiveStylesheet(stylesheet);
      if (!hasChanges) {
        setEditedStylesheet(stylesheet);
      }
    });
    return () => unsubscribe();
  }, [stylesheetService, hasChanges]);

  // Track changes
  useEffect(() => {
    const changed = JSON.stringify(editedStylesheet) !== JSON.stringify(activeStylesheet);
    setHasChanges(changed);
  }, [editedStylesheet, activeStylesheet]);

  // Select a preset
  const handlePresetChange = useCallback(
    (id: string) => {
      stylesheetService.setActiveStylesheet(id);
      const stylesheet = stylesheetService.getStylesheet(id);
      if (stylesheet) {
        setEditedStylesheet(stylesheet);
      }
    },
    [stylesheetService]
  );

  // Update colors
  const updateColor = useCallback((key: keyof NotationColors, value: string) => {
    setEditedStylesheet((prev) => ({
      ...prev,
      colors: { ...prev.colors, [key]: value },
    }));
  }, []);

  // Update layout
  const updateLayout = useCallback(
    (key: keyof NotationLayout, value: number | string) => {
      setEditedStylesheet((prev) => ({
        ...prev,
        layout: { ...prev.layout, [key]: value },
      }));
    },
    []
  );

  // Update display
  const updateDisplay = useCallback(
    (key: keyof NotationDisplay, value: boolean | string) => {
      setEditedStylesheet((prev) => ({
        ...prev,
        display: { ...prev.display, [key]: value },
      }));
    },
    []
  );

  // Update font
  const updateFont = useCallback(
    (
      fontKey: keyof NotationStylesheet['fonts'],
      property: keyof NotationFont,
      value: string | number
    ) => {
      setEditedStylesheet((prev) => ({
        ...prev,
        fonts: {
          ...prev.fonts,
          [fontKey]: { ...prev.fonts[fontKey], [property]: value },
        },
      }));
    },
    []
  );

  // Save changes
  const handleSave = useCallback(() => {
    if (editedStylesheet.isBuiltIn) {
      // Create a new stylesheet based on the built-in
      const newStylesheet = stylesheetService.createStylesheet({
        ...editedStylesheet,
        name: `${editedStylesheet.name} (Custom)`,
      });
      stylesheetService.setActiveStylesheet(newStylesheet.id);
      setStylesheets(stylesheetService.getAllStylesheets());
    } else {
      stylesheetService.updateStylesheet(editedStylesheet.id, editedStylesheet);
    }
    setHasChanges(false);
  }, [editedStylesheet, stylesheetService]);

  // Revert changes
  const handleRevert = useCallback(() => {
    setEditedStylesheet(activeStylesheet);
    setHasChanges(false);
  }, [activeStylesheet]);

  // Create new stylesheet
  const handleCreate = useCallback(() => {
    if (!newStylesheetName.trim()) return;

    const newStylesheet = stylesheetService.createStylesheet({
      ...editedStylesheet,
      name: newStylesheetName.trim(),
    });
    stylesheetService.setActiveStylesheet(newStylesheet.id);
    setStylesheets(stylesheetService.getAllStylesheets());
    setNewStylesheetName('');
    setShowCreateDialog(false);
  }, [newStylesheetName, editedStylesheet, stylesheetService]);

  // Duplicate stylesheet
  const handleDuplicate = useCallback(() => {
    const duplicated = stylesheetService.duplicateStylesheet(activeStylesheet.id);
    if (duplicated) {
      stylesheetService.setActiveStylesheet(duplicated.id);
      setStylesheets(stylesheetService.getAllStylesheets());
    }
  }, [activeStylesheet, stylesheetService]);

  // Delete stylesheet
  const handleDelete = useCallback(() => {
    if (activeStylesheet.isBuiltIn) return;
    if (confirm(`Delete stylesheet "${activeStylesheet.name}"?`)) {
      stylesheetService.deleteStylesheet(activeStylesheet.id);
      setStylesheets(stylesheetService.getAllStylesheets());
    }
  }, [activeStylesheet, stylesheetService]);

  // Export stylesheet
  const handleExport = useCallback(() => {
    const json = stylesheetService.exportStylesheet(activeStylesheet.id);
    if (json) {
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${activeStylesheet.name.replace(/\s+/g, '-').toLowerCase()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
  }, [activeStylesheet, stylesheetService]);

  // Import stylesheet
  const handleImport = useCallback(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const text = await file.text();
        const imported = stylesheetService.importStylesheet(text);
        if (imported) {
          stylesheetService.setActiveStylesheet(imported.id);
          setStylesheets(stylesheetService.getAllStylesheets());
        }
      }
    };
    input.click();
  }, [stylesheetService]);

  // Generate CSS variables for preview
  const cssVars = stylesheetService.toCssVariables(editedStylesheet);

  return (
    <div className={styles.container}>
      {/* Preset Selection */}
      <div className={styles.presetRow}>
        <Dropdown
          className={styles.presetDropdown}
          value={activeStylesheet.name}
          onOptionSelect={(_, data) => {
            const id = stylesheets.find((s) => s.name === data.optionValue)?.id;
            if (id) handlePresetChange(id);
          }}
        >
          {stylesheets.map((s) => (
            <Option key={s.id} value={s.name}>
              {s.name}
              {s.isBuiltIn && ' (Built-in)'}
            </Option>
          ))}
        </Dropdown>

        <Tooltip content="Duplicate" relationship="label">
          <Button icon={<Copy24Regular />} appearance="subtle" onClick={handleDuplicate} />
        </Tooltip>

        <Dialog open={showCreateDialog} onOpenChange={(_, data) => setShowCreateDialog(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Tooltip content="Create New" relationship="label">
              <Button icon={<Add24Regular />} appearance="subtle" />
            </Tooltip>
          </DialogTrigger>
          <DialogSurface>
            <DialogBody>
              <DialogTitle>Create New Stylesheet</DialogTitle>
              <DialogContent>
                <Field label="Stylesheet Name">
                  <Input
                    value={newStylesheetName}
                    onChange={(_, data) => setNewStylesheetName(data.value)}
                    placeholder="My Custom Style"
                  />
                </Field>
              </DialogContent>
              <DialogActions>
                <Button appearance="secondary" onClick={() => setShowCreateDialog(false)}>
                  Cancel
                </Button>
                <Button appearance="primary" onClick={handleCreate}>
                  Create
                </Button>
              </DialogActions>
            </DialogBody>
          </DialogSurface>
        </Dialog>

        {!activeStylesheet.isBuiltIn && (
          <Tooltip content="Delete" relationship="label">
            <Button
              icon={<Delete24Regular />}
              appearance="subtle"
              onClick={handleDelete}
            />
          </Tooltip>
        )}
      </div>

      {activeStylesheet.description && (
        <div style={{ fontSize: '12px', color: tokens.colorNeutralForeground3 }}>
          {activeStylesheet.description}
        </div>
      )}

      {/* Tabs */}
      <TabList
        selectedValue={selectedTab}
        onTabSelect={(_, data) => setSelectedTab(data.value as TabType)}
        className={styles.tabs}
      >
        <Tab icon={<PaintBrush24Regular />} value="colors">
          Colors
        </Tab>
        <Tab icon={<TextFont24Regular />} value="fonts">
          Fonts
        </Tab>
        <Tab icon={<LayoutColumnTwoSplitRight24Regular />} value="layout">
          Layout
        </Tab>
        <Tab icon={<Eye24Regular />} value="display">
          Display
        </Tab>
      </TabList>

      {/* Tab Content */}
      <div className={styles.tabContent}>
        {selectedTab === 'colors' && (
          <>
            <ColorSetting label="Primary Text" color={editedStylesheet.colors.primary} onChange={(c) => updateColor('primary', c)} />
            <ColorSetting label="Secondary Text" color={editedStylesheet.colors.secondary} onChange={(c) => updateColor('secondary', c)} />
            <ColorSetting label="Background" color={editedStylesheet.colors.background} onChange={(c) => updateColor('background', c)} />
            <ColorSetting label="Staff Lines" color={editedStylesheet.colors.staffLines} onChange={(c) => updateColor('staffLines', c)} />
            <ColorSetting label="Cursor" color={editedStylesheet.colors.cursor} onChange={(c) => updateColor('cursor', c)} />
            <ColorSetting label="Selection" color={editedStylesheet.colors.selection} onChange={(c) => updateColor('selection', c)} />
            <Divider style={{ margin: '12px 0' }} />
            <ColorSetting label="In Tune" color={editedStylesheet.colors.inTune} onChange={(c) => updateColor('inTune', c)} />
            <ColorSetting label="Flat" color={editedStylesheet.colors.flat} onChange={(c) => updateColor('flat', c)} />
            <ColorSetting label="Sharp" color={editedStylesheet.colors.sharp} onChange={(c) => updateColor('sharp', c)} />
          </>
        )}

        {selectedTab === 'fonts' && (
          <>
            <FontSetting
              name="Title"
              font={editedStylesheet.fonts.title}
              onChange={(prop, val) => updateFont('title', prop, val)}
            />
            <FontSetting
              name="Subtitle"
              font={editedStylesheet.fonts.subtitle}
              onChange={(prop, val) => updateFont('subtitle', prop, val)}
            />
            <FontSetting
              name="Tablature"
              font={editedStylesheet.fonts.tablature}
              onChange={(prop, val) => updateFont('tablature', prop, val)}
            />
            <FontSetting
              name="Fret Numbers"
              font={editedStylesheet.fonts.fret}
              onChange={(prop, val) => updateFont('fret', prop, val)}
            />
            <FontSetting
              name="Chord Names"
              font={editedStylesheet.fonts.chord}
              onChange={(prop, val) => updateFont('chord', prop, val)}
            />
            <FontSetting
              name="Lyrics"
              font={editedStylesheet.fonts.lyrics}
              onChange={(prop, val) => updateFont('lyrics', prop, val)}
            />
          </>
        )}

        {selectedTab === 'layout' && (
          <>
            <SliderSetting
              label="Scale"
              value={editedStylesheet.layout.scale}
              min={0.5}
              max={2}
              step={0.05}
              format={(v) => `${Math.round(v * 100)}%`}
              onChange={(v) => updateLayout('scale', v)}
            />
            <SliderSetting
              label="Stave Spacing"
              value={editedStylesheet.layout.staveSpacing}
              min={4}
              max={40}
              step={2}
              format={(v) => `${v}px`}
              onChange={(v) => updateLayout('staveSpacing', v)}
            />
            <SliderSetting
              label="System Spacing"
              value={editedStylesheet.layout.systemSpacing}
              min={10}
              max={60}
              step={2}
              format={(v) => `${v}px`}
              onChange={(v) => updateLayout('systemSpacing', v)}
            />
            <SliderSetting
              label="Left Margin"
              value={editedStylesheet.layout.marginLeft}
              min={0}
              max={100}
              step={5}
              format={(v) => `${v}px`}
              onChange={(v) => updateLayout('marginLeft', v)}
            />
            <SliderSetting
              label="Right Margin"
              value={editedStylesheet.layout.marginRight}
              min={0}
              max={100}
              step={5}
              format={(v) => `${v}px`}
              onChange={(v) => updateLayout('marginRight', v)}
            />
            <Divider style={{ margin: '12px 0' }} />
            <div className={styles.settingRow}>
              <span className={styles.settingLabel}>Layout Mode</span>
              <Dropdown
                value={editedStylesheet.layout.layoutMode === 'horizontal' ? 'Horizontal' : 'Page'}
                onOptionSelect={(_, data) =>
                  updateLayout('layoutMode', data.optionValue === 'Horizontal' ? 'horizontal' : 'page')
                }
              >
                <Option value="Page">Page</Option>
                <Option value="Horizontal">Horizontal</Option>
              </Dropdown>
            </div>
            <div className={styles.settingRow}>
              <span className={styles.settingLabel}>Stave Profile</span>
              <Dropdown
                value={
                  { default: 'Default', tab: 'Tab Only', score: 'Score Only', 'tab-mixed': 'Tab + Score', 'score-tab': 'Score + Tab' }[
                    editedStylesheet.layout.staveProfile
                  ]
                }
                onOptionSelect={(_, data) => {
                  const profiles: Record<string, NotationLayout['staveProfile']> = {
                    Default: 'default',
                    'Tab Only': 'tab',
                    'Score Only': 'score',
                    'Tab + Score': 'tab-mixed',
                    'Score + Tab': 'score-tab',
                  };
                  updateLayout('staveProfile', profiles[data.optionValue ?? 'Default']);
                }}
              >
                <Option value="Default">Default</Option>
                <Option value="Tab Only">Tab Only</Option>
                <Option value="Score Only">Score Only</Option>
                <Option value="Tab + Score">Tab + Score</Option>
                <Option value="Score + Tab">Score + Tab</Option>
              </Dropdown>
            </div>
          </>
        )}

        {selectedTab === 'display' && (
          <>
            <SwitchSetting
              label="Standard Notation"
              checked={editedStylesheet.display.showStandardNotation}
              onChange={(v) => updateDisplay('showStandardNotation', v)}
            />
            <SwitchSetting
              label="Tablature"
              checked={editedStylesheet.display.showTablature}
              onChange={(v) => updateDisplay('showTablature', v)}
            />
            <SwitchSetting
              label="Chord Diagrams"
              checked={editedStylesheet.display.showChordDiagrams}
              onChange={(v) => updateDisplay('showChordDiagrams', v)}
            />
            <SwitchSetting
              label="Lyrics"
              checked={editedStylesheet.display.showLyrics}
              onChange={(v) => updateDisplay('showLyrics', v)}
            />
            <SwitchSetting
              label="Fingering"
              checked={editedStylesheet.display.showFingering}
              onChange={(v) => updateDisplay('showFingering', v)}
            />
            <SwitchSetting
              label="Dynamics"
              checked={editedStylesheet.display.showDynamics}
              onChange={(v) => updateDisplay('showDynamics', v)}
            />
            <SwitchSetting
              label="Tempo Markings"
              checked={editedStylesheet.display.showTempoMarkings}
              onChange={(v) => updateDisplay('showTempoMarkings', v)}
            />
            <SwitchSetting
              label="Bar Numbers"
              checked={editedStylesheet.display.showBarNumbers}
              onChange={(v) => updateDisplay('showBarNumbers', v)}
            />
            <SwitchSetting
              label="Track Names"
              checked={editedStylesheet.display.showTrackNames}
              onChange={(v) => updateDisplay('showTrackNames', v)}
            />
            <SwitchSetting
              label="Time Signatures"
              checked={editedStylesheet.display.showTimeSignatures}
              onChange={(v) => updateDisplay('showTimeSignatures', v)}
            />
            <SwitchSetting
              label="Key Signatures"
              checked={editedStylesheet.display.showKeySignatures}
              onChange={(v) => updateDisplay('showKeySignatures', v)}
            />
            <SwitchSetting
              label="Copyright"
              checked={editedStylesheet.display.showCopyright}
              onChange={(v) => updateDisplay('showCopyright', v)}
            />
            <SwitchSetting
              label="Rhythm on Tab"
              checked={editedStylesheet.display.showRhythmOnTab}
              onChange={(v) => updateDisplay('showRhythmOnTab', v)}
            />
            <SwitchSetting
              label="Effects (bends, slides)"
              checked={editedStylesheet.display.showEffects}
              onChange={(v) => updateDisplay('showEffects', v)}
            />
            <Divider style={{ margin: '12px 0' }} />
            <div className={styles.settingRow}>
              <span className={styles.settingLabel}>Notation Mode</span>
              <Dropdown
                value={
                  { 'guitar-pro': 'Guitar Pro', songbook: 'Songbook', classic: 'Classic' }[
                    editedStylesheet.display.notationMode
                  ]
                }
                onOptionSelect={(_, data) => {
                  const modes: Record<string, NotationDisplay['notationMode']> = {
                    'Guitar Pro': 'guitar-pro',
                    Songbook: 'songbook',
                    Classic: 'classic',
                  };
                  updateDisplay('notationMode', modes[data.optionValue ?? 'Guitar Pro']);
                }}
              >
                <Option value="Guitar Pro">Guitar Pro</Option>
                <Option value="Songbook">Songbook</Option>
                <Option value="Classic">Classic</Option>
              </Dropdown>
            </div>
          </>
        )}
      </div>

      {/* Preview */}
      <div className={styles.previewArea} style={cssVars as React.CSSProperties}>
        <div className={styles.previewTitle}>Song Title</div>
        <div className={styles.previewTab}>e|--0--2--3--|</div>
        <div className={styles.previewTab}>B|--0--0--0--|</div>
      </div>

      {/* Actions */}
      <div className={styles.actions}>
        <Tooltip content="Export Stylesheet" relationship="label">
          <Button icon={<ArrowDownload24Regular />} appearance="subtle" onClick={handleExport} />
        </Tooltip>
        <Tooltip content="Import Stylesheet" relationship="label">
          <Button icon={<ArrowUpload24Regular />} appearance="subtle" onClick={handleImport} />
        </Tooltip>
        <div style={{ flex: 1 }} />
        {hasChanges && (
          <>
            <Button appearance="secondary" onClick={handleRevert}>
              Revert
            </Button>
            <Button appearance="primary" onClick={handleSave}>
              {editedStylesheet.isBuiltIn ? 'Save as New' : 'Save'}
            </Button>
          </>
        )}
        {onClose && (
          <Button appearance="secondary" onClick={onClose}>
            Close
          </Button>
        )}
      </div>
    </div>
  );
}

// Helper Components

interface ColorSettingProps {
  label: string;
  color: string;
  onChange: (color: string) => void;
}

function ColorSetting({ label, color, onChange }: ColorSettingProps) {
  const styles = useStyles();
  return (
    <div className={styles.settingRow}>
      <span className={styles.settingLabel}>{label}</span>
      <div className={styles.colorInput}>
        <input
          type="color"
          value={color.startsWith('#') ? color : '#000000'}
          onChange={(e) => onChange(e.target.value)}
          className={styles.colorSwatch}
        />
        <Input
          value={color}
          onChange={(_, data) => onChange(data.value)}
          style={{ width: '100px', fontFamily: 'monospace', fontSize: '12px' }}
        />
      </div>
    </div>
  );
}

interface SliderSettingProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  format: (value: number) => string;
  onChange: (value: number) => void;
}

function SliderSetting({ label, value, min, max, step, format, onChange }: SliderSettingProps) {
  const styles = useStyles();
  return (
    <div className={styles.settingRow}>
      <span className={styles.settingLabel}>{label}</span>
      <div className={styles.sliderWithValue}>
        <Slider
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(_, data) => onChange(data.value)}
          style={{ flex: 1 }}
        />
        <span className={styles.sliderValue}>{format(value)}</span>
      </div>
    </div>
  );
}

interface SwitchSettingProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function SwitchSetting({ label, checked, onChange }: SwitchSettingProps) {
  const styles = useStyles();
  return (
    <div className={styles.settingRow}>
      <span className={styles.settingLabel}>{label}</span>
      <Switch checked={checked} onChange={(_, data) => onChange(data.checked)} />
    </div>
  );
}

interface FontSettingProps {
  name: string;
  font: NotationFont;
  onChange: (property: keyof NotationFont, value: string | number) => void;
}

function FontSetting({ name, font, onChange }: FontSettingProps) {
  const styles = useStyles();
  return (
    <div className={styles.fontRow}>
      <div className={styles.fontName}>{name}</div>
      <div className={styles.fontControls}>
        <Input
          value={font.family}
          onChange={(_, data) => onChange('family', data.value)}
          style={{ flex: 1, minWidth: '150px' }}
          placeholder="Font family"
        />
        <Input
          type="number"
          value={font.size.toString()}
          onChange={(_, data) => onChange('size', parseInt(data.value, 10) || 12)}
          style={{ width: '60px' }}
        />
        <Dropdown
          value={font.style}
          onOptionSelect={(_, data) => onChange('style', data.optionValue ?? 'normal')}
          style={{ width: '100px' }}
        >
          <Option value="normal">Normal</Option>
          <Option value="bold">Bold</Option>
          <Option value="italic">Italic</Option>
          <Option value="bold-italic">Bold Italic</Option>
        </Dropdown>
      </div>
    </div>
  );
}

/**
 * Dialog wrapper for StylesheetSettings
 */
interface StylesheetSettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function StylesheetSettingsDialog({ open, onOpenChange }: StylesheetSettingsDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(_, data) => onOpenChange(data.open)}>
      <DialogSurface style={{ maxWidth: '600px' }}>
        <DialogBody>
          <DialogTitle>Notation Stylesheet</DialogTitle>
          <DialogContent>
            <StylesheetSettings onClose={() => onOpenChange(false)} />
          </DialogContent>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
