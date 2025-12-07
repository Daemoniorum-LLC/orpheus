/**
 * Parametric EQ Component
 * 4-band parametric equalizer with frequency, gain, and Q controls
 */

import { Label, Slider } from '@persona-framework/ui';
import { useState } from 'react';

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
    <div className="p-4 border border-border rounded-lg bg-muted">
      <div className="text-sm font-semibold mb-3">Parametric EQ</div>
      <div className="grid grid-cols-4 gap-3">
        {eqBands.map((band, index) => (
          <div key={index} className="flex flex-col gap-2">
            <div className="text-[11px] font-semibold text-center text-muted-foreground">
              {getBandName(index)}
            </div>

            {/* Frequency */}
            <div className="flex flex-col gap-1">
              <Label className="text-[10px] text-muted-foreground">FREQ</Label>
              <Slider
                min={20}
                max={20000}
                step={10}
                value={[band.frequency]}
                onValueChange={(value) => updateBand(index, { frequency: value[0] })}
              />
              <div className="text-[10px] font-mono text-center text-primary">
                {formatFreq(band.frequency)}Hz
              </div>
            </div>

            {/* Gain */}
            <div className="flex flex-col gap-1">
              <Label className="text-[10px] text-muted-foreground">GAIN</Label>
              <Slider
                min={-12}
                max={12}
                step={0.5}
                value={[band.gain]}
                onValueChange={(value) => updateBand(index, { gain: value[0] })}
              />
              <div className="text-[10px] font-mono text-center text-primary">
                {band.gain > 0 ? '+' : ''}{band.gain.toFixed(1)} dB
              </div>
            </div>

            {/* Q (Bandwidth) */}
            <div className="flex flex-col gap-1">
              <Label className="text-[10px] text-muted-foreground">Q</Label>
              <Slider
                min={0.1}
                max={10}
                step={0.1}
                value={[band.q]}
                onValueChange={(value) => updateBand(index, { q: value[0] })}
              />
              <div className="text-[10px] font-mono text-center text-primary">
                {band.q.toFixed(1)}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
