/**
 * Effects Rack Component
 * Reverb, Delay, Chorus, and other time-based effects
 */

import { Label, Slider, Switch } from '@persona-framework/ui';
import { useState } from 'react';

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
    <div className="p-4 border border-border rounded-lg bg-muted">
      <div className="flex justify-between items-center mb-4">
        <div className="text-sm font-semibold">Effects</div>
      </div>

      {/* Reverb */}
      <div className="mb-4 p-3 rounded-md bg-background">
        <div className="flex justify-between items-center mb-3">
          <div className="text-xs font-semibold">REVERB</div>
          <div className="flex items-center gap-2">
            <Label htmlFor="reverb-switch" className="text-xs">
              {effects.reverb.enabled ? 'ON' : 'OFF'}
            </Label>
            <Switch
              id="reverb-switch"
              checked={effects.reverb.enabled}
              onCheckedChange={(checked) => updateReverb({ enabled: checked })}
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">ROOM SIZE</Label>
            <Slider
              min={0}
              max={1}
              step={0.01}
              value={[effects.reverb.roomSize]}
              onValueChange={(value) => updateReverb({ roomSize: value[0] })}
              disabled={!effects.reverb.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">
              {(effects.reverb.roomSize * 100).toFixed(0)}%
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">DECAY</Label>
            <Slider
              min={0.1}
              max={10}
              step={0.1}
              value={[effects.reverb.decay]}
              onValueChange={(value) => updateReverb({ decay: value[0] })}
              disabled={!effects.reverb.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">
              {effects.reverb.decay.toFixed(1)}s
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">PRE-DELAY</Label>
            <Slider
              min={0}
              max={100}
              step={1}
              value={[effects.reverb.preDelay]}
              onValueChange={(value) => updateReverb({ preDelay: value[0] })}
              disabled={!effects.reverb.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">
              {effects.reverb.preDelay}ms
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">WET/DRY</Label>
            <Slider
              min={0}
              max={100}
              step={1}
              value={[effects.reverb.wetDry]}
              onValueChange={(value) => updateReverb({ wetDry: value[0] })}
              disabled={!effects.reverb.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">
              {effects.reverb.wetDry}%
            </div>
          </div>
        </div>
      </div>

      {/* Delay */}
      <div className="p-3 rounded-md bg-background">
        <div className="flex justify-between items-center mb-3">
          <div className="text-xs font-semibold">DELAY</div>
          <div className="flex items-center gap-2">
            <Label htmlFor="delay-switch" className="text-xs">
              {effects.delay.enabled ? 'ON' : 'OFF'}
            </Label>
            <Switch
              id="delay-switch"
              checked={effects.delay.enabled}
              onCheckedChange={(checked) => updateDelay({ enabled: checked })}
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">TIME</Label>
            <Slider
              min={1}
              max={2000}
              step={1}
              value={[effects.delay.time]}
              onValueChange={(value) => updateDelay({ time: value[0] })}
              disabled={!effects.delay.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">
              {effects.delay.time}ms
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">FEEDBACK</Label>
            <Slider
              min={0}
              max={95}
              step={1}
              value={[effects.delay.feedback]}
              onValueChange={(value) => updateDelay({ feedback: value[0] })}
              disabled={!effects.delay.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">
              {effects.delay.feedback}%
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">WET/DRY</Label>
            <Slider
              min={0}
              max={100}
              step={1}
              value={[effects.delay.wetDry]}
              onValueChange={(value) => updateDelay({ wetDry: value[0] })}
              disabled={!effects.delay.enabled}
            />
            <div className="text-[10px] font-mono text-center text-primary">
              {effects.delay.wetDry}%
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-[10px] text-muted-foreground">SYNC</Label>
            <div className="flex items-center gap-2">
              <Switch
                checked={effects.delay.sync}
                onCheckedChange={(checked) => updateDelay({ sync: checked })}
                disabled={!effects.delay.enabled}
              />
              <span className="text-[10px] text-muted-foreground">
                {effects.delay.sync ? 'Tempo Sync' : 'Free'}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
