/**
 * Compressor Component
 * Professional dynamics compressor with threshold, ratio, attack, release, and makeup gain
 */

import { makeStyles, shorthands, tokens, Slider, Label, Switch } from '@fluentui/react-components';
import { useState } from 'react';

const useStyles = makeStyles({
  container: {
    ...shorthands.padding('16px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    backgroundColor: tokens.colorNeutralBackground2,
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
  },
  title: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
  },
  controls: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    ...shorthands.gap('16px'),
  },
  control: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  label: {
    fontSize: '11px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground2,
  },
  value: {
    fontSize: '12px',
    fontFamily: 'monospace',
    textAlign: 'center',
    color: tokens.colorBrandForeground1,
    marginTop: '4px',
  },
  meter: {
    marginTop: '16px',
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('12px'),
  },
  meterLabel: {
    fontSize: '10px',
    color: tokens.colorNeutralForeground2,
    minWidth: '30px',
  },
  meterBar: {
    flex: 1,
    height: '20px',
    backgroundColor: tokens.colorNeutralBackground5,
    ...shorthands.borderRadius('4px'),
    position: 'relative',
    ...shorthands.overflow('hidden'),
  },
  meterFill: {
    height: '100%',
    backgroundColor: tokens.colorPaletteGreenBackground3,
    transition: 'width 0.1s ease-out',
  },
});

export interface CompressorSettings {
  enabled: boolean;
  threshold: number; // dB
  ratio: number; // ratio (1:1 to 20:1)
  attack: number; // ms
  release: number; // ms
  knee: number; // dB
  makeupGain: number; // dB
}

export interface CompressorProps {
  settings?: CompressorSettings;
  onSettingsChange?: (settings: CompressorSettings) => void;
}

const DEFAULT_SETTINGS: CompressorSettings = {
  enabled: false,
  threshold: -20,
  ratio: 4,
  attack: 10,
  release: 100,
  knee: 2,
  makeupGain: 0,
};

export function Compressor({ settings = DEFAULT_SETTINGS, onSettingsChange }: CompressorProps) {
  const styles = useStyles();
  const [compressor, setCompressor] = useState<CompressorSettings>(settings);
  const [gainReduction, setGainReduction] = useState(0);

  // Simulate gain reduction meter
  useState(() => {
    const interval = setInterval(() => {
      if (compressor.enabled) {
        setGainReduction(Math.random() * Math.min(compressor.threshold / -2, 12));
      } else {
        setGainReduction(0);
      }
    }, 100);
    return () => clearInterval(interval);
  });

  const updateSetting = <K extends keyof CompressorSettings>(
    key: K,
    value: CompressorSettings[K]
  ) => {
    const newSettings = { ...compressor, [key]: value };
    setCompressor(newSettings);
    onSettingsChange?.(newSettings);
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>Compressor</div>
        <Switch
          checked={compressor.enabled}
          onChange={(_, data) => updateSetting('enabled', data.checked)}
          label={compressor.enabled ? 'ON' : 'OFF'}
        />
      </div>

      <div className={styles.controls}>
        {/* Threshold */}
        <div className={styles.control}>
          <Label className={styles.label}>THRESHOLD</Label>
          <Slider
            min={-60}
            max={0}
            step={1}
            value={compressor.threshold}
            onChange={(_, data) => updateSetting('threshold', data.value)}
            disabled={!compressor.enabled}
          />
          <div className={styles.value}>{compressor.threshold} dB</div>
        </div>

        {/* Ratio */}
        <div className={styles.control}>
          <Label className={styles.label}>RATIO</Label>
          <Slider
            min={1}
            max={20}
            step={0.5}
            value={compressor.ratio}
            onChange={(_, data) => updateSetting('ratio', data.value)}
            disabled={!compressor.enabled}
          />
          <div className={styles.value}>{compressor.ratio.toFixed(1)}:1</div>
        </div>

        {/* Attack */}
        <div className={styles.control}>
          <Label className={styles.label}>ATTACK</Label>
          <Slider
            min={0.1}
            max={100}
            step={0.1}
            value={compressor.attack}
            onChange={(_, data) => updateSetting('attack', data.value)}
            disabled={!compressor.enabled}
          />
          <div className={styles.value}>{compressor.attack.toFixed(1)} ms</div>
        </div>

        {/* Release */}
        <div className={styles.control}>
          <Label className={styles.label}>RELEASE</Label>
          <Slider
            min={10}
            max={1000}
            step={10}
            value={compressor.release}
            onChange={(_, data) => updateSetting('release', data.value)}
            disabled={!compressor.enabled}
          />
          <div className={styles.value}>{compressor.release} ms</div>
        </div>

        {/* Knee */}
        <div className={styles.control}>
          <Label className={styles.label}>KNEE</Label>
          <Slider
            min={0}
            max={12}
            step={1}
            value={compressor.knee}
            onChange={(_, data) => updateSetting('knee', data.value)}
            disabled={!compressor.enabled}
          />
          <div className={styles.value}>{compressor.knee} dB</div>
        </div>

        {/* Makeup Gain */}
        <div className={styles.control}>
          <Label className={styles.label}>MAKEUP</Label>
          <Slider
            min={0}
            max={24}
            step={0.5}
            value={compressor.makeupGain}
            onChange={(_, data) => updateSetting('makeupGain', data.value)}
            disabled={!compressor.enabled}
          />
          <div className={styles.value}>+{compressor.makeupGain.toFixed(1)} dB</div>
        </div>
      </div>

      {/* Gain Reduction Meter */}
      <div className={styles.meter}>
        <div className={styles.meterLabel}>GR</div>
        <div className={styles.meterBar}>
          <div
            className={styles.meterFill}
            style={{
              width: `${(gainReduction / 12) * 100}%`,
              backgroundColor: compressor.enabled
                ? tokens.colorPaletteRedBackground3
                : tokens.colorNeutralBackground4,
            }}
          />
        </div>
        <div className={styles.meterLabel}>-{gainReduction.toFixed(1)} dB</div>
      </div>
    </div>
  );
}
