/**
 * Waveform Visualizer Component
 * Real-time waveform and spectrum visualization
 */

import { useRef, useEffect } from 'react';
import * as Tone from 'tone';

export interface WaveformVisualizerProps {
  /** Width in pixels */
  width?: number;
  /** Height in pixels */
  height?: number;
  /** Visualization type */
  type?: 'waveform' | 'spectrum' | 'both';
  /** FFT size for spectrum */
  fftSize?: 128 | 256 | 512 | 1024 | 2048 | 4096 | 8192;
  /** Color scheme */
  color?: string;
  /** Background color */
  backgroundColor?: string;
}

export function WaveformVisualizer({
  width = 800,
  height = 200,
  type = 'waveform',
  fftSize = 2048,
  color = '#667eea',
  backgroundColor = '#1a1a1a',
}: WaveformVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const analyzerRef = useRef<Tone.Analyser | null>(null);
  const animationFrameRef = useRef<number | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    // Set canvas size
    canvas.width = width;
    canvas.height = height;

    // Create analyzer
    const analyzer = new Tone.Analyser(type === 'spectrum' ? 'fft' : 'waveform', fftSize);
    analyzerRef.current = analyzer;

    // Connect to master output
    Tone.getDestination().connect(analyzer);

    // Start visualization loop
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const draw = () => {
      if (!ctx || !analyzerRef.current) return;

      // Clear canvas
      ctx.fillStyle = backgroundColor;
      ctx.fillRect(0, 0, width, height);

      if (type === 'waveform' || type === 'both') {
        drawWaveform(ctx, analyzerRef.current, width, height, color, type === 'both' ? height / 2 : height);
      }

      if (type === 'spectrum' || type === 'both') {
        const yOffset = type === 'both' ? height / 2 : 0;
        const specHeight = type === 'both' ? height / 2 : height;
        drawSpectrum(ctx, analyzerRef.current, width, specHeight, color, yOffset);
      }

      animationFrameRef.current = requestAnimationFrame(draw);
    };

    draw();

    // Cleanup
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      if (analyzerRef.current) {
        analyzerRef.current.dispose();
      }
    };
  }, [width, height, type, fftSize, color, backgroundColor]);

  return (
    <div className="w-full h-full bg-background rounded border border-border relative overflow-hidden">
      <canvas ref={canvasRef} className="w-full h-full block" />
    </div>
  );
}

/**
 * Draw waveform visualization
 */
function drawWaveform(
  ctx: CanvasRenderingContext2D,
  analyzer: Tone.Analyser,
  width: number,
  height: number,
  color: string,
  yOffset: number = 0
): void {
  const values = analyzer.getValue() as Float32Array;
  const sliceWidth = width / values.length;

  ctx.beginPath();
  ctx.strokeStyle = color;
  ctx.lineWidth = 2;

  let x = 0;
  for (let i = 0; i < values.length; i++) {
    const v = values[i] as number;
    const y = ((v + 1) / 2) * height + yOffset;

    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }

    x += sliceWidth;
  }

  ctx.stroke();

  // Draw zero line
  ctx.beginPath();
  ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
  ctx.lineWidth = 1;
  ctx.moveTo(0, height / 2 + yOffset);
  ctx.lineTo(width, height / 2 + yOffset);
  ctx.stroke();
}

/**
 * Draw spectrum visualization
 */
function drawSpectrum(
  ctx: CanvasRenderingContext2D,
  analyzer: Tone.Analyser,
  width: number,
  height: number,
  color: string,
  yOffset: number = 0
): void {
  const values = analyzer.getValue() as Float32Array;
  const barWidth = width / values.length;
  const gradient = ctx.createLinearGradient(0, yOffset, 0, height + yOffset);

  // Create gradient based on color
  const rgb = hexToRgb(color);
  gradient.addColorStop(0, `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.8)`);
  gradient.addColorStop(0.5, `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.5)`);
  gradient.addColorStop(1, `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.2)`);

  ctx.fillStyle = gradient;

  for (let i = 0; i < values.length; i++) {
    const v = values[i] as number;
    // Map dB values (-100 to 0) to height
    const normalized = (v + 100) / 100;
    const barHeight = normalized * height;

    const x = i * barWidth;
    const y = height - barHeight + yOffset;

    ctx.fillRect(x, y, barWidth - 1, barHeight);
  }

  // Draw frequency labels (for reference)
  if (height > 100) {
    ctx.fillStyle = 'rgba(255, 255, 255, 0.3)';
    ctx.font = '10px monospace';
    ctx.textAlign = 'center';

    const frequencies = [100, 1000, 10000];
    const maxFreq = analyzer.context.sampleRate / 2;

    frequencies.forEach((freq) => {
      if (freq < maxFreq) {
        const x = (freq / maxFreq) * width;
        ctx.fillText(`${freq >= 1000 ? freq / 1000 + 'k' : freq}Hz`, x, yOffset + 15);
      }
    });
  }
}

/**
 * Convert hex color to RGB
 */
function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16),
      }
    : { r: 102, g: 126, b: 234 };
}
