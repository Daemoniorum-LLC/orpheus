/**
 * JamPlayer - Voice-controlled backing track player for jamming
 * Search for backing tracks and play along with your instrument
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  type MusicTrack,
  type MusicSearchResult,
  subscribeToPlayback,
  playTrack,
  pausePlayback,
  resumePlayback,
  stopPlayback,
  getPlaybackState,
  getSuggestedSearches,
  searchMusic,
  parseNaturalMusicQuery,
  MUSIC_GENRES,
  MUSICAL_KEYS,
} from '../services/music-search';
import { setOnSearchResults, getMusicVoiceState } from '../services/voice-commands-music';
import { showSuccess, showInfo } from '../services/toast';

interface JamPlayerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function JamPlayer({ isOpen, onClose }: JamPlayerProps) {
  const [searchResults, setSearchResults] = useState<MusicTrack[]>([]);
  const [currentTrack, setCurrentTrack] = useState<MusicTrack | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedGenre, setSelectedGenre] = useState<string>('');
  const [selectedKey, setSelectedKey] = useState<string>('');
  const [isSearching, setIsSearching] = useState(false);
  const [showSuggestions, setShowSuggestions] = useState(true);

  // Subscribe to playback state changes
  useEffect(() => {
    const unsubscribe = subscribeToPlayback((state) => {
      setCurrentTrack(state.currentTrack);
      setIsPlaying(state.isPlaying);
    });

    return unsubscribe;
  }, []);

  // Register for voice search results
  useEffect(() => {
    setOnSearchResults((results: MusicSearchResult) => {
      setSearchResults(results.tracks);
      setShowSuggestions(false);
    });

    return () => setOnSearchResults(null);
  }, []);

  // Sync with voice state
  useEffect(() => {
    const interval = setInterval(() => {
      const voiceState = getMusicVoiceState();
      if (voiceState.lastSearchResults.length > 0 && voiceState.lastSearchResults !== searchResults) {
        setSearchResults(voiceState.lastSearchResults);
        setShowSuggestions(false);
      }
    }, 500);

    return () => clearInterval(interval);
  }, [searchResults]);

  const handleSearch = useCallback(async () => {
    if (!searchQuery && !selectedGenre && !selectedKey) {
      showInfo('Enter a search term or select filters');
      return;
    }

    setIsSearching(true);
    setShowSuggestions(false);

    try {
      const queryText = [
        searchQuery,
        selectedGenre,
        selectedKey ? `in ${selectedKey}` : '',
      ].filter(Boolean).join(' ');

      const query = parseNaturalMusicQuery(queryText);
      if (selectedGenre) query.genre = selectedGenre;
      if (selectedKey) query.key = selectedKey;

      const results = await searchMusic(query);
      setSearchResults(results.tracks);

      if (results.tracks.length > 0) {
        showSuccess(`Found ${results.tracks.length} tracks!`);
      } else {
        showInfo('No tracks found. Try different filters.');
      }
    } catch (e) {
      console.error('Search failed:', e);
    }

    setIsSearching(false);
  }, [searchQuery, selectedGenre, selectedKey]);

  const handleSuggestionClick = useCallback(async (suggestion: ReturnType<typeof getSuggestedSearches>[0]) => {
    setIsSearching(true);
    setShowSuggestions(false);

    try {
      const results = await searchMusic(suggestion);
      setSearchResults(results.tracks);

      if (results.tracks.length > 0) {
        showSuccess(`Found ${results.tracks.length} ${suggestion.genre} tracks!`);
      }
    } catch (e) {
      console.error('Search failed:', e);
    }

    setIsSearching(false);
  }, []);

  const handlePlayTrack = useCallback((track: MusicTrack) => {
    playTrack(track);
  }, []);

  const handlePlayPause = useCallback(() => {
    if (isPlaying) {
      pausePlayback();
    } else {
      resumePlayback();
    }
  }, [isPlaying]);

  const handleStop = useCallback(() => {
    stopPlayback();
  }, []);

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  if (!isOpen) return null;

  const suggestions = getSuggestedSearches();

  return (
    <div className="jam-player-overlay">
      <div className="jam-player">
        {/* Header */}
        <div className="jam-player-header">
          <div className="jam-player-title">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M9 18V5l12-2v13" />
              <circle cx="6" cy="18" r="3" />
              <circle cx="18" cy="16" r="3" />
            </svg>
            <span>Jam Player</span>
          </div>
          <button className="jam-player-close" onClick={onClose}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Voice hint */}
        <div className="jam-player-voice-hint">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
            <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
            <line x1="12" y1="19" x2="12" y2="23" />
          </svg>
          <span>Say: "Find blues in A" or "Play some funk"</span>
        </div>

        {/* Search bar */}
        <div className="jam-player-search">
          <input
            type="text"
            placeholder="Search for backing tracks..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
          />
          <button onClick={handleSearch} disabled={isSearching}>
            {isSearching ? (
              <div className="spinner-small" />
            ) : (
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="11" cy="11" r="8" />
                <path d="m21 21-4.35-4.35" />
              </svg>
            )}
          </button>
        </div>

        {/* Filters */}
        <div className="jam-player-filters">
          <select
            value={selectedGenre}
            onChange={(e) => setSelectedGenre(e.target.value)}
          >
            <option value="">Any Genre</option>
            {MUSIC_GENRES.map(genre => (
              <option key={genre} value={genre}>
                {genre.charAt(0).toUpperCase() + genre.slice(1)}
              </option>
            ))}
          </select>

          <select
            value={selectedKey}
            onChange={(e) => setSelectedKey(e.target.value)}
          >
            <option value="">Any Key</option>
            {MUSICAL_KEYS.slice(0, 17).map(key => (
              <option key={key} value={key}>{key}</option>
            ))}
            <option disabled>───────</option>
            {MUSICAL_KEYS.slice(17).map(key => (
              <option key={key} value={key}>{key}</option>
            ))}
          </select>
        </div>

        {/* Suggestions or Results */}
        <div className="jam-player-content">
          {showSuggestions && searchResults.length === 0 ? (
            <div className="jam-player-suggestions">
              <h4>Quick Start</h4>
              <div className="suggestion-grid">
                {suggestions.map((suggestion, i) => (
                  <button
                    key={i}
                    className="suggestion-card"
                    onClick={() => handleSuggestionClick(suggestion)}
                  >
                    <span className="suggestion-genre">{suggestion.genre}</span>
                    {suggestion.key && <span className="suggestion-key">{suggestion.key}</span>}
                  </button>
                ))}
              </div>
            </div>
          ) : (
            <div className="jam-player-results">
              {searchResults.length === 0 && !isSearching && (
                <div className="no-results">
                  <p>No tracks found</p>
                  <button onClick={() => setShowSuggestions(true)}>
                    Show suggestions
                  </button>
                </div>
              )}

              {searchResults.map((track, index) => (
                <div
                  key={track.id}
                  className={`track-item ${currentTrack?.id === track.id ? 'active' : ''}`}
                  onClick={() => handlePlayTrack(track)}
                >
                  <div className="track-number">{index + 1}</div>
                  <img
                    src={track.thumbnail}
                    alt=""
                    className="track-thumbnail"
                    onError={(e) => {
                      (e.target as HTMLImageElement).src = 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect fill="%23333" width="100" height="100"/><text x="50" y="55" text-anchor="middle" fill="%23666" font-size="30">♪</text></svg>';
                    }}
                  />
                  <div className="track-info">
                    <div className="track-title">{track.title}</div>
                    <div className="track-artist">{track.artist}</div>
                  </div>
                  <div className="track-meta">
                    {track.key && <span className="track-key">{track.key}</span>}
                    {track.tempo && <span className="track-tempo">{track.tempo} BPM</span>}
                  </div>
                  <div className="track-duration">{formatDuration(track.duration)}</div>
                  {currentTrack?.id === track.id && isPlaying && (
                    <div className="playing-indicator">
                      <div className="bar"></div>
                      <div className="bar"></div>
                      <div className="bar"></div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Now Playing bar */}
        {currentTrack && (
          <div className="jam-player-now-playing">
            <img
              src={currentTrack.thumbnail}
              alt=""
              className="now-playing-thumb"
              onError={(e) => {
                (e.target as HTMLImageElement).src = 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect fill="%23333" width="100" height="100"/><text x="50" y="55" text-anchor="middle" fill="%23666" font-size="30">♪</text></svg>';
              }}
            />
            <div className="now-playing-info">
              <div className="now-playing-title">{currentTrack.title}</div>
              <div className="now-playing-artist">{currentTrack.artist}</div>
            </div>
            <div className="now-playing-controls">
              <button className="control-btn" onClick={handleStop}>
                <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                  <rect x="6" y="6" width="12" height="12" rx="1" />
                </svg>
              </button>
              <button className="control-btn primary" onClick={handlePlayPause}>
                {isPlaying ? (
                  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                    <rect x="6" y="5" width="4" height="14" rx="1" />
                    <rect x="14" y="5" width="4" height="14" rx="1" />
                  </svg>
                ) : (
                  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                    <polygon points="5,3 19,12 5,21" />
                  </svg>
                )}
              </button>
            </div>
          </div>
        )}
      </div>

      <style>{`
        .jam-player-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.7);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
          animation: fadeIn 0.2s ease-out;
        }

        @keyframes fadeIn {
          from { opacity: 0; }
          to { opacity: 1; }
        }

        .jam-player {
          background: var(--surface, #1a1a2e);
          border-radius: 16px;
          width: 90%;
          max-width: 600px;
          max-height: 85vh;
          display: flex;
          flex-direction: column;
          overflow: hidden;
          box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
          animation: slideUp 0.3s ease-out;
        }

        @keyframes slideUp {
          from { transform: translateY(20px); opacity: 0; }
          to { transform: translateY(0); opacity: 1; }
        }

        .jam-player-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 16px 20px;
          border-bottom: 1px solid var(--border, #2a2a4a);
        }

        .jam-player-title {
          display: flex;
          align-items: center;
          gap: 10px;
          font-size: 18px;
          font-weight: 600;
          color: var(--text-primary, #fff);
        }

        .jam-player-title svg {
          color: var(--accent, #7c3aed);
        }

        .jam-player-close {
          background: transparent;
          border: none;
          padding: 8px;
          cursor: pointer;
          color: var(--text-secondary, #888);
          border-radius: 8px;
          transition: all 0.2s;
        }

        .jam-player-close:hover {
          background: var(--hover, #2a2a4a);
          color: var(--text-primary, #fff);
        }

        .jam-player-voice-hint {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 10px 20px;
          background: var(--accent, #7c3aed)15;
          color: var(--accent, #7c3aed);
          font-size: 13px;
        }

        .jam-player-search {
          display: flex;
          gap: 8px;
          padding: 16px 20px;
        }

        .jam-player-search input {
          flex: 1;
          padding: 12px 16px;
          border: 1px solid var(--border, #2a2a4a);
          border-radius: 8px;
          background: var(--input-bg, #0a0a1a);
          color: var(--text-primary, #fff);
          font-size: 14px;
        }

        .jam-player-search input:focus {
          outline: none;
          border-color: var(--accent, #7c3aed);
        }

        .jam-player-search button {
          padding: 12px 16px;
          background: var(--accent, #7c3aed);
          border: none;
          border-radius: 8px;
          color: white;
          cursor: pointer;
          transition: all 0.2s;
        }

        .jam-player-search button:hover:not(:disabled) {
          background: var(--accent-hover, #6d28d9);
        }

        .jam-player-search button:disabled {
          opacity: 0.7;
          cursor: not-allowed;
        }

        .jam-player-filters {
          display: flex;
          gap: 8px;
          padding: 0 20px 16px;
        }

        .jam-player-filters select {
          flex: 1;
          padding: 10px 12px;
          border: 1px solid var(--border, #2a2a4a);
          border-radius: 8px;
          background: var(--input-bg, #0a0a1a);
          color: var(--text-primary, #fff);
          font-size: 13px;
          cursor: pointer;
        }

        .jam-player-content {
          flex: 1;
          overflow-y: auto;
          padding: 0 20px 20px;
        }

        .jam-player-suggestions h4 {
          margin: 0 0 12px;
          font-size: 14px;
          color: var(--text-secondary, #888);
          font-weight: 500;
        }

        .suggestion-grid {
          display: grid;
          grid-template-columns: repeat(3, 1fr);
          gap: 10px;
        }

        .suggestion-card {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 4px;
          padding: 16px 12px;
          background: var(--card-bg, #151528);
          border: 1px solid var(--border, #2a2a4a);
          border-radius: 12px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .suggestion-card:hover {
          background: var(--hover, #2a2a4a);
          border-color: var(--accent, #7c3aed);
          transform: translateY(-2px);
        }

        .suggestion-genre {
          font-size: 14px;
          font-weight: 500;
          color: var(--text-primary, #fff);
          text-transform: capitalize;
        }

        .suggestion-key {
          font-size: 12px;
          color: var(--accent, #7c3aed);
          font-weight: 600;
        }

        .no-results {
          text-align: center;
          padding: 40px 20px;
          color: var(--text-secondary, #888);
        }

        .no-results button {
          margin-top: 12px;
          padding: 8px 16px;
          background: var(--accent, #7c3aed);
          border: none;
          border-radius: 6px;
          color: white;
          cursor: pointer;
        }

        .track-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px;
          border-radius: 10px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .track-item:hover {
          background: var(--hover, #2a2a4a);
        }

        .track-item.active {
          background: var(--accent, #7c3aed)20;
        }

        .track-number {
          width: 24px;
          text-align: center;
          font-size: 13px;
          color: var(--text-secondary, #888);
        }

        .track-thumbnail {
          width: 48px;
          height: 48px;
          border-radius: 6px;
          object-fit: cover;
          background: var(--card-bg, #151528);
        }

        .track-info {
          flex: 1;
          min-width: 0;
        }

        .track-title {
          font-size: 14px;
          font-weight: 500;
          color: var(--text-primary, #fff);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .track-artist {
          font-size: 12px;
          color: var(--text-secondary, #888);
        }

        .track-meta {
          display: flex;
          gap: 8px;
        }

        .track-key, .track-tempo {
          font-size: 11px;
          padding: 3px 8px;
          border-radius: 4px;
          background: var(--card-bg, #151528);
          color: var(--text-secondary, #888);
        }

        .track-key {
          color: var(--accent, #7c3aed);
        }

        .track-duration {
          font-size: 12px;
          color: var(--text-secondary, #888);
          min-width: 40px;
          text-align: right;
        }

        .playing-indicator {
          display: flex;
          gap: 2px;
          align-items: flex-end;
          height: 16px;
        }

        .playing-indicator .bar {
          width: 3px;
          background: var(--accent, #7c3aed);
          border-radius: 2px;
          animation: sound 0.5s infinite ease-in-out;
        }

        .playing-indicator .bar:nth-child(1) { height: 60%; animation-delay: 0s; }
        .playing-indicator .bar:nth-child(2) { height: 100%; animation-delay: 0.1s; }
        .playing-indicator .bar:nth-child(3) { height: 40%; animation-delay: 0.2s; }

        @keyframes sound {
          0%, 100% { transform: scaleY(0.5); }
          50% { transform: scaleY(1); }
        }

        .jam-player-now-playing {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px 20px;
          background: var(--card-bg, #151528);
          border-top: 1px solid var(--border, #2a2a4a);
        }

        .now-playing-thumb {
          width: 48px;
          height: 48px;
          border-radius: 6px;
          object-fit: cover;
        }

        .now-playing-info {
          flex: 1;
          min-width: 0;
        }

        .now-playing-title {
          font-size: 14px;
          font-weight: 500;
          color: var(--text-primary, #fff);
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .now-playing-artist {
          font-size: 12px;
          color: var(--text-secondary, #888);
        }

        .now-playing-controls {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .control-btn {
          background: transparent;
          border: none;
          padding: 10px;
          cursor: pointer;
          color: var(--text-secondary, #888);
          border-radius: 50%;
          transition: all 0.2s;
        }

        .control-btn:hover {
          background: var(--hover, #2a2a4a);
          color: var(--text-primary, #fff);
        }

        .control-btn.primary {
          background: var(--accent, #7c3aed);
          color: white;
        }

        .control-btn.primary:hover {
          background: var(--accent-hover, #6d28d9);
        }

        .spinner-small {
          width: 18px;
          height: 18px;
          border: 2px solid transparent;
          border-top-color: white;
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }

        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

export default JamPlayer;
