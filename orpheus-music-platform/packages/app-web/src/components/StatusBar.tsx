/**
 * StatusBar - Global status bar showing app state
 * Memoized for performance
 */

import { Circle, Mic, Play, Pencil, Cloud, CloudOff } from 'lucide-react';
import { useAppStore, type AppMode } from '../store/app-store';
import { getAudioRecorder } from '../services/audio-recorder';
import { getAutoSaveManager } from '../services/project-save';
import { useState, useEffect, memo } from 'react';
import { ScreenReaderAnnouncer } from '../utils/accessibility';

const MODE_LABELS: Record<AppMode, string> = {
  compose: '🎼 Compose',
  record: '🎙️ Record',
  mix: '🎚️ Mix',
  master: '✨ Master',
  practice: '🎸 Practice',
  distribute: '🌍 Distribute',
};

export const StatusBar = memo(function StatusBar() {
  const {
    mode,
    project,
    isPlaying,
    currentTime,
    projectModified,
  } = useAppStore();

  const [isRecording, setIsRecording] = useState(false);
  const [recordingTime, setRecordingTime] = useState(0);
  const [autoSaveEnabled, setAutoSaveEnabled] = useState(false);

  // Monitor auto-save state
  useEffect(() => {
    const manager = getAutoSaveManager();
    // Check initial state
    setAutoSaveEnabled(manager.enabled());

    // Poll for changes (auto-save manager doesn't have an event system)
    const interval = setInterval(() => {
      setAutoSaveEnabled(manager.enabled());
    }, 1000);

    return () => clearInterval(interval);
  }, [project]);

  // Monitor recording state
  useEffect(() => {
    const recorder = getAudioRecorder();
    const unsubscribe = recorder.onStateChange((state) => {
      const wasRecording = isRecording;
      setIsRecording(state.isRecording);
      setRecordingTime(state.currentTime);

      // Announce recording state changes to screen readers
      if (state.isRecording && !wasRecording) {
        ScreenReaderAnnouncer.announce('Recording started', 'assertive');
      } else if (!state.isRecording && wasRecording) {
        ScreenReaderAnnouncer.announce('Recording stopped', 'assertive');
      }
    });

    return unsubscribe;
  }, [isRecording]);

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const projectName = project?.project.metadata.title || 'No Project';

  return (
    <div
      className="flex items-center justify-between px-4 py-1 bg-secondary border-t border-border text-[11px] text-muted-foreground h-6 flex-shrink-0"
      role="status"
      aria-label="Application status bar"
    >
      <div className="flex items-center gap-4">
        {/* Current Mode */}
        <div className="flex items-center gap-1" aria-label={`Current mode: ${MODE_LABELS[mode]}`}>
          <span className="font-semibold capitalize">{MODE_LABELS[mode]}</span>
        </div>

        {/* Project Name */}
        <div className="flex items-center gap-1" aria-label={`Project name: ${projectName}`}>
          <span>{projectName}</span>
        </div>

        {/* Modified Indicator */}
        {projectModified && (
          <div className="flex items-center gap-1 text-yellow-500 font-semibold" aria-label="Project has unsaved changes">
            <Pencil className="h-3 w-3" aria-hidden="true" />
            <span>Modified</span>
          </div>
        )}

        {/* Auto-save Indicator */}
        {project && (
          <div
            className={`flex items-center gap-1 ${autoSaveEnabled ? 'text-green-500' : 'text-muted-foreground/50'}`}
            aria-label={autoSaveEnabled ? 'Auto-save enabled' : 'Auto-save disabled'}
            title={autoSaveEnabled ? 'Auto-save enabled' : 'Auto-save disabled'}
          >
            {autoSaveEnabled ? (
              <Cloud className="h-3 w-3" aria-hidden="true" />
            ) : (
              <CloudOff className="h-3 w-3" aria-hidden="true" />
            )}
          </div>
        )}
      </div>

      <div className="flex items-center gap-4">
        {/* Recording Status */}
        {isRecording && (
          <div className="flex items-center gap-1 text-red-500 font-semibold" aria-label={`Recording: ${formatTime(recordingTime)}`}>
            <Mic className="h-3 w-3" aria-hidden="true" />
            <span>REC {formatTime(recordingTime)}</span>
          </div>
        )}

        {/* Playback Status */}
        {isPlaying && !isRecording && (
          <div className="flex items-center gap-1 text-primary font-semibold" aria-label={`Playing: ${formatTime(currentTime)}`}>
            <Play className="h-3 w-3" aria-hidden="true" />
            <span>{formatTime(currentTime)}</span>
          </div>
        )}

        {/* Ready Status */}
        {!isRecording && !isPlaying && (
          <div className="flex items-center gap-1" aria-label="System ready">
            <Circle className="h-2 w-2 fill-green-500 text-green-500" aria-hidden="true" />
            <span>Ready</span>
          </div>
        )}
      </div>
    </div>
  );
});
