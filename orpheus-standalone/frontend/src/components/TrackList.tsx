import React, { useState } from 'react';
import { Track } from './Track';
import './TrackList.css';

interface TrackData {
  id: string;
  projectId: string;
  name: string;
  instrument: string;
  trackNumber: number;
  volume: number;
  pan: number;
  muted: boolean;
  soloed: boolean;
  audioFileUrl?: string;
}

interface TrackListProps {
  tracks: TrackData[];
  onAddTrack: (track: Partial<TrackData>) => void;
  onUpdateTrack: (trackId: string, updates: Partial<TrackData>) => void;
  onDeleteTrack: (trackId: string) => void;
}

export const TrackList: React.FC<TrackListProps> = ({
  tracks,
  onAddTrack,
  onUpdateTrack,
  onDeleteTrack,
}) => {
  const [showNewTrackDialog, setShowNewTrackDialog] = useState(false);
  const [newTrackData, setNewTrackData] = useState({
    name: '',
    instrument: 'electric-guitar',
    trackNumber: tracks.length + 1,
  });

  const handleCreateTrack = (e: React.FormEvent) => {
    e.preventDefault();
    onAddTrack({
      ...newTrackData,
      volume: 0.75,
      pan: 0.5,
      muted: false,
      soloed: false,
    });
    setShowNewTrackDialog(false);
    setNewTrackData({
      name: '',
      instrument: 'electric-guitar',
      trackNumber: tracks.length + 2,
    });
  };

  return (
    <div className="track-list" data-testid="track-list">
      <div className="track-list-header">
        <h3>Tracks</h3>
        <button
          className="btn-add-track"
          data-testid="add-track-button"
          onClick={() => setShowNewTrackDialog(true)}
          title="Add Track"
        >
          +
        </button>
      </div>

      {tracks.length === 0 ? (
        <div className="track-list-empty">
          <p>No tracks yet</p>
          <button
            className="btn-primary-small"
            onClick={() => setShowNewTrackDialog(true)}
          >
            Add Track
          </button>
        </div>
      ) : (
        tracks
          .sort((a, b) => a.trackNumber - b.trackNumber)
          .map(track => (
            <Track
              key={track.id}
              track={track}
              onUpdate={(updates) => onUpdateTrack(track.id, updates)}
              onDelete={() => onDeleteTrack(track.id)}
            />
          ))
      )}

      {/* New Track Dialog */}
      {showNewTrackDialog && (
        <div className="modal-overlay" onClick={() => setShowNewTrackDialog(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h2>Add New Track</h2>

            <form onSubmit={handleCreateTrack}>
              <div className="form-group">
                <label htmlFor="trackName">Track Name</label>
                <input
                  type="text"
                  id="trackName"
                  data-testid="track-name-input"
                  value={newTrackData.name}
                  onChange={(e) =>
                    setNewTrackData({ ...newTrackData, name: e.target.value })
                  }
                  required
                  autoFocus
                  placeholder="Guitar, Bass, Drums..."
                />
              </div>

              <div className="form-group">
                <label htmlFor="instrument">Instrument</label>
                <select
                  id="instrument"
                  data-testid="track-instrument-select"
                  value={newTrackData.instrument}
                  onChange={(e) =>
                    setNewTrackData({ ...newTrackData, instrument: e.target.value })
                  }
                >
                  <option value="electric-guitar">Electric Guitar</option>
                  <option value="acoustic-guitar">Acoustic Guitar</option>
                  <option value="bass">Bass</option>
                  <option value="drums">Drums</option>
                  <option value="piano">Piano</option>
                  <option value="synth">Synthesizer</option>
                  <option value="vocals">Vocals</option>
                  <option value="strings">Strings</option>
                  <option value="brass">Brass</option>
                  <option value="woodwinds">Woodwinds</option>
                  <option value="percussion">Percussion</option>
                  <option value="other">Other</option>
                </select>
              </div>

              <div className="modal-actions">
                <button
                  type="button"
                  className="btn-secondary"
                  onClick={() => setShowNewTrackDialog(false)}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="btn-primary"
                  data-testid="create-track-submit"
                >
                  Add Track
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
