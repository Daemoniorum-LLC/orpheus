/**
 * Metronome Component
 * Visual and audio metronome with BPM control, time signatures, and count-in
 */

import { makeStyles, shorthands, tokens, Button, Slider, Label, Dropdown, Option, Switch } from '@fluentui/react-components';
import { Play24Regular, Stop24Regular } from '@fluentui/react-icons';
import { useState, useEffect, useRef } from 'react';
import * as Tone from 'tone';

const useStyles = makeStyles({
  container: {
    ...shorthands.padding('16px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    backgroundColor: tokens.colorNeutralBackground2,
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
  },
  controls: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
    ...shorthands.gap('16px'),
  },
  control: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('6px'),
  },
  label: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground2,
  },
  value: {
    fontSize: '12px',
    fontFamily: 'monospace',
    textAlign: 'center',
    color: tokens.colorBrandForeground1,
    fontWeight: tokens.fontWeightSemibold,
  },
  visualBeats: {
    display: 'flex',
    ...shorthands.gap('8px'),
    justifyContent: 'center',
    ...shorthands.padding('12px'),
  },
  beat: {
    width: '40px',
    height: '40px',
    ...shorthands.borderRadius('50%'),
    backgroundColor: tokens.colorNeutralBackground4,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    transition: 'all 0.1s ease-out',
  },
  beatActive: {
    backgroundColor: tokens.colorBrandBackground,
    color: tokens.colorNeutralForegroundInverted,
    transform: 'scale(1.2)',
    boxShadow: '0 0 20px rgba(102, 126, 234, 0.5)',
  },
  beatDownbeat: {
    backgroundColor: tokens.colorPaletteRedBackground3,
    color: tokens.colorNeutralForegroundInverted,
    transform: 'scale(1.2)',
    boxShadow: '0 0 20px rgba(255, 0, 0, 0.5)',
  },
  buttonGroup: {
    display: 'flex',
    ...shorthands.gap('8px'),
    justifyContent: 'center',
  },
});

export interface MetronomeProps {
  /** Initial BPM */
  initialBPM?: number;
  /** Initial time signature */
  initialTimeSignature?: [number, number];
  /** Auto-start on mount */
  autoStart?: boolean;
  /** Callback when metronome starts/stops */
  onPlayingChange?: (isPlaying: boolean) => void;
}

const TIME_SIGNATURES: Array<[number, number]> = [
  [2, 4],
  [3, 4],
  [4, 4],
  [5, 4],
  [6, 8],
  [7, 8],
  [9, 8],
  [12, 8],
];

