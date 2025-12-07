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
    backgroundColor: 'var(--color-charcoal-700)',
    ...shorthands.borderRadius('6px'),
    ...shorthands.border('1px', 'solid', 'var(--color-charcoal-500)'),
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    fontFamily: 'var(--font-display)',
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    color: 'var(--color-phthalo-highlight)',
  },
  timeline: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  timelineBar: {
    height: '40px',
    backgroundColor: 'var(--color-charcoal-850)',
    ...shorthands.borderRadius('4px'),
    ...shorthands.border('1px', 'solid', 'var(--color-charcoal-500)'),
    position: 'relative',
    cursor: 'pointer',
    ...shorthands.overflow('hidden'),
  },
  loopRegion: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    backgroundColor: 'var(--color-phthalo-base)',
    opacity: 0.4,
    cursor: 'ew-resize',
  },
  loopHandle: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    width: '8px',
    backgroundColor: 'var(--color-phthalo-medium)',
    cursor: 'ew-resize',
    ...shorthands.transition('background-color', '100ms'),
    ':hover': {
      backgroundColor: 'var(--color-phthalo-highlight)',
    },
  },
  playhead: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    width: '2px',
    backgroundColor: '#e06060',  // Muted red for visibility
    pointerEvents: 'none',
    zIndex: 2,
    boxShadow: '0 0 4px rgba(224, 96, 96, 0.5)',
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
    whiteSpace: 'nowrap',
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
    backgroundColor: 'var(--color-charcoal-800)',
    ...shorthands.border('1px', 'solid', 'var(--color-charcoal-600)'),
    boxShadow: '0 2px 6px rgba(0, 0, 0, 0.25)',
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

  // Track previous position to detect loop reset
  const prevPositionRef = useRef(position);

  // Count loops when position jumps back (indicating a loop)
  useEffect(() => {
    if (loopEnabled && isPlaying) {
      const prevPos = prevPositionRef.current;
      // Detect when position jumps backwards significantly (loop occurred)
      if (prevPos > localLoopEnd - 0.05 && position < localLoopStart + 0.05) {
        setLoopCount((c) => c + 1);
      }
    }
    prevPositionRef.current = position;
  }, [position, loopEnabled, isPlaying, localLoopStart, localLoopEnd]);

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

  // Track initial drag position for region dragging
  const dragStartRef = useRef<{ pos: number; loopStart: number; loopEnd: number } | null>(null);

  const handleDragStart = useCallback((e: React.MouseEvent, type: 'start' | 'end' | 'region') => {
    e.stopPropagation();
    setDragging(type);

    // Store initial state for region drag
    if (type === 'region') {
      const initialPos = getPositionFromEvent(e);
      dragStartRef.current = {
        pos: initialPos,
        loopStart: localLoopStart,
        loopEnd: localLoopEnd,
      };
    }

    const handleMove = (moveEvent: MouseEvent) => {
      const pos = getPositionFromEvent(moveEvent);

      if (type === 'start') {
        const newStart = Math.max(0, Math.min(pos, localLoopEnd - 0.05));
        setLocalLoopStart(newStart);
      } else if (type === 'end') {
        const newEnd = Math.min(1, Math.max(pos, localLoopStart + 0.05));
        setLocalLoopEnd(newEnd);
      } else if (type === 'region' && dragStartRef.current) {
        // Move entire region while maintaining size
        const delta = pos - dragStartRef.current.pos;
        const regionSize = dragStartRef.current.loopEnd - dragStartRef.current.loopStart;
        let newStart = dragStartRef.current.loopStart + delta;
        let newEnd = dragStartRef.current.loopEnd + delta;

        // Clamp to valid range
        if (newStart < 0) {
          newStart = 0;
          newEnd = regionSize;
        }
        if (newEnd > 1) {
          newEnd = 1;
          newStart = 1 - regionSize;
        }

        setLocalLoopStart(newStart);
        setLocalLoopEnd(newEnd);
      }
    };

    const handleUp = () => {
      setDragging(null);
      dragStartRef.current = null;
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

  // Determine which measures should show labels to avoid overlap
  const getMeasureLabelInterval = useCallback((): number => {
    if (measures <= 16) return 1; // Show all labels
    if (measures <= 32) return 2; // Show every 2nd measure
    if (measures <= 64) return 4; // Show every 4th measure
    if (measures <= 128) return 8; // Show every 8th measure
    return 16; // Show every 16th measure
  }, [measures]);

  const shouldShowMeasureLabel = useCallback((measureNum: number): boolean => {
    const interval = getMeasureLabelInterval();
    // Always show measure 1, and then at intervals
    return measureNum === 1 || measureNum % interval === 0;
  }, [getMeasureLabelInterval]);

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
                {shouldShowMeasureLabel(i + 1) && (
                  <span className={styles.measureLabel}>{i + 1}</span>
                )}
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

      {/* Preset sections - measure-based for musical relevance */}
      <div>
        <div className={styles.infoLabel} style={{ marginBottom: '8px' }}>
          Quick Select Section
        </div>
        <div className={styles.presets}>
          {/* Common practice section lengths */}
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0, Math.min(4 / measures, 1))}
          >
            First 4 bars
          </Button>
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0, Math.min(8 / measures, 1))}
          >
            First 8 bars
          </Button>
          {measures >= 16 && (
            <Button
              className={styles.presetButton}
              appearance="subtle"
              onClick={() => handlePreset(0, Math.min(16 / measures, 1))}
            >
              First 16 bars
            </Button>
          )}
          {/* Half sections */}
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0, 0.5)}
          >
            First half (M1-{Math.ceil(measures / 2)})
          </Button>
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(0.5, 1)}
          >
            Second half (M{Math.ceil(measures / 2) + 1}-{measures})
          </Button>
          {/* Last N bars - for practicing endings */}
          <Button
            className={styles.presetButton}
            appearance="subtle"
            onClick={() => handlePreset(Math.max(0, 1 - 8 / measures), 1)}
          >
            Last 8 bars
          </Button>
          {/* Full song */}
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
