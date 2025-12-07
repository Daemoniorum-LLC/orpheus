/**
 * Metronome Component
 * Visual and audio metronome with BPM control, time signatures, and count-in
 */

import {
  Button,
  Label,
  Switch,
  Slider,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@persona-framework/ui';
import { Play, Square } from 'lucide-react';
import { useState, useEffect, useRef } from 'react';
import * as Tone from 'tone';

export interface MetronomeProps {
  initialBPM?: number;
  initialTimeSignature?: [number, number];
  autoStart?: boolean;
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

      loopRef.current = new Tone.Loop((time) => {
        const isDownbeat = beatCount % beatsPerBar === 0;
        const frequency = isDownbeat ? 880 : 440;

        synthRef.current?.triggerAttackRelease(frequency, '8n', time);

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

  return (
    <div className="p-4 border border-border rounded-lg bg-secondary flex flex-col gap-4">
      <div className="flex justify-between items-center">
        <div className="text-sm font-semibold">Metronome</div>
        <div className="flex items-center gap-2">
          <Switch
            checked={countIn}
            onCheckedChange={setCountIn}
          />
          <Label className="text-xs">
            {countIn ? `Count-in: ${countInBars} bar${countInBars > 1 ? 's' : ''}` : 'Count-in: Off'}
          </Label>
        </div>
      </div>

      {/* Visual Beats */}
      <div className="flex gap-2 justify-center p-3">
        {Array.from({ length: timeSignature[0] }, (_, i) => (
          <div
            key={i}
            className={`w-10 h-10 rounded-full flex items-center justify-center text-sm font-semibold transition-all duration-100 ${
              isPlaying && currentBeat === i
                ? i === 0
                  ? 'bg-red-500 text-white scale-125 shadow-[0_0_20px_rgba(239,68,68,0.5)]'
                  : 'bg-primary text-primary-foreground scale-125 shadow-[0_0_20px_rgba(102,126,234,0.5)]'
                : 'bg-muted'
            }`}
          >
            {i + 1}
          </div>
        ))}
      </div>

      {/* Controls */}
      <div className="grid grid-cols-3 gap-4">
        <div className="flex flex-col gap-1.5">
          <Label className="text-[11px] text-muted-foreground">TEMPO (BPM)</Label>
          <Slider
            min={40}
            max={240}
            step={1}
            value={[bpm]}
            onValueChange={(value) => handleBPMChange(value[0])}
          />
          <div className="text-xs font-mono text-center text-primary font-semibold">{bpm} BPM</div>
        </div>

        <div className="flex flex-col gap-1.5">
          <Label className="text-[11px] text-muted-foreground">TIME SIGNATURE</Label>
          <Select
            value={`${timeSignature[0]}/${timeSignature[1]}`}
            onValueChange={handleTimeSignatureChange}
            disabled={isPlaying}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {TIME_SIGNATURES.map(([num, den]) => (
                <SelectItem key={`${num}/${den}`} value={`${num}/${den}`}>
                  {num}/{den}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <div className="text-xs font-mono text-center text-primary font-semibold">
            {timeSignature[0]}/{timeSignature[1]}
          </div>
        </div>

        {countIn && (
          <div className="flex flex-col gap-1.5">
            <Label className="text-[11px] text-muted-foreground">COUNT-IN BARS</Label>
            <Slider
              min={1}
              max={4}
              step={1}
              value={[countInBars]}
              onValueChange={(value) => setCountInBars(value[0])}
            />
            <div className="text-xs font-mono text-center text-primary font-semibold">
              {countInBars} bar{countInBars > 1 ? 's' : ''}
            </div>
          </div>
        )}
      </div>

      {/* Play/Stop Button */}
      <div className="flex gap-2 justify-center">
        <Button
          size="lg"
          onClick={handlePlayStop}
        >
          {isPlaying ? (
            <Square className="h-4 w-4 mr-2" />
          ) : (
            <Play className="h-4 w-4 mr-2" />
          )}
          {isPlaying ? 'Stop' : 'Start'} Metronome
        </Button>
      </div>
    </div>
  );
}
