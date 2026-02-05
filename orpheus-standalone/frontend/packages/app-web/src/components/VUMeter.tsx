/**
 * VU Meter Component
 * Professional-grade VU meter with peak hold, stereo display, and dB scale
 */

import { makeStyles, shorthands, tokens } from '@fluentui/react-components';
import { useEffect, useRef, useState } from 'react';
import * as Tone from 'tone';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    ...shorthands.gap('4px'),
  },
  meterWrapper: {
    display: 'flex',
    ...shorthands.gap('2px'),
    alignItems: 'flex-end',
  },
  meterContainer: {
    position: 'relative',
    backgroundColor: 'var(--color-charcoal-900)',
    ...shorthands.borderRadius('2px'),
    ...shorthands.border('1px', 'solid', 'var(--color-charcoal-500)'),
    ...shorthands.overflow('hidden'),
  },
  meterFill: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    transition: 'height 50ms ease-out',
  },
  peakIndicator: {
    position: 'absolute',
    left: 0,
    right: 0,
    height: '2px',
    backgroundColor: tokens.colorPaletteRedForeground1,
    transition: 'bottom 100ms ease-out',
  },
  clipIndicator: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    height: '4px',
    transition: 'background-color 100ms',
  },
  // Colorblind-friendly threshold markers
  thresholdMarker: {
    position: 'absolute',
    left: 0,
    right: 0,
    height: '1px',
    backgroundColor: tokens.colorNeutralForeground3,
    pointerEvents: 'none',
    zIndex: 1,
  },
  thresholdTick: {
    position: 'absolute',
    right: '-3px',
    width: '6px',
    height: '3px',
    backgroundColor: tokens.colorNeutralForeground2,
    ...shorthands.borderRadius('1px'),
    transform: 'translateY(-50%)',
  },
  scaleContainer: {
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'space-between',
    fontSize: '8px',
    color: 'var(--color-charcoal-300)',
    fontFamily: 'var(--font-mono)',
    height: '100%',
    marginRight: '4px',
  },
  label: {
    fontSize: '10px',
    color: tokens.colorNeutralForeground2,
    fontWeight: 500,
    fontFamily: 'var(--font-display)',
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
  },
  dbValue: {
    fontSize: '9px',
    fontFamily: 'var(--font-mono)',
    color: 'var(--color-charcoal-300)',
    textAlign: 'center',
    minWidth: '36px',
  },
});

interface VUMeterProps {
  /** Audio node to measure */
  audioNode?: Tone.ToneAudioNode;
  /** Manual level input (0-1) for left channel */
  levelL?: number;
  /** Manual level input (0-1) for right channel */
  levelR?: number;
  /** Width of each meter bar */
  width?: number;
  /** Height of the meter */
  height?: number;
  /** Show stereo (two bars) or mono (one bar) */
  stereo?: boolean;
  /** Show dB scale on the side */
  showScale?: boolean;
  /** Show peak hold indicator */
  showPeakHold?: boolean;
  /** Peak hold time in ms */
  peakHoldTime?: number;
  /** Label to show below */
  label?: string;
  /** Show dB value below */
  showDbValue?: boolean;
  /** Show colorblind-friendly threshold markers at -12dB, -6dB, -3dB */
  showThresholdMarkers?: boolean;
}

/**
 * Convert linear amplitude (0-1) to dB
 */
function linearToDb(value: number): number {
  if (value <= 0) return -Infinity;
  return 20 * Math.log10(value);
}

/**
 * Get color for meter level (using phthalo green for safe levels)
 */
function getMeterColor(db: number): string {
  if (db > -3) return '#4a1e1e';   // Dark red for danger
  if (db > -6) return '#4a3e18';   // Dark amber for warning
  if (db > -12) return '#3d4a18'; // Dark yellow-green for caution
  return '#1a4a2c';                // Phthalo green for safe
}

/**
 * Calculate meter height percentage from dB value
 * Maps -60dB to 0dB to 0-100%
 */
function dbToHeight(db: number, minDb: number = -60, maxDb: number = 0): number {
  if (db <= minDb) return 0;
  if (db >= maxDb) return 100;
  return ((db - minDb) / (maxDb - minDb)) * 100;
}

