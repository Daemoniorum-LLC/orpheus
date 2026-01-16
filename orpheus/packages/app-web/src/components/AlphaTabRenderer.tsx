/**
 * AlphaTab Renderer Component
 * Professional tablature rendering using alphaTab library
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import {
  makeStyles,
  shorthands,
  tokens,
  Button,
  Slider,
  Tooltip,
} from '@fluentui/react-components';
import {
  PlayRegular,
  PauseRegular,
  StopRegular,
  PreviousRegular,
  NextRegular,
  Speaker2Regular,
  SpeakerMuteRegular,
} from '@fluentui/react-icons';
import * as alphaTab from '@coderline/alphatab';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('8px'),
    ...shorthands.overflow('hidden'),
  },
  toolbar: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    ...shorthands.padding('12px', '16px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke2),
  },
  playbackControls: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('4px'),
  },
  volumeControl: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    marginLeft: 'auto',
    width: '150px',
  },
  timeDisplay: {
    fontFamily: 'monospace',
    fontSize: '14px',
    color: tokens.colorNeutralForeground2,
    minWidth: '100px',
    textAlign: 'center',
  },
  progressContainer: {
    ...shorthands.padding('0', '16px'),
    backgroundColor: tokens.colorNeutralBackground2,
  },
  progressBar: {
    width: '100%',
    height: '4px',
    backgroundColor: tokens.colorNeutralStroke2,
    ...shorthands.borderRadius('2px'),
    cursor: 'pointer',
    position: 'relative',
  },
  progressFill: {
    position: 'absolute',
    height: '100%',
    backgroundColor: tokens.colorBrandBackground,
    ...shorthands.borderRadius('2px'),
    transitionProperty: 'width',
    transitionDuration: '100ms',
  },
  rendererContainer: {
    flex: 1,
    ...shorthands.overflow('auto'),
    ...shorthands.padding('16px'),
  },
  alphaTabMain: {
    width: '100%',
    minHeight: '400px',
  },
  loadingOverlay: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '300px',
    color: tokens.colorNeutralForeground3,
  },
  trackSelector: {
    display: 'flex',
    ...shorthands.gap('4px'),
    flexWrap: 'wrap',
    marginLeft: '16px',
  },
  trackButton: {
    minWidth: 'auto',
  },
  trackButtonActive: {
    backgroundColor: tokens.colorBrandBackground,
    color: tokens.colorNeutralForegroundOnBrand,
  },
});

interface AlphaTabRendererProps {
  /** Raw Guitar Pro file data (ArrayBuffer) */
  gpData?: ArrayBuffer;
  /** Track indices to display (default: all) */
  visibleTracks?: number[];
  /** Show playback controls */
  showControls?: boolean;
  /** Callback when playback position changes */
  onPositionChange?: (position: number, duration: number) => void;
  /** Callback when track is clicked */
  onTrackClick?: (trackIndex: number) => void;
}

