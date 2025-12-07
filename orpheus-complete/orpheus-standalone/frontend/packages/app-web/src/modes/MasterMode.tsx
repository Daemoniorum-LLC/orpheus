/**
 * Master Mode - AI-powered mastering (Nexus DAW)
 */

import { makeStyles, shorthands, tokens, Button, Card, Dropdown, Option } from '@fluentui/react-components';
import { ArrowDownload24Regular, BotRegular, FolderOpen24Regular } from '@fluentui/react-icons';
import { useState, useEffect } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { MasteringChain, type MasteringChainSettings } from '../components/MasteringChain';
import { LUFSMeter } from '../components/LUFSMeter';
import { getAudioProcessingManager } from '../services/audio-processing';

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

  // Wire up mastering chain to audio processing
  useEffect(() => {
    if (masteringSettings) {
      const manager = getAudioProcessingManager();
      const master = manager.getMaster();
      master.updateChain(masteringSettings);
      console.log('[MasterMode] Updated mastering chain');
    }
  }, [masteringSettings]);

  const platforms = [
    { id: 'spotify', name: 'Spotify', target: -14, color: '#1DB954' },
    { id: 'apple', name: 'Apple Music', target: -16, color: '#FA243C' },
    { id: 'youtube', name: 'YouTube', target: -14, color: '#FF0000' },
    { id: 'tidal', name: 'Tidal', target: -14, color: '#000000' },
    { id: 'soundcloud', name: 'SoundCloud', target: -8, color: '#FF5500' },
  ];

  const exportFormats = [
    { name: 'WAV', format: '44.1kHz/24-bit', size: '~50MB' },
    { name: 'WAV', format: '48kHz/24-bit', size: '~55MB' },
    { name: 'FLAC', format: 'Lossless', size: '~35MB' },
    { name: 'MP3', format: '320kbps', size: '~12MB' },
    { name: 'AAC', format: '256kbps', size: '~10MB' },
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
              {exportFormats.map((fmt, idx) => (
                <Button
                  key={idx}
                  icon={<ArrowDownload24Regular />}
                  appearance="subtle"
                  style={{ justifyContent: 'flex-start' }}
                >
                  <div style={{ flex: 1, display: 'flex', justifyContent: 'space-between' }}>
                    <span>
                      {fmt.name} - {fmt.format}
                    </span>
                    <span style={{ color: tokens.colorNeutralForeground2, fontSize: '11px' }}>
                      {fmt.size}
                    </span>
                  </div>
                </Button>
              ))}
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
