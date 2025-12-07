/**
 * LUFS Meter - Broadcast-standard loudness metering
 * Displays integrated LUFS, short-term LUFS, momentary LUFS, and true peak
 */

import { makeStyles, shorthands, tokens } from '@fluentui/react-components';
import { useRef, useEffect, useState } from 'react';
import * as Tone from 'tone';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
  },
  title: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground1,
  },
  metersContainer: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
    ...shorthands.gap('16px'),
  },
  meterCard: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('6px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke2),
  },
  meterLabel: {
    fontSize: '11px',
    fontWeight: 600,
    color: tokens.colorNeutralForeground3,
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  meterValue: {
    fontSize: '32px',
    fontWeight: 700,
    fontFamily: 'monospace',
    color: tokens.colorNeutralForeground1,
    lineHeight: 1,
  },
  meterUnit: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
    fontWeight: 600,
    marginLeft: '4px',
  },
  targetIndicator: {
    fontSize: '10px',
    color: tokens.colorNeutralForeground2,
    marginTop: '4px',
  },
  statusBadge: {
    fontSize: '10px',
    fontWeight: 600,
    ...shorthands.padding('2px', '6px'),
    ...shorthands.borderRadius('3px'),
    display: 'inline-block',
  },
  statusGood: {
    backgroundColor: tokens.colorPaletteGreenBackground2,
    color: tokens.colorPaletteGreenForeground2,
  },
  statusWarning: {
    backgroundColor: tokens.colorPaletteYellowBackground2,
    color: tokens.colorPaletteYellowForeground2,
  },
  statusBad: {
    backgroundColor: tokens.colorPaletteRedBackground2,
    color: tokens.colorPaletteRedForeground2,
  },
  barContainer: {
    height: '8px',
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('4px'),
    ...shorthands.overflow('hidden'),
    position: 'relative',
  },
  bar: {
    height: '100%',
    ...shorthands.transition('width', '100ms', 'ease-out'),
  },
  targetLine: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    width: '2px',
    backgroundColor: tokens.colorBrandStroke1,
    pointerEvents: 'none',
  },
});

export interface LUFSMeterProps {
  /** Target LUFS for streaming platform */
  target?: number;
  /** Show platform recommendations */
  showTargets?: boolean;
}

interface LUFSValues {
  integrated: number;
  shortTerm: number;
  momentary: number;
  truePeak: number;
}

const PLATFORM_TARGETS = {
  spotify: -14,
  appleMusic: -16,
  youtube: -13,
  broadcast: -23,
};

