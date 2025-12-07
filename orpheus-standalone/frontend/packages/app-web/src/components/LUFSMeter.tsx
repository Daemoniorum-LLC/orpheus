/**
 * LUFS Meter - Broadcast-standard loudness metering
 * Displays integrated LUFS, short-term LUFS, momentary LUFS, and true peak
 * Uses ITU-R BS.1770 compliant K-weighted measurement
 */

import { makeStyles, shorthands, tokens, Button } from '@fluentui/react-components';
import { ArrowReset24Regular } from '@fluentui/react-icons';
import { useEffect, useState, useCallback } from 'react';
import { getLUFSAnalyzer, type LUFSMeasurement } from '../services/lufs-analyzer';

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
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground1,
  },
  subtitle: {
    fontSize: '10px',
    color: tokens.colorNeutralForeground3,
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
  activeIndicator: {
    width: '8px',
    height: '8px',
    ...shorthands.borderRadius('50%'),
    transition: 'background-color 200ms',
  },
  rangeValue: {
    fontSize: '14px',
    fontWeight: 600,
    fontFamily: 'monospace',
    color: tokens.colorNeutralForeground2,
  },
});

export interface LUFSMeterProps {
  /** Target LUFS for streaming platform */
  target?: number;
  /** Show platform recommendations */
  showTargets?: boolean;
}

const PLATFORM_TARGETS = {
  spotify: -14,
  appleMusic: -16,
  youtube: -13,
  broadcast: -23,
};

