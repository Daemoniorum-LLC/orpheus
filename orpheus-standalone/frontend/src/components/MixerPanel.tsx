import React, { useState } from 'react';
import './MixerPanel.css';

interface Track {
  id: string;
  name: string;
  volume: number;
  pan: number;
  muted: boolean;
  soloed: boolean;
  peakLevel: number;
  rmsLevel: number;
  effects: Effect[];
}

interface Effect {
  id: string;
  type: string;
  name: string;
  enabled: boolean;
  parameters: Record<string, number>;
}

interface MixerPanelProps {
  tracks: Track[];
  onUpdateTrack: (trackId: string, updates: Partial<Track>) => void;
}

export const MixerPanel: React.FC<MixerPanelProps> = ({ tracks, onUpdateTrack }) => {
  const [selectedTrack, setSelectedTrack] = useState<string | null>(null);
  const [showEffects, setShowEffects] = useState(false);
  const [showMasterBus, setShowMasterBus] = useState(false);

  const handleVolumeChange = (trackId: string, volume: number) => {
    onUpdateTrack(trackId, { volume });
  };

  const handlePanChange = (trackId: string, pan: number) => {
    onUpdateTrack(trackId, { pan });
  };

  const handleEffectsClick = (trackId: string) => {
    setSelectedTrack(trackId);
    setShowEffects(true);
  };

  const handleAddEffect = async (effectType: string) => {
    if (!selectedTrack) return;

    const newEffect: Effect = {
      id: `effect-${Date.now()}`,
      type: effectType,
      name: effectType.charAt(0).toUpperCase() + effectType.slice(1),
      enabled: true,
      parameters: {},
    };

    const track = tracks.find(t => t.id === selectedTrack);
    if (track) {
      const updatedEffects = [...track.effects, newEffect];
      onUpdateTrack(selectedTrack, { effects: updatedEffects });
    }
  };

  const handleRemoveEffect = (trackId: string, effectId: string) => {
    const track = tracks.find(t => t.id === trackId);
    if (track) {
      const updatedEffects = track.effects.filter(e => e.id !== effectId);
      onUpdateTrack(trackId, { effects: updatedEffects });
    }
  };

  const handleToggleEffect = (trackId: string, effectId: string) => {
    const track = tracks.find(t => t.id === trackId);
    if (track) {
      const updatedEffects = track.effects.map(e =>
        e.id === effectId ? { ...e, enabled: !e.enabled } : e
      );
      onUpdateTrack(trackId, { effects: updatedEffects });
    }
  };

  const getPanLabel = (pan: number): string => {
    if (pan === 0) return 'C';
    if (pan < 0) return `L${Math.abs(Math.round(pan * 100))}`;
    return `R${Math.round(pan * 100)}`;
  };

  return (
    <div className="mixer-panel" data-testid="mixer-panel">
      <div className="mixer-header">
        <h3>Mixer</h3>
        <button
          className="master-bus-button"
          onClick={() => setShowMasterBus(!showMasterBus)}
          data-testid="master-bus"
        >
          Master
        </button>
      </div>

      <div className="mixer-channels">
        {tracks.map((track) => (
          <div
            key={track.id}
            className={`mixer-channel ${selectedTrack === track.id ? 'selected' : ''}`}
            data-testid="mixer-channel"
          >
            <div className="channel-header">
              <span className="channel-name">{track.name}</span>
              <button
                className="effects-button"
                onClick={() => handleEffectsClick(track.id)}
                data-testid="track-effects-button"
              >
                FX ({track.effects.length})
              </button>
            </div>

            <div className="channel-meter">
              <div className="meter-scale">
                <div className="meter-markers">
                  <span>0</span>
                  <span>-6</span>
                  <span>-12</span>
                  <span>-24</span>
                  <span>-∞</span>
                </div>
                <div className="meter-bars">
                  <div
                    className="peak-meter"
                    style={{ height: `${track.peakLevel * 100}%` }}
                    data-testid="peak-meter"
                  />
                  <div
                    className="rms-meter"
                    style={{ height: `${track.rmsLevel * 100}%` }}
                    data-testid="rms-meter"
                  />
                </div>
              </div>
            </div>

            <div className="channel-pan">
              <label>Pan</label>
              <input
                type="range"
                min="-1"
                max="1"
                step="0.01"
                value={track.pan}
                onChange={(e) => handlePanChange(track.id, parseFloat(e.target.value))}
                data-testid="track-pan-slider"
              />
              <span className="pan-value">{getPanLabel(track.pan)}</span>
            </div>

            <div className="channel-fader">
              <label>Volume</label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={track.volume}
                orient="vertical"
                onChange={(e) => handleVolumeChange(track.id, parseFloat(e.target.value))}
                data-testid="track-volume-slider"
              />
              <span className="volume-value">{Math.round(track.volume * 100)}%</span>
            </div>

            <div className="channel-effects">
              {track.effects.map((effect) => (
                <div
                  key={effect.id}
                  className={`effect-chip ${!effect.enabled ? 'disabled' : ''}`}
                  data-testid="effect-chip"
                >
                  <button
                    className="effect-toggle"
                    onClick={() => handleToggleEffect(track.id, effect.id)}
                    data-testid="effect-bypass"
                  >
                    {effect.name}
                  </button>
                  <button
                    className="effect-remove"
                    onClick={() => handleRemoveEffect(track.id, effect.id)}
                    data-testid="remove-effect"
                  >
                    ×
                  </button>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>

      {showEffects && selectedTrack && (
        <div className="effects-panel" data-testid="effects-panel">
          <div className="effects-header">
            <h4>Effects - {tracks.find(t => t.id === selectedTrack)?.name}</h4>
            <button onClick={() => setShowEffects(false)}>×</button>
          </div>

          <div className="effects-add">
            <button onClick={() => setShowEffects(false)} data-testid="add-effect-button">
              Add Effect
            </button>
            <div className="effects-menu">
              <button onClick={() => handleAddEffect('eq')} data-testid="effect-eq">
                EQ
              </button>
              <button onClick={() => handleAddEffect('compressor')} data-testid="effect-compressor">
                Compressor
              </button>
              <button onClick={() => handleAddEffect('reverb')} data-testid="effect-reverb">
                Reverb
              </button>
              <button onClick={() => handleAddEffect('delay')} data-testid="effect-delay">
                Delay
              </button>
              <button onClick={() => handleAddEffect('distortion')} data-testid="effect-distortion">
                Distortion
              </button>
              <button onClick={() => handleAddEffect('limiter')} data-testid="effect-limiter">
                Limiter
              </button>
            </div>
          </div>

          <div className="effects-chain">
            {tracks
              .find(t => t.id === selectedTrack)
              ?.effects.map((effect) => (
                <div key={effect.id} className="effect-slot" data-testid="effect-slot">
                  <div className="effect-name">
                    {effect.name}
                    <button
                      className={`bypass-button ${!effect.enabled ? 'active' : ''}`}
                      onClick={() => handleToggleEffect(selectedTrack, effect.id)}
                      data-testid="effect-bypass"
                    >
                      Bypass
                    </button>
                  </div>
                  <button
                    className="remove-button"
                    onClick={() => handleRemoveEffect(selectedTrack, effect.id)}
                    data-testid="remove-effect"
                  >
                    Remove
                  </button>
                </div>
              ))}
          </div>
        </div>
      )}

      {showMasterBus && (
        <div className="master-bus-panel" data-testid="master-bus-panel">
          <div className="master-header">
            <h4>Master Bus</h4>
            <button onClick={() => setShowMasterBus(false)}>×</button>
          </div>

          <div className="master-meter">
            <div className="stereo-meter">
              <div className="meter-channel">
                <span>L</span>
                <div className="meter-bar" />
              </div>
              <div className="meter-channel">
                <span>R</span>
                <div className="meter-bar" />
              </div>
            </div>
          </div>

          <div className="master-controls">
            <button data-testid="add-effect-button">Add Effect</button>
          </div>
        </div>
      )}
    </div>
  );
};
