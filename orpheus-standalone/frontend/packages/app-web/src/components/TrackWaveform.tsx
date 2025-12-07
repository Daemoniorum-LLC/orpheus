/**
 * Track Waveform Display
 * Renders a static waveform visualization for recorded audio
 */

import { useEffect, useRef, useState } from 'react';
import { makeStyles, tokens } from '@fluentui/react-components';

const useStyles = makeStyles({
  container: {
    width: '100%',
    height: '60px',
    position: 'relative',
    backgroundColor: tokens.colorNeutralBackground3,
    borderRadius: '4px',
    overflow: 'hidden',
    cursor: 'pointer',
  },
  canvas: {
    width: '100%',
    height: '100%',
  },
  playhead: {
    position: 'absolute',
    top: 0,
    bottom: 0,
    width: '2px',
    backgroundColor: tokens.colorBrandForeground1,
    pointerEvents: 'none',
    transition: 'left 0.1s linear',
  },
  timeMarkers: {
    position: 'absolute',
    bottom: '2px',
    left: '4px',
    right: '4px',
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '9px',
    color: tokens.colorNeutralForeground3,
    pointerEvents: 'none',
  },
});

interface TrackWaveformProps {
  audioBlob: Blob;
  duration: number;
  progress?: number;
  isPlaying?: boolean;
  color?: string;
  height?: number;
  onSeek?: (position: number) => void;
}

export function TrackWaveform({
  audioBlob,
  duration,
  progress = 0,
  isPlaying = false,
  color = '#667eea',
  height = 60,
  onSeek,
}: TrackWaveformProps) {
  const styles = useStyles();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [waveformData, setWaveformData] = useState<number[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // Decode audio and generate waveform data
  useEffect(() => {
    const generateWaveform = async () => {
      setIsLoading(true);
      try {
        const audioContext = new AudioContext();
        const arrayBuffer = await audioBlob.arrayBuffer();
        const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);

        // Get the audio data from the first channel
        const channelData = audioBuffer.getChannelData(0);
        const samples = 200; // Number of bars in the waveform
        const blockSize = Math.floor(channelData.length / samples);
        const waveform: number[] = [];

        for (let i = 0; i < samples; i++) {
          const start = blockSize * i;
          let sum = 0;

          for (let j = 0; j < blockSize; j++) {
            sum += Math.abs(channelData[start + j] || 0);
          }

          // RMS value
          const rms = Math.sqrt(sum / blockSize);
          waveform.push(rms);
        }

        // Normalize
        const max = Math.max(...waveform, 0.001);
        const normalized = waveform.map((v) => v / max);

        setWaveformData(normalized);
        audioContext.close();
      } catch (err) {
        console.error('[TrackWaveform] Failed to decode audio:', err);
        // Generate placeholder waveform
        setWaveformData(Array(200).fill(0.1));
      }
      setIsLoading(false);
    };

    if (audioBlob) {
      generateWaveform();
    }
  }, [audioBlob]);

  // Draw waveform
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || waveformData.length === 0) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const canvasHeight = rect.height;
    const barWidth = width / waveformData.length;
    const centerY = canvasHeight / 2;

    // Clear canvas
    ctx.clearRect(0, 0, width, canvasHeight);

    // Draw waveform bars
    waveformData.forEach((value, index) => {
      const barHeight = value * (canvasHeight - 8) * 0.9;
      const x = index * barWidth;
      const playedPercent = progress;

      // Determine color based on playback position
      const barProgress = index / waveformData.length;
      if (barProgress <= playedPercent && isPlaying) {
        ctx.fillStyle = color;
      } else {
        ctx.fillStyle = `${color}66`; // Semi-transparent
      }

      // Draw symmetric bars
      ctx.fillRect(x, centerY - barHeight / 2, barWidth - 1, barHeight);
    });

    // Draw center line
    ctx.strokeStyle = `${color}33`;
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, centerY);
    ctx.lineTo(width, centerY);
    ctx.stroke();
  }, [waveformData, progress, isPlaying, color]);

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!onSeek || !containerRef.current) return;

    const rect = containerRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const position = x / rect.width;
    onSeek(Math.max(0, Math.min(1, position)));
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div
      ref={containerRef}
      className={styles.container}
      style={{ height: `${height}px` }}
      onClick={handleClick}
    >
      <canvas ref={canvasRef} className={styles.canvas} />

      {/* Playhead */}
      {isPlaying && (
        <div
          className={styles.playhead}
          style={{ left: `${progress * 100}%` }}
        />
      )}

      {/* Time markers */}
      <div className={styles.timeMarkers}>
        <span>0:00</span>
        <span>{formatTime(duration / 2)}</span>
        <span>{formatTime(duration)}</span>
      </div>

      {/* Loading indicator */}
      {isLoading && (
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            fontSize: '12px',
            color: tokens.colorNeutralForeground3,
          }}
        >
          Loading...
        </div>
      )}
    </div>
  );
}

export default TrackWaveform;
