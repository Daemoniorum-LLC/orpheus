/**
 * Mix Mode - Professional mixing console (Nexus DAW)
 */

import { makeStyles, shorthands, tokens, Button, Tooltip, ToggleButton } from '@fluentui/react-components';
import { Add24Regular, BotRegular, FolderOpen24Regular, Eye24Regular, EyeOff24Regular, MusicNote224Regular } from '@fluentui/react-icons';
import { TunerDialog } from '../components/Tuner';
import { useState, useEffect, useMemo, useCallback } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { ChannelStripWithProcessors } from '../components/ChannelStripWithProcessors';
import { ChannelStrip } from '../components/ChannelStrip';
import { ConfirmDialog } from '../components/ConfirmDialog';
import type { MaestroProject } from '@orpheus/shared-types';

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
  focusedTrack: {
    transform: 'scale(1.02)',
    boxShadow: `0 0 20px ${tokens.colorBrandBackground}`,
    zIndex: 10,
  },
  unfocusedTrack: {
    opacity: 0.4,
    filter: 'grayscale(50%)',
    transform: 'scale(0.98)',
    ...shorthands.transition('all', '200ms', 'ease-out'),
  },
  focusModeIndicator: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    ...shorthands.padding('8px', '16px'),
    backgroundColor: tokens.colorBrandBackground,
    color: tokens.colorNeutralForegroundOnBrand,
    ...shorthands.borderRadius('4px'),
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
  },
  trackWrapper: {
    ...shorthands.transition('all', '200ms', 'ease-out'),
  },
});

interface MixTrack {
  id: string;
  name: string;
  volume: number;
  pan: number;
  solo: boolean;
  mute: boolean;
}

/**
 * Derives mixer tracks from project composition and mixing data.
 * Combines track names from composition with volume/pan/solo/mute from mixing state.
 */
function deriveTracksFromProject(project: MaestroProject): MixTrack[] {
  const compositionTracks = project.project.composition.tracks || [];
  const mixingSnapshots = project.project.mixing?.snapshots || [];
  const currentSnapshot = mixingSnapshots[mixingSnapshots.length - 1];
  const trackStates = currentSnapshot?.trackStates || {};

  return compositionTracks.map((track) => {
    const mixState = trackStates[track.id];
    return {
      id: track.id,
      name: track.name,
      volume: mixState?.volume ?? 0,
      pan: mixState?.pan ?? 0,
      solo: mixState?.solo ?? false,
      mute: mixState?.muted ?? false,
    };
  });
}

