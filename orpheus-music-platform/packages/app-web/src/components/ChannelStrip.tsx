/**
 * Channel Strip Component
 * Professional mixing channel with fader, pan, effects, meters
 */

import { Button, Slider } from '@persona-framework/ui';
import { Volume2, VolumeX, Trash2 } from 'lucide-react';
import { useState } from 'react';

export interface ChannelStripProps {
  trackId: string;
  trackName: string;
  volume?: number;
  pan?: number;
  solo?: boolean;
  mute?: boolean;
  onVolumeChange?: (value: number) => void;
  onPanChange?: (value: number) => void;
  onSoloToggle?: () => void;
  onMuteToggle?: () => void;
  onDelete?: () => void;
}

export function ChannelStrip({
  trackName,
  volume = 0,
  pan = 0,
  solo = false,
  mute = false,
  onVolumeChange,
  onPanChange,
  onSoloToggle,
  onMuteToggle,
  onDelete,
}: ChannelStripProps) {
  const [meterLevel, setMeterLevel] = useState(0);

  // Simulate meter animation
  useState(() => {
    const interval = setInterval(() => {
      setMeterLevel(Math.random() * (mute ? 0 : volume + 20));
    }, 50);
    return () => clearInterval(interval);
  });

  const getMeterColor = (level: number) => {
    if (level > 85) return 'bg-red-500';
    if (level > 70) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  return (
    <div className="w-20 h-full flex flex-col p-3 border border-border rounded-lg bg-muted gap-3">
      <div className="text-xs font-semibold text-center overflow-hidden text-ellipsis whitespace-nowrap">
        {trackName}
      </div>

      {/* Level Meters */}
      <div className="flex gap-1 justify-center">
        <div className="w-1.5 h-20 bg-background/50 rounded-sm relative overflow-hidden">
          <div
            className={`absolute bottom-0 left-0 right-0 transition-[height] duration-50 ${getMeterColor(meterLevel)}`}
            style={{ height: meterLevel + '%' }}
          />
        </div>
        <div className="w-1.5 h-20 bg-background/50 rounded-sm relative overflow-hidden">
          <div
            className={`absolute bottom-0 left-0 right-0 transition-[height] duration-50 ${getMeterColor(meterLevel * 0.9)}`}
            style={{ height: (meterLevel * 0.9) + '%' }}
          />
        </div>
      </div>

      {/* Fader */}
      <div className="flex-1 flex flex-col items-center gap-2">
        <div className="text-[11px] font-mono text-muted-foreground">
          {volume.toFixed(1)} dB
        </div>
        <Slider
          orientation="vertical"
          min={-60}
          max={12}
          step={0.1}
          value={[volume]}
          onValueChange={(value) => onVolumeChange?.(value[0])}
          className="h-[200px]"
        />
      </div>

      {/* Pan Control */}
      <div className="flex flex-col items-center gap-2">
        <div className="text-[11px] font-mono text-muted-foreground">
          {pan === 0 ? 'C' : pan > 0 ? `R${pan}` : `L${Math.abs(pan)}`}
        </div>
        <Slider
          min={-50}
          max={50}
          step={1}
          value={[pan]}
          onValueChange={(value) => onPanChange?.(value[0])}
        />
      </div>

      {/* Control Buttons */}
      <div className="flex flex-col gap-1.5">
        <Button
          variant={solo ? 'default' : 'secondary'}
          size="sm"
          className="min-w-[56px] text-[10px]"
          onClick={onSoloToggle}
        >
          S
        </Button>
        <Button
          variant={mute ? 'default' : 'secondary'}
          size="sm"
          className="min-w-[56px]"
          onClick={onMuteToggle}
        >
          {mute ? <VolumeX className="h-4 w-4" /> : <Volume2 className="h-4 w-4" />}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="min-w-[56px]"
          onClick={onDelete}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
