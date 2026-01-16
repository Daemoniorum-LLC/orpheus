/**
 * Mix Mode - Professional mixing console (Nexus DAW)
 */

import { Button } from '@persona-framework/ui';
import { Plus, Bot, FolderOpen } from 'lucide-react';
import { useState, useEffect, useMemo } from 'react';
import { useProject, useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { ChannelStripWithProcessors } from '../components/ChannelStripWithProcessors';
import { ChannelStrip } from '../components/ChannelStrip';
import { ConfirmDialog } from '../components/ConfirmDialog';

interface MixSettings {
  volume: number;
  pan: number;
  solo: boolean;
  mute: boolean;
}

// Default mix settings for a new track
const DEFAULT_MIX_SETTINGS: MixSettings = {
  volume: 0,
  pan: 0,
  solo: false,
  mute: false,
};

export function MixMode() {
  const project = useProject();
  const { setAIAssistantOpen, setProject, setMode } = useAppStore();

  // Store mix settings per track ID (survives track list changes)
  const [mixSettingsMap, setMixSettingsMap] = useState<Record<string, MixSettings>>({});
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

  // Derive tracks from the actual project
  const projectTracks = useMemo(() => {
    if (!project?.project.composition.tracks) return [];
    return project.project.composition.tracks.map((track: any) => ({
      id: track.id,
      name: track.name || 'Untitled Track',
      instrument: track.instrument?.type || 'Unknown',
    }));
  }, [project]);

  // Initialize mix settings for new tracks
  useEffect(() => {
    if (projectTracks.length > 0) {
      setMixSettingsMap((prev) => {
        const updated = { ...prev };
        projectTracks.forEach((track) => {
          if (!updated[track.id]) {
            updated[track.id] = { ...DEFAULT_MIX_SETTINGS };
          }
        });
        return updated;
      });
    }
  }, [projectTracks]);

  // Get mix settings for a track (with defaults)
  const getMixSettings = (trackId: string): MixSettings => {
    return mixSettingsMap[trackId] || { ...DEFAULT_MIX_SETTINGS };
  };

  // Update mix settings for a track
  const updateMixSettings = (trackId: string, updates: Partial<MixSettings>) => {
    setMixSettingsMap((prev) => ({
      ...prev,
      [trackId]: { ...getMixSettings(trackId), ...updates },
    }));
  };

  const handleDeleteClick = (trackId: string, trackName: string) => {
    setDeleteConfirm({
      open: true,
      trackId,
      trackName,
    });
  };

  const confirmDeleteTrack = () => {
    if (!project) return;

    // Remove track from project composition
    const updatedTracks = project.project.composition.tracks?.filter(
      (track: any) => track.id !== deleteConfirm.trackId
    ) || [];

    // Create updated project
    const updatedProject = {
      ...project,
      project: {
        ...project.project,
        composition: {
          ...project.project.composition,
          tracks: updatedTracks,
        },
        metadata: {
          ...project.project.metadata,
          modified: new Date().toISOString(),
        },
      },
    };

    // Update project in store
    setProject(updatedProject);

    // Clear mix settings for deleted track
    setMixSettingsMap((prev) => {
      const updated = { ...prev };
      delete updated[deleteConfirm.trackId];
      return updated;
    });

    // Close dialog
    setDeleteConfirm({ open: false, trackId: '', trackName: '' });
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
      <div className="h-full flex flex-col">
        <div className="p-4 border-b border-border flex justify-between items-center">
          <div className="text-xl font-semibold">🎚️ Mix - Nexus DAW</div>
        </div>
        <div className="flex-1 flex items-center justify-center flex-col gap-4">
          <h3 className="text-lg font-semibold">No Project Loaded</h3>
          <p className="text-muted-foreground mb-6">
            To use Mix mode, you need to load a project first.
          </p>
          <div className="flex gap-3 justify-center">
            <Button onClick={handleImportFile}>
              <FolderOpen className="h-4 w-4 mr-2" />
              Import Guitar Pro File
            </Button>
            <Button variant="secondary" onClick={() => setMode('compose')}>
              Go to Compose Mode
            </Button>
          </div>
          <p className="text-xs text-muted-foreground mt-4">
            Or press <strong>Ctrl+1</strong> to switch to Compose mode
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 border-b border-border flex justify-between items-center">
        <div className="text-xl font-semibold">🎚️ Mix - Nexus DAW</div>
        <div className="flex gap-3">
          <Button variant="secondary" onClick={() => setAIAssistantOpen(true)}>
            <Bot className="h-4 w-4 mr-2" />
            AI Mix Suggestions
          </Button>
        </div>
      </div>

      <div className="flex-1 flex p-6 gap-4 overflow-auto bg-secondary">
        <div className="flex gap-3 items-stretch">
          {/* Track Channels from actual project */}
          {projectTracks.length === 0 ? (
            <div className="flex items-center justify-center text-muted-foreground p-8">
              <div className="text-center">
                <p className="mb-2">No tracks in project</p>
                <p className="text-sm">Import a Guitar Pro file with tracks to mix</p>
              </div>
            </div>
          ) : (
            projectTracks.map((track) => {
              const settings = getMixSettings(track.id);
              return (
                <ChannelStripWithProcessors
                  key={track.id}
                  trackId={track.id}
                  trackName={track.name}
                  volume={settings.volume}
                  pan={settings.pan}
                  solo={settings.solo}
                  mute={settings.mute}
                  onVolumeChange={(v) => updateMixSettings(track.id, { volume: v })}
                  onPanChange={(p) => updateMixSettings(track.id, { pan: p })}
                  onSoloToggle={() => updateMixSettings(track.id, { solo: !settings.solo })}
                  onMuteToggle={() => updateMixSettings(track.id, { mute: !settings.mute })}
                  onDelete={() => handleDeleteClick(track.id, track.name)}
                />
              );
            })
          )}

          {/* Master Channel */}
          <div className="border-l-2 border-primary pl-3">
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