export function MixMode() {
  const styles = useStyles();
  const project = useProject();
  const { setAIAssistantOpen, setProject, updateProject, setMode } = useAppStore();

  // Derive tracks from project, or use empty array if no project
  const projectTracks = useMemo(() => {
    if (!project) return [];
    return deriveTracksFromProject(project);
  }, [project]);

  // Local track state for real-time UI updates
  const [tracks, setTracks] = useState<MixTrack[]>(projectTracks);
  const [masterVolume, setMasterVolume] = useState(0);
  const [masterPan, setMasterPan] = useState(0);

  // Focus mode - isolate a single track for detailed work
  const [focusedTrackId, setFocusedTrackId] = useState<string | null>(null);
  const isFocusMode = focusedTrackId !== null;

  // Tuner dialog
  const [tunerOpen, setTunerOpen] = useState(false);

  // Sync local state when project changes
  useEffect(() => {
    if (projectTracks.length > 0) {
      setTracks(projectTracks);
    }
  }, [projectTracks]);

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
    const newTrack: MixTrack = {
      id: Date.now().toString(),
      name: `Track ${tracks.length + 1}`,
      volume: 0,
      pan: 0,
      solo: false,
      mute: false,
    };
    setTracks([...tracks, newTrack]);
  };

  /**
   * Updates a track's mixing parameters and syncs to project state.
   */
  const updateTrack = useCallback((id: string, updates: Partial<MixTrack>) => {
    // Update local state immediately for responsive UI
    setTracks(prevTracks => prevTracks.map((t) => (t.id === id ? { ...t, ...updates } : t)));

    // Update project state for persistence
    if (project) {
      const updatedProject = structuredClone(project);
      const snapshots = updatedProject.project.mixing.snapshots || [];

      // Get or create current snapshot
      let currentSnapshot = snapshots[snapshots.length - 1];
      if (!currentSnapshot) {
        currentSnapshot = {
          id: `snapshot-${Date.now()}`,
          name: 'Current Mix',
          timestamp: new Date().toISOString(),
          trackStates: {},
          busStates: {},
        };
        snapshots.push(currentSnapshot);
      }

      // Update track state in snapshot
      const trackState = currentSnapshot.trackStates[id] || {
        volume: 0,
        pan: 0,
        muted: false,
        solo: false,
        plugins: [],
      };

      if (updates.volume !== undefined) trackState.volume = updates.volume;
      if (updates.pan !== undefined) trackState.pan = updates.pan;
      if (updates.mute !== undefined) trackState.muted = updates.mute;
      if (updates.solo !== undefined) trackState.solo = updates.solo;

      currentSnapshot.trackStates[id] = trackState;
      currentSnapshot.timestamp = new Date().toISOString();
      updatedProject.project.mixing.snapshots = snapshots;

      updateProject(updatedProject);
    }
  }, [project, updateProject]);

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

  // Toggle focus on a track
  const toggleFocus = useCallback((trackId: string) => {
    setFocusedTrackId((prev) => (prev === trackId ? null : trackId));
  }, []);

  // Clear focus mode
  const clearFocus = useCallback(() => {
    setFocusedTrackId(null);
  }, []);

  // Get track class based on focus state
  const getTrackClassName = useCallback((trackId: string) => {
    if (!isFocusMode) return styles.trackWrapper;
    if (trackId === focusedTrackId) {
      return `${styles.trackWrapper} ${styles.focusedTrack}`;
    }
    return `${styles.trackWrapper} ${styles.unfocusedTrack}`;
  }, [isFocusMode, focusedTrackId, styles]);

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
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <div className={styles.title}>🎚️ Mix - Nexus DAW</div>
          {isFocusMode && (
            <div className={styles.focusModeIndicator}>
              <Eye24Regular />
              Focus: {tracks.find((t) => t.id === focusedTrackId)?.name || 'Unknown'}
              <Button
                appearance="transparent"
                size="small"
                icon={<EyeOff24Regular />}
                onClick={clearFocus}
                style={{ marginLeft: '8px' }}
              />
            </div>
          )}
        </div>
        <div style={{ display: 'flex', gap: '12px' }}>
          <Tooltip content="Guitar Tuner" relationship="label">
            <Button
              icon={<MusicNote224Regular />}
              appearance="subtle"
              onClick={() => setTunerOpen(true)}
            >
              Tuner
            </Button>
          </Tooltip>
          <Button icon={<Add24Regular />} onClick={handleAddTrack}>
            Add Track
          </Button>
          <Button icon={<BotRegular />} appearance="subtle" onClick={() => setAIAssistantOpen(true)}>
            AI Mix Suggestions
          </Button>
        </div>
      </div>

      <div className={styles.content}>
        {tracks.length === 0 ? (
          <div className={styles.emptyState}>
            <h3>No Tracks to Mix</h3>
            <p style={{ color: tokens.colorNeutralForeground2, marginBottom: '24px' }}>
              Your project doesn't have any tracks yet. Add tracks in Compose mode or click "Add Track" below.
            </p>
            <div style={{ display: 'flex', gap: '12px', justifyContent: 'center' }}>
              <Button
                appearance="primary"
                icon={<Add24Regular />}
                onClick={handleAddTrack}
              >
                Add Track
              </Button>
              <Button
                appearance="secondary"
                onClick={() => setMode('compose')}
              >
                Go to Compose Mode
              </Button>
            </div>
          </div>
        ) : (
        <div className={styles.mixerSection}>
          {/* Track Channels */}
          {tracks.map((track) => (
            <div key={track.id} className={getTrackClassName(track.id)}>
              <ChannelStripWithProcessors
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
                onFocusToggle={() => toggleFocus(track.id)}
                isFocused={focusedTrackId === track.id}
              />
            </div>
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
        )}
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

      {/* Tuner Dialog */}
      <TunerDialog open={tunerOpen} onOpenChange={setTunerOpen} />
    </div>
  );
}
