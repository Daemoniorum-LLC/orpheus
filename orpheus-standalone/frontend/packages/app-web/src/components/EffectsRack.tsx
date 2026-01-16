/**
 * Effects Rack Component
 * Reverb, Delay, Chorus, and other time-based effects
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
  effect: {
    marginBottom: '16px',
    ...shorthands.padding('12px'),
    ...shorthands.borderRadius('6px'),
    backgroundColor: tokens.colorNeutralBackground3,
  },
  effectHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  effectTitle: {
    fontSize: '12px',
    fontWeight: tokens.fontWeightSemibold,
  },
  controls: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    ...shorthands.gap('12px'),
  },
  control: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('6px'),
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

export interface ReverbSettings {
  enabled: boolean;
  roomSize: number; // 0-1
  decay: number; // seconds
  preDelay: number; // ms
  wetDry: number; // 0-100%
}

export interface DelaySettings {
  enabled: boolean;
  time: number; // ms
  feedback: number; // 0-100%
  wetDry: number; // 0-100%
  sync: boolean;
}

export interface EffectsSettings {
  reverb: ReverbSettings;
  delay: DelaySettings;
}

export interface EffectsRackProps {
  settings?: EffectsSettings;
  onSettingsChange?: (settings: EffectsSettings) => void;
}

const DEFAULT_SETTINGS: EffectsSettings = {
  reverb: {
    enabled: false,
    roomSize: 0.5,
    decay: 1.5,
    preDelay: 20,
    wetDry: 30,
  },
  delay: {
    enabled: false,
    time: 250,
    feedback: 40,
    wetDry: 25,
    sync: false,
  },
};

export function EffectsRack({ settings = DEFAULT_SETTINGS, onSettingsChange }: EffectsRackProps) {
  const styles = useStyles();
  const [effects, setEffects] = useState<EffectsSettings>(settings);

  const updateReverb = (updates: Partial<ReverbSettings>) => {
    const newSettings = {
      ...effects,
      reverb: { ...effects.reverb, ...updates },
    };
    setEffects(newSettings);
    onSettingsChange?.(newSettings);
  };

  const updateDelay = (updates: Partial<DelaySettings>) => {
    const newSettings = {
      ...effects,
      delay: { ...effects.delay, ...updates },
    };
    setEffects(newSettings);
    onSettingsChange?.(newSettings);
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>Effects</div>
      </div>

      {/* Reverb */}
      <div className={styles.effect}>
        <div className={styles.effectHeader}>
          <div className={styles.effectTitle}>REVERB</div>
          <Switch
            checked={effects.reverb.enabled}
            onChange={(_, data) => updateReverb({ enabled: data.checked })}
            label={effects.reverb.enabled ? 'ON' : 'OFF'}
          />
        </div>

        <div className={styles.controls}>
          <div className={styles.control}>
            <Label className={styles.label}>ROOM SIZE</Label>
            <Slider
              min={0}
              max={1}
              step={0.01}
              value={effects.reverb.roomSize}
              onChange={(_, data) => updateReverb({ roomSize: data.value })}
              disabled={!effects.reverb.enabled}
            />
            <div className={styles.value}>{(effects.reverb.roomSize * 100).toFixed(0)}%</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>DECAY</Label>
            <Slider
              min={0.1}
              max={10}
              step={0.1}
              value={effects.reverb.decay}
              onChange={(_, data) => updateReverb({ decay: data.value })}
              disabled={!effects.reverb.enabled}
            />
            <div className={styles.value}>{effects.reverb.decay.toFixed(1)}s</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>PRE-DELAY</Label>
            <Slider
              min={0}
              max={100}
              step={1}
              value={effects.reverb.preDelay}
              onChange={(_, data) => updateReverb({ preDelay: data.value })}
              disabled={!effects.reverb.enabled}
            />
            <div className={styles.value}>{effects.reverb.preDelay}ms</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>WET/DRY</Label>
            <Slider
              min={0}
              max={100}
              step={1}
              value={effects.reverb.wetDry}
              onChange={(_, data) => updateReverb({ wetDry: data.value })}
              disabled={!effects.reverb.enabled}
            />
            <div className={styles.value}>{effects.reverb.wetDry}%</div>
          </div>
        </div>
      </div>

      {/* Delay */}
      <div className={styles.effect}>
        <div className={styles.effectHeader}>
          <div className={styles.effectTitle}>DELAY</div>
          <Switch
            checked={effects.delay.enabled}
            onChange={(_, data) => updateDelay({ enabled: data.checked })}
            label={effects.delay.enabled ? 'ON' : 'OFF'}
          />
        </div>

        <div className={styles.controls}>
          <div className={styles.control}>
            <Label className={styles.label}>TIME</Label>
            <Slider
              min={1}
              max={2000}
              step={1}
              value={effects.delay.time}
              onChange={(_, data) => updateDelay({ time: data.value })}
              disabled={!effects.delay.enabled}
            />
            <div className={styles.value}>{effects.delay.time}ms</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>FEEDBACK</Label>
            <Slider
              min={0}
              max={95}
              step={1}
              value={effects.delay.feedback}
              onChange={(_, data) => updateDelay({ feedback: data.value })}
              disabled={!effects.delay.enabled}
            />
            <div className={styles.value}>{effects.delay.feedback}%</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>WET/DRY</Label>
            <Slider
              min={0}
              max={100}
              step={1}
              value={effects.delay.wetDry}
              onChange={(_, data) => updateDelay({ wetDry: data.value })}
              disabled={!effects.delay.enabled}
            />
            <div className={styles.value}>{effects.delay.wetDry}%</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>SYNC</Label>
            <Switch
              checked={effects.delay.sync}
              onChange={(_, data) => updateDelay({ sync: data.checked })}
              disabled={!effects.delay.enabled}
              label={effects.delay.sync ? 'Tempo Sync' : 'Free'}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
