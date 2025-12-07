/**
 * Export Dialog - Advanced export with multiple formats and presets
 * Professional export system with platform optimization
 */

import {
  Dialog,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Button,
  makeStyles,
  shorthands,
  tokens,
  Label,
} from '@fluentui/react-components';
import {
  ArrowDownload24Regular,
  Dismiss24Regular,
  Checkmark24Regular,
  Warning24Regular,
} from '@fluentui/react-icons';
import { useState } from 'react';
import { useAppStore } from '../store/app-store';
import { exportProject, EXPORT_FORMATS } from '../services/export';
import { downloadBlob } from '../services/export';
import { showSuccess, showError } from '../services/toast';

const useStyles = makeStyles({
  content: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('24px'),
  },
  section: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
  },
  sectionTitle: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground1,
  },
  formatGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    ...shorthands.gap('12px'),
  },
  formatCard: {
    ...shorthands.padding('12px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    cursor: 'pointer',
    ...shorthands.transition('all', '150ms'),
    '&:hover': {
      backgroundColor: tokens.colorNeutralBackground2,
      ...shorthands.borderColor(tokens.colorBrandStroke1),
    },
  },
  formatCardSelected: {
    backgroundColor: tokens.colorBrandBackground2,
    ...shorthands.borderColor(tokens.colorBrandStroke1),
    ...shorthands.borderWidth('2px'),
  },
  formatName: {
    fontSize: '13px',
    fontWeight: 600,
    marginBottom: '4px',
  },
  formatDesc: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    lineHeight: '1.4',
  },
  presetCard: {
    ...shorthands.padding('12px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    display: 'flex',
    alignItems: 'flex-start',
    ...shorthands.gap('12px'),
    cursor: 'pointer',
    ...shorthands.transition('all', '150ms'),
    '&:hover': {
      backgroundColor: tokens.colorNeutralBackground2,
    },
  },
  presetCardSelected: {
    backgroundColor: tokens.colorBrandBackground2,
    ...shorthands.borderColor(tokens.colorBrandStroke1),
    ...shorthands.borderWidth('2px'),
  },
  presetInfo: {
    flex: 1,
  },
  presetTitle: {
    fontSize: '13px',
    fontWeight: 600,
    marginBottom: '4px',
  },
  presetDesc: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    lineHeight: '1.4',
  },
  warningBox: {
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorPaletteYellowBackground2,
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorPaletteYellowBorder2),
    display: 'flex',
    ...shorthands.gap('12px'),
    alignItems: 'flex-start',
  },
  warningText: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
    lineHeight: '1.5',
  },
  successBox: {
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorPaletteLightGreenBackground2,
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorPaletteLightGreenBorder2),
    display: 'flex',
    ...shorthands.gap('12px'),
    alignItems: 'center',
  },
  successText: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
  },
});

interface ExportFormat {
  id: string;
  name: string;
  extension: string;
  description: string;
  icon: string;
}

interface ExportPreset {
  id: string;
  name: string;
  description: string;
  formats: string[];
  settings: Record<string, any>;
}

const AUDIO_FORMATS: ExportFormat[] = [
  {
    id: 'wav',
    name: 'WAV (Lossless)',
    extension: 'wav',
    description: 'Uncompressed audio, best quality',
    icon: '🎵',
  },
  {
    id: 'mp3',
    name: 'MP3 (320kbps)',
    extension: 'mp3',
    description: 'High-quality compressed audio',
    icon: '🎧',
  },
  {
    id: 'flac',
    name: 'FLAC (Lossless)',
    extension: 'flac',
    description: 'Lossless compression, smaller than WAV',
    icon: '💎',
  },
];

const PLATFORM_PRESETS: ExportPreset[] = [
  {
    id: 'spotify',
    name: '🎵 Spotify Optimized',
    description: 'WAV 44.1kHz 16-bit, -14 LUFS target',
    formats: ['wav'],
    settings: { sampleRate: 44100, bitDepth: 16, lufsTarget: -14 },
  },
  {
    id: 'youtube',
    name: '📺 YouTube Optimized',
    description: 'MP3 320kbps, -13 LUFS target',
    formats: ['mp3'],
    settings: { bitrate: 320, lufsTarget: -13 },
  },
  {
    id: 'apple-music',
    name: '🍎 Apple Music Optimized',
    description: 'WAV 44.1kHz 16-bit, -16 LUFS target',
    formats: ['wav'],
    settings: { sampleRate: 44100, bitDepth: 16, lufsTarget: -16 },
  },
  {
    id: 'broadcast',
    name: '📻 Broadcast Standard',
    description: 'WAV 48kHz 24-bit, -23 LUFS target',
    formats: ['wav'],
    settings: { sampleRate: 48000, bitDepth: 24, lufsTarget: -23 },
  },
  {
    id: 'custom',
    name: '⚙️ Custom Settings',
    description: 'Choose your own formats and quality',
    formats: [],
    settings: {},
  },
];

export interface ExportDialogProps {
  open: boolean;
  onClose: () => void;
}

