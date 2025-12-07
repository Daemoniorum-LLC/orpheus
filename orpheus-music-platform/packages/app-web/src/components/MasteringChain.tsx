/**
 * Mastering Chain Component
 * Professional mastering processor chain: EQ → Compression → Stereo → Limiter
 */

import { Label, Slider, Switch } from '@persona-framework/ui';
import { useState } from 'react';

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
    <div className="flex flex-col gap-4">
      <div className="text-base font-semibold mb-2">Mastering Chain</div>

      {/* Master EQ */}
      <div className="p-4 border border-border rounded-lg bg-muted">
        <div className="flex justify-between items-center mb-3">
          <div className="text-sm font-semibold">1. Master EQ</div>
          <div className="flex items-center gap-2">
            <Label htmlFor="eq-switch" className="text-xs">{chain.eq.enabled ? 'ON' : 'OFF'}</Label>
            <Switch
              id="eq-switch"
              checked={chain.eq.enabled}
              onCheckedChange={(checked) => updateProcessor('eq', { enabled: checked })}
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">LOW CUT</Label>
            <Slider
              min={20}
              max={120}
              step={1}
              value={[chain.eq.lowCut]}
              onValueChange={(value) => updateProcessor('eq', { lowCut: value[0] })}
              disabled={!chain.eq.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.eq.lowCut}Hz</div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">LOW SHELF (80Hz)</Label>
            <Slider
              min={-6}
              max={6}
              step={0.5}
              value={[chain.eq.lowShelf]}
              onValueChange={(value) => updateProcessor('eq', { lowShelf: value[0] })}
              disabled={!chain.eq.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.eq.lowShelf > 0 ? '+' : ''}{chain.eq.lowShelf.toFixed(1)} dB</div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">PRESENCE (3kHz)</Label>
            <Slider
              min={-6}
              max={6}
              step={0.5}
              value={[chain.eq.presence]}
              onValueChange={(value) => updateProcessor('eq', { presence: value[0] })}
              disabled={!chain.eq.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.eq.presence > 0 ? '+' : ''}{chain.eq.presence.toFixed(1)} dB</div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">AIR BAND (12kHz)</Label>
            <Slider
              min={-6}
              max={6}
              step={0.5}
              value={[chain.eq.airBand]}
              onValueChange={(value) => updateProcessor('eq', { airBand: value[0] })}
              disabled={!chain.eq.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.eq.airBand > 0 ? '+' : ''}{chain.eq.airBand.toFixed(1)} dB</div>
          </div>
        </div>
      </div>

      {/* Glue Compressor */}
      <div className="p-4 border border-border rounded-lg bg-muted">
        <div className="flex justify-between items-center mb-3">
          <div className="text-sm font-semibold">2. Glue Compressor</div>
          <div className="flex items-center gap-2">
            <Label htmlFor="comp-switch" className="text-xs">{chain.compressor.enabled ? 'ON' : 'OFF'}</Label>
            <Switch
              id="comp-switch"
              checked={chain.compressor.enabled}
              onCheckedChange={(checked) => updateProcessor('compressor', { enabled: checked })}
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">THRESHOLD</Label>
            <Slider
              min={-24}
              max={0}
              step={0.5}
              value={[chain.compressor.threshold]}
              onValueChange={(value) => updateProcessor('compressor', { threshold: value[0] })}
              disabled={!chain.compressor.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.compressor.threshold} dB</div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">RATIO</Label>
            <Slider
              min={1}
              max={4}
              step={0.1}
              value={[chain.compressor.ratio]}
              onValueChange={(value) => updateProcessor('compressor', { ratio: value[0] })}
              disabled={!chain.compressor.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.compressor.ratio.toFixed(1)}:1</div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">ATTACK</Label>
            <Slider
              min={1}
              max={100}
              step={1}
              value={[chain.compressor.attack]}
              onValueChange={(value) => updateProcessor('compressor', { attack: value[0] })}
              disabled={!chain.compressor.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.compressor.attack}ms</div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">RELEASE</Label>
            <Slider
              min={50}
              max={1000}
              step={10}
              value={[chain.compressor.release]}
              onValueChange={(value) => updateProcessor('compressor', { release: value[0] })}
              disabled={!chain.compressor.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.compressor.release}ms</div>
          </div>
        </div>
      </div>

      {/* Stereo Imager */}
      <div className="p-4 border border-border rounded-lg bg-muted">
        <div className="flex justify-between items-center mb-3">
          <div className="text-sm font-semibold">3. Stereo Imager</div>
          <div className="flex items-center gap-2">
            <Label htmlFor="stereo-switch" className="text-xs">{chain.stereo.enabled ? 'ON' : 'OFF'}</Label>
            <Switch
              id="stereo-switch"
              checked={chain.stereo.enabled}
              onCheckedChange={(checked) => updateProcessor('stereo', { enabled: checked })}
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">WIDTH</Label>
            <Slider
              min={0}
              max={200}
              step={1}
              value={[chain.stereo.width]}
              onValueChange={(value) => updateProcessor('stereo', { width: value[0] })}
              disabled={!chain.stereo.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.stereo.width}%</div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">BALANCE</Label>
            <Slider
              min={-50}
              max={50}
              step={1}
              value={[chain.stereo.balance]}
              onValueChange={(value) => updateProcessor('stereo', { balance: value[0] })}
              disabled={!chain.stereo.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">
              {chain.stereo.balance === 0 ? 'C' : chain.stereo.balance > 0 ? `R${chain.stereo.balance}` : `L${Math.abs(chain.stereo.balance)}`}
            </div>
          </div>
        </div>
      </div>

      {/* Limiter */}
      <div className="p-4 border border-border rounded-lg bg-muted">
        <div className="flex justify-between items-center mb-3">
          <div className="text-sm font-semibold">4. True Peak Limiter</div>
          <div className="flex items-center gap-2">
            <Label htmlFor="limiter-switch" className="text-xs">{chain.limiter.enabled ? 'ON' : 'OFF'}</Label>
            <Switch
              id="limiter-switch"
              checked={chain.limiter.enabled}
              onCheckedChange={(checked) => updateProcessor('limiter', { enabled: checked })}
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">CEILING</Label>
            <Slider
              min={-3}
              max={0}
              step={0.1}
              value={[chain.limiter.ceiling]}
              onValueChange={(value) => updateProcessor('limiter', { ceiling: value[0] })}
              disabled={!chain.limiter.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.limiter.ceiling.toFixed(1)} dBTP</div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">RELEASE</Label>
            <Slider
              min={10}
              max={1000}
              step={10}
              value={[chain.limiter.release]}
              onValueChange={(value) => updateProcessor('limiter', { release: value[0] })}
              disabled={!chain.limiter.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">{chain.limiter.release}ms</div>
          </div>
        </div>
      </div>
    </div>
  );
}
