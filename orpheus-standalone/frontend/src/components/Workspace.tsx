import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { TrackList } from './TrackList';
import { Timeline } from './Timeline';
import { Transport } from './Transport';
import { MixerPanel } from './MixerPanel';
import './Workspace.css';

interface Project {
  id: string;
  title: string;
  artist?: string;
  bpm: number;
  timeSignature: string;
  key?: string;
  created: string;
  modified: string;
}

interface Track {
  id: string;
  projectId: string;
  name: string;
  instrument: string;
  trackNumber: number;
  volume: number;
  pan: number;
  muted: boolean;
  soloed: boolean;
  audioFileUrl?: string;
}

export const Workspace: React.FC = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [tracks, setTracks] = useState<Track[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [playheadPosition, setPlayheadPosition] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [autoSaveStatus, setAutoSaveStatus] = useState<'saved' | 'saving' | 'error'>('saved');

  useEffect(() => {
    if (projectId) {
      loadProject();
      loadTracks();
    }
  }, [projectId]);

  const loadProject = async () => {
    try {
      const response = await fetch(`/api/v1/projects/${projectId}`);
      if (!response.ok) throw new Error('Failed to load project');
      const data = await response.json();
      setProject(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const loadTracks = async () => {
    try {
      const response = await fetch(`/api/v1/projects/${projectId}/tracks`);
      if (!response.ok) throw new Error('Failed to load tracks');
      const data = await response.json();
      setTracks(data);
    } catch (err) {
      console.error('Failed to load tracks:', err);
    }
  };

  const handleAddTrack = async (trackData: Partial<Track>) => {
    try {
      const response = await fetch(`/api/v1/projects/${projectId}/tracks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(trackData),
      });

      if (!response.ok) throw new Error('Failed to add track');

      const newTrack = await response.json();
      setTracks([...tracks, newTrack]);
      triggerAutoSave();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  const handleUpdateTrack = async (trackId: string, updates: Partial<Track>) => {
    try {
      const response = await fetch(`/api/v1/tracks/${trackId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates),
      });

      if (!response.ok) throw new Error('Failed to update track');

      const updatedTrack = await response.json();
      setTracks(tracks.map(t => t.id === trackId ? updatedTrack : t));
      triggerAutoSave();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  const handleDeleteTrack = async (trackId: string) => {
    try {
      const response = await fetch(`/api/v1/tracks/${trackId}`, {
        method: 'DELETE',
      });

      if (!response.ok) throw new Error('Failed to delete track');

      setTracks(tracks.filter(t => t.id !== trackId));
      triggerAutoSave();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  const triggerAutoSave = () => {
    setAutoSaveStatus('saving');
    setTimeout(() => {
      setAutoSaveStatus('saved');
    }, 1000);
  };

  const handlePlayPause = () => {
    setIsPlaying(!isPlaying);
  };

  const handleStop = () => {
    setIsPlaying(false);
    setPlayheadPosition(0);
  };

  if (loading) {
    return (
      <div className="workspace-loading">
        Loading project...
      </div>
    );
  }

  if (error || !project) {
    return (
      <div className="workspace-error">
        Error: {error || 'Project not found'}
      </div>
    );
  }

  return (
    <div className="workspace" data-testid="project-workspace">
      {/* Top Bar */}
      <div className="workspace-header">
        <div className="project-info">
          <h1 data-testid="project-title">{project.title}</h1>
          <div className="project-meta">
            <span data-testid="project-bpm">{project.bpm}</span>
            <span>BPM</span>
            <span className="separator">•</span>
            <span>{project.timeSignature}</span>
            {project.key && (
              <>
                <span className="separator">•</span>
                <span data-testid="project-key">{project.key}</span>
              </>
            )}
          </div>
        </div>

        <div className="workspace-actions">
          <span
            className="auto-save-indicator"
            data-testid="auto-save-indicator"
          >
            {autoSaveStatus === 'saving' && '💾 Saving...'}
            {autoSaveStatus === 'saved' && '✓ Saved'}
            {autoSaveStatus === 'error' && '⚠ Error saving'}
          </span>

          <button
            className="btn-icon"
            data-testid="project-settings-button"
            title="Project Settings"
          >
            ⚙️
          </button>

          <button
            className="btn-icon"
            data-testid="export-project-button"
            title="Export Project"
          >
            📤
          </button>

          <button
            className="btn-icon"
            data-testid="share-project-button"
            title="Share Project"
          >
            👥
          </button>
        </div>

        <div
          className="project-id"
          data-testid="project-id"
          data-id={project.id}
          style={{ display: 'none' }}
        />
      </div>

      {/* Main Content */}
      <div className="workspace-content">
        {/* Left Sidebar - Track Controls */}
        <div className="track-controls">
          <TrackList
            tracks={tracks}
            onAddTrack={handleAddTrack}
            onUpdateTrack={handleUpdateTrack}
            onDeleteTrack={handleDeleteTrack}
          />
        </div>

        {/* Center - Timeline */}
        <div className="timeline-container">
          <Timeline
            tracks={tracks}
            playheadPosition={playheadPosition}
            bpm={project.bpm}
            timeSignature={project.timeSignature}
            isPlaying={isPlaying}
          />
        </div>

        {/* Right Sidebar - Mixer */}
        <div className="mixer-container">
          <MixerPanel tracks={tracks} onUpdateTrack={handleUpdateTrack} />
        </div>
      </div>

      {/* Bottom - Transport */}
      <div className="transport-container">
        <Transport
          isPlaying={isPlaying}
          playheadPosition={playheadPosition}
          bpm={project.bpm}
          onPlayPause={handlePlayPause}
          onStop={handleStop}
          onSeek={setPlayheadPosition}
        />
      </div>
    </div>
  );
};