export function LUFSMeter({ target = PLATFORM_TARGETS.spotify, showTargets = true }: LUFSMeterProps) {
  const styles = useStyles();
  const [measurement, setMeasurement] = useState<LUFSMeasurement>({
    integrated: -Infinity,
    shortTerm: -Infinity,
    momentary: -Infinity,
    truePeak: -Infinity,
    range: 0,
    active: false,
  });

  // Get analyzer and connect
  useEffect(() => {
    const analyzer = getLUFSAnalyzer();
    analyzer.connect();

    // Update measurements periodically
    const interval = setInterval(() => {
      const m = analyzer.getMeasurement();
      setMeasurement(m);
    }, 100);

    return () => {
      clearInterval(interval);
      analyzer.disconnect();
    };
  }, []);

  // Reset measurements
  const handleReset = useCallback(() => {
    const analyzer = getLUFSAnalyzer();
    analyzer.reset();
    setMeasurement({
      integrated: -Infinity,
      shortTerm: -Infinity,
      momentary: -Infinity,
      truePeak: -Infinity,
      range: 0,
      active: false,
    });
  }, []);

  const getStatus = (value: number, targetValue: number): 'good' | 'warning' | 'bad' => {
    if (!isFinite(value)) return 'bad';
    const diff = Math.abs(value - targetValue);
    if (diff <= 1) return 'good';
    if (diff <= 3) return 'warning';
    return 'bad';
  };

  const getBarWidth = (value: number): number => {
    // Map -70 to 0 LUFS to 0-100%
    if (!isFinite(value) || value < -70) return 0;
    return Math.max(0, Math.min(100, ((value + 70) / 70) * 100));
  };

  const getBarColor = (value: number, targetValue: number): string => {
    const status = getStatus(value, targetValue);
    if (status === 'good') return tokens.colorPaletteGreenBackground3;
    if (status === 'warning') return tokens.colorPaletteYellowBackground3;
    return tokens.colorPaletteRedBackground3;
  };

  const formatLUFS = (value: number): string => {
    if (!isFinite(value) || value < -70) return '-∞';
    return value.toFixed(1);
  };

  const formatRange = (value: number): string => {
    if (!isFinite(value) || value <= 0) return '0.0';
    return value.toFixed(1);
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div>
          <div className={styles.title}>LUFS Metering</div>
          <div className={styles.subtitle}>ITU-R BS.1770 K-weighted</div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div
            className={styles.activeIndicator}
            style={{
              backgroundColor: measurement.active
                ? tokens.colorPaletteGreenBackground3
                : tokens.colorNeutralBackground4,
            }}
            title={measurement.active ? 'Audio active' : 'No audio signal'}
          />
          <Button
            icon={<ArrowReset24Regular />}
            appearance="subtle"
            size="small"
            onClick={handleReset}
            title="Reset integrated measurements"
          >
            Reset
          </Button>
        </div>
      </div>

      <div className={styles.metersContainer}>
        {/* Integrated LUFS */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>Integrated (Program)</div>
          <div>
            <span className={styles.meterValue}>{formatLUFS(measurement.integrated)}</span>
            <span className={styles.meterUnit}>LUFS</span>
          </div>
          <div className={styles.barContainer}>
            <div
              className={styles.bar}
              style={{
                width: `${getBarWidth(measurement.integrated)}%`,
                backgroundColor: getBarColor(measurement.integrated, target),
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
                getStatus(measurement.integrated, target) === 'good' ? styles.statusGood :
                getStatus(measurement.integrated, target) === 'warning' ? styles.statusWarning :
                styles.statusBad
              }`}>
                {!isFinite(measurement.integrated) ? '⏸ No Signal' :
                 getStatus(measurement.integrated, target) === 'good' ? '✓ On Target' :
                 getStatus(measurement.integrated, target) === 'warning' ? '⚠ Close' :
                 measurement.integrated > target ? '↑ Too Loud' : '↓ Too Quiet'}
              </span>
            </>
          )}
        </div>

        {/* Short-term LUFS */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>Short-term (3s)</div>
          <div>
            <span className={styles.meterValue}>{formatLUFS(measurement.shortTerm)}</span>
            <span className={styles.meterUnit}>LUFS</span>
          </div>
          <div className={styles.barContainer}>
            <div
              className={styles.bar}
              style={{
                width: `${getBarWidth(measurement.shortTerm)}%`,
                backgroundColor: getBarColor(measurement.shortTerm, target),
              }}
            />
          </div>
        </div>

        {/* Momentary LUFS */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>Momentary (400ms)</div>
          <div>
            <span className={styles.meterValue}>{formatLUFS(measurement.momentary)}</span>
            <span className={styles.meterUnit}>LUFS</span>
          </div>
          <div className={styles.barContainer}>
            <div
              className={styles.bar}
              style={{
                width: `${getBarWidth(measurement.momentary)}%`,
                backgroundColor: getBarColor(measurement.momentary, target),
              }}
            />
          </div>
        </div>

        {/* True Peak */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>True Peak</div>
          <div>
            <span className={styles.meterValue}>{formatLUFS(measurement.truePeak)}</span>
            <span className={styles.meterUnit}>dBTP</span>
          </div>
          <div className={styles.barContainer}>
            <div
              className={styles.bar}
              style={{
                width: `${getBarWidth(measurement.truePeak)}%`,
                backgroundColor: measurement.truePeak > -1 ? tokens.colorPaletteRedBackground3 : tokens.colorPaletteGreenBackground3,
              }}
            />
          </div>
          <div className={styles.targetIndicator}>
            {!isFinite(measurement.truePeak) ? '⏸ No Signal' :
             measurement.truePeak > -0.3 ? '⚠ Clipping!' :
             measurement.truePeak > -1 ? '⚠ Clipping Risk' : '✓ Safe (-1 dBTP limit)'}
          </div>
        </div>

        {/* Loudness Range (LRA) */}
        <div className={styles.meterCard}>
          <div className={styles.meterLabel}>Loudness Range (LRA)</div>
          <div>
            <span className={styles.meterValue}>{formatRange(measurement.range)}</span>
            <span className={styles.meterUnit}>LU</span>
          </div>
          <div className={styles.targetIndicator}>
            {measurement.range <= 0 ? '⏸ Measuring...' :
             measurement.range < 4 ? '↓ Very Compressed' :
             measurement.range < 8 ? '○ Moderate' :
             measurement.range < 15 ? '✓ Dynamic' : '↑ Very Dynamic'}
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