export function VUMeter({
  audioNode,
  levelL = 0,
  levelR,
  width = 8,
  height = 100,
  stereo = true,
  showScale = false,
  showPeakHold = true,
  peakHoldTime = 1500,
  label,
  showDbValue = false,
  showThresholdMarkers = true, // Enabled by default for accessibility
}: VUMeterProps) {
  const styles = useStyles();
  const [levels, setLevels] = useState({ left: 0, right: 0 });
  const [peaks, setPeaks] = useState({ left: -Infinity, right: -Infinity });
  const [clipping, setClipping] = useState({ left: false, right: false });
  const meterRef = useRef<Tone.Meter | null>(null);
  const splitRef = useRef<Tone.Split | null>(null);
  const meterLRef = useRef<Tone.Meter | null>(null);
  const meterRRef = useRef<Tone.Meter | null>(null);
  const animationRef = useRef<number | null>(null);
  const idleTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isIdleRef = useRef(false);
  const idleFrameCountRef = useRef(0);
  const peakTimersRef = useRef<{ left: NodeJS.Timeout | null; right: NodeJS.Timeout | null }>({
    left: null,
    right: null,
  });

  // Idle detection constants
  const IDLE_THRESHOLD_DB = -60; // Consider idle when below -60dB
  const IDLE_FRAME_COUNT = 120; // ~2 seconds at 60fps before going idle
  const IDLE_POLL_INTERVAL = 500; // Poll every 500ms when idle

  // Set up audio metering
  useEffect(() => {
    if (audioNode) {
      try {
        // Create meters for stereo analysis
        if (stereo) {
          splitRef.current = new Tone.Split();
          meterLRef.current = new Tone.Meter({ smoothing: 0.8 });
          meterRRef.current = new Tone.Meter({ smoothing: 0.8 });

          audioNode.connect(splitRef.current);
          splitRef.current.left.connect(meterLRef.current);
          splitRef.current.right.connect(meterRRef.current);
        } else {
          meterRef.current = new Tone.Meter({ smoothing: 0.8 });
          audioNode.connect(meterRef.current);
        }
      } catch (err) {
        console.error('[VUMeter] Failed to connect audio node:', err);
        return;
      }

      // Schedule next update based on idle state
      const scheduleNextUpdate = (updateFn: () => void) => {
        if (isIdleRef.current) {
          // In idle mode, use slower polling
          idleTimeoutRef.current = setTimeout(() => {
            animationRef.current = requestAnimationFrame(updateFn);
          }, IDLE_POLL_INTERVAL);
        } else {
          // Active mode, use full frame rate
          animationRef.current = requestAnimationFrame(updateFn);
        }
      };

      // Start animation loop
      const updateMeters = () => {
        try {
          let leftDb = -Infinity;
          let rightDb = -Infinity;

          if (stereo && meterLRef.current && meterRRef.current) {
            leftDb = meterLRef.current.getValue() as number;
            rightDb = meterRRef.current.getValue() as number;
          } else if (meterRef.current) {
            leftDb = meterRef.current.getValue() as number;
            rightDb = leftDb;
          }

          // Validate values (protect against NaN/undefined)
          if (!isFinite(leftDb)) leftDb = -Infinity;
          if (!isFinite(rightDb)) rightDb = -Infinity;

          // Idle detection: check if audio is below threshold
          const isCurrentlyIdle = leftDb < IDLE_THRESHOLD_DB && rightDb < IDLE_THRESHOLD_DB;

          if (isCurrentlyIdle) {
            idleFrameCountRef.current++;
            if (idleFrameCountRef.current >= IDLE_FRAME_COUNT && !isIdleRef.current) {
              // Enter idle mode
              isIdleRef.current = true;
            }
          } else {
            // Audio detected, wake up from idle
            idleFrameCountRef.current = 0;
            if (isIdleRef.current) {
              isIdleRef.current = false;
              // Clear any pending idle timeout
              if (idleTimeoutRef.current) {
                clearTimeout(idleTimeoutRef.current);
                idleTimeoutRef.current = null;
              }
            }
          }

          // Update levels
          setLevels({ left: leftDb, right: rightDb });

          // Update peaks
          setPeaks((prev) => {
            const newPeaks = { ...prev };

            if (leftDb > prev.left) {
              newPeaks.left = leftDb;
              // Reset peak timer
              if (peakTimersRef.current.left) {
                clearTimeout(peakTimersRef.current.left);
              }
              peakTimersRef.current.left = setTimeout(() => {
                setPeaks((p) => ({ ...p, left: -Infinity }));
              }, peakHoldTime);
            }

            if (rightDb > prev.right) {
              newPeaks.right = rightDb;
              if (peakTimersRef.current.right) {
                clearTimeout(peakTimersRef.current.right);
              }
              peakTimersRef.current.right = setTimeout(() => {
                setPeaks((p) => ({ ...p, right: -Infinity }));
              }, peakHoldTime);
            }

            return newPeaks;
          });

          // Check for clipping
          setClipping({
            left: leftDb > -0.1,
            right: rightDb > -0.1,
          });
        } catch (err) {
          // Silently ignore metering errors (e.g., disposed nodes)
        }

        scheduleNextUpdate(updateMeters);
      };

      animationRef.current = requestAnimationFrame(updateMeters);

      return () => {
        if (animationRef.current) {
          cancelAnimationFrame(animationRef.current);
        }
        if (idleTimeoutRef.current) {
          clearTimeout(idleTimeoutRef.current);
        }
        if (peakTimersRef.current.left) {
          clearTimeout(peakTimersRef.current.left);
        }
        if (peakTimersRef.current.right) {
          clearTimeout(peakTimersRef.current.right);
        }
        // Reset idle state
        isIdleRef.current = false;
        idleFrameCountRef.current = 0;
        try {
          meterRef.current?.dispose();
          meterLRef.current?.dispose();
          meterRRef.current?.dispose();
          splitRef.current?.dispose();
        } catch (err) {
          // Ignore disposal errors
        }
      };
    }
  }, [audioNode, stereo, peakHoldTime]);

  // Use manual levels if no audio node
  useEffect(() => {
    if (!audioNode) {
      const leftDb = linearToDb(levelL);
      const rightDb = linearToDb(levelR ?? levelL);
      setLevels({ left: leftDb, right: rightDb });
    }
  }, [audioNode, levelL, levelR]);

  // Threshold levels for colorblind-friendly markers
  const THRESHOLD_LEVELS = [
    { db: -12, label: 'Safe' },   // Green to Yellow transition
    { db: -6, label: 'Caution' }, // Yellow to Orange transition
    { db: -3, label: 'Warning' }, // Orange to Red transition
  ];

  const renderMeter = (level: number, peak: number, isClipping: boolean, key: string) => {
    const heightPercent = dbToHeight(level);
    const peakHeightPercent = dbToHeight(peak);
    const color = getMeterColor(level);

    return (
      <div
        key={key}
        className={styles.meterContainer}
        style={{ width: `${width}px`, height: `${height}px` }}
        role="meter"
        aria-valuenow={Math.round(level)}
        aria-valuemin={-60}
        aria-valuemax={0}
        aria-label={`Audio level: ${level > -60 ? `${level.toFixed(1)} dB` : 'silent'}${isClipping ? ', clipping!' : ''}`}
      >
        {/* Clip indicator */}
        <div
          className={styles.clipIndicator}
          style={{
            backgroundColor: isClipping ? tokens.colorPaletteRedBackground3 : 'transparent',
          }}
        />

        {/* Colorblind-friendly threshold markers */}
        {showThresholdMarkers && THRESHOLD_LEVELS.map(({ db, label }) => {
          const markerHeight = dbToHeight(db);
          return (
            <div
              key={`threshold-${db}`}
              className={styles.thresholdMarker}
              style={{ bottom: `${markerHeight}%` }}
              title={`${db} dB (${label})`}
            >
              <div className={styles.thresholdTick} />
            </div>
          );
        })}

        {/* Main meter fill */}
        <div
          className={styles.meterFill}
          style={{
            height: `${heightPercent}%`,
            background: 'linear-gradient(to top, #1a4a2c 0%, #1e5e38 50%, #3d4a18 70%, #4a3e18 85%, #4a1e1e 100%)',
          }}
        />

        {/* Peak hold indicator */}
        {showPeakHold && peak > -60 && (
          <div
            className={styles.peakIndicator}
            style={{ bottom: `${peakHeightPercent}%` }}
          />
        )}
      </div>
    );
  };

  const leftDb = levels.left;
  const rightDb = stereo ? levels.right : levels.left;
  const displayDb = stereo ? Math.max(leftDb, rightDb) : leftDb;

  return (
    <div className={styles.container}>
      <div className={styles.meterWrapper}>
        {showScale && (
          <div className={styles.scaleContainer} style={{ height: `${height}px`, position: 'relative' }}>
            {/* Labels positioned at actual dB values */}
            <span style={{ position: 'absolute', top: `${100 - dbToHeight(0)}%`, transform: 'translateY(-50%)' }}>0</span>
            <span style={{ position: 'absolute', top: `${100 - dbToHeight(-6)}%`, transform: 'translateY(-50%)' }}>-6</span>
            <span style={{ position: 'absolute', top: `${100 - dbToHeight(-12)}%`, transform: 'translateY(-50%)' }}>-12</span>
            <span style={{ position: 'absolute', top: `${100 - dbToHeight(-24)}%`, transform: 'translateY(-50%)' }}>-24</span>
            <span style={{ position: 'absolute', top: `${100 - dbToHeight(-48)}%`, transform: 'translateY(-50%)' }}>-48</span>
          </div>
        )}
        {renderMeter(leftDb, peaks.left, clipping.left, 'left')}
        {stereo && renderMeter(rightDb, peaks.right, clipping.right, 'right')}
      </div>
      {label && <div className={styles.label}>{label}</div>}
      {showDbValue && (
        <div className={styles.dbValue}>
          {displayDb > -60 ? `${displayDb.toFixed(1)} dB` : '-∞'}
        </div>
      )}
    </div>
  );
}

/**
 * Create a test tone generator for meter testing
 */
export function createTestTone(frequency: number = 440): Tone.Oscillator {
  const osc = new Tone.Oscillator(frequency, 'sine');
  osc.volume.value = -12;
  return osc;
}
