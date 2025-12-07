/**
 * StatusBar - Global status bar showing app state
 * Memoized for performance
 */

import { makeStyles, shorthands, tokens } from '@fluentui/react-components';
import {
  Circle12Filled,
  RecordFilled,
  PlayFilled,
  EditRegular,
} from '@fluentui/react-icons';
import { useAppStore, type AppMode } from '../store/app-store';
import { getAudioRecorder } from '../services/audio-recorder';
import { useState, useEffect, memo } from 'react';
import { ScreenReaderAnnouncer } from '../utils/accessibility';

const useStyles = makeStyles({
  statusBar: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    ...shorthands.padding('4px', '16px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderTop('1px', 'solid', tokens.colorNeutralStroke1),
    fontSize: '11px',
    color: tokens.colorNeutralForeground2,
    height: '24px',
    flexShrink: 0,
  },
  leftSection: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('16px'),
  },
  rightSection: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('16px'),
  },
  statusItem: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('4px'),
  },
  modeName: {
    fontWeight: 600,
    textTransform: 'capitalize',
  },
  recording: {
    color: tokens.colorPaletteRedForeground1,
    fontWeight: 600,
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('4px'),
  },
  playing: {
    color: tokens.colorBrandForeground1,
    fontWeight: 600,
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('4px'),
  },
  modified: {
    color: tokens.colorPaletteYellowForeground2,
    fontWeight: 600,
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('4px'),
  },
  dot: {
    fontSize: '8px',
  },
});

const MODE_LABELS: Record<AppMode, string> = {
  compose: '🎼 Compose',
  record: '🎙️ Record',
  mix: '🎚️ Mix',
  master: '✨ Master',
  practice: '🎸 Practice',
  distribute: '🌍 Distribute',
};

export const StatusBar = memo(function StatusBar() {
  const styles = useStyles();
  const {
    mode,
    project,
    isPlaying,
    currentTime,
    projectModified,
  } = useAppStore();

  const [isRecording, setIsRecording] = useState(false);
  const [recordingTime, setRecordingTime] = useState(0);

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
    <div className={styles.statusBar} role="status" aria-label="Application status bar">
      <div className={styles.leftSection}>
        {/* Current Mode */}
        <div className={styles.statusItem} aria-label={`Current mode: ${MODE_LABELS[mode]}`}>
          <span className={styles.modeName}>{MODE_LABELS[mode]}</span>
        </div>

        {/* Project Name */}
        <div className={styles.statusItem} aria-label={`Project name: ${projectName}`}>
          <span>{projectName}</span>
        </div>

        {/* Modified Indicator */}
        {projectModified && (
          <div className={styles.modified} aria-label="Project has unsaved changes">
            <EditRegular style={{ fontSize: '12px' }} aria-hidden="true" />
            <span>Modified</span>
          </div>
        )}
      </div>

      <div className={styles.rightSection}>
        {/* Recording Status */}
        {isRecording && (
          <div className={styles.recording} aria-label={`Recording: ${formatTime(recordingTime)}`}>
            <RecordFilled style={{ fontSize: '10px' }} aria-hidden="true" />
            <span>REC {formatTime(recordingTime)}</span>
          </div>
        )}

        {/* Playback Status */}
        {isPlaying && !isRecording && (
          <div className={styles.playing} aria-label={`Playing: ${formatTime(currentTime)}`}>
            <PlayFilled style={{ fontSize: '10px' }} aria-hidden="true" />
            <span>{formatTime(currentTime)}</span>
          </div>
        )}

        {/* Ready Status */}
        {!isRecording && !isPlaying && (
          <div className={styles.statusItem} aria-label="System ready">
            <Circle12Filled className={styles.dot} style={{ color: tokens.colorPaletteGreenForeground2 }} aria-hidden="true" />
            <span>Ready</span>
          </div>
        )}
      </div>
    </div>
  );
});
