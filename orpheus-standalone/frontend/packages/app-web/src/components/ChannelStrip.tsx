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
import { useState, useEffect, useRef, useCallback } from 'react';

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
    transition: 'all 0.2s ease',
  },
  stripSolo: {
    borderColor: tokens.colorPaletteYellowBorder1,
    boxShadow: `0 0 8px ${tokens.colorPaletteYellowBackground3}`,
  },
  stripMuted: {
    opacity: 0.6,
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
    ...shorthands.gap('2px'),
    justifyContent: 'center',
  },
  meter: {
    width: '8px',
    height: '80px',
    backgroundColor: tokens.colorNeutralBackground5,
    ...shorthands.borderRadius('2px'),
    position: 'relative',
    ...shorthands.overflow('hidden'),
  },
  meterFill: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    transition: 'height 50ms ease-out',
    background: `linear-gradient(to top,
      ${tokens.colorPaletteGreenBackground3} 0%,
      ${tokens.colorPaletteGreenBackground3} 60%,
      ${tokens.colorPaletteYellowBackground3} 75%,
      ${tokens.colorPaletteDarkOrangeBackground3} 85%,
      ${tokens.colorPaletteRedBackground3} 100%)`,
  },
  peakIndicator: {
    position: 'absolute',
    left: 0,
    right: 0,
    height: '2px',
    backgroundColor: tokens.colorPaletteRedForeground1,
    transition: 'bottom 100ms ease-out',
  },
  clipIndicator: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    height: '3px',
    transition: 'background-color 100ms',
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
  soloActive: {
    backgroundColor: `${tokens.colorPaletteYellowBackground3} !important`,
    color: '#000 !important',
  },
  muteActive: {
    backgroundColor: `${tokens.colorPaletteRedBackground3} !important`,
  },
});

export interface ChannelStripProps {
  trackId: string;
  trackName: string;
  volume?: number;
  pan?: number;
  solo?: boolean;
  mute?: boolean;
  /** Effective mute state (considering solo from other tracks) */
  effectiveMute?: boolean;
  /** Audio level for left channel (0-1) - if not provided, simulates based on volume */
  levelL?: number;
  /** Audio level for right channel (0-1) */
  levelR?: number;
  /** Track color for identification */
  color?: string;
  onVolumeChange?: (value: number) => void;
  onPanChange?: (value: number) => void;
  /** Solo toggle - receives mouse event for exclusive mode (Ctrl+click) */
  onSoloToggle?: (event?: React.MouseEvent) => void;
  onMuteToggle?: () => void;
  onDelete?: () => void;
}

