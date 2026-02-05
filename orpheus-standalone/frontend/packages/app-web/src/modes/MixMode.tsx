/**
 * Mix Mode - Professional mixing console (Nexus DAW)
 */

import { makeStyles, shorthands, tokens, Button } from '@fluentui/react-components';
import { Add24Regular, BotRegular, FolderOpen24Regular, ChartMultiple24Regular } from '@fluentui/react-icons';
import { useState, useEffect, useMemo, useCallback } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { ChannelStripWithProcessors } from '../components/ChannelStripWithProcessors';
import { ChannelStrip } from '../components/ChannelStrip';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { AutomationLanesContainer, type AutomationPoint, type AutomationParameter } from '../components/AutomationLane';

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
  automationSection: {
    width: '100%',
    ...shorthands.borderTop('1px', 'solid', tokens.colorNeutralStroke1),
    paddingTop: '16px',
    marginTop: '16px',
  },
  automationHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  automationTitle: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground2,
  },
});

interface MixerTrack {
  id: string;
  name: string;
  volume: number;
  pan: number;
  solo: boolean;
  mute: boolean;
  type: 'audio' | 'midi' | 'instrument' | 'aux';
  color?: string;
  sourceType: 'session' | 'composition' | 'added';
}

/**
 * Create mixer tracks from project data
 */
function createTracksFromProject(project: any): MixerTrack[] {
  const mixerTracks: MixerTrack[] = [];

  // First, check if we have session tracks (audio/midi recordings)
  if (project.project.session?.tracks?.length > 0) {
    for (const track of project.project.session.tracks) {
      mixerTracks.push({
        id: track.id,
        name: track.name,
        volume: track.volume ?? 0,
        pan: track.pan ?? 0,
        solo: track.solo ?? false,
        mute: track.muted ?? false,
        type: track.type || 'audio',
        color: track.color,
        sourceType: 'session',
      });
    }
  }

  // Also add composition tracks (from Guitar Pro import, etc.)
  if (project.project.composition?.tracks?.length > 0) {
    for (const track of project.project.composition.tracks) {
      // Don't duplicate if already in session
      if (!mixerTracks.find((t) => t.id === track.id)) {
        mixerTracks.push({
          id: track.id,
          name: track.name,
          volume: 0,
          pan: 0,
          solo: false,
          mute: false,
          type: 'midi',
          sourceType: 'composition',
        });
      }
    }
  }

  // If no tracks found, create a default track
  if (mixerTracks.length === 0) {
    mixerTracks.push({
      id: 'track-1',
      name: project.project.metadata?.title || 'Main Track',
      volume: 0,
      pan: 0,
      solo: false,
      mute: false,
      type: 'audio',
      sourceType: 'added',
    });
  }

  return mixerTracks;
}

export function MixMode() {
  const styles = useStyles();
  const project = useProject();
  const { setAIAssistantOpen, setProject, setMode, updateProject } = useAppStore();
  const [tracks, setTracks] = useState<MixerTrack[]>([]);
  const [masterVolume, setMasterVolume] = useState(0);
  const [masterPan, setMasterPan] = useState(0);
  const [tracksInitialized, setTracksInitialized] = useState(false);
  const [showAutomation, setShowAutomation] = useState(false);
  const [automationData, setAutomationData] = useState<Record<string, Record<AutomationParameter, AutomationPoint[]>>>({});

  // Initialize tracks from project when project changes
  useEffect(() => {
    if (project && !tracksInitialized) {
      const projectTracks = createTracksFromProject(project);
      setTracks(projectTracks);
      setTracksInitialized(true);
      console.log('[MixMode] Loaded tracks from project:', projectTracks.length);
    }
  }, [project, tracksInitialized]);

  // Reset initialization when project changes
  useEffect(() => {
    setTracksInitialized(false);
  }, [project?.project?.metadata?.id]);

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
    const newTrack: MixerTrack = {
      id: `track-${Date.now()}`,
      name: `Track ${tracks.length + 1}`,
      volume: 0,
      pan: 0,
      solo: false,
      mute: false,
      type: 'audio',
      sourceType: 'added',
    };
    setTracks([...tracks, newTrack]);
  };

  // Check if any track is soloed
  const anySoloed = useMemo(() => tracks.some((t) => t.solo), [tracks]);

  const updateTrack = useCallback((id: string, updates: Partial<MixerTrack>) => {
    setTracks((prevTracks) => prevTracks.map((t) => (t.id === id ? { ...t, ...updates } : t)));
  }, []);

  // Handle solo toggle with exclusive mode option
  const handleSoloToggle = useCallback((id: string, exclusive: boolean = false) => {
    setTracks((prevTracks) => {
      const track = prevTracks.find((t) => t.id === id);
      if (!track) return prevTracks;

      const newSoloState = !track.solo;

      if (exclusive && newSoloState) {
        // Exclusive mode: unsolo all other tracks
        return prevTracks.map((t) => ({
          ...t,
          solo: t.id === id ? true : false,
        }));
      } else {
        // Additive mode: just toggle this track's solo
        return prevTracks.map((t) =>
          t.id === id ? { ...t, solo: newSoloState } : t
        );
      }
    });
  }, []);

  // Get effective mute state (considering solo)
  const getEffectiveMute = useCallback((track: MixerTrack): boolean => {
    // If track is explicitly muted, it's muted
    if (track.mute) return true;
    // If any track is soloed and this one isn't, it's effectively muted
    if (anySoloed && !track.solo) return true;
    return false;
  }, [anySoloed]);

  // Handle automation data changes
  const handleAutomationChange = useCallback((trackId: string, parameter: AutomationParameter, points: AutomationPoint[]) => {
    setAutomationData((prev) => ({
      ...prev,
      [trackId]: {
        ...(prev[trackId] || {}),
        [parameter]: points,
      },
    }));
    console.log(`[MixMode] Automation updated: ${trackId} - ${parameter}`, points.length, 'points');
  }, []);

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
          <Button
            icon={<ChartMultiple24Regular />}
            appearance={showAutomation ? 'primary' : 'secondary'}
            onClick={() => setShowAutomation(!showAutomation)}
          >
            Automation
          </Button>
          <Button icon={<BotRegular />} appearance="subtle" onClick={() => setAIAssistantOpen(true)}>
            AI Mix Suggestions
          </Button>
        </div>
      </div>

      <div className={styles.content}>
        <div style={{ display: 'flex', flexDirection: 'column', flex: 1 }}>
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
                effectiveMute={getEffectiveMute(track)}
                color={track.color}
                onVolumeChange={(v) => updateTrack(track.id, { volume: v })}
                onPanChange={(p) => updateTrack(track.id, { pan: p })}
                onSoloToggle={(e) => handleSoloToggle(track.id, e?.ctrlKey || e?.metaKey)}
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

          {/* Automation Lanes */}
          {showAutomation && tracks.length > 0 && (
            <div className={styles.automationSection}>
              <div className={styles.automationHeader}>
                <span className={styles.automationTitle}>Automation Lanes</span>
                <span style={{ fontSize: '11px', color: tokens.colorNeutralForeground3 }}>
                  Click to add points, drag to move, Delete key to remove
                </span>
              </div>
              <AutomationLanesContainer
                tracks={tracks.map((t) => ({ id: t.id, name: t.name, color: t.color }))}
                automationData={automationData}
                duration={project?.project?.metadata?.duration || 180}
                onAutomationChange={handleAutomationChange}
              />
            </div>
          )}
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
