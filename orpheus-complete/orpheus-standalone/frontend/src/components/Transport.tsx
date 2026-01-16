import React, { useState } from 'react';
import './Transport.css';

interface TransportProps {
  isPlaying: boolean;
  playheadPosition: number;
  bpm: number;
  onPlayPause: () => void;
  onStop: () => void;
  onSeek: (position: number) => void;
}

export const Transport: React.FC<TransportProps> = ({
  isPlaying,
  playheadPosition,
  bpm,
  onPlayPause,
  onStop,
  onSeek,
}) => {
  const [isRecording, setIsRecording] = useState(false);
  const [isLooping, setIsLooping] = useState(false);
  const [metronomeEnabled, setMetronomeEnabled] = useState(false);
  const [timeFormat, setTimeFormat] = useState<'bars' | 'seconds' | 'samples'>('bars');

  const formatTime = (position: number): string => {
    if (timeFormat === 'bars') {
      const beatsPerBar = 4;
      const secondsPerBeat = 60 / bpm;
      const totalBeats = position / secondsPerBeat;
      const bars = Math.floor(totalBeats / beatsPerBar) + 1;
      const beats = Math.floor(totalBeats % beatsPerBar) + 1;
      const ticks = Math.floor((totalBeats % 1) * 960);
      return `${bars}:${beats}:${ticks}`;
    } else if (timeFormat === 'seconds') {
      return `${position.toFixed(2)}s`;
    } else {
      const sampleRate = 48000;
      const samples = Math.floor(position * sampleRate);
      return `${samples} samples`;
    }
  };

  return (
    <div className="transport">
      <div className="transport-left">
        <div className="transport-buttons">
          <button
            className="transport-btn"
            data-testid="transport-stop-button"
            onClick={onStop}
            title="Stop (Space)"
          >
            ⏹
          </button>

          <button
            className={`transport-btn ${isPlaying ? 'playing' : ''}`}
            data-testid="transport-play-button"
            onClick={onPlayPause}
            title="Play/Pause (Space)"
          >
            {isPlaying ? '⏸' : '▶'}
          </button>

          <button
            className={`transport-btn ${isRecording ? 'recording' : ''}`}
            data-testid="transport-record-button"
            onClick={() => setIsRecording(!isRecording)}
            title="Record (R)"
          >
            ⏺
          </button>

          <button
            className={`transport-btn ${isLooping ? 'active' : ''}`}
            data-testid="loop-button"
            onClick={() => setIsLooping(!isLooping)}
            title="Loop (L)"
          >
            ↻
          </button>
        </div>

        <div className="transport-time">
          <div
            className="time-display"
            data-testid="time-display"
            onClick={() => {
              const formats: Array<'bars' | 'seconds' | 'samples'> = ['bars', 'seconds', 'samples'];
              const currentIndex = formats.indexOf(timeFormat);
              setTimeFormat(formats[(currentIndex + 1) % formats.length]);
            }}
            title="Click to change format"
          >
            {formatTime(playheadPosition)}
          </div>
        </div>
      </div>

      <div className="transport-center">
        <div className="tempo-control">
          <label>BPM</label>
          <div
            className="tempo-display"
            data-testid="tempo-display"
            title="Click to change tempo"
          >
            {bpm}
          </div>
        </div>

        <button
          className={`transport-icon-btn ${metronomeEnabled ? 'active' : ''}`}
          data-testid="metronome-button"
          onClick={() => setMetronomeEnabled(!metronomeEnabled)}
          title="Metronome (M)"
        >
          🎵
          {metronomeEnabled && (
            <span
              className="metronome-indicator flashing"
              data-testid="metronome-indicator"
            />
          )}
        </button>

        <button
          className="transport-icon-btn"
          data-testid="snap-button"
          title="Snap to Grid"
        >
          🧲
        </button>
      </div>

      <div className="transport-right">
        <button
          className="transport-icon-btn"
          data-testid="zoom-out-button"
          title="Zoom Out (-)"
        >
          🔍−
        </button>

        <button
          className="transport-icon-btn"
          data-testid="zoom-in-button"
          title="Zoom In (+)"
        >
          🔍+
        </button>

        <button
          className="transport-icon-btn"
          data-testid="zoom-fit-button"
          title="Zoom to Fit"
        >
          ⬌
        </button>

        <button
          className="transport-icon-btn"
          data-testid="follow-playhead-button"
          title="Follow Playhead"
        >
          ➜
        </button>

        <div className="audio-latency" data-testid="audio-latency" title="Audio Latency">
          12.5ms
        </div>
      </div>
    </div>
  );
};
