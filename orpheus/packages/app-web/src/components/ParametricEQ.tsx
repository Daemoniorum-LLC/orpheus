/**
 * Parametric EQ Component
 * 4-band parametric equalizer with frequency, gain, and Q controls
 */

import { makeStyles, shorthands, tokens, Slider, Label } from '@fluentui/react-components';
import { useState } from 'react';

const useStyles = makeStyles({
  container: {
    ...shorthands.padding('16px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    backgroundColor: tokens.colorNeutralBackground2,
  },
  title: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '12px',
  },
  bands: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    ...shorthands.gap('12px'),
  },
  band: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  bandTitle: {
    fontSize: '11px',
    fontWeight: tokens.fontWeightSemibold,
    textAlign: 'center',
    color: tokens.colorNeutralForeground2,
  },
  control: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('4px'),
  },
  label: {
    fontSize: '10px',
    color: tokens.colorNeutralForeground2,
  },
  value: {
    fontSize: '10px',
    fontFamily: 'monospace',
    textAlign: 'center',
    color: tokens.colorBrandForeground1,
  },
});

export interface EQBand {
  frequency: number;
  gain: number;
  q: number;
}

export interface ParametricEQProps {
  bands?: EQBand[];
  onBandsChange?: (bands: EQBand[]) => void;
}

const DEFAULT_BANDS: EQBand[] = [
  { frequency: 100, gain: 0, q: 1.0 },   // Low
  { frequency: 400, gain: 0, q: 1.0 },   // Low-Mid
  { frequency: 2000, gain: 0, q: 1.0 },  // Mid
  { frequency: 8000, gain: 0, q: 1.0 },  // High
];

export function ParametricEQ({ bands = DEFAULT_BANDS, onBandsChange }: ParametricEQProps) {
  const styles = useStyles();
  const [eqBands, setEQBands] = useState<EQBand[]>(bands);

  const updateBand = (index: number, updates: Partial<EQBand>) => {
    const newBands = [...eqBands];
    newBands[index] = { ...newBands[index], ...updates };
    setEQBands(newBands);
    onBandsChange?.(newBands);
  };

  const formatFreq = (freq: number): string => {
    return freq >= 1000 ? `${(freq / 1000).toFixed(1)}k` : `${freq}`;
  };

  const getBandName = (index: number): string => {
    const names = ['LOW', 'LOW-MID', 'MID', 'HIGH'];
    return names[index] || `BAND ${index + 1}`;
  };

  return (
    <div className={styles.container}>
      <div className={styles.title}>Parametric EQ</div>
      <div className={styles.bands}>
        {eqBands.map((band, index) => (
          <div key={index} className={styles.band}>
            <div className={styles.bandTitle}>{getBandName(index)}</div>

            {/* Frequency */}
            <div className={styles.control}>
              <Label className={styles.label}>FREQ</Label>
              <Slider
                min={20}
                max={20000}
                step={10}
                value={band.frequency}
                onChange={(_, data) => updateBand(index, { frequency: data.value })}
              />
              <div className={styles.value}>{formatFreq(band.frequency)}Hz</div>
            </div>

            {/* Gain */}
            <div className={styles.control}>
              <Label className={styles.label}>GAIN</Label>
              <Slider
                min={-12}
                max={12}
                step={0.5}
                value={band.gain}
                onChange={(_, data) => updateBand(index, { gain: data.value })}
              />
              <div className={styles.value}>{band.gain > 0 ? '+' : ''}{band.gain.toFixed(1)} dB</div>
            </div>

            {/* Q (Bandwidth) */}
            <div className={styles.control}>
              <Label className={styles.label}>Q</Label>
              <Slider
                min={0.1}
                max={10}
                step={0.1}
                value={band.q}
                onChange={(_, data) => updateBand(index, { q: data.value })}
              />
              <div className={styles.value}>{band.q.toFixed(1)}</div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
