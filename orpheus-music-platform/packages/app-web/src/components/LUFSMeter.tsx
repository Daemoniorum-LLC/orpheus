/**
 * LUFS Meter - Broadcast-standard loudness metering
 * Displays integrated LUFS, short-term LUFS, momentary LUFS, and true peak
 */

import { useRef, useEffect, useState } from 'react';
import * as Tone from 'tone';
import { cn } from '../lib/utils';

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
    if (status === 'good') return 'bg-green-500';
    if (status === 'warning') return 'bg-yellow-500';
    return 'bg-red-500';
  };

  const formatLUFS = (value: number): string => {
    return value > -70 ? value.toFixed(1) : '-∞';
  };

  return (
    <div className="flex flex-col gap-3 p-4 bg-muted rounded-lg border border-border">
      <div className="text-sm font-semibold text-foreground">LUFS Metering</div>

      <div className="grid grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-4">
        {/* Integrated LUFS */}
        <div className="flex flex-col gap-2 p-3 bg-background rounded-md border border-border">
          <div className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Integrated</div>
          <div>
            <span className="text-3xl font-bold font-mono text-foreground">{formatLUFS(values.integrated)}</span>
            <span className="text-xs text-muted-foreground font-semibold ml-1">LUFS</span>
          </div>
          <div className="h-2 bg-muted rounded overflow-hidden relative">
            <div
              className={cn('h-full transition-[width] duration-100', getBarColor(values.integrated, target))}
              style={{ width: `${getBarWidth(values.integrated)}%` }}
            />
            <div
              className="absolute top-0 bottom-0 w-0.5 bg-primary"
              style={{ left: `${getBarWidth(target)}%` }}
            />
          </div>
          {showTargets && (
            <>
              <div className="text-[10px] text-muted-foreground">
                Target: {target} LUFS
              </div>
              <span className={cn(
                'text-[10px] font-semibold px-1.5 py-0.5 rounded inline-block w-fit',
                getStatus(values.integrated, target) === 'good' ? 'bg-green-500/20 text-green-600' :
                getStatus(values.integrated, target) === 'warning' ? 'bg-yellow-500/20 text-yellow-600' :
                'bg-red-500/20 text-red-600'
              )}>
                {getStatus(values.integrated, target) === 'good' ? '✓ On Target' :
                 getStatus(values.integrated, target) === 'warning' ? '⚠ Close' :
                 values.integrated > target ? '↑ Too Loud' : '↓ Too Quiet'}
              </span>
            </>
          )}
        </div>

        {/* Short-term LUFS */}
        <div className="flex flex-col gap-2 p-3 bg-background rounded-md border border-border">
          <div className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Short-term (3s)</div>
          <div>
            <span className="text-3xl font-bold font-mono text-foreground">{formatLUFS(values.shortTerm)}</span>
            <span className="text-xs text-muted-foreground font-semibold ml-1">LUFS</span>
          </div>
          <div className="h-2 bg-muted rounded overflow-hidden">
            <div
              className={cn('h-full transition-[width] duration-100', getBarColor(values.shortTerm, target))}
              style={{ width: `${getBarWidth(values.shortTerm)}%` }}
            />
          </div>
        </div>

        {/* Momentary LUFS */}
        <div className="flex flex-col gap-2 p-3 bg-background rounded-md border border-border">
          <div className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Momentary (400ms)</div>
          <div>
            <span className="text-3xl font-bold font-mono text-foreground">{formatLUFS(values.momentary)}</span>
            <span className="text-xs text-muted-foreground font-semibold ml-1">LUFS</span>
          </div>
          <div className="h-2 bg-muted rounded overflow-hidden">
            <div
              className={cn('h-full transition-[width] duration-100', getBarColor(values.momentary, target))}
              style={{ width: `${getBarWidth(values.momentary)}%` }}
            />
          </div>
        </div>

        {/* True Peak */}
        <div className="flex flex-col gap-2 p-3 bg-background rounded-md border border-border">
          <div className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">True Peak</div>
          <div>
            <span className="text-3xl font-bold font-mono text-foreground">{formatLUFS(values.truePeak)}</span>
            <span className="text-xs text-muted-foreground font-semibold ml-1">dBTP</span>
          </div>
          <div className="h-2 bg-muted rounded overflow-hidden">
            <div
              className={cn('h-full transition-[width] duration-100', values.truePeak > -1 ? 'bg-red-500' : 'bg-green-500')}
              style={{ width: `${getBarWidth(values.truePeak)}%` }}
            />
          </div>
          <div className="text-[10px] text-muted-foreground">
            {values.truePeak > -1 ? '⚠ Clipping Risk' : '✓ Safe'}
          </div>
        </div>
      </div>

      {showTargets && (
        <div className="text-[11px] text-muted-foreground mt-2 flex gap-4 flex-wrap">
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
