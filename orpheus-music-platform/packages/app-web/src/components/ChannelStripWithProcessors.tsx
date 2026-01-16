/**
 * Channel Strip with Processors
 * Enhanced channel strip with EQ, Compressor, and Effects
 */

import { Button } from '@persona-framework/ui';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
} from '@persona-framework/ui';
import { Settings } from 'lucide-react';
import { useState, useEffect } from 'react';
import { ChannelStrip, type ChannelStripProps } from './ChannelStrip';
import { ParametricEQ, type EQBand } from './ParametricEQ';
import { Compressor, type CompressorSettings } from './Compressor';
import { EffectsRack, type EffectsSettings } from './EffectsRack';
import { getAudioProcessingManager } from '../services/audio-processing';

export interface ChannelProcessors {
  eq: EQBand[];
  compressor: CompressorSettings;
  effects: EffectsSettings;
}

export interface ChannelStripWithProcessorsProps extends ChannelStripProps {
  processors?: ChannelProcessors;
  onProcessorsChange?: (processors: ChannelProcessors) => void;
}

const DEFAULT_PROCESSORS: ChannelProcessors = {
  eq: [
    { frequency: 100, gain: 0, q: 1.0 },
    { frequency: 400, gain: 0, q: 1.0 },
    { frequency: 2000, gain: 0, q: 1.0 },
    { frequency: 8000, gain: 0, q: 1.0 },
  ],
  compressor: {
    enabled: false,
    threshold: -20,
    ratio: 4,
    attack: 10,
    release: 100,
    knee: 2,
    makeupGain: 0,
  },
  effects: {
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
  },
};

export function ChannelStripWithProcessors({
  processors = DEFAULT_PROCESSORS,
  onProcessorsChange,
  ...channelProps
}: ChannelStripWithProcessorsProps) {
  const [localProcessors, setLocalProcessors] = useState<ChannelProcessors>(processors);
  const [dialogOpen, setDialogOpen] = useState(false);

  // Wire up audio processing
  useEffect(() => {
    const manager = getAudioProcessingManager();
    const channel = manager.getChannel(channelProps.trackId);

    // Update processors when settings change
    channel.updateEQ(localProcessors.eq);
    channel.updateCompressor(localProcessors.compressor);
    channel.updateEffects(localProcessors.effects);

    console.log(`[ChannelStripWithProcessors] Updated processors for ${channelProps.trackName}`);
  }, [localProcessors, channelProps.trackId, channelProps.trackName]);

  const handleEQChange = (eq: EQBand[]) => {
    const newProcessors = { ...localProcessors, eq };
    setLocalProcessors(newProcessors);
    onProcessorsChange?.(newProcessors);
  };

  const handleCompressorChange = (compressor: CompressorSettings) => {
    const newProcessors = { ...localProcessors, compressor };
    setLocalProcessors(newProcessors);
    onProcessorsChange?.(newProcessors);
  };

  const handleEffectsChange = (effects: EffectsSettings) => {
    const newProcessors = { ...localProcessors, effects };
    setLocalProcessors(newProcessors);
    onProcessorsChange?.(newProcessors);
  };

  return (
    <div className="flex flex-col gap-2">
      {/* Channel Strip */}
      <ChannelStrip {...channelProps} />

      {/* Processor Settings Dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogTrigger asChild>
          <Button
            variant="ghost"
            size="sm"
            className="w-full"
          >
            <Settings className="h-4 w-4 mr-2" />
            FX
          </Button>
        </DialogTrigger>
        <DialogContent className="max-w-2xl w-[95vw] sm:w-auto max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {channelProps.trackName} - Processors
            </DialogTitle>
          </DialogHeader>
          <div className="flex flex-col gap-4">
            <ParametricEQ
              bands={localProcessors.eq}
              onBandsChange={handleEQChange}
            />
            <Compressor
              settings={localProcessors.compressor}
              onSettingsChange={handleCompressorChange}
            />
            <EffectsRack
              settings={localProcessors.effects}
              onSettingsChange={handleEffectsChange}
            />
          </div>
          <DialogFooter>
            <Button onClick={() => setDialogOpen(false)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
