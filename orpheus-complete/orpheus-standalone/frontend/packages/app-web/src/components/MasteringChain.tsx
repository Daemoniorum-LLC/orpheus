/**
 * Mastering Chain Component
 * Professional mastering processor chain: EQ → Compression → Stereo → Limiter
 */

import { makeStyles, shorthands, tokens, Slider, Label, Switch } from '@fluentui/react-components';
import { useState } from 'react';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
  title: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '8px',
  },
  processor: {
    ...shorthands.padding('16px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    backgroundColor: tokens.colorNeutralBackground2,
  },
  processorHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  processorTitle: {
    fontSize: '14px',
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

export interface MasteringChainSettings {
  eq: {
    enabled: boolean;
    lowCut: number;
    lowShelf: number;
    presence: number;
    airBand: number;
  };
  compressor: {
    enabled: boolean;
    threshold: number;
    ratio: number;
    attack: number;
    release: number;
  };
  stereo: {
    enabled: boolean;
    width: number;
    balance: number;
  };
  limiter: {
    enabled: boolean;
    ceiling: number;
    release: number;
  };
}

export interface MasteringChainProps {
  settings?: MasteringChainSettings;
  onSettingsChange?: (settings: MasteringChainSettings) => void;
}

const DEFAULT_SETTINGS: MasteringChainSettings = {
  eq: {
    enabled: false,
    lowCut: 30,
    lowShelf: 0,
    presence: 0,
    airBand: 0,
  },
  compressor: {
    enabled: false,
    threshold: -6,
    ratio: 1.5,
    attack: 30,
    release: 300,
  },
  stereo: {
    enabled: false,
    width: 100,
    balance: 0,
  },
  limiter: {
    enabled: true,
    ceiling: -0.3,
    release: 50,
  },
};

export function MasteringChain({
  settings = DEFAULT_SETTINGS,
  onSettingsChange,
}: MasteringChainProps) {
  const styles = useStyles();
  const [chain, setChain] = useState<MasteringChainSettings>(settings);

  const updateProcessor = <K extends keyof MasteringChainSettings>(
    processor: K,
    updates: Partial<MasteringChainSettings[K]>
  ) => {
    const newSettings = {
      ...chain,
      [processor]: { ...chain[processor], ...updates },
    };
    setChain(newSettings);
    onSettingsChange?.(newSettings);
  };

  return (
    <div className={styles.container}>
      <div className={styles.title}>Mastering Chain</div>

      {/* Master EQ */}
      <div className={styles.processor}>
        <div className={styles.processorHeader}>
          <div className={styles.processorTitle}>1. Master EQ</div>
          <Switch
            checked={chain.eq.enabled}
            onChange={(_, data) => updateProcessor('eq', { enabled: data.checked })}
            label={chain.eq.enabled ? 'ON' : 'OFF'}
          />
        </div>

        <div className={styles.controls}>
          <div className={styles.control}>
            <Label className={styles.label}>LOW CUT</Label>
            <Slider
              min={20}
              max={120}
              step={1}
              value={chain.eq.lowCut}
              onChange={(_, data) => updateProcessor('eq', { lowCut: data.value })}
              disabled={!chain.eq.enabled}
            />
            <div className={styles.value}>{chain.eq.lowCut}Hz</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>LOW SHELF (80Hz)</Label>
            <Slider
              min={-6}
              max={6}
              step={0.5}
              value={chain.eq.lowShelf}
              onChange={(_, data) => updateProcessor('eq', { lowShelf: data.value })}
              disabled={!chain.eq.enabled}
            />
            <div className={styles.value}>{chain.eq.lowShelf > 0 ? '+' : ''}{chain.eq.lowShelf.toFixed(1)} dB</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>PRESENCE (3kHz)</Label>
            <Slider
              min={-6}
              max={6}
              step={0.5}
              value={chain.eq.presence}
              onChange={(_, data) => updateProcessor('eq', { presence: data.value })}
              disabled={!chain.eq.enabled}
            />
            <div className={styles.value}>{chain.eq.presence > 0 ? '+' : ''}{chain.eq.presence.toFixed(1)} dB</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>AIR BAND (12kHz)</Label>
            <Slider
              min={-6}
              max={6}
              step={0.5}
              value={chain.eq.airBand}
              onChange={(_, data) => updateProcessor('eq', { airBand: data.value })}
              disabled={!chain.eq.enabled}
            />
            <div className={styles.value}>{chain.eq.airBand > 0 ? '+' : ''}{chain.eq.airBand.toFixed(1)} dB</div>
          </div>
        </div>
      </div>

      {/* Glue Compressor */}
      <div className={styles.processor}>
        <div className={styles.processorHeader}>
          <div className={styles.processorTitle}>2. Glue Compressor</div>
          <Switch
            checked={chain.compressor.enabled}
            onChange={(_, data) => updateProcessor('compressor', { enabled: data.checked })}
            label={chain.compressor.enabled ? 'ON' : 'OFF'}
          />
        </div>

        <div className={styles.controls}>
          <div className={styles.control}>
            <Label className={styles.label}>THRESHOLD</Label>
            <Slider
              min={-24}
              max={0}
              step={0.5}
              value={chain.compressor.threshold}
              onChange={(_, data) => updateProcessor('compressor', { threshold: data.value })}
              disabled={!chain.compressor.enabled}
            />
            <div className={styles.value}>{chain.compressor.threshold} dB</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>RATIO</Label>
            <Slider
              min={1}
              max={4}
              step={0.1}
              value={chain.compressor.ratio}
              onChange={(_, data) => updateProcessor('compressor', { ratio: data.value })}
              disabled={!chain.compressor.enabled}
            />
            <div className={styles.value}>{chain.compressor.ratio.toFixed(1)}:1</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>ATTACK</Label>
            <Slider
              min={1}
              max={100}
              step={1}
              value={chain.compressor.attack}
              onChange={(_, data) => updateProcessor('compressor', { attack: data.value })}
              disabled={!chain.compressor.enabled}
            />
            <div className={styles.value}>{chain.compressor.attack}ms</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>RELEASE</Label>
            <Slider
              min={50}
              max={1000}
              step={10}
              value={chain.compressor.release}
              onChange={(_, data) => updateProcessor('compressor', { release: data.value })}
              disabled={!chain.compressor.enabled}
            />
            <div className={styles.value}>{chain.compressor.release}ms</div>
          </div>
        </div>
      </div>

      {/* Stereo Imager */}
      <div className={styles.processor}>
        <div className={styles.processorHeader}>
          <div className={styles.processorTitle}>3. Stereo Imager</div>
          <Switch
            checked={chain.stereo.enabled}
            onChange={(_, data) => updateProcessor('stereo', { enabled: data.checked })}
            label={chain.stereo.enabled ? 'ON' : 'OFF'}
          />
        </div>

        <div className={styles.controls}>
          <div className={styles.control}>
            <Label className={styles.label}>WIDTH</Label>
            <Slider
              min={0}
              max={200}
              step={1}
              value={chain.stereo.width}
              onChange={(_, data) => updateProcessor('stereo', { width: data.value })}
              disabled={!chain.stereo.enabled}
            />
            <div className={styles.value}>{chain.stereo.width}%</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>BALANCE</Label>
            <Slider
              min={-50}
              max={50}
              step={1}
              value={chain.stereo.balance}
              onChange={(_, data) => updateProcessor('stereo', { balance: data.value })}
              disabled={!chain.stereo.enabled}
            />
            <div className={styles.value}>
              {chain.stereo.balance === 0 ? 'C' : chain.stereo.balance > 0 ? `R${chain.stereo.balance}` : `L${Math.abs(chain.stereo.balance)}`}
            </div>
          </div>
        </div>
      </div>

      {/* Limiter */}
      <div className={styles.processor}>
        <div className={styles.processorHeader}>
          <div className={styles.processorTitle}>4. True Peak Limiter</div>
          <Switch
            checked={chain.limiter.enabled}
            onChange={(_, data) => updateProcessor('limiter', { enabled: data.checked })}
            label={chain.limiter.enabled ? 'ON' : 'OFF'}
          />
        </div>

        <div className={styles.controls}>
          <div className={styles.control}>
            <Label className={styles.label}>CEILING</Label>
            <Slider
              min={-3}
              max={0}
              step={0.1}
              value={chain.limiter.ceiling}
              onChange={(_, data) => updateProcessor('limiter', { ceiling: data.value })}
              disabled={!chain.limiter.enabled}
            />
            <div className={styles.value}>{chain.limiter.ceiling.toFixed(1)} dBTP</div>
          </div>

          <div className={styles.control}>
            <Label className={styles.label}>RELEASE</Label>
            <Slider
              min={10}
              max={1000}
              step={10}
              value={chain.limiter.release}
              onChange={(_, data) => updateProcessor('limiter', { release: data.value })}
              disabled={!chain.limiter.enabled}
            />
            <div className={styles.value}>{chain.limiter.release}ms</div>
          </div>
        </div>
      </div>
    </div>
  );
}
