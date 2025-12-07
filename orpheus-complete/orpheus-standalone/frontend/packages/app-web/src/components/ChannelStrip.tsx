/**
 * Channel Strip Component
 * Professional mixing channel with fader, pan, effects, meters
 */

import { makeStyles, shorthands, tokens, Button, Slider } from '@fluentui/react-components';
import {
  Speaker224Regular,
  SpeakerMute24Regular,
  Delete24Regular,
} from '@fluentui/react-icons';
import { useState } from 'react';

const useStyles = makeStyles({
  strip: {
    width: '80px',
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.padding('12px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.gap('12px'),
  },
  header: {
    fontSize: '12px',
    fontWeight: tokens.fontWeightSemibold,
    textAlign: 'center',
    ...shorthands.overflow('hidden'),
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  meters: {
    display: 'flex',
    ...shorthands.gap('4px'),
    justifyContent: 'center',
  },
  meter: {
    width: '6px',
    height: '80px',
    backgroundColor: tokens.colorNeutralBackground5,
    ...shorthands.borderRadius('3px'),
    position: 'relative',
    ...shorthands.overflow('hidden'),
  },
  meterFill: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    transition: 'height 0.05s ease-out',
  },
  fader: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  faderLabel: {
    fontSize: '11px',
    fontFamily: 'monospace',
    color: tokens.colorNeutralForeground2,
  },
  controls: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('6px'),
  },
  controlButton: {
    minWidth: '56px',
    fontSize: '10px',
  },
});

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
  const styles = useStyles();
  const [meterLevel, setMeterLevel] = useState(0);

  // Simulate meter animation
  useState(() => {
    const interval = setInterval(() => {
      setMeterLevel(Math.random() * (mute ? 0 : volume + 20));
    }, 50);
    return () => clearInterval(interval);
  });

  const getMeterColor = (level: number) => {
    if (level > 85) return tokens.colorPaletteRedBackground3;
    if (level > 70) return tokens.colorPaletteYellowBackground3;
    return tokens.colorPaletteGreenBackground3;
  };

  return (
    <div className={styles.strip}>
      <div className={styles.header}>{trackName}</div>

      {/* Level Meters */}
      <div className={styles.meters}>
        <div className={styles.meter}>
          <div
            className={styles.meterFill}
            style={{
              height: meterLevel + '%',
              backgroundColor: getMeterColor(meterLevel),
            }}
          />
        </div>
        <div className={styles.meter}>
          <div
            className={styles.meterFill}
            style={{
              height: (meterLevel * 0.9) + '%',
              backgroundColor: getMeterColor(meterLevel * 0.9),
            }}
          />
        </div>
      </div>

      {/* Fader */}
      <div className={styles.fader}>
        <div className={styles.faderLabel}>{volume.toFixed(1)} dB</div>
        <Slider
          vertical
          min={-60}
          max={12}
          step={0.1}
          value={volume}
          onChange={(_, data) => onVolumeChange?.(data.value)}
          style={{ height: '200px' }}
        />
      </div>

      {/* Pan Control */}
      <div className={styles.fader}>
        <div className={styles.faderLabel}>
          {pan === 0 ? 'C' : pan > 0 ? `R${pan}` : `L${Math.abs(pan)}`}
        </div>
        <Slider
          min={-50}
          max={50}
          step={1}
          value={pan}
          onChange={(_, data) => onPanChange?.(data.value)}
        />
      </div>

      {/* Control Buttons */}
      <div className={styles.controls}>
        <Button
          className={styles.controlButton}
          appearance={solo ? 'primary' : 'secondary'}
          size="small"
          onClick={onSoloToggle}
        >
          S
        </Button>
        <Button
          className={styles.controlButton}
          appearance={mute ? 'primary' : 'secondary'}
          icon={mute ? <SpeakerMute24Regular /> : <Speaker224Regular />}
          size="small"
          onClick={onMuteToggle}
        />
        <Button
          className={styles.controlButton}
          icon={<Delete24Regular />}
          appearance="subtle"
          size="small"
          onClick={onDelete}
        />
      </div>
    </div>
  );
}
