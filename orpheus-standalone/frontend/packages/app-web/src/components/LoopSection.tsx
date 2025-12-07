/**
 * Loop Section Component
 * Allows users to select and loop specific sections of a piece for practice
 */

import { makeStyles, shorthands, tokens, Button, Slider, Card, ToggleButton } from '@fluentui/react-components';
import { ArrowRepeatAll24Regular, Play24Regular, Pause24Regular, Previous24Regular, Next24Regular } from '@fluentui/react-icons';
import { useState, useCallback, useEffect, useRef } from 'react';
import type { MaestroProject } from '@maestro-ai/shared-types';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  timeline: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  timelineBar: {
    height: '40px',
    backgroundColor: tokens.colorNeutralBackground4,
    ...shorthands.borderRadius('4px'),
    position: 'relative',
    cursor: 'pointer',
    ...shorthands.overflow('hidden'),
  },
  loopRegion: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    backgroundColor: tokens.colorBrandBackground,
    opacity: 0.3,
    cursor: 'ew-resize',
  },
  loopHandle: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    width: '8px',
    backgroundColor: tokens.colorBrandBackground,
    cursor: 'ew-resize',
    ...shorthands.transition('background-color', '100ms'),
    ':hover': {
      backgroundColor: tokens.colorBrandForeground1,
    },
  },
  playhead: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    width: '2px',
    backgroundColor: tokens.colorPaletteRedBackground3,
    pointerEvents: 'none',
    zIndex: 2,
  },
  measureMarkers: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    left: 0,
    right: 0,
    display: 'flex',
    pointerEvents: 'none',
  },
  measureMarker: {
    ...shorthands.borderLeft('1px', 'solid', tokens.colorNeutralStroke2),
    flex: 1,
    position: 'relative',
  },
  measureLabel: {
    position: 'absolute',
    top: '2px',
    left: '4px',
    fontSize: '9px',
    color: tokens.colorNeutralForeground3,
  },
  controls: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    ...shorthands.gap('12px'),
  },
  info: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    ...shorthands.gap('12px'),
  },
  infoCard: {
    ...shorthands.padding('12px'),
    textAlign: 'center',
  },
  infoLabel: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    textTransform: 'uppercase',
    marginBottom: '4px',
  },
  infoValue: {
    fontSize: '18px',
    fontWeight: 600,
    fontFamily: 'monospace',
  },
  presets: {
    display: 'flex',
    flexWrap: 'wrap',
    ...shorthands.gap('8px'),
  },
  presetButton: {
    fontSize: '11px',
  },
});

export interface LoopSectionProps {
  project: MaestroProject;
  /** Total number of measures */
  totalMeasures?: number;
  /** Current playback position (0-1) */
  position?: number;
  /** Is currently playing */
  isPlaying?: boolean;
  /** Loop enabled */
  loopEnabled?: boolean;
  /** Loop start position (0-1) */
  loopStart?: number;
  /** Loop end position (0-1) */
  loopEnd?: number;
  /** Callbacks */
  onLoopChange?: (start: number, end: number) => void;
  onLoopToggle?: (enabled: boolean) => void;
  onSeek?: (position: number) => void;
  onPlay?: () => void;
  onPause?: () => void;
}