export function Metronome({
  initialBPM = 120,
  initialTimeSignature = [4, 4],
  autoStart = false,
  onPlayingChange,
}: MetronomeProps) {
  const styles = useStyles();
  const [bpm, setBPM] = useState(initialBPM);
  const [timeSignature, setTimeSignature] = useState(initialTimeSignature);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentBeat, setCurrentBeat] = useState(0);
  const [countIn, setCountIn] = useState(false);
  const [countInBars, setCountInBars] = useState(1);

  const synthRef = useRef<Tone.Synth | null>(null);
  const loopRef = useRef<Tone.Loop | null>(null);

  // Initialize synth
  useEffect(() => {
    synthRef.current = new Tone.Synth({
      oscillator: { type: 'sine' },
      envelope: {
        attack: 0.001,
        decay: 0.1,
        sustain: 0,
        release: 0.1,
      },
    }).toDestination();

    synthRef.current.volume.value = -10;

    return () => {
      if (synthRef.current) {
        synthRef.current.dispose();
      }
      if (loopRef.current) {
        loopRef.current.dispose();
      }
    };
  }, []);

  // Start/stop metronome
  useEffect(() => {
    if (!synthRef.current) return;

    if (isPlaying) {
      Tone.Transport.bpm.value = bpm;

      let beatCount = 0;
      const beatsPerBar = timeSignature[0];
      const noteValue = `${timeSignature[1]}n`;

      // Create metronome loop
      loopRef.current = new Tone.Loop((time) => {
        const isDownbeat = beatCount % beatsPerBar === 0;
        const frequency = isDownbeat ? 880 : 440; // A5 for downbeat, A4 for other beats

        synthRef.current?.triggerAttackRelease(frequency, '8n', time);

        // Update visual beat (sync with audio)
        Tone.Draw.schedule(() => {
          setCurrentBeat(beatCount % beatsPerBar);
        }, time);

        beatCount++;
      }, noteValue);

      loopRef.current.start(0);
      Tone.Transport.start();
      onPlayingChange?.(true);
    } else {
      if (loopRef.current) {
        loopRef.current.stop();
        loopRef.current.dispose();
        loopRef.current = null;
      }
      Tone.Transport.stop();
      setCurrentBeat(0);
      onPlayingChange?.(false);
    }

    return () => {
      if (loopRef.current) {
        loopRef.current.stop();
        loopRef.current.dispose();
        loopRef.current = null;
      }
    };
  }, [isPlaying, bpm, timeSignature, onPlayingChange]);

  // Auto-start
  useEffect(() => {
    if (autoStart) {
      setIsPlaying(true);
    }
  }, [autoStart]);

  const handlePlayStop = async () => {
    if (!isPlaying) {
      await Tone.start();
    }
    setIsPlaying(!isPlaying);
  };

  const handleBPMChange = (value: number) => {
    setBPM(value);
    if (isPlaying) {
      Tone.Transport.bpm.value = value;
    }
  };

  const handleTimeSignatureChange = (value: string) => {
    const [numerator, denominator] = value.split('/').map(Number);
    setTimeSignature([numerator, denominator]);
    setCurrentBeat(0);
  };

  const timeSignatureOptions = TIME_SIGNATURES.map(([num, den]) => ({
    value: `${num}/${den}`,
    label: `${num}/${den}`,
  }));

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>Metronome</div>
        <Switch
          checked={countIn}
          onChange={(_, data) => setCountIn(data.checked)}
          label={countIn ? `Count-in: ${countInBars} bar${countInBars > 1 ? 's' : ''}` : 'Count-in: Off'}
        />
      </div>

      {/* Visual Beats */}
      <div className={styles.visualBeats}>
        {Array.from({ length: timeSignature[0] }, (_, i) => (
          <div
            key={i}
            className={`${styles.beat} ${
              isPlaying && currentBeat === i
                ? i === 0
                  ? styles.beatDownbeat
                  : styles.beatActive
                : ''
            }`}
          >
            {i + 1}
          </div>
        ))}
      </div>

      {/* Controls */}
      <div className={styles.controls}>
        <div className={styles.control}>
          <Label className={styles.label}>TEMPO (BPM)</Label>
          <Slider
            min={40}
            max={240}
            step={1}
            value={bpm}
            onChange={(_, data) => handleBPMChange(data.value)}
          />
          <div className={styles.value}>{bpm} BPM</div>
        </div>

        <div className={styles.control}>
          <Label className={styles.label}>TIME SIGNATURE</Label>
          <Dropdown
            value={`${timeSignature[0]}/${timeSignature[1]}`}
            onOptionSelect={(_, data) => handleTimeSignatureChange(data.optionValue as string)}
            disabled={isPlaying}
          >
            {timeSignatureOptions.map((opt) => (
              <Option key={opt.value} value={opt.value}>
                {opt.label}
              </Option>
            ))}
          </Dropdown>
          <div className={styles.value}>{timeSignature[0]}/{timeSignature[1]}</div>
        </div>

        {countIn && (
          <div className={styles.control}>
            <Label className={styles.label}>COUNT-IN BARS</Label>
            <Slider
              min={1}
              max={4}
              step={1}
              value={countInBars}
              onChange={(_, data) => setCountInBars(data.value)}
            />
            <div className={styles.value}>{countInBars} bar{countInBars > 1 ? 's' : ''}</div>
          </div>
        )}
      </div>

      {/* Play/Stop Button */}
      <div className={styles.buttonGroup}>
        <Button
          icon={isPlaying ? <Stop24Regular /> : <Play24Regular />}
          appearance="primary"
          onClick={handlePlayStop}
          size="large"
        >
          {isPlaying ? 'Stop' : 'Start'} Metronome
        </Button>
      </div>
    </div>
  );
}
