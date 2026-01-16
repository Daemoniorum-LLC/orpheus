/**
 * Voice Commands for Music Search - Find backing tracks and songs by voice
 * "Find me a blues backing track in A" -> searches and plays
 */

import { type VoiceCommand } from './voice-commands';
import { getAudioFeedback } from './audio-feedback';
import {
  searchMusic,
  parseNaturalMusicQuery,
  playTrack,
  pausePlayback,
  stopPlayback,
  resumePlayback,
  getPlaybackState,
  type MusicTrack,
  type MusicSearchResult,
} from './music-search';
import { showSuccess, showInfo, showError } from './toast';
import { useAppStore } from '../store/app-store';

// Store for search results and current track
interface MusicVoiceState {
  lastSearchResults: MusicTrack[];
  currentTrackIndex: number;
  isSearching: boolean;
}

const musicVoiceState: MusicVoiceState = {
  lastSearchResults: [],
  currentTrackIndex: -1,
  isSearching: false,
};

// Callback for when search results are ready (to open UI)
type SearchResultsCallback = (results: MusicSearchResult) => void;
let onSearchResultsCallback: SearchResultsCallback | null = null;

export function setOnSearchResults(callback: SearchResultsCallback | null): void {
  onSearchResultsCallback = callback;
}

// Callback to toggle the JamPlayer UI
type JamPlayerToggleCallback = (open: boolean) => void;
let jamPlayerToggleCallback: JamPlayerToggleCallback | null = null;

export function setJamPlayerToggle(callback: JamPlayerToggleCallback | null): void {
  jamPlayerToggleCallback = callback;
}

export function openJamPlayer(): void {
  if (jamPlayerToggleCallback) {
    jamPlayerToggleCallback(true);
  }
}

export function closeJamPlayer(): void {
  if (jamPlayerToggleCallback) {
    jamPlayerToggleCallback(false);
  }
}

/**
 * Parse music search from voice command
 */