export function LoopSection({
  project,
  totalMeasures,
  position = 0,
  isPlaying = false,
  loopEnabled = false,
  loopStart = 0,
  loopEnd = 1,
  onLoopChange,
  onLoopToggle,
  onSeek,
  onPlay,
  onPause,
}: LoopSectionProps) {
  const styles = useStyles();
  const timelineRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState<'start' | 'end' | 'region' | null>(null);
  const [localLoopStart, setLocalLoopStart] = useState(loopStart);
  const [localLoopEnd, setLocalLoopEnd] = useState(loopEnd);
  const [loopCount, setLoopCount] = useState(0);

  // Calculate number of measures
  const measures = totalMeasures ||
    (project.project.composition?.tracks?.[0] as any)?.measures?.length || 16;

  // Sync external loop state
  useEffect(() => {
    setLocalLoopStart(loopStart);
    setLocalLoopEnd(loopEnd);
  }, [loopStart, loopEnd]);

  // Count loops when position resets
  useEffect(() => {
    if (loopEnabled && position < localLoopStart + 0.01 && position > localLoopStart - 0.01) {
      setLoopCount((c) => c + 1);
    }
  }, [position, loopEnabled, localLoopStart]);

  const getPositionFromEvent = useCallback((e: MouseEvent | React.MouseEvent): number => {
    if (!timelineRef.current) return 0;
    const rect = timelineRef.current.getBoundingClientRect();
    return Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
  }, []);

  const handleTimelineClick = useCallback((e: React.MouseEvent) => {
    if (dragging) return;
    const pos = getPositionFromEvent(e);
    onSeek?.(pos);
  }, [dragging, getPositionFromEvent, onSeek]);

  const handleDragStart = useCallback((e: React.MouseEvent, type: 'start' | 'end' | 'region') => {
    e.stopPropagation();
    setDragging(type);

    const handleMove = (moveEvent: MouseEvent) => {
      const pos = getPositionFromEvent(moveEvent);

      if (type === 'start') {
        const newStart = Math.min(pos, localLoopEnd - 0.05);
        setLocalLoopStart(newStart);
      } else if (type === 'end') {
        const newEnd = Math.max(pos, localLoopStart + 0.05);
        setLocalLoopEnd(newEnd);
      } else if (type === 'region') {
        // Move entire region (would need initial offset tracking)
      }
    };

    const handleUp = () => {
      setDragging(null);
      onLoopChange?.(localLoopStart, localLoopEnd);
      document.removeEventListener('mousemove', handleMove);
      document.removeEventListener('mouseup', handleUp);
    };

    document.addEventListener('mousemove', handleMove);
    document.addEventListener('mouseup', handleUp);
  }, [localLoopStart, localLoopEnd, getPositionFromEvent, onLoopChange]);

  // Preset sections
  const handlePreset = useCallback((start: number, end: number) => {
    setLocalLoopStart(start);
    setLocalLoopEnd(end);
    onLoopChange?.(start, end);
    setLoopCount(0);
  }, [onLoopChange]);

  // Convert position to measure number
  const positionToMeasure = useCallback((pos: number): number => {
    return Math.floor(pos * measures) + 1;
  }, [measures]);

  // Convert measure to position
  const measureToPosition = useCallback((measure: number): number => {
    return (measure - 1) / measures;
  }, [measures]);

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>
          <ArrowRepeatAll24Regular />
          Loop Section Practice
        </div>
        <ToggleButton
          checked={loopEnabled}
          onClick={() => {
            onLoopToggle?.(!loopEnabled);
            if (!loopEnabled) setLoopCount(0);
          }}
          icon={<ArrowRepeatAll24Regular />}
          appearance={loopEnabled ? 'primary' : 'secondary'}
        >
          {loopEnabled ? 'Loop On' : 'Loop Off'}
        </ToggleButton>
      </div>

      {/* Timeline with loop region */}
      <div className={styles.timeline}>
        <div
          ref={timelineRef}
          className={styles.timelineBar}
          onClick={handleTimelineClick}
        >
          {/* Measure markers */}
          <div className={styles.measureMarkers}>
            {Array.from({ length: measures }, (_, i) => (
              <div key={i} className={styles.measureMarker}>
                <span className={styles.measureLabel}>{i + 1}</span>
              </div>
            ))}
          </div>

          {/* Loop region */}
          {loopEnabled && (
            <>
              <div
                className={styles.loopRegion}
                style={{
                  left: `${localLoopStart * 100}%`,
                  width: `${(localLoopEnd - localLoopStart) * 100}%`,
                }}
                onMouseDown={(e) => handleDragStart(e, 'region')}
              />
              <div
                className={styles.loopHandle}
                style={{ left: `${localLoopStart * 100}%`, transform: 'translateX(-50%)' }}
                onMouseDown={(e) => handleDragStart(e, 'start')}
              />
              <div
                className={styles.loopHandle}
                style={{ left: `${localLoopEnd * 100}%`, transform: 'translateX(-50%)' }}
                onMouseDown={(e) => handleDragStart(e, 'end')}
              />
            </>
          )}

          {/* Playhead */}
          <div
            className={styles.playhead}
            style={{ left: `${position * 100}%` }}
          />
        </div>
      </div>

      {/* Playback controls */}
      <div className={styles.controls}>
        <Button
          icon={<Previous24Regular />}
          appearance="subtle"
          onClick={() => onSeek?.(loopEnabled ? localLoopStart : 0)}
          title="Go to loop start"
        />
        <Button
          icon={isPlaying ? <Pause24Regular /> : <Play24Regular />}
          appearance="primary"
          onClick={() => (isPlaying ? onPause?.() : onPlay?.())}
        >
          {isPlaying ? 'Pause' : 'Play'}
        </Button>
        <Button
          icon={<Next24Regular />}
          appearance="subtle"
          onClick={() => onSeek?.(loopEnabled ? localLoopEnd : 1)}
          title="Go to loop end"
        />
      </div>

      {/* Loop info */}
      <div className={styles.info}>
        <Card className={styles.infoCard}>
          <div className={styles.infoLabel}>Loop Start</div>
          <div className={styles.infoValue}>
            M{positionToMeasure(localLoopStart)}
          </div>
        </Card>
        <Card className={styles.infoCard}>
          <div className={styles.infoLabel}>Loop End</div>
          <div className={styles.infoValue}>
            M{positionToMeasure(localLoopEnd)}
          </div>
        </Card>
        <Card className={styles.infoCard}>
          <div className={styles.infoLabel}>Loop Count</div>
          <div className={styles.infoValue}>
            {loopCount}
          </div>
        </Card>
      </div>

      {/* Preset sections */}
      <div>
        <div className={styles.infoLabel} style={{ marginBottom: '8px' }}>
          Quick Select Section
        </div>
        <div className={styles.presets}>
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0, 0.25)}
          >
            Intro (M1-{Math.ceil(measures * 0.25)})
          </Button>
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0.25, 0.5)}
          >
            Verse ({Math.ceil(measures * 0.25 + 1)}-{Math.ceil(measures * 0.5)})
          </Button>
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0.5, 0.75)}
          >
            Chorus ({Math.ceil(measures * 0.5 + 1)}-{Math.ceil(measures * 0.75)})
          </Button>
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0.75, 1)}
          >
            Outro ({Math.ceil(measures * 0.75 + 1)}-{measures})
          </Button>
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0, 1)}
          >
            Full Song
          </Button>
        </div>
      </div>
    </div>
  );
}
