/**
 * Master Mode - AI-powered mastering (Nexus DAW)
 */

import { makeStyles, shorthands, tokens, Button, Card, Dropdown, Option, Spinner } from '@fluentui/react-components';
import { ArrowDownload24Regular, BotRegular, FolderOpen24Regular, Checkmark24Regular } from '@fluentui/react-icons';
import { useState, useEffect } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { MasteringChain, type MasteringChainSettings } from '../components/MasteringChain';
import { LUFSMeter } from '../components/LUFSMeter';
import { getAudioProcessingManager } from '../services/audio-processing';
import { AudioExporter, createTestAudioBuffer } from '../services/audio-export';
import { MessageDialog, type MessageType } from '../components/MessageDialog';

const useStyles = makeStyles({
  container: {
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
  },
  header: {
    ...shorthands.padding('16px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '20px',
    fontWeight: tokens.fontWeightSemibold,
  },
  content: {
    flex: 1,
    display: 'flex',
    ...shorthands.padding('24px'),
    ...shorthands.gap('24px'),
    overflow: 'auto',
  },
  leftPanel: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('20px'),
  },
  rightPanel: {
    width: '400px',
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('20px'),
  },
  card: {
    ...shorthands.padding('20px'),
  },
  cardTitle: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '16px',
  },
  meterSection: {
    display: 'flex',
    ...shorthands.gap('24px'),
    alignItems: 'center',
  },
  meter: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    ...shorthands.gap('12px'),
  },
  meterBar: {
    width: '60px',
    height: '200px',
    backgroundColor: tokens.colorNeutralBackground5,
    ...shorthands.borderRadius('6px'),
    position: 'relative',
    ...shorthands.overflow('hidden'),
  },
  meterFill: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    transition: 'height 0.1s ease-out',
  },
  meterValue: {
    fontSize: '24px',
    fontWeight: 700,
    fontFamily: 'monospace',
  },
  meterLabel: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
    textTransform: 'uppercase',
  },
  targetList: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
  },
  targetItem: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('6px'),
  },
  formatList: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  emptyState: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
});