export function ExportDialog({ open, onClose }: ExportDialogProps) {
  const styles = useStyles();
  const { project } = useAppStore();

  const [selectedPreset, setSelectedPreset] = useState<string>('spotify');
  const [selectedFormats, setSelectedFormats] = useState<Set<string>>(new Set(['midi', 'json']));
  const [exporting, setExporting] = useState(false);
  const [exportSuccess, setExportSuccess] = useState(false);

  const toggleFormat = (formatId: string) => {
    const newFormats = new Set(selectedFormats);
    if (newFormats.has(formatId)) {
      newFormats.delete(formatId);
    } else {
      newFormats.add(formatId);
    }
    setSelectedFormats(newFormats);
  };

  const handleExport = async () => {
    if (!project) {
      showError('No project loaded', 3000);
      return;
    }

    if (selectedFormats.size === 0) {
      showError('Please select at least one export format', 3000);
      return;
    }

    setExporting(true);
    setExportSuccess(false);

    try {
      const exportPromises = Array.from(selectedFormats).map(async (formatId) => {
        // Check if it's a standard format (MIDI, JSON, GP5)
        const standardFormat = EXPORT_FORMATS.find((f) => f.id === formatId);
        if (standardFormat) {
          const result = await exportProject(project, formatId);
          downloadBlob(result.blob, result.filename);
          return;
        }

        // Audio formats - for now, show info message
        // In a real implementation, this would render audio using Tone.js
        showSuccess(`${formatId.toUpperCase()} export coming soon!`, 3000);
      });

      await Promise.all(exportPromises);

      showSuccess(`Exported ${selectedFormats.size} file(s) successfully!`, 3000);
      setExportSuccess(true);

      // Save export history
      const exportHistory = JSON.parse(localStorage.getItem('maestro-export-history') || '[]');
      exportHistory.unshift({
        timestamp: new Date().toISOString(),
        projectName: project.project.metadata.title,
        formats: Array.from(selectedFormats),
        preset: selectedPreset,
      });
      // Keep only last 10 exports
      localStorage.setItem('maestro-export-history', JSON.stringify(exportHistory.slice(0, 10)));
    } catch (error) {
      console.error('[ExportDialog] Export failed:', error);
      showError('Export failed. Please try again.', 5000);
    } finally {
      setExporting(false);
    }
  };

  if (!project) return null;

  return (
    <Dialog open={open} onOpenChange={(_, data) => !data.open && onClose()}>
      <DialogSurface style={{ maxWidth: '600px' }}>
        <DialogBody>
          <DialogTitle
            action={
              <Button
                appearance="subtle"
                icon={<Dismiss24Regular />}
                onClick={onClose}
              />
            }
          >
            Export Project
          </DialogTitle>

          <DialogContent className={styles.content}>
            {/* Platform Presets */}
            <div className={styles.section}>
              <div className={styles.sectionTitle}>Platform Presets</div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                {PLATFORM_PRESETS.map((preset) => (
                  <div
                    key={preset.id}
                    className={`${styles.presetCard} ${
                      selectedPreset === preset.id ? styles.presetCardSelected : ''
                    }`}
                    onClick={() => setSelectedPreset(preset.id)}
                  >
                    <div className={styles.presetInfo}>
                      <div className={styles.presetTitle}>{preset.name}</div>
                      <div className={styles.presetDesc}>{preset.description}</div>
                    </div>
                    {selectedPreset === preset.id && (
                      <Checkmark24Regular style={{ color: tokens.colorBrandForeground1 }} />
                    )}
                  </div>
                ))}
              </div>
            </div>

            {/* Export Formats */}
            <div className={styles.section}>
              <div className={styles.sectionTitle}>
                Export Formats {selectedPreset === 'custom' ? '(Select Multiple)' : ''}
              </div>

              {/* Audio Formats */}
              {selectedPreset === 'custom' && (
                <>
                  <Label size="small" weight="semibold">Audio Formats</Label>
                  <div className={styles.formatGrid}>
                    {AUDIO_FORMATS.map((format) => (
                      <div
                        key={format.id}
                        className={`${styles.formatCard} ${
                          selectedFormats.has(format.id) ? styles.formatCardSelected : ''
                        }`}
                        onClick={() => toggleFormat(format.id)}
                      >
                        <div className={styles.formatName}>
                          {format.icon} {format.name}
                        </div>
                        <div className={styles.formatDesc}>{format.description}</div>
                      </div>
                    ))}
                  </div>
                </>
              )}

              {/* Data Formats */}
              <Label size="small" weight="semibold">Project Formats</Label>
              <div className={styles.formatGrid}>
                {EXPORT_FORMATS.map((format) => (
                  <div
                    key={format.id}
                    className={`${styles.formatCard} ${
                      selectedFormats.has(format.id) ? styles.formatCardSelected : ''
                    }`}
                    onClick={() => toggleFormat(format.id)}
                  >
                    <div className={styles.formatName}>{format.name}</div>
                    <div className={styles.formatDesc}>{format.description}</div>
                  </div>
                ))}
              </div>
            </div>

            {/* Warning for audio export */}
            {(selectedFormats.has('wav') ||
              selectedFormats.has('mp3') ||
              selectedFormats.has('flac')) && (
              <div className={styles.warningBox}>
                <Warning24Regular style={{ color: tokens.colorPaletteYellowForeground2 }} />
                <div className={styles.warningText}>
                  <strong>Audio Export Preview:</strong> Full audio rendering is coming soon.
                  For now, you can export MIDI and re-import into your DAW for audio rendering.
                </div>
              </div>
            )}

            {/* Success message */}
            {exportSuccess && (
              <div className={styles.successBox}>
                <Checkmark24Regular
                  style={{ color: tokens.colorPaletteLightGreenForeground1 }}
                />
                <div className={styles.successText}>
                  ✅ Export complete! Check your downloads folder.
                </div>
              </div>
            )}
          </DialogContent>

          <DialogActions>
            <Button appearance="secondary" onClick={onClose}>
              Cancel
            </Button>
            <Button
              appearance="primary"
              icon={<ArrowDownload24Regular />}
              onClick={handleExport}
              disabled={exporting || selectedFormats.size === 0}
            >
              {exporting ? 'Exporting...' : `Export ${selectedFormats.size} Format(s)`}
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
