/**
 * Channel Strip with Processors
 * Enhanced channel strip with EQ, Compressor, and Effects
 */

import { makeStyles, shorthands, Button, Dialog, DialogTrigger, DialogSurface, DialogTitle, DialogBody, DialogContent, DialogActions, Tooltip } from '@fluentui/react-components';
import { Settings24Regular, Eye24Regular, EyeOff24Regular } from '@fluentui/react-icons';
import { useState, useEffect } from 'react';
import { ChannelStrip, type ChannelStripProps } from './ChannelStrip';
import { ParametricEQ, type EQBand } from './ParametricEQ';
import { Compressor, type CompressorSettings } from './Compressor';
import { EffectsRack, type EffectsSettings } from './EffectsRack';
import { getAudioProcessingManager } from '../services/audio-processing';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  dialogContent: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
    maxHeight: '600px',
    ...shorthands.overflow('auto'),
  },
  settingsButton: {
    width: '100%',
  },
});

export interface ChannelProcessors {
  eq: EQBand[];
  compressor: CompressorSettings;
  effects: EffectsSettings;
}

export interface ChannelStripWithProcessorsProps extends ChannelStripProps {
  processors?: ChannelProcessors;
  onProcessorsChange?: (processors: ChannelProcessors) => void;
  onFocusToggle?: () => void;
  isFocused?: boolean;
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
  onFocusToggle,
  isFocused = false,
  ...channelProps
}: ChannelStripWithProcessorsProps) {
  const styles = useStyles();
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
    <div className={styles.container}>
      {/* Channel Strip */}
      <ChannelStrip {...channelProps} />

      {/* Focus and FX Buttons */}
      <div style={{ display: 'flex', gap: '4px' }}>
        {onFocusToggle && (
          <Tooltip content={isFocused ? "Exit Focus Mode" : "Focus on this track"} relationship="label">
            <Button
              icon={isFocused ? <EyeOff24Regular /> : <Eye24Regular />}
              appearance={isFocused ? "primary" : "subtle"}
              size="small"
              onClick={onFocusToggle}
              style={{ flex: 1 }}
            />
          </Tooltip>
        )}

        {/* Processor Settings Dialog */}
        <Dialog open={dialogOpen} onOpenChange={(_, data) => setDialogOpen(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Button
              icon={<Settings24Regular />}
              appearance="subtle"
              size="small"
              style={{ flex: 1 }}
            >
              FX
            </Button>
          </DialogTrigger>
        <DialogSurface>
          <DialogBody>
            <DialogTitle>
              {channelProps.trackName} - Processors
            </DialogTitle>
            <DialogContent className={styles.dialogContent}>
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
            </DialogContent>
            <DialogActions>
              <Button appearance="primary" onClick={() => setDialogOpen(false)}>
                Close
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
        </Dialog>
      </div>
    </div>
  );
}