function extractMusicSearch(transcript: string): string | null {
  // Patterns to extract the search query
  const patterns = [
    /(?:find|search|play|get)\s+(?:me\s+)?(?:a\s+)?(?:some\s+)?(.+?)(?:\s+(?:to|for)\s+(?:jam|play))?$/i,
    /(?:i\s+want\s+to\s+jam\s+(?:to|with)\s+)(.+)/i,
    /(?:let's\s+jam\s+(?:to|with)\s+)(.+)/i,
    /(?:put\s+on\s+)(.+)/i,
    /(?:backing\s+track\s+(?:in|for)\s+)(.+)/i,
  ];

  for (const pattern of patterns) {
    const match = transcript.match(pattern);
    if (match && match[1]) {
      return match[1].trim();
    }
  }

  // If no pattern matches, use the whole transcript
  return transcript;
}

/**
 * Get music search voice commands
 */
export function getMusicVoiceCommands(): VoiceCommand[] {
  const audioFeedback = getAudioFeedback();

  return [
    // ========================================
    // JAM PLAYER UI COMMANDS
    // ========================================
    {
      id: 'open-jam-player',
      phrases: [
        'open jam player', 'show jam player', 'jam player',
        'open music', 'show music', 'music player',
        'i want to jam', 'let\'s find something', 'find music'
      ],
      category: 'system',
      description: 'Open the Jam Player',
      action: () => {
        openJamPlayer();
        audioFeedback.play('confirm');
        showInfo('Opening Jam Player');
      },
    },
    {
      id: 'close-jam-player',
      phrases: [
        'close jam player', 'hide jam player', 'close music',
        'hide music', 'done jamming', 'close player'
      ],
      category: 'system',
      description: 'Close the Jam Player',
      action: () => {
        closeJamPlayer();
        audioFeedback.play('confirm');
        showInfo('Jam Player closed');
      },
    },

    // ========================================
    // SEARCH COMMANDS
    // ========================================
    {
      id: 'find-backing-track',
      phrases: [
        'find', 'find me', 'search for', 'search', 'look for',
        'get me', 'play some', 'i want', "i'd like",
        'backing track', 'jam track', 'play along'
      ],
      category: 'system',
      description: 'Search for backing tracks to jam with',
      action: async () => {
        // This is a partial match - the actual search term follows
        showInfo('Say what you want to jam to! (e.g., "blues in A", "rock in E minor")');
        audioFeedback.play('listening');
      },
    },
    {
      id: 'find-blues',
      phrases: [
        'find blues', 'blues backing track', 'play some blues',
        'blues in', 'blues jam', 'twelve bar blues', '12 bar blues',
        'shuffle blues', 'slow blues', 'fast blues'
      ],
      category: 'system',
      description: 'Search for blues backing tracks',
      action: async () => {
        musicVoiceState.isSearching = true;
        showInfo('Searching for blues...');
        audioFeedback.play('confirm');

        try {
          const query = parseNaturalMusicQuery('blues backing track');
          query.genre = 'blues';
          const results = await searchMusic(query);
          handleSearchResults(results);
        } catch (e) {
          showError('Search failed');
          audioFeedback.play('error');
        }
        musicVoiceState.isSearching = false;
      },
    },
    {
      id: 'find-rock',
      phrases: [
        'find rock', 'rock backing track', 'play some rock',
        'rock in', 'rock jam', 'hard rock', 'classic rock'
      ],
      category: 'system',
      description: 'Search for rock backing tracks',
      action: async () => {
        musicVoiceState.isSearching = true;
        showInfo('Searching for rock...');
        audioFeedback.play('confirm');

        try {
          const query = parseNaturalMusicQuery('rock backing track');
          query.genre = 'rock';
          const results = await searchMusic(query);
          handleSearchResults(results);
        } catch (e) {
          showError('Search failed');
          audioFeedback.play('error');
        }
        musicVoiceState.isSearching = false;
      },
    },
    {
      id: 'find-jazz',
      phrases: [
        'find jazz', 'jazz backing track', 'play some jazz',
        'jazz in', 'jazz jam', 'smooth jazz', 'bebop', 'jazz fusion'
      ],
      category: 'system',
      description: 'Search for jazz backing tracks',
      action: async () => {
        musicVoiceState.isSearching = true;
        showInfo('Searching for jazz...');
        audioFeedback.play('confirm');

        try {
          const query = parseNaturalMusicQuery('jazz backing track');
          query.genre = 'jazz';
          const results = await searchMusic(query);
          handleSearchResults(results);
        } catch (e) {
          showError('Search failed');
          audioFeedback.play('error');
        }
        musicVoiceState.isSearching = false;
      },
    },
    {
      id: 'find-funk',
      phrases: [
        'find funk', 'funk backing track', 'play some funk',
        'funk in', 'funk jam', 'funky', 'groove'
      ],
      category: 'system',
      description: 'Search for funk backing tracks',
      action: async () => {
        musicVoiceState.isSearching = true;
        showInfo('Searching for funk...');
        audioFeedback.play('confirm');

        try {
          const query = parseNaturalMusicQuery('funk backing track');
          query.genre = 'funk';
          const results = await searchMusic(query);
          handleSearchResults(results);
        } catch (e) {
          showError('Search failed');
          audioFeedback.play('error');
        }
        musicVoiceState.isSearching = false;
      },
    },
    {
      id: 'find-metal',
      phrases: [
        'find metal', 'metal backing track', 'play some metal',
        'metal in', 'metal jam', 'heavy metal', 'thrash'
      ],
      category: 'system',
      description: 'Search for metal backing tracks',
      action: async () => {
        musicVoiceState.isSearching = true;
        showInfo('Searching for metal...');
        audioFeedback.play('confirm');

        try {
          const query = parseNaturalMusicQuery('metal backing track');
          query.genre = 'metal';
          const results = await searchMusic(query);
          handleSearchResults(results);
        } catch (e) {
          showError('Search failed');
          audioFeedback.play('error');
        }
        musicVoiceState.isSearching = false;
      },
    },

    // Key-specific searches
    {
      id: 'find-in-a',
      phrases: [
        'in a', 'in a major', 'key of a', 'a major', 'jam in a'
      ],
      category: 'system',
      description: 'Search in key of A',
      action: async () => {
        await searchByKey('A');
      },
    },
    {
      id: 'find-in-am',
      phrases: [
        'in a minor', 'a minor', 'jam in a minor', 'key of a minor'
      ],
      category: 'system',
      description: 'Search in key of Am',
      action: async () => {
        await searchByKey('Am');
      },
    },
    {
      id: 'find-in-e',
      phrases: [
        'in e', 'in e major', 'key of e', 'e major', 'jam in e'
      ],
      category: 'system',
      description: 'Search in key of E',
      action: async () => {
        await searchByKey('E');
      },
    },
    {
      id: 'find-in-em',
      phrases: [
        'in e minor', 'e minor', 'jam in e minor', 'key of e minor'
      ],
      category: 'system',
      description: 'Search in key of Em',
      action: async () => {
        await searchByKey('Em');
      },
    },
    {
      id: 'find-in-g',
      phrases: [
        'in g', 'in g major', 'key of g', 'g major', 'jam in g'
      ],
      category: 'system',
      description: 'Search in key of G',
      action: async () => {
        await searchByKey('G');
      },
    },
    {
      id: 'find-in-d',
      phrases: [
        'in d', 'in d major', 'key of d', 'd major', 'jam in d'
      ],
      category: 'system',
      description: 'Search in key of D',
      action: async () => {
        await searchByKey('D');
      },
    },
    {
      id: 'find-in-dm',
      phrases: [
        'in d minor', 'd minor', 'jam in d minor', 'key of d minor'
      ],
      category: 'system',
      description: 'Search in key of Dm',
      action: async () => {
        await searchByKey('Dm');
      },
    },

    // ========================================
    // PLAYBACK COMMANDS FOR JAM TRACKS
    // ========================================
    {
      id: 'play-this-track',
      phrases: [
        'play this', 'play it', 'play that', 'start this',
        'jam to this', "let's hear it"
      ],
      category: 'playback',
      description: 'Play the current/first search result',
      action: () => {
        if (musicVoiceState.lastSearchResults.length > 0) {
          const track = musicVoiceState.lastSearchResults[0];
          playTrack(track);
          musicVoiceState.currentTrackIndex = 0;
          audioFeedback.play('success');
          showSuccess(`Playing: ${track.title}`);
        } else {
          showInfo('No track to play. Search for something first!');
        }
      },
    },
    {
      id: 'next-track',
      phrases: [
        'next', 'next track', 'skip', 'next one', 'play next',
        'something else', 'try another'
      ],
      category: 'playback',
      description: 'Play the next search result',
      action: () => {
        if (musicVoiceState.lastSearchResults.length > 0) {
          musicVoiceState.currentTrackIndex =
            (musicVoiceState.currentTrackIndex + 1) % musicVoiceState.lastSearchResults.length;
          const track = musicVoiceState.lastSearchResults[musicVoiceState.currentTrackIndex];
          playTrack(track);
          audioFeedback.play('confirm');
          showSuccess(`Playing: ${track.title}`);
        } else {
          showInfo('No tracks. Search for something first!');
        }
      },
    },
    {
      id: 'previous-track',
      phrases: [
        'previous', 'previous track', 'go back', 'last one',
        'play previous', 'before'
      ],
      category: 'playback',
      description: 'Play the previous search result',
      action: () => {
        if (musicVoiceState.lastSearchResults.length > 0) {
          musicVoiceState.currentTrackIndex =
            (musicVoiceState.currentTrackIndex - 1 + musicVoiceState.lastSearchResults.length) %
            musicVoiceState.lastSearchResults.length;
          const track = musicVoiceState.lastSearchResults[musicVoiceState.currentTrackIndex];
          playTrack(track);
          audioFeedback.play('confirm');
          showSuccess(`Playing: ${track.title}`);
        } else {
          showInfo('No tracks. Search for something first!');
        }
      },
    },
    {
      id: 'stop-jam-track',
      phrases: [
        'stop track', 'stop the music', 'stop jam', 'turn it off',
        'stop backing track', 'silence'
      ],
      category: 'playback',
      description: 'Stop the jam track',
      action: () => {
        stopPlayback();
        audioFeedback.play('confirm');
        showInfo('Jam track stopped');
      },
    },
    {
      id: 'pause-jam-track',
      phrases: [
        'pause track', 'pause music', 'pause jam', 'hold on',
        'wait a sec'
      ],
      category: 'playback',
      description: 'Pause the jam track',
      action: () => {
        pausePlayback();
        audioFeedback.play('confirm');
        showInfo('Jam track paused');
      },
    },
    {
      id: 'resume-jam-track',
      phrases: [
        'resume track', 'resume music', 'continue', 'keep going',
        'unpause', 'go on'
      ],
      category: 'playback',
      description: 'Resume the jam track',
      action: () => {
        resumePlayback();
        audioFeedback.play('confirm');
        showInfo('Resuming...');
      },
    },

    // ========================================
    // DRUM TRACK SPECIFIC
    // ========================================
    {
      id: 'find-drum-track',
      phrases: [
        'drum track', 'drums only', 'find drums', 'play drums',
        'just drums', 'drum backing', 'need drums'
      ],
      category: 'system',
      description: 'Search for drum-only tracks',
      action: async () => {
        musicVoiceState.isSearching = true;
        showInfo('Searching for drum tracks...');
        audioFeedback.play('confirm');

        try {
          const query = parseNaturalMusicQuery('drum track drums only');
          query.category = 'drum-track';
          const results = await searchMusic(query);
          handleSearchResults(results);
        } catch (e) {
          showError('Search failed');
          audioFeedback.play('error');
        }
        musicVoiceState.isSearching = false;
      },
    },
    {
      id: 'find-bass-track',
      phrases: [
        'bass track', 'bass only', 'find bass', 'play bass',
        'just bass', 'bass backing', 'need bass'
      ],
      category: 'system',
      description: 'Search for bass-only tracks',
      action: async () => {
        musicVoiceState.isSearching = true;
        showInfo('Searching for bass tracks...');
        audioFeedback.play('confirm');

        try {
          const query = parseNaturalMusicQuery('bass track bass only');
          query.category = 'bass-track';
          const results = await searchMusic(query);
          handleSearchResults(results);
        } catch (e) {
          showError('Search failed');
          audioFeedback.play('error');
        }
        musicVoiceState.isSearching = false;
      },
    },

    // ========================================
    // NATURAL LANGUAGE SEARCHES
    // ========================================
    {
      id: 'find-something-to-jam',
      phrases: [
        'find something', 'find me something', 'something to jam',
        "let's jam", 'wanna jam', 'jam session', 'random track'
      ],
      category: 'system',
      description: 'Find a random jam track',
      action: async () => {
        const genres = ['blues', 'rock', 'funk', 'jazz'];
        const randomGenre = genres[Math.floor(Math.random() * genres.length)];
        const keys = ['A', 'E', 'G', 'D', 'Am', 'Em'];
        const randomKey = keys[Math.floor(Math.random() * keys.length)];

        musicVoiceState.isSearching = true;
        showInfo(`Finding ${randomGenre} in ${randomKey}...`);
        audioFeedback.play('confirm');

        try {
          const query = parseNaturalMusicQuery(`${randomGenre} backing track ${randomKey}`);
          const results = await searchMusic(query);
          handleSearchResults(results);

          // Auto-play first result
          if (results.tracks.length > 0) {
            setTimeout(() => {
              playTrack(results.tracks[0]);
              audioFeedback.play('success');
              audioFeedback.speak(`Playing ${randomGenre} in ${randomKey}`);
            }, 500);
          }
        } catch (e) {
          showError('Search failed');
          audioFeedback.play('error');
        }
        musicVoiceState.isSearching = false;
      },
    },
  ];
}

/**
 * Search by musical key
 */
async function searchByKey(key: string): Promise<void> {
  const audioFeedback = getAudioFeedback();
  musicVoiceState.isSearching = true;
  showInfo(`Searching in ${key}...`);
  audioFeedback.play('confirm');

  try {
    const query = parseNaturalMusicQuery(`backing track in ${key}`);
    query.key = key;
    const results = await searchMusic(query);
    handleSearchResults(results);
  } catch (e) {
    showError('Search failed');
    audioFeedback.play('error');
  }
  musicVoiceState.isSearching = false;
}

/**
 * Handle search results
 */
function handleSearchResults(results: MusicSearchResult): void {
  const audioFeedback = getAudioFeedback();

  musicVoiceState.lastSearchResults = results.tracks;
  musicVoiceState.currentTrackIndex = -1;

  if (results.tracks.length > 0) {
    showSuccess(`Found ${results.tracks.length} tracks!`);
    audioFeedback.speak(`Found ${results.tracks.length} tracks. Say play this or next to browse.`);

    // Trigger UI callback if registered
    if (onSearchResultsCallback) {
      onSearchResultsCallback(results);
    }
  } else {
    showInfo('No tracks found. Try a different search.');
    audioFeedback.play('error');
  }
}

/**
 * Get current music voice state
 */
export function getMusicVoiceState(): MusicVoiceState {
  return { ...musicVoiceState };
}
