/**
 * Record Mode - Multi-track audio recording (Nexus DAW)
 */

import { makeStyles, shorthands, tokens, Button, Input, Card, Dropdown, Option, ProgressBar, Menu, MenuTrigger, MenuPopover, MenuList, MenuItem, Spinner } from '@fluentui/react-components';
import {
  Record24Regular,
  Stop24Regular,
  Pause24Regular,
  Play24Regular,
  Delete24Regular,
  ArrowDownload24Regular,
  Mic24Regular,
  ChevronDown16Regular,
} from '@fluentui/react-icons';
import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import * as Tone from 'tone';
import { WaveformVisualizer } from '../components/WaveformVisualizer';
import { Metronome } from '../components/Metronome';
import { InputLevelMeter } from '../components/InputLevelMeter';
import { TrackWaveform } from '../components/TrackWaveform';
import { getAudioRecorder, type RecordingTrack, type RecorderState, type AudioDevice } from '../services/audio-recorder';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { AudioExporter } from '../services/audio-export';

type ExportFormat = 'webm' | 'wav-16' | 'wav-24';

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
    flexDirection: 'column',
    ...shorthands.padding('24px'),
    ...shorthands.gap('24px'),
    overflow: 'auto',
  },
  visualizerSection: {
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground2,
  },
  visualizerHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
  },
  visualizerTitle: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  recordingTime: {
    fontSize: '24px',
    fontFamily: 'monospace',
    fontWeight: 700,
    color: tokens.colorPaletteRedForeground1,
  },
  controls: {
    display: 'flex',
    ...shorthands.gap('12px'),
    alignItems: 'center',
  },
  tracksSection: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
  },
  sectionTitle: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '8px',
  },
  trackCard: {
    ...shorthands.padding('16px'),
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  trackInfo: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('4px'),
  },
  trackName: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
  },
  trackMeta: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
  },
  trackActions: {
    display: 'flex',
    ...shorthands.gap('8px'),
  },
  emptyState: {
    textAlign: 'center',
    ...shorthands.padding('40px'),
    color: tokens.colorNeutralForeground2,
  },
  recordingIndicator: {
    position: 'fixed',
    top: '80px',
    right: '20px',
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    ...shorthands.padding('12px', '20px'),
    backgroundColor: tokens.colorPaletteRedBackground3,
    color: '#ffffff',
    ...shorthands.borderRadius('24px'),
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
    zIndex: 1000,
    fontWeight: 600,
    fontSize: '14px',
  },
  pulsingDot: {
    width: '12px',
    height: '12px',
    ...shorthands.borderRadius('50%'),
    backgroundColor: '#ff0000',
    boxShadow: '0 0 10px rgba(255, 0, 0, 0.8)',
    animationName: 'pulse',
    animationDuration: '1.5s',
    animationIterationCount: 'infinite',
    '@keyframes pulse': {
      '0%': {
        transform: 'scale(1)',
        opacity: '1',
      },
      '50%': {
        transform: 'scale(1.3)',
        opacity: '0.6',
      },
      '100%': {
        transform: 'scale(1)',
        opacity: '1',
      },
    },
  },
  recordingBorder: {
    ...shorthands.border('2px', 'solid', tokens.colorPaletteRedForeground1),
    boxShadow: `0 0 20px ${tokens.colorPaletteRedBackground3}`,
  },
});