export function ChannelStrip({
  trackName,
  volume = 0,
  pan = 0,
  solo = false,
  mute = false,
  effectiveMute,
  levelL,
  levelR,
  color,
  onVolumeChange,
  onPanChange,
  onSoloToggle,
  onMuteToggle,
  onDelete,
}: ChannelStripProps) {
  // Use effectiveMute if provided, otherwise fall back to mute
  const isEffectivelyMuted = effectiveMute ?? mute;
  const styles = useStyles();
  const [meterLevelL, setMeterLevelL] = useState(0);
  const [meterLevelR, setMeterLevelR] = useState(0);
  const [peakL, setPeakL] = useState(0);
  const [peakR, setPeakR] = useState(0);
  const [clippingL, setClippingL] = useState(false);
  const [clippingR, setClippingR] = useState(false);
  const peakTimerLRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const peakTimerRRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const clipTimerLRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const clipTimerRRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const animationRef = useRef<number | null>(null);

  // Update peak with hold
  const updatePeak = useCallback((channel: 'L' | 'R', level: number) => {
    const setPeak = channel === 'L' ? setPeakL : setPeakR;
    const timerRef = channel === 'L' ? peakTimerLRef : peakTimerRRef;
    const setClipping = channel === 'L' ? setClippingL : setClippingR;
    const clipTimerRef = channel === 'L' ? clipTimerLRef : clipTimerRRef;

    if (level > (channel === 'L' ? peakL : peakR)) {
      setPeak(level);
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => setPeak(0), 1500);
    }

    // Check for clipping
    if (level > 95) {
      setClipping(true);
      if (clipTimerRef.current) clearTimeout(clipTimerRef.current);
      clipTimerRef.current = setTimeout(() => setClipping(false), 2000);
    }
  }, [peakL, peakR]);

  // Handle external level input
  useEffect(() => {
    if (levelL !== undefined) {
      const effectiveLevel = isEffectivelyMuted ? 0 : levelL * 100;
      setMeterLevelL(effectiveLevel);
      updatePeak('L', effectiveLevel);
    }

    if (levelR !== undefined) {
      const effectiveLevel = isEffectivelyMuted ? 0 : levelR * 100;
      setMeterLevelR(effectiveLevel);
      updatePeak('R', effectiveLevel);
    }
  }, [levelL, levelR, isEffectivelyMuted, updatePeak]);

  // Simulate meter animation when no levels provided
  useEffect(() => {
    if (levelL === undefined) {
      const updateMeters = () => {
        // Simulate audio levels based on volume setting
        // Map -60 to +12 dB to 0-1 range, then add some noise
        const baseLevel = isEffectivelyMuted ? 0 : Math.max(0, (volume + 60) / 72);
        const noiseL = (Math.random() - 0.5) * 0.3 * baseLevel;
        const noiseR = (Math.random() - 0.5) * 0.3 * baseLevel;

        const newLevelL = Math.max(0, Math.min(100, (baseLevel + noiseL) * 100));
        const newLevelR = Math.max(0, Math.min(100, (baseLevel + noiseR) * 100));

        setMeterLevelL(newLevelL);
        setMeterLevelR(newLevelR);

        updatePeak('L', newLevelL);
        updatePeak('R', newLevelR);

        animationRef.current = requestAnimationFrame(updateMeters);
      };

      // Throttle updates to ~30fps
      const intervalId = setInterval(() => {
        if (animationRef.current) cancelAnimationFrame(animationRef.current);
        animationRef.current = requestAnimationFrame(updateMeters);
      }, 33);

      return () => {
        clearInterval(intervalId);
        if (animationRef.current) cancelAnimationFrame(animationRef.current);
        if (peakTimerLRef.current) clearTimeout(peakTimerLRef.current);
        if (peakTimerRRef.current) clearTimeout(peakTimerRRef.current);
        if (clipTimerLRef.current) clearTimeout(clipTimerLRef.current);
        if (clipTimerRRef.current) clearTimeout(clipTimerRRef.current);
      };
    }
  }, [levelL, volume, isEffectivelyMuted, updatePeak]);

  const stripClassName = [
    styles.strip,
    solo ? styles.stripSolo : '',
    isEffectivelyMuted ? styles.stripMuted : '',
  ].filter(Boolean).join(' ');

  const renderMeter = (level: number, peak: number, clipping: boolean) => (
    <div className={styles.meter}>
      {/* Clip indicator */}
      <div
        className={styles.clipIndicator}
        style={{
          backgroundColor: clipping ? tokens.colorPaletteRedBackground3 : 'transparent',
        }}
      />
      {/* Main meter fill */}
      <div
        className={styles.meterFill}
        style={{ height: `${Math.min(100, level)}%` }}
      />
      {/* Peak hold indicator */}
      {peak > 5 && (
        <div
          className={styles.peakIndicator}
          style={{ bottom: `${Math.min(100, peak)}%` }}
        />
      )}
    </div>
  );

  return (
    <div className={stripClassName} style={color ? { borderLeftColor: color, borderLeftWidth: '3px' } : undefined}>
      <div className={styles.header} title={trackName}>{trackName}</div>

      {/* Level Meters */}
      <div className={styles.meters}>
        {renderMeter(meterLevelL, peakL, clippingL)}
        {renderMeter(meterLevelR, peakR, clippingR)}
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
          className={`${styles.controlButton} ${solo ? styles.soloActive : ''}`}
          appearance={solo ? 'primary' : 'secondary'}
          size="small"
          onClick={(e) => onSoloToggle?.(e)}
          title="Solo - hear only this track (Ctrl+click for exclusive)"
        >
          S
        </Button>
        <Button
          className={`${styles.controlButton} ${mute ? styles.muteActive : ''}`}
          appearance={mute ? 'primary' : 'secondary'}
          icon={mute ? <SpeakerMute24Regular /> : <Speaker224Regular />}
          size="small"
          onClick={onMuteToggle}
          title="Mute track"
        />
        <Button
          className={styles.controlButton}
          icon={<Delete24Regular />}
          appearance="subtle"
          size="small"
          onClick={onDelete}
          title="Delete track"
        />
      </div>
    </div>
  );
}