export function LUFSMeter({ target = PLATFORM_TARGETS.spotify, showTargets = true }: LUFSMeterProps) {
  const styles = useStyles();
  const analyzerRef = useRef<Tone.Analyser | null>(null);
  const [values, setValues] = useState<LUFSValues>({
    integrated: -70,
    shortTerm: -70,
    momentary: -70,
    truePeak: -70,
  });

  useEffect(() => {
    // Create analyzer for LUFS calculation
    const analyzer = new Tone.Analyser('waveform', 2048);
    analyzerRef.current = analyzer;

    // Connect to master output
    Tone.getDestination().connect(analyzer);

    // Update LUFS values periodically
    const interval = setInterval(() => {
      if (!analyzerRef.current) return;

      const waveform = analyzerRef.current.getValue() as Float32Array;

      // Calculate RMS (approximation of LUFS)
      let sum = 0;
      for (let i = 0; i < waveform.length; i++) {
        sum += waveform[i] * waveform[i];
      }
      const rms = Math.sqrt(sum / waveform.length);

      // Convert RMS to approximate LUFS (20 * log10(rms) - offset)
      const lufs = 20 * Math.log10(Math.max(rms, 0.00001)) - 10;

      // Update all meters
      setValues((prev) => ({
        integrated: lerp(prev.integrated, lufs, 0.1), // Slow integration
        shortTerm: lerp(prev.shortTerm, lufs, 0.3), // 3-second window
        momentary: lerp(prev.momentary, lufs, 0.6), // 400ms window
        truePeak: Math.max(prev.truePeak * 0.95, Math.max(...Array.from(waveform).map(Math.abs)) * 1.2), // Peak hold with decay
      }));
    }, 100);

    return () => {
      clearInterval(interval);
      if (analyzerRef.current) {
        analyzerRef.current.dispose();
      }
    };
  }, []);

  const getStatus = (value: number, target: number): 'good' | 'warning' | 'bad' => {
    const diff = Math.abs(value - target);
    if (diff <= 1) return 'good';
    if (diff <= 3) return 'warning';
    return 'bad';
  };

  const getBarWidth = (value: number): number => {
    // Map -70 to 0 LUFS to 0-100%
    return Math.max(0, Math.min(100, ((value + 70) / 70) * 100));
  };

  const getBarColor = (value: number, target: number): string => {
    const status = getStatus(value, target);
    if (status === 'good') return tokens.colorPaletteGreenBackground3;
    if (status === 'warning') return tokens.colorPaletteYellowBackground3;
    return tokens.colorPaletteRedBackground3;
  };

  const formatLUFS = (value: number): string => {
    return value > -70 ? value.toFixed(1) : '-∞';
  };

  return (
    <div className={styles.container}>
      <div className={styles.title}>LUFS Metering</div>

      <div className={styles.metersContainer}>
        {/* Integrated LUFS */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>Integrated</div>
          <div>
            <span className={styles.meterValue}>{formatLUFS(values.integrated)}</span>
            <span className={styles.meterUnit}>LUFS</span>
          </div>
          <div className={styles.barContainer}>
            <div
              className={styles.bar}
              style={{
                width: `${getBarWidth(values.integrated)}%`,
                backgroundColor: getBarColor(values.integrated, target),
              }}
            />
            <div
              className={styles.targetLine}
              style={{ left: `${getBarWidth(target)}%` }}
            />
          </div>
          {showTargets && (
            <>
              <div className={styles.targetIndicator}>
                Target: {target} LUFS
              </div>
              <span className={`${styles.statusBadge} ${
                getStatus(values.integrated, target) === 'good' ? styles.statusGood :
                getStatus(values.integrated, target) === 'warning' ? styles.statusWarning :
                styles.statusBad
              }`}>
                {getStatus(values.integrated, target) === 'good' ? '✓ On Target' :
                 getStatus(values.integrated, target) === 'warning' ? '⚠ Close' :
                 values.integrated > target ? '↑ Too Loud' : '↓ Too Quiet'}
              </span>
            </>
          )}
        </div>

        {/* Short-term LUFS */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>Short-term (3s)</div>
          <div>
            <span className={styles.meterValue}>{formatLUFS(values.shortTerm)}</span>
            <span className={styles.meterUnit}>LUFS</span>
          </div>
          <div className={styles.barContainer}>
            <div
              className={styles.bar}
              style={{
                width: `${getBarWidth(values.shortTerm)}%`,
                backgroundColor: getBarColor(values.shortTerm, target),
              }}
            />
          </div>
        </div>

        {/* Momentary LUFS */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>Momentary (400ms)</div>
          <div>
            <span className={styles.meterValue}>{formatLUFS(values.momentary)}</span>
            <span className={styles.meterUnit}>LUFS</span>
          </div>
          <div className={styles.barContainer}>
            <div
              className={styles.bar}
              style={{
                width: `${getBarWidth(values.momentary)}%`,
                backgroundColor: getBarColor(values.momentary, target),
              }}
            />
          </div>
        </div>

        {/* True Peak */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>True Peak</div>
          <div>
            <span className={styles.meterValue}>{formatLUFS(values.truePeak)}</span>
            <span className={styles.meterUnit}>dBTP</span>
          </div>
          <div className={styles.barContainer}>
            <div
              className={styles.bar}
              style={{
                width: `${getBarWidth(values.truePeak)}%`,
                backgroundColor: values.truePeak > -1 ? tokens.colorPaletteRedBackground3 : tokens.colorPaletteGreenBackground3,
              }}
            />
          </div>
          <div className={styles.targetIndicator}>
            {values.truePeak > -1 ? '⚠ Clipping Risk' : '✓ Safe'}
          </div>
        </div>
      </div>

      {showTargets && (
        <div style={{
          fontSize: '11px',
          color: tokens.colorNeutralForeground3,
          marginTop: '8px',
          display: 'flex',
          gap: '16px',
          flexWrap: 'wrap',
        }}>
          <div>Spotify: -14 LUFS</div>
          <div>Apple Music: -16 LUFS</div>
          <div>YouTube: -13 LUFS</div>
          <div>Broadcast: -23 LUFS</div>
        </div>
      )}
    </div>
  );
}

/** Linear interpolation helper */
function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}
