/**
 * Export Dialog - Advanced export with multiple formats and presets
 * Professional export system with platform optimization
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  Button,
} from '@persona-framework/ui';
import { Download, X, Check, AlertTriangle } from 'lucide-react';
import { useState } from 'react';
import { useAppStore } from '../store/app-store';
import { exportProject, EXPORT_FORMATS, downloadBlob, type ExportResult } from '../services/export';
import { showSuccess, showError, showWarning } from '../services/toast';
import { logger } from '../utils/logger';
import { safeStorage } from '../utils/storage';

interface ExportPreset {
  id: string;
  name: string;
  description: string;
  formats: string[];
  settings: Record<string, any>;
}

const AUDIO_FORMATS = [
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
      const warnings: string[] = [];
      const exportPromises = Array.from(selectedFormats).map(async (formatId) => {
        const standardFormat = EXPORT_FORMATS.find((f) => f.id === formatId);
        if (standardFormat) {
          const result = await exportProject(project, formatId);
          downloadBlob(result.blob, result.filename);

          // Collect warnings for display
          if (result.warning) {
            warnings.push(result.warning);
          }
          return;
        }

        showSuccess(`${formatId.toUpperCase()} export coming soon!`, 3000);
      });

      await Promise.all(exportPromises);

      // Show warnings first (with longer duration)
      warnings.forEach((warning) => {
        showWarning(warning, 8000);
      });

      // Show success message
      const successCount = selectedFormats.size - warnings.length;
      if (successCount > 0 && warnings.length > 0) {
        showSuccess(`Exported ${successCount} file(s) successfully!`, 3000);
      } else if (warnings.length === 0) {
        showSuccess(`Exported ${selectedFormats.size} file(s) successfully!`, 3000);
      }
      setExportSuccess(true);

      const exportHistory = JSON.parse(safeStorage.getItem('maestro-export-history') || '[]');
      exportHistory.unshift({
        timestamp: new Date().toISOString(),
        projectName: project.project.metadata.title,
        formats: Array.from(selectedFormats),
        preset: selectedPreset,
      });
      safeStorage.setItem('maestro-export-history', JSON.stringify(exportHistory.slice(0, 10)));
    } catch (error) {
      logger.export.error('Export failed:', error);
      showError('Export failed. Please try again.', 5000);
    } finally {
      setExporting(false);
    }
  };

  if (!project) return null;

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <DialogContent className="max-w-[600px] w-[95vw] sm:w-auto max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex justify-between items-start">
            <DialogTitle>Export Project</DialogTitle>
            <Button variant="ghost" size="icon" onClick={onClose} className="-mt-2 -mr-2">
              <X className="h-4 w-4" />
            </Button>
          </div>
          <DialogDescription className="sr-only">
            Export your project in various formats
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-6 py-4">
          {/* Platform Presets */}
          <div className="flex flex-col gap-3">
            <h3 className="text-sm font-semibold" id="platform-presets-label">Platform Presets</h3>
            <div className="flex flex-col gap-2" role="radiogroup" aria-labelledby="platform-presets-label">
              {PLATFORM_PRESETS.map((preset) => {
                const isSelected = selectedPreset === preset.id;
                return (
                  <div
                    key={preset.id}
                    role="radio"
                    aria-checked={isSelected}
                    aria-label={`${preset.name}: ${preset.description}`}
                    tabIndex={0}
                    className={`p-3 border rounded-lg cursor-pointer transition-colors flex items-start gap-3 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ${
                      isSelected
                        ? 'bg-primary/10 border-primary'
                        : 'border-border hover:bg-muted'
                    }`}
                    onClick={() => setSelectedPreset(preset.id)}
                    onKeyDown={(e) => e.key === 'Enter' || e.key === ' ' ? (e.preventDefault(), setSelectedPreset(preset.id)) : null}
                  >
                    <div className="flex-1">
                      <div className="text-sm font-semibold">{preset.name}</div>
                      <div className="text-xs text-muted-foreground">{preset.description}</div>
                    </div>
                    {isSelected && (
                      <Check className="h-5 w-5 text-primary flex-shrink-0 animate-success-scale" aria-hidden="true" />
                    )}
                  </div>
                );
              })}
            </div>
          </div>

          {/* Export Formats */}
          <div className="flex flex-col gap-3">
            <h3 className="text-sm font-semibold">
              Export Formats {selectedPreset === 'custom' ? '(Select Multiple)' : ''}
            </h3>

            {/* Audio Formats (only show in custom mode) */}
            {selectedPreset === 'custom' && (
              <>
                <span className="text-xs font-semibold text-muted-foreground" id="audio-formats-label">Audio Formats</span>
                <div className="grid grid-cols-2 gap-3" role="group" aria-labelledby="audio-formats-label">
                  {AUDIO_FORMATS.map((format) => {
                    const isSelected = selectedFormats.has(format.id);
                    return (
                      <div
                        key={format.id}
                        role="checkbox"
                        aria-checked={isSelected}
                        aria-label={`${format.name}: ${format.description}`}
                        tabIndex={0}
                        className={`p-3 min-h-[60px] border rounded-lg cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ${
                          isSelected
                            ? 'bg-primary/10 border-primary'
                            : 'border-border hover:bg-muted'
                        }`}
                        onClick={() => toggleFormat(format.id)}
                        onKeyDown={(e) => e.key === 'Enter' || e.key === ' ' ? (e.preventDefault(), toggleFormat(format.id)) : null}
                      >
                        <div className="text-sm font-semibold">
                          {format.icon} {format.name}
                        </div>
                        <div className="text-xs text-muted-foreground">{format.description}</div>
                      </div>
                    );
                  })}
                </div>
              </>
            )}

            {/* Project Formats */}
            <span className="text-xs font-semibold text-muted-foreground" id="project-formats-label">Project Formats</span>
            <div className="grid grid-cols-2 gap-3" role="group" aria-labelledby="project-formats-label">
              {EXPORT_FORMATS.map((format) => {
                const isSelected = selectedFormats.has(format.id);
                return (
                  <div
                    key={format.id}
                    role="checkbox"
                    aria-checked={isSelected}
                    aria-label={`${format.name}: ${format.description}`}
                    tabIndex={0}
                    className={`p-3 min-h-[60px] border rounded-lg cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ${
                      isSelected
                        ? 'bg-primary/10 border-primary'
                        : 'border-border hover:bg-muted'
                    }`}
                    onClick={() => toggleFormat(format.id)}
                    onKeyDown={(e) => e.key === 'Enter' || e.key === ' ' ? (e.preventDefault(), toggleFormat(format.id)) : null}
                  >
                    <div className="text-sm font-semibold">{format.name}</div>
                    <div className="text-xs text-muted-foreground">{format.description}</div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Warning for audio export */}
          {(selectedFormats.has('wav') ||
            selectedFormats.has('mp3') ||
            selectedFormats.has('flac')) && (
            <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-lg flex gap-3 items-start">
              <AlertTriangle className="h-5 w-5 text-yellow-500 flex-shrink-0" />
              <div className="text-xs text-muted-foreground leading-relaxed">
                <strong>Audio Export Preview:</strong> Full audio rendering is coming soon.
                For now, you can export MIDI and re-import into your DAW for audio rendering.
              </div>
            </div>
          )}

          {/* Success message */}
          {exportSuccess && (
            <div className="p-3 bg-green-500/10 border border-green-500/20 rounded-lg flex gap-3 items-center">
              <Check className="h-5 w-5 text-green-500" />
              <div className="text-xs text-muted-foreground">
                Export complete! Check your downloads folder.
              </div>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={handleExport}
            disabled={exporting || selectedFormats.size === 0}
          >
            <Download className="h-4 w-4 mr-2" />
            {exporting ? 'Exporting...' : `Export ${selectedFormats.size} Format(s)`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
