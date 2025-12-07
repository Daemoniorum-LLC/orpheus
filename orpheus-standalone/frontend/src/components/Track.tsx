import React from 'react';
import './Track.css';

interface TrackData {
  id: string;
  name: string;
  instrument: string;
  trackNumber: number;
  volume: number;
  pan: number;
  muted: boolean;
  soloed: boolean;
  audioFileUrl?: string;
}

interface TrackProps {
  track: TrackData;
  onUpdate: (updates: Partial<TrackData>) => void;
  onDelete: () => void;
}

export const Track: React.FC<TrackProps> = ({ track, onUpdate, onDelete }) => {
  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onUpdate({ volume: parseFloat(e.target.value) });
  };

  const handlePanChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onUpdate({ pan: parseFloat(e.target.value) });
  };

  const toggleMute = () => {
    onUpdate({ muted: !track.muted });
  };

  const toggleSolo = () => {
    onUpdate({ soloed: !track.soloed });
  };

  return (
    <div
      className={`track-item ${track.muted ? 'muted' : ''} ${track.soloed ? 'soloed' : ''}`}
      data-testid="track-item"
    >
      <div className="track-header">
        <span className="track-number">{track.trackNumber}</span>
        <input
          type="text"
          className="track-name"
          data-testid="track-name"
          value={track.name}
          onChange={(e) => onUpdate({ name: e.target.value })}
          placeholder="Track Name"
        />
      </div>

      <div className="track-controls">
        <div className="track-control-row">
          <button
            className={`btn-track-control ${track.muted ? 'active' : ''}`}
            data-testid="track-mute-button"
            data-muted={track.muted}
            onClick={toggleMute}
            title="Mute"
          >
            M
          </button>
          <button
            className={`btn-track-control ${track.soloed ? 'active' : ''}`}
            data-testid="track-solo-button"
            data-soloed={track.soloed}
            onClick={toggleSolo}
            title="Solo"
          >
            S
          </button>
          <button
            className="btn-track-control"
            data-testid="track-record-arm"
            title="Arm for Recording"
          >
            ⏺
          </button>
        </div>

        <div className="track-control-group">
          <label>Volume</label>
          <input
            type="range"
            className="slider"
            data-testid="track-volume-slider"
            min="0"
            max="1"
            step="0.01"
            value={track.volume}
            onChange={handleVolumeChange}
          />
          <span className="slider-value">{Math.round(track.volume * 100)}%</span>
        </div>

        <div className="track-control-group">
          <label>Pan</label>
          <input
            type="range"
            className="slider"
            data-testid="track-pan-slider"
            min="0"
            max="1"
            step="0.01"
            value={track.pan}
            onChange={handlePanChange}
          />
          <span className="slider-value">
            {track.pan < 0.5 ? `L${Math.round((0.5 - track.pan) * 200)}` :
             track.pan > 0.5 ? `R${Math.round((track.pan - 0.5) * 200)}` :
             'C'}
          </span>
        </div>

        <div className="track-actions">
          <button
            className="btn-track-icon"
            data-testid="track-effects-button"
            title="Effects"
          >
            🎛️
          </button>
          <button
            className="btn-track-icon"
            data-testid="track-upload-button"
            title="Upload Audio"
          >
            📁
          </button>
          <button
            className="btn-track-icon"
            data-testid="track-menu-button"
            title="More Options"
          >
            ⋮
          </button>
        </div>
      </div>

      {/* Peak Meters */}
      <div className="track-meters">
        <div className="meter" data-testid="peak-meter" data-level="-20">
          <div className="meter-bar" style={{ height: '40%' }}></div>
        </div>
        <div className="meter" data-testid="rms-meter">
          <div className="meter-bar" style={{ height: '30%' }}></div>
        </div>
      </div>
    </div>
  );
};
