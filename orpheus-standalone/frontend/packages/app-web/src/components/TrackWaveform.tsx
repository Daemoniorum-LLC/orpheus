/**
 * Track Waveform Display
 * Renders a static waveform visualization for recorded audio
 */

import { useEffect, useRef, useState } from 'react';
import { makeStyles, tokens, Spinner } from '@fluentui/react-components';

// Shared AudioContext to avoid browser limits
let sharedAudioContext: AudioContext | null = null;

function getSharedAudioContext(): AudioContext {
  if (!sharedAudioContext || sharedAudioContext.state === 'closed') {
    sharedAudioContext = new AudioContext();
  }
  return sharedAudioContext;
}

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
  /** Called when user clicks to seek - receives position 0-1 */
  onSeek?: (position: number) => void;
  /** Called when user clicks to play from position - receives position 0-1 */
  onPlayFrom?: (position: number) => void;
}

export function TrackWaveform({
  audioBlob,
  duration,
  progress = 0,
  isPlaying = false,
  color = '#667eea',
  height = 60,
  onSeek,
  onPlayFrom,
}: TrackWaveformProps) {
  const styles = useStyles();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [waveformData, setWaveformData] = useState<number[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [hoverPosition, setHoverPosition] = useState<number | null>(null);

  // Decode audio and generate waveform data using shared context
  useEffect(() => {
    let cancelled = false;

    const generateWaveform = async () => {
      setIsLoading(true);
      try {
        const audioContext = getSharedAudioContext();

        // Resume if suspended (browser autoplay policy)
        if (audioContext.state === 'suspended') {
          await audioContext.resume();
        }

        const arrayBuffer = await audioBlob.arrayBuffer();
        const audioBuffer = await audioContext.decodeAudioData(arrayBuffer.slice(0));

        if (cancelled) return;

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
        let max = 0;
        for (const v of waveform) {
          if (v > max) max = v;
        }
        max = max || 0.001;
        const normalized = waveform.map((v) => v / max);

        if (!cancelled) {
          setWaveformData(normalized);
        }
      } catch (err) {
        console.error('[TrackWaveform] Failed to decode audio:', err);
        if (!cancelled) {
          // Generate placeholder waveform
          setWaveformData(Array(200).fill(0.1));
        }
      }
      if (!cancelled) {
        setIsLoading(false);
      }
    };

    if (audioBlob) {
      generateWaveform();
    }

    return () => {
      cancelled = true;
    };
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

  const getPositionFromEvent = (e: React.MouseEvent<HTMLDivElement>): number => {
    if (!containerRef.current) return 0;
    const rect = containerRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    return Math.max(0, Math.min(1, x / rect.width));
  };

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const position = getPositionFromEvent(e);

    if (isPlaying && onSeek) {
      // If playing, seek to position
      onSeek(position);
    } else if (onPlayFrom) {
      // If not playing, start playback from this position
      onPlayFrom(position);
    } else if (onSeek) {
      // Fallback: just seek
      onSeek(position);
    }
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    setHoverPosition(getPositionFromEvent(e));
  };

  const handleMouseLeave = () => {
    setHoverPosition(null);
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
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      role="slider"
      aria-label={`Audio waveform, ${formatTime(duration)} duration. Click to ${isPlaying ? 'seek' : 'play from position'}.`}
      aria-valuenow={Math.round(progress * 100)}
      aria-valuemin={0}
      aria-valuemax={100}
      tabIndex={0}
    >
      <canvas ref={canvasRef} className={styles.canvas} />

      {/* Playhead - show when playing or when there's progress */}
      {(isPlaying || progress > 0) && (
        <div
          className={styles.playhead}
          style={{ left: `${progress * 100}%` }}
        />
      )}

      {/* Hover indicator */}
      {hoverPosition !== null && (
        <div
          style={{
            position: 'absolute',
            left: `${hoverPosition * 100}%`,
            top: 0,
            bottom: 0,
            width: '1px',
            backgroundColor: `${color}88`,
            pointerEvents: 'none',
            zIndex: 1,
          }}
        />
      )}

      {/* Hover time tooltip */}
      {hoverPosition !== null && (
        <div
          style={{
            position: 'absolute',
            left: `${hoverPosition * 100}%`,
            top: '-20px',
            transform: 'translateX(-50%)',
            backgroundColor: tokens.colorNeutralBackground1,
            padding: '2px 6px',
            borderRadius: '4px',
            fontSize: '10px',
            fontFamily: 'monospace',
            color: tokens.colorNeutralForeground1,
            boxShadow: '0 1px 4px rgba(0,0,0,0.2)',
            pointerEvents: 'none',
            zIndex: 10,
          }}
        >
          {formatTime(hoverPosition * duration)}
        </div>
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
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
          }}
        >
          <Spinner size="tiny" />
          <span style={{ fontSize: '12px', color: tokens.colorNeutralForeground3 }}>
            Loading waveform...
          </span>
        </div>
      )}
    </div>
  );
}

export default TrackWaveform;