export function AlphaTabRenderer({
  gpData,
  visibleTracks,
  showControls = true,
  onPositionChange,
  onTrackClick,
}: AlphaTabRendererProps) {
  const styles = useStyles();
  const containerRef = useRef<HTMLDivElement>(null);
  const apiRef = useRef<alphaTab.AlphaTabApi | null>(null);

  const [isLoading, setIsLoading] = useState(true);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [totalTime, setTotalTime] = useState(0);
  const [volume, setVolume] = useState(1);
  const [isMuted, setIsMuted] = useState(false);
  const [tracks, setTracks] = useState<alphaTab.model.Track[]>([]);
  const [activeTracks, setActiveTracks] = useState<number[]>([]);

  // Initialize alphaTab
  useEffect(() => {
    if (!containerRef.current) return;

    const settings = new alphaTab.Settings();
    settings.core.engine = 'html5';
    settings.core.logLevel = alphaTab.LogLevel.Warning;
    settings.display.staveProfile = alphaTab.StaveProfile.Tab;
    settings.display.layoutMode = alphaTab.LayoutMode.Page;
    settings.notation.notationMode = alphaTab.NotationMode.GuitarPro;
    settings.player.enablePlayer = true;
    settings.player.enableCursor = true;
    settings.player.enableUserInteraction = true;
    settings.player.soundFont = '/soundfont/sonivox.sf2'; // Would need actual soundfont

    const api = new alphaTab.AlphaTabApi(containerRef.current, settings);

    // Event handlers
    api.renderStarted.on(() => {
      setIsLoading(true);
    });

    api.renderFinished.on(() => {
      setIsLoading(false);
    });

    api.scoreLoaded.on((score) => {
      if (score) {
        setTracks(score.tracks);
        setActiveTracks(score.tracks.map((_, i) => i));
      }
    });

    api.playerStateChanged.on((args) => {
      setIsPlaying(args.state === alphaTab.synth.PlayerState.Playing);
    });

    api.playerPositionChanged.on((args) => {
      setCurrentTime(args.currentTime);
      setTotalTime(args.endTime);
      onPositionChange?.(args.currentTime, args.endTime);
    });

    apiRef.current = api;

    return () => {
      api.destroy();
      apiRef.current = null;
    };
  }, [onPositionChange]);

  // Load GP data when provided
  useEffect(() => {
    if (!apiRef.current || !gpData) return;

    const uint8Array = new Uint8Array(gpData);
    apiRef.current.load(uint8Array);
  }, [gpData]);

  // Update visible tracks
  useEffect(() => {
    if (!apiRef.current || !tracks.length) return;

    if (visibleTracks) {
      const tracksToRender = tracks.filter((_, i) => visibleTracks.includes(i));
      apiRef.current.renderTracks(tracksToRender);
    }
  }, [visibleTracks, tracks]);

  // Playback controls
  const handlePlayPause = useCallback(() => {
    if (!apiRef.current) return;
    apiRef.current.playPause();
  }, []);

  const handleStop = useCallback(() => {
    if (!apiRef.current) return;
    apiRef.current.stop();
  }, []);

  const handlePrevious = useCallback(() => {
    if (!apiRef.current) return;
    // Go to previous bar
    const player = apiRef.current;
    player.tickPosition = Math.max(0, player.tickPosition - 960); // 960 = quarter note
  }, []);

  const handleNext = useCallback(() => {
    if (!apiRef.current) return;
    // Go to next bar
    const player = apiRef.current;
    player.tickPosition = player.tickPosition + 960;
  }, []);

  const handleVolumeChange = useCallback((value: number) => {
    if (!apiRef.current) return;
    setVolume(value);
    apiRef.current.masterVolume = value;
  }, []);

  const handleMuteToggle = useCallback(() => {
    if (!apiRef.current) return;
    const newMuted = !isMuted;
    setIsMuted(newMuted);
    apiRef.current.masterVolume = newMuted ? 0 : volume;
  }, [isMuted, volume]);

  const handleProgressClick = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (!apiRef.current || totalTime === 0) return;

    const rect = e.currentTarget.getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;
    const newTime = percent * totalTime;

    apiRef.current.timePosition = newTime;
  }, [totalTime]);

  const handleTrackToggle = useCallback((trackIndex: number) => {
    if (!apiRef.current) return;

    setActiveTracks(prev => {
      const newActive = prev.includes(trackIndex)
        ? prev.filter(i => i !== trackIndex)
        : [...prev, trackIndex];

      // Update alphaTab rendering
      const tracksToRender = tracks.filter((_, i) => newActive.includes(i));
      if (tracksToRender.length > 0) {
        apiRef.current?.renderTracks(tracksToRender);
      }

      return newActive;
    });

    onTrackClick?.(trackIndex);
  }, [tracks, onTrackClick]);

  const formatTime = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  };

  const progressPercent = totalTime > 0 ? (currentTime / totalTime) * 100 : 0;

  return (
    <div className={styles.container}>
      {showControls && (
        <>
          <div className={styles.toolbar}>
            <div className={styles.playbackControls}>
              <Tooltip content="Previous" relationship="label">
                <Button
                  icon={<PreviousRegular />}
                  appearance="subtle"
                  onClick={handlePrevious}
                />
              </Tooltip>
              <Tooltip content={isPlaying ? 'Pause' : 'Play'} relationship="label">
                <Button
                  icon={isPlaying ? <PauseRegular /> : <PlayRegular />}
                  appearance="primary"
                  onClick={handlePlayPause}
                />
              </Tooltip>
              <Tooltip content="Stop" relationship="label">
                <Button
                  icon={<StopRegular />}
                  appearance="subtle"
                  onClick={handleStop}
                />
              </Tooltip>
              <Tooltip content="Next" relationship="label">
                <Button
                  icon={<NextRegular />}
                  appearance="subtle"
                  onClick={handleNext}
                />
              </Tooltip>
            </div>

            <div className={styles.timeDisplay}>
              {formatTime(currentTime)} / {formatTime(totalTime)}
            </div>

            {tracks.length > 1 && (
              <div className={styles.trackSelector}>
                {tracks.map((track, i) => (
                  <Button
                    key={i}
                    size="small"
                    appearance={activeTracks.includes(i) ? 'primary' : 'secondary'}
                    className={styles.trackButton}
                    onClick={() => handleTrackToggle(i)}
                  >
                    {track.name || `Track ${i + 1}`}
                  </Button>
                ))}
              </div>
            )}

            <div className={styles.volumeControl}>
              <Tooltip content={isMuted ? 'Unmute' : 'Mute'} relationship="label">
                <Button
                  icon={isMuted ? <SpeakerMuteRegular /> : <Speaker2Regular />}
                  appearance="subtle"
                  onClick={handleMuteToggle}
                />
              </Tooltip>
              <Slider
                min={0}
                max={1}
                step={0.05}
                value={isMuted ? 0 : volume}
                onChange={(_, data) => handleVolumeChange(data.value)}
                style={{ flex: 1 }}
              />
            </div>
          </div>

          <div className={styles.progressContainer}>
            <div className={styles.progressBar} onClick={handleProgressClick}>
              <div
                className={styles.progressFill}
                style={{ width: `${progressPercent}%` }}
              />
            </div>
          </div>
        </>
      )}

      <div className={styles.rendererContainer}>
        {isLoading && !gpData && (
          <div className={styles.loadingOverlay}>
            Drop a Guitar Pro file or import one to view tablature
          </div>
        )}
        <div ref={containerRef} className={styles.alphaTabMain} />
      </div>
    </div>
  );
}
