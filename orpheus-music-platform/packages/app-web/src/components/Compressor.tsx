/**
 * Compressor Component
 * Professional dynamics compressor with threshold, ratio, attack, release, and makeup gain
 */

import { Label, Slider, Switch } from '@persona-framework/ui';
import { useState } from 'react';

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
    <div className="p-4 border border-border rounded-lg bg-muted">
      <div className="flex justify-between items-center mb-4">
        <div className="text-sm font-semibold">Compressor</div>
        <div className="flex items-center gap-2">
          <Label htmlFor="compressor-switch" className="text-xs">
            {compressor.enabled ? 'ON' : 'OFF'}
          </Label>
          <Switch
            id="compressor-switch"
            checked={compressor.enabled}
            onCheckedChange={(checked) => updateSetting('enabled', checked)}
          />
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        {/* Threshold */}
        <div className="flex flex-col gap-2">
          <Label className="text-[11px] font-semibold text-muted-foreground">THRESHOLD</Label>
          <Slider
            min={-60}
            max={0}
            step={1}
            value={[compressor.threshold]}
            onValueChange={(value) => updateSetting('threshold', value[0])}
            disabled={!compressor.enabled}
          />
          <div className="text-xs font-mono text-center text-primary">
            {compressor.threshold} dB
          </div>
        </div>

        {/* Ratio */}
        <div className="flex flex-col gap-2">
          <Label className="text-[11px] font-semibold text-muted-foreground">RATIO</Label>
          <Slider
            min={1}
            max={20}
            step={0.5}
            value={[compressor.ratio]}
            onValueChange={(value) => updateSetting('ratio', value[0])}
            disabled={!compressor.enabled}
          />
          <div className="text-xs font-mono text-center text-primary">
            {compressor.ratio.toFixed(1)}:1
          </div>
        </div>

        {/* Attack */}
        <div className="flex flex-col gap-2">
          <Label className="text-[11px] font-semibold text-muted-foreground">ATTACK</Label>
          <Slider
            min={0.1}
            max={100}
            step={0.1}
            value={[compressor.attack]}
            onValueChange={(value) => updateSetting('attack', value[0])}
            disabled={!compressor.enabled}
          />
          <div className="text-xs font-mono text-center text-primary">
            {compressor.attack.toFixed(1)} ms
          </div>
        </div>

        {/* Release */}
        <div className="flex flex-col gap-2">
          <Label className="text-[11px] font-semibold text-muted-foreground">RELEASE</Label>
          <Slider
            min={10}
            max={1000}
            step={10}
            value={[compressor.release]}
            onValueChange={(value) => updateSetting('release', value[0])}
            disabled={!compressor.enabled}
          />
          <div className="text-xs font-mono text-center text-primary">
            {compressor.release} ms
          </div>
        </div>

        {/* Knee */}
        <div className="flex flex-col gap-2">
          <Label className="text-[11px] font-semibold text-muted-foreground">KNEE</Label>
          <Slider
            min={0}
            max={12}
            step={1}
            value={[compressor.knee]}
            onValueChange={(value) => updateSetting('knee', value[0])}
            disabled={!compressor.enabled}
          />
          <div className="text-xs font-mono text-center text-primary">
            {compressor.knee} dB
          </div>
        </div>

        {/* Makeup Gain */}
        <div className="flex flex-col gap-2">
          <Label className="text-[11px] font-semibold text-muted-foreground">MAKEUP</Label>
          <Slider
            min={0}
            max={24}
            step={0.5}
            value={[compressor.makeupGain]}
            onValueChange={(value) => updateSetting('makeupGain', value[0])}
            disabled={!compressor.enabled}
          />
          <div className="text-xs font-mono text-center text-primary">
            +{compressor.makeupGain.toFixed(1)} dB
          </div>
        </div>
      </div>

      {/* Gain Reduction Meter */}
      <div className="mt-4 flex items-center gap-3">
        <div className="text-[10px] text-muted-foreground min-w-[30px]">GR</div>
        <div className="flex-1 h-5 bg-background/50 rounded relative overflow-hidden">
          <div
            className="h-full transition-[width] duration-100"
            style={{
              width: `${(gainReduction / 12) * 100}%`,
              backgroundColor: compressor.enabled ? 'rgb(239 68 68)' : 'rgb(115 115 115)',
            }}
          />
        </div>
        <div className="text-[10px] text-muted-foreground min-w-[30px]">
          -{gainReduction.toFixed(1)} dB
        </div>
      </div>
    </div>
  );
}
