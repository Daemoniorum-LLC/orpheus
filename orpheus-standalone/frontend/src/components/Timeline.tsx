import React, { useRef, useEffect } from 'react';
import './Timeline.css';

interface Track {
  id: string;
  name: string;
  audioFileUrl?: string;
}

interface TimelineProps {
  tracks: Track[];
  playheadPosition: number;
  bpm: number;
  timeSignature: string;
  isPlaying: boolean;
}

export const Timeline: React.FC<TimelineProps> = ({
  tracks,
  playheadPosition,
  bpm,
  timeSignature,
  isPlaying,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    drawTimeline();
  }, [tracks, bpm, timeSignature]);

  const drawTimeline = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = 5000; // 5000px width for timeline
    canvas.height = 50;

    // Clear canvas
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw time ruler
    const pixelsPerSecond = 50;
    const secondsPerBeat = 60 / bpm;
    const beatsPerBar = parseInt(timeSignature.split('/')[0]);

    // Draw bars
    for (let bar = 0; bar < 100; bar++) {
      const x = bar * beatsPerBar * secondsPerBeat * pixelsPerSecond;

      // Bar line
      ctx.strokeStyle = '#3a3a3a';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvas.height);
      ctx.stroke();

      // Bar number
      ctx.fillStyle = '#9ca3af';
      ctx.font = '11px Inter, system-ui, sans-serif';
      ctx.fillText(`${bar + 1}`, x + 4, 15);

      // Beat lines
      for (let beat = 1; beat < beatsPerBar; beat++) {
        const beatX = x + (beat * secondsPerBeat * pixelsPerSecond);
        ctx.strokeStyle = '#2a2a2a';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(beatX, 0);
        ctx.lineTo(beatX, canvas.height);
        ctx.stroke();
      }
    }
  };

  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    // Handle timeline click for seeking
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    // Position calculation would go here
  };

  return (
    <div className="timeline" data-testid="timeline" onClick={handleClick}>
      <div className="timeline-header">
        <canvas ref={canvasRef} className="timeline-ruler" />
      </div>

      <div
        className="timeline-scroll"
        ref={scrollRef}
        data-testid="timeline-scroll"
      >
        <div className="timeline-tracks">
          {tracks.map((track, index) => (
            <div
              key={track.id}
              className="timeline-track"
              style={{ height: '80px' }}
            >
              {track.audioFileUrl && (
                <div
                  className="audio-region"
                  data-testid="audio-region"
                  data-start="0"
                  style={{
                    left: '50px',
                    width: '200px',
                  }}
                >
                  <div className="audio-waveform">
                    {/* Waveform visualization would go here */}
                    <div className="waveform-placeholder" />
                  </div>
                  <div
                    className="region-end-handle"
                    data-testid="region-end-handle"
                  />
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Playhead */}
        <div
          className={`playhead ${isPlaying ? 'playing' : ''}`}
          data-testid="playhead"
          data-position={playheadPosition}
          style={{
            left: `${playheadPosition * 50}px`, // 50px per second
          }}
        />
      </div>
    </div>
  );
};