export function MasterMode() {
  const styles = useStyles();
  const project = useProject();
  const { setAIAssistantOpen, setProject, setMode } = useAppStore();
  const [selectedPlatform, setSelectedPlatform] = useState('spotify');
  const [masteringSettings, setMasteringSettings] = useState<MasteringChainSettings | null>(null);
  const [exportingFormat, setExportingFormat] = useState<string | null>(null);
  const [exportedFormat, setExportedFormat] = useState<string | null>(null);

  // Message dialog state
  const [messageDialog, setMessageDialog] = useState<{
    open: boolean;
    title: string;
    message: string;
    type: MessageType;
  }>({
    open: false,
    title: '',
    message: '',
    type: 'info',
  });

  // Wire up mastering chain to audio processing
  useEffect(() => {
    if (masteringSettings) {
      const manager = getAudioProcessingManager();
      const master = manager.getMaster();
      master.updateChain(masteringSettings);
      console.log('[MasterMode] Updated mastering chain');
    }
  }, [masteringSettings]);

  // Handle export
  const handleExport = async (format: string, sampleRate: number, bitDepth: number) => {
    setExportingFormat(format);
    setExportedFormat(null);

    try {
      // Get project duration (default to 60 seconds for test)
      const duration = project?.project?.metadata?.duration || 60;

      // Create a test audio buffer (in a real implementation, this would render the project)
      // For now, we'll create a simple test tone as a placeholder
      console.log(`[MasterMode] Exporting ${format} at ${sampleRate}Hz/${bitDepth}-bit`);

      const audioBuffer = await createTestAudioBuffer(Math.min(duration, 10), sampleRate);

      let result;
      if (format === 'mp3') {
        result = await AudioExporter.exportToMP3(audioBuffer, 320);
      } else if (format === 'flac') {
        result = await AudioExporter.exportToFLAC(audioBuffer);
      } else if (format === 'aac') {
        result = await AudioExporter.export(audioBuffer, {
          format: 'aac',
          sampleRate: sampleRate as 44100 | 48000,
          bitDepth: bitDepth as 16 | 24,
          channels: 2,
        });
      } else {
        result = await AudioExporter.exportToWAV(audioBuffer, {
          sampleRate: sampleRate as 44100 | 48000,
          bitDepth: bitDepth as 16 | 24,
        });
      }

      // Download the file
      const filename = `${project?.project?.metadata?.title || 'master'}-${format}.${result.filename.split('.').pop()}`;
      AudioExporter.downloadBlob(result.blob, filename);

      setExportedFormat(format);
      setMessageDialog({
        open: true,
        title: 'Export Complete',
        message: `Successfully exported ${result.format}\nFile size: ${AudioExporter.formatFileSize(result.fileSize)}\nDuration: ${result.duration.toFixed(1)}s`,
        type: 'success',
      });

      // Clear exported indicator after 3 seconds
      setTimeout(() => setExportedFormat(null), 3000);
    } catch (error) {
      console.error('[MasterMode] Export error:', error);
      setMessageDialog({
        open: true,
        title: 'Export Failed',
        message: error instanceof Error ? error.message : 'Unknown error occurred',
        type: 'error',
      });
    } finally {
      setExportingFormat(null);
    }
  };

  const platforms = [
    { id: 'spotify', name: 'Spotify', target: -14, color: '#1DB954' },
    { id: 'apple', name: 'Apple Music', target: -16, color: '#FA243C' },
    { id: 'youtube', name: 'YouTube', target: -14, color: '#FF0000' },
    { id: 'tidal', name: 'Tidal', target: -14, color: '#000000' },
    { id: 'soundcloud', name: 'SoundCloud', target: -8, color: '#FF5500' },
  ];

  const exportFormats = [
    { id: 'wav-44', name: 'WAV', format: '44.1kHz/24-bit', size: '~50MB', exportFormat: 'wav', sampleRate: 44100, bitDepth: 24 },
    { id: 'wav-48', name: 'WAV', format: '48kHz/24-bit', size: '~55MB', exportFormat: 'wav', sampleRate: 48000, bitDepth: 24 },
    { id: 'flac', name: 'FLAC', format: 'Lossless', size: '~35MB', exportFormat: 'flac', sampleRate: 48000, bitDepth: 24 },
    { id: 'mp3', name: 'MP3', format: '320kbps', size: '~12MB', exportFormat: 'mp3', sampleRate: 44100, bitDepth: 16 },
    { id: 'aac', name: 'AAC', format: '256kbps', size: '~10MB', exportFormat: 'aac', sampleRate: 44100, bitDepth: 16 },
  ];

  const handleImportFile = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.gp,.gpx,.gp5,.gp4,.gp3,.maestro';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const result = await importFile(file);
        if (result.success && result.project) {
          setProject(result.project);
        }
      }
    };
    input.click();
  };

  if (!project) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <div className={styles.title}>✨ Master - AI Mastering</div>
        </div>
        <div className={styles.emptyState}>
          <h3>No Project Loaded</h3>
          <p style={{ color: tokens.colorNeutralForeground2, marginBottom: '24px' }}>
            To use Master mode, you need to load a project first.
          </p>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'center' }}>
            <Button
              appearance="primary"
              icon={<FolderOpen24Regular />}
              onClick={handleImportFile}
            >
              Import Guitar Pro File
            </Button>
            <Button
              appearance="secondary"
              onClick={() => setMode('compose')}
            >
              Go to Compose Mode
            </Button>
          </div>
          <p style={{ fontSize: '12px', color: tokens.colorNeutralForeground3, marginTop: '16px' }}>
            Or press <strong>Ctrl+1</strong> to switch to Compose mode
          </p>
        </div>
      </div>
    );
  }

  const selectedTarget = platforms.find((p) => p.id === selectedPlatform);

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>✨ Master - AI Mastering</div>
        <Button icon={<BotRegular />} appearance="primary" onClick={() => setAIAssistantOpen(true)}>
          AI Auto-Master
        </Button>
      </div>

      <div className={styles.content}>
        {/* Left Panel - Meters */}
        <div className={styles.leftPanel}>
          {/* Professional LUFS Metering */}
          <LUFSMeter
            target={selectedTarget?.target || -14}
            showTargets={true}
          />

          {/* Mastering Chain */}
          <Card className={styles.card}>
            <MasteringChain onSettingsChange={setMasteringSettings} />
          </Card>
        </div>

        {/* Right Panel - Platform Targets & Export */}
        <div className={styles.rightPanel}>
          <Card className={styles.card}>
            <div className={styles.cardTitle}>Platform Targets</div>
            <Dropdown
              placeholder="Select platform"
              value={selectedTarget?.name}
              onOptionSelect={(_, data) => setSelectedPlatform(data.optionValue as string)}
              style={{ marginBottom: '16px' }}
            >
              {platforms.map((p) => (
                <Option key={p.id} value={p.id} text={p.name + ' (' + p.target + ' LUFS)'}>
                  {p.name} ({p.target} LUFS)
                </Option>
              ))}
            </Dropdown>

            <div className={styles.targetList}>
              {platforms.map((platform) => (
                <div key={platform.id} className={styles.targetItem}>
                  <span style={{ fontWeight: 600 }}>{platform.name}</span>
                  <span style={{ fontFamily: 'monospace', color: platform.color }}>
                    {platform.target} LUFS
                  </span>
                </div>
              ))}
            </div>
          </Card>

          <Card className={styles.card}>
            <div className={styles.cardTitle}>Export</div>
            <div className={styles.formatList}>
              {exportFormats.map((fmt) => (
                <Button
                  key={fmt.id}
                  icon={
                    exportingFormat === fmt.id ? (
                      <Spinner size="tiny" />
                    ) : exportedFormat === fmt.id ? (
                      <Checkmark24Regular />
                    ) : (
                      <ArrowDownload24Regular />
                    )
                  }
                  appearance={exportedFormat === fmt.id ? 'primary' : 'subtle'}
                  style={{ justifyContent: 'flex-start' }}
                  disabled={exportingFormat !== null}
                  onClick={() => handleExport(fmt.id, fmt.sampleRate, fmt.bitDepth)}
                >
                  <div style={{ flex: 1, display: 'flex', justifyContent: 'space-between' }}>
                    <span>
                      {fmt.name} - {fmt.format}
                    </span>
                    <span style={{ color: tokens.colorNeutralForeground2, fontSize: '11px' }}>
                      {exportingFormat === fmt.id ? 'Exporting...' : fmt.size}
                    </span>
                  </div>
                </Button>
              ))}
            </div>
          </Card>
        </div>
      </div>

      <MessageDialog
        open={messageDialog.open}
        onClose={() => setMessageDialog({ ...messageDialog, open: false })}
        title={messageDialog.title}
        message={messageDialog.message}
        type={messageDialog.type}
      />
    </div>
  );
}
