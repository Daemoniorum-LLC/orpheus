/**
 * Mix Mode - Professional mixing console (Nexus DAW)
 */

import { makeStyles, shorthands, tokens, Button } from '@fluentui/react-components';
import { Add24Regular, BotRegular, FolderOpen24Regular } from '@fluentui/react-icons';
import { useState } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { ChannelStripWithProcessors } from '../components/ChannelStripWithProcessors';
import { ChannelStrip } from '../components/ChannelStrip';
import { ConfirmDialog } from '../components/ConfirmDialog';

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
    ...shorthands.gap('16px'),
    overflow: 'auto',
    backgroundColor: tokens.colorNeutralBackground3,
  },
  emptyState: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
  mixerSection: {
    display: 'flex',
    ...shorthands.gap('12px'),
    alignItems: 'stretch',
  },
  master: {
    ...shorthands.borderLeft('2px', 'solid', tokens.colorBrandBackground),
    paddingLeft: '12px',
  },
});

interface Track {
  id: string;
  name: string;
  volume: number;
  pan: number;
  solo: boolean;
  mute: boolean;
}

export function MixMode() {
  const styles = useStyles();
  const project = useProject();
  const { setAIAssistantOpen, setProject, setMode } = useAppStore();
  const [tracks, setTracks] = useState<Track[]>([
    { id: '1', name: 'Guitar', volume: 0, pan: 0, solo: false, mute: false },
    { id: '2', name: 'Bass', volume: -3, pan: 0, solo: false, mute: false },
    { id: '3', name: 'Drums', volume: -6, pan: 0, solo: false, mute: false },
    { id: '4', name: 'Vocals', volume: -2, pan: 0, solo: false, mute: false },
  ]);
  const [masterVolume, setMasterVolume] = useState(0);
  const [masterPan, setMasterPan] = useState(0);

  // Confirm dialog state
  const [deleteConfirm, setDeleteConfirm] = useState<{
    open: boolean;
    trackId: string;
    trackName: string;
  }>({
    open: false,
    trackId: '',
    trackName: '',
  });

  const handleAddTrack = () => {
    const newTrack: Track = {
      id: Date.now().toString(),
      name: `Track ${tracks.length + 1}`,
      volume: 0,
      pan: 0,
      solo: false,
      mute: false,
    };
    setTracks([...tracks, newTrack]);
  };

  const updateTrack = (id: string, updates: Partial<Track>) => {
    setTracks(tracks.map((t) => (t.id === id ? { ...t, ...updates } : t)));
  };

  const handleDeleteClick = (id: string) => {
    const track = tracks.find((t) => t.id === id);
    if (track) {
      setDeleteConfirm({
        open: true,
        trackId: id,
        trackName: track.name,
      });
    }
  };

  const confirmDeleteTrack = () => {
    setTracks(tracks.filter((t) => t.id !== deleteConfirm.trackId));
  };

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
          <div className={styles.title}>🎚️ Mix - Nexus DAW</div>
        </div>
        <div className={styles.emptyState}>
          <h3>No Project Loaded</h3>
          <p style={{ color: tokens.colorNeutralForeground2, marginBottom: '24px' }}>
            To use Mix mode, you need to load a project first.
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

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>🎚️ Mix - Nexus DAW</div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <Button icon={<Add24Regular />} onClick={handleAddTrack}>
            Add Track
          </Button>
          <Button icon={<BotRegular />} appearance="subtle" onClick={() => setAIAssistantOpen(true)}>
            AI Mix Suggestions
          </Button>
        </div>
      </div>

      <div className={styles.content}>
        <div className={styles.mixerSection}>
          {/* Track Channels */}
          {tracks.map((track) => (
            <ChannelStripWithProcessors
              key={track.id}
              trackId={track.id}
              trackName={track.name}
              volume={track.volume}
              pan={track.pan}
              solo={track.solo}
              mute={track.mute}
              onVolumeChange={(v) => updateTrack(track.id, { volume: v })}
              onPanChange={(p) => updateTrack(track.id, { pan: p })}
              onSoloToggle={() => updateTrack(track.id, { solo: !track.solo })}
              onMuteToggle={() => updateTrack(track.id, { mute: !track.mute })}
              onDelete={() => handleDeleteClick(track.id)}
            />
          ))}

          {/* Master Channel */}
          <div className={styles.master}>
            <ChannelStrip
              trackId="master"
              trackName="MASTER"
              volume={masterVolume}
              pan={masterPan}
              onVolumeChange={setMasterVolume}
              onPanChange={setMasterPan}
            />
          </div>
        </div>
      </div>

      <ConfirmDialog
        open={deleteConfirm.open}
        onConfirm={confirmDeleteTrack}
        onCancel={() => setDeleteConfirm({ ...deleteConfirm, open: false })}
        title="Delete Track?"
        message={`Are you sure you want to delete the track "${deleteConfirm.trackName}"? This action cannot be undone.`}
        confirmText="Delete"
        cancelText="Cancel"
        type="danger"
      />
    </div>
  );
}