export function RecordMode() {
  const styles = useStyles();
  const [recorderState, setRecorderState] = useState<RecorderState>({
    isRecording: false,
    isPaused: false,
    currentTime: 0,
    tracks: [],
  });
  const [trackName, setTrackName] = useState('');
  const [initialized, setInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string>('');

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

  // Playback state
  const [playingTrackId, setPlayingTrackId] = useState<string | null>(null);
  const [playbackProgress, setPlaybackProgress] = useState<Record<string, number>>({});
  const playerRef = useRef<Tone.Player | null>(null);
  const playbackIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Export state
  const [exportingTrackId, setExportingTrackId] = useState<string | null>(null);

  const recorder = getAudioRecorder();

  // Initialize recorder
  useEffect(() => {
    const init = async () => {
      try {
        // Initialize recorder
        await recorder.initialize();

        // Enumerate audio devices
        const devices = await recorder.enumerateDevices();
        setAudioDevices(devices);
        if (devices.length > 0) {
          setSelectedDevice(devices[0].deviceId);
        }

        // Start level monitoring
        recorder.startLevelMonitoring();

        setInitialized(true);
        console.log('[RecordMode] Recorder initialized');
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : 'Initialization failed');
        console.error('[RecordMode] Initialization error:', err);
      }
    };

    init();

    // Subscribe to state changes
    const unsubscribe = recorder.onStateChange(setRecorderState);

    // Update time while recording
    const interval = setInterval(() => {
      if (recorder.getState().isRecording && !recorder.getState().isPaused) {
        setRecorderState(recorder.getState());
      }
    }, 100);

    return () => {
      unsubscribe();
      clearInterval(interval);
      recorder.stopLevelMonitoring();
    };
  }, [recorder]);

  const handleDeviceChange = async (_: unknown, data: { optionValue?: string }) => {
    if (!data.optionValue) return;

    try {
      setSelectedDevice(data.optionValue);
      await recorder.selectDevice(data.optionValue);
      recorder.startLevelMonitoring(); // Restart level monitoring after device change
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to change device');
    }
  };

  const handleStartRecording = async () => {
    try {
      await recorder.startRecording();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start recording');
    }
  };

  const handlePauseResume = () => {
    if (recorderState.isPaused) {
      recorder.resumeRecording();
    } else {
      recorder.pauseRecording();
    }
  };

  const handleStopRecording = async () => {
    try {
      await recorder.stopRecording(trackName || undefined);
      setTrackName('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to stop recording');
    }
  };

  const handleDeleteClick = (track: RecordingTrack) => {
    setDeleteConfirm({
      open: true,
      trackId: track.id,
      trackName: track.name,
    });
  };

  const confirmDeleteTrack = () => {
    recorder.deleteTrack(deleteConfirm.trackId);
  };

  const handleExportTrack = async (track: RecordingTrack, format: ExportFormat = 'webm') => {
    if (!track.blob) {
      setError('No audio data available for export');
      return;
    }

    setExportingTrackId(track.id);

    try {
      if (format === 'webm') {
        // Export as original WebM
        const blob = await recorder.exportTrack(track.id);
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = track.name + '.webm';
        a.click();
        URL.revokeObjectURL(url);
        console.log(`[RecordMode] Exported ${track.name} as WebM`);
      } else {
        // Convert to WAV
        const audioContext = new AudioContext();
        const arrayBuffer = await track.blob.arrayBuffer();
        const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);

        const bitDepth = format === 'wav-24' ? 24 : 16;
        const result = await AudioExporter.exportToWAV(audioBuffer, {
          sampleRate: audioBuffer.sampleRate as 44100 | 48000,
          bitDepth,
          channels: audioBuffer.numberOfChannels as 1 | 2,
        });

        // Download the WAV file
        const safeName = track.name.replace(/[^a-zA-Z0-9-_]/g, '_');
        AudioExporter.downloadBlob(result.blob, `${safeName}.wav`);

        console.log(`[RecordMode] Exported ${track.name} as WAV (${bitDepth}-bit), size: ${AudioExporter.formatFileSize(result.fileSize)}`);
        audioContext.close();
      }
    } catch (err) {
      console.error('[RecordMode] Export error:', err);
      setError(err instanceof Error ? err.message : 'Failed to export track');
    } finally {
      setExportingTrackId(null);
    }
  };

  // Play a recorded track
  const handlePlayTrack = async (track: RecordingTrack) => {
    try {
      // Stop any currently playing track
      await handleStopPlayback();

      // Start Tone.js if not started
      await Tone.start();

      if (!track.blob) {
        setError('No audio data available for this track');
        return;
      }

      // Create URL from blob
      const url = URL.createObjectURL(track.blob);

      // Create player
      const player = new Tone.Player(url);
      player.toDestination();

      // Wait for player to load
      await new Promise<void>((resolve, reject) => {
        player.buffer.onload = () => resolve();
        player.buffer.onerror = reject;
        // Also resolve after a timeout if already loaded
        if (player.buffer.loaded) {
          resolve();
        }
      });

      playerRef.current = player;
      setPlayingTrackId(track.id);

      // Start playback
      player.start();

      // Track progress
      const startTime = Tone.now();
      playbackIntervalRef.current = setInterval(() => {
        const elapsed = Tone.now() - startTime;
        const progress = Math.min(elapsed / track.duration, 1);
        setPlaybackProgress((prev) => ({ ...prev, [track.id]: progress }));

        if (progress >= 1) {
          handleStopPlayback();
        }
      }, 100);

      // Handle playback end
      player.onstop = () => {
        handleStopPlayback();
      };

      console.log(`[RecordMode] Playing track: ${track.name}`);
    } catch (err) {
      console.error('[RecordMode] Playback error:', err);
      setError(err instanceof Error ? err.message : 'Failed to play track');
      handleStopPlayback();
    }
  };

  // Stop playback
  const handleStopPlayback = async () => {
    if (playbackIntervalRef.current) {
      clearInterval(playbackIntervalRef.current);
      playbackIntervalRef.current = null;
    }

    if (playerRef.current) {
      try {
        playerRef.current.stop();
        playerRef.current.dispose();
      } catch {
        // Ignore errors when stopping
      }
      playerRef.current = null;
    }

    setPlayingTrackId(null);
    setPlaybackProgress({});
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      handleStopPlayback();
    };
  }, []);

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 100);
    return mins + ':' + secs.toString().padStart(2, '0') + '.' + ms.toString().padStart(2, '0');
  };

  if (error) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <div className={styles.title}>🎙️ Record - Nexus DAW</div>
        </div>
        <div className={styles.content}>
          <div className={styles.emptyState}>
            <h3 style={{ color: tokens.colorPaletteRedForeground1 }}>⚠️ Error</h3>
            <p>{error}</p>
            <p style={{ marginTop: '16px', fontSize: '12px' }}>
              Make sure you have granted microphone permission and are using HTTPS.
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (!initialized) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <div className={styles.title}>🎙️ Record - Nexus DAW</div>
        </div>
        <div className={styles.content}>
          <div className={styles.emptyState}>
            <p>Initializing audio recorder...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      {/* Floating Recording Indicator */}
      {recorderState.isRecording && (
        <div className={styles.recordingIndicator}>
          <div className={styles.pulsingDot} />
          <span>● REC</span>
          <span style={{ fontFamily: 'monospace', fontWeight: 700 }}>
            {formatTime(recorderState.currentTime)}
          </span>
        </div>
      )}

      <div className={styles.header}>
        <div className={styles.title}>🎙️ Record - Nexus DAW</div>
        <div className={styles.controls}>
          {recorderState.isRecording && (
            <div className={styles.recordingTime}>{formatTime(recorderState.currentTime)}</div>
          )}
          <Button
            icon={<Record24Regular />}
            appearance="primary"
            onClick={handleStartRecording}
            disabled={recorderState.isRecording}
            style={
              recorderState.isRecording
                ? {}
                : { backgroundColor: tokens.colorPaletteRedBackground3 }
            }
          >
            {recorderState.isRecording ? 'Recording...' : 'Record'}
          </Button>
          {recorderState.isRecording && (
            <>
              <Button
                icon={recorderState.isPaused ? <Play24Regular /> : <Pause24Regular />}
                onClick={handlePauseResume}
              >
                {recorderState.isPaused ? 'Resume' : 'Pause'}
              </Button>
              <Button icon={<Stop24Regular />} onClick={handleStopRecording}>
                Stop
              </Button>
            </>
          )}
        </div>
      </div>

      <div className={styles.content}>
        {/* Metronome */}
        <Metronome initialBPM={120} initialTimeSignature={[4, 4]} />

        {/* Audio Input Selection */}
        {audioDevices.length > 0 && (
          <div style={{ maxWidth: '400px' }}>
            <Dropdown
              placeholder="Select microphone"
              value={audioDevices.find((d) => d.deviceId === selectedDevice)?.label || ''}
              onOptionSelect={handleDeviceChange}
              disabled={recorderState.isRecording}
            >
              {audioDevices.map((device) => (
                <Option key={device.deviceId} value={device.deviceId}>
                  {device.label}
                </Option>
              ))}
            </Dropdown>
          </div>
        )}

        {/* Input Level Meter */}
        {!recorderState.isRecording && (
          <div style={{ maxWidth: '600px' }}>
            <InputLevelMeter level={recorderState.inputLevel || 0} />
          </div>
        )}

        {/* Waveform Visualizer */}
        <div className={`${styles.visualizerSection} ${recorderState.isRecording ? styles.recordingBorder : ''}`}>
          <div className={styles.visualizerHeader}>
            <div className={styles.visualizerTitle}>
              <Mic24Regular />
              <span>Live Input Monitor</span>
            </div>
            {recorderState.isRecording && (
              <Input
                placeholder="Track name (optional)"
                value={trackName}
                onChange={(e) => setTrackName(e.target.value)}
                style={{ maxWidth: '200px' }}
              />
            )}
          </div>
          <WaveformVisualizer width={1200} height={150} type="both" color="#667eea" />
        </div>

        {/* Recorded Tracks */}
        <div className={styles.tracksSection}>
          <div className={styles.sectionTitle}>
            Recorded Tracks ({recorderState.tracks.length})
          </div>

          {recorderState.tracks.length === 0 ? (
            <div className={styles.emptyState}>
              <p>No tracks recorded yet</p>
              <p style={{ fontSize: '12px', marginTop: '8px' }}>
                Click the Record button above to start recording
              </p>
            </div>
          ) : (
            recorderState.tracks.map((track) => {
              const isPlaying = playingTrackId === track.id;
              const isExporting = exportingTrackId === track.id;
              const progress = playbackProgress[track.id] || 0;

              return (
                <Card key={track.id} className={styles.trackCard}>
                  <div style={{ flex: 1 }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                      <div className={styles.trackInfo}>
                        <div className={styles.trackName}>{track.name}</div>
                        <div className={styles.trackMeta}>
                          Duration: {formatTime(track.duration)} • ID: {track.id.slice(-8)}
                        </div>
                      </div>
                      <div className={styles.trackActions}>
                        <Button
                          icon={isPlaying ? <Stop24Regular /> : <Play24Regular />}
                          size="small"
                          appearance={isPlaying ? 'primary' : 'secondary'}
                          onClick={() => isPlaying ? handleStopPlayback() : handlePlayTrack(track)}
                          disabled={recorderState.isRecording}
                        >
                          {isPlaying ? 'Stop' : 'Play'}
                        </Button>

                        {/* Export Format Menu */}
                        <Menu>
                          <MenuTrigger disableButtonEnhancement>
                            <Button
                              icon={isExporting ? <Spinner size="tiny" /> : <ArrowDownload24Regular />}
                              size="small"
                              disabled={isExporting}
                            >
                              Export
                              <ChevronDown16Regular />
                            </Button>
                          </MenuTrigger>
                          <MenuPopover>
                            <MenuList>
                              <MenuItem onClick={() => handleExportTrack(track, 'wav-24')}>
                                WAV (24-bit) - Highest Quality
                              </MenuItem>
                              <MenuItem onClick={() => handleExportTrack(track, 'wav-16')}>
                                WAV (16-bit) - CD Quality
                              </MenuItem>
                              <MenuItem onClick={() => handleExportTrack(track, 'webm')}>
                                WebM (Original)
                              </MenuItem>
                            </MenuList>
                          </MenuPopover>
                        </Menu>

                        <Button
                          icon={<Delete24Regular />}
                          size="small"
                          appearance="subtle"
                          onClick={() => handleDeleteClick(track)}
                          disabled={isPlaying || isExporting}
                        >
                          Delete
                        </Button>
                      </div>
                    </div>

                    {/* Waveform display */}
                    {track.blob && (
                      <div style={{ marginTop: '12px' }}>
                        <TrackWaveform
                          audioBlob={track.blob}
                          duration={track.duration}
                          progress={progress}
                          isPlaying={isPlaying}
                          color="#667eea"
                          height={50}
                          onSeek={(pos) => {
                            // Seek within the track if playing
                            if (isPlaying && playerRef.current) {
                              const seekTime = pos * track.duration;
                              playerRef.current.seek(seekTime);
                              setPlaybackProgress((prev) => ({ ...prev, [track.id]: pos }));
                            }
                          }}
                        />
                      </div>
                    )}

                    {/* Playback progress when playing */}
                    {isPlaying && (
                      <div style={{ marginTop: '8px', display: 'flex', justifyContent: 'space-between' }}>
                        <span style={{ fontSize: '11px', color: tokens.colorNeutralForeground2 }}>
                          {formatTime(progress * track.duration)}
                        </span>
                        <span style={{ fontSize: '11px', color: tokens.colorNeutralForeground2 }}>
                          {formatTime(track.duration)}
                        </span>
                      </div>
                    )}
                  </div>
                </Card>
              );
            })
          )}
        </div>
      </div>

      <ConfirmDialog
        open={deleteConfirm.open}
        onConfirm={confirmDeleteTrack}
        onCancel={() => setDeleteConfirm({ ...deleteConfirm, open: false })}
        title="Delete Recording?"
        message={`Are you sure you want to delete the recording "${deleteConfirm.trackName}"? This action cannot be undone.`}
        confirmText="Delete"
        cancelText="Cancel"
        type="danger"
      />
    </div>
  );
}
