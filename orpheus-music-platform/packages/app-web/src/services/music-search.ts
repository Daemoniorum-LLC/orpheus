/**
 * Music Search Service - Search and play music from various platforms
 * Supports YouTube, Spotify (future), and Jamendo for backing tracks
 */

export type MusicPlatform = 'youtube' | 'spotify' | 'jamendo' | 'all';

export type MusicCategory =
  | 'backing-track'
  | 'song'
  | 'loop'
  | 'drum-track'
  | 'bass-track'
  | 'metronome'
  | 'any';

export interface MusicSearchQuery {
  query: string;
  key?: string;           // Musical key (A, Bm, C#, etc.)
  genre?: string;         // blues, rock, jazz, metal, etc.
  tempo?: number;         // BPM
  category?: MusicCategory;
  platform?: MusicPlatform;
}

export interface MusicTrack {
  id: string;
  title: string;
  artist: string;
  duration: number;       // seconds
  thumbnail: string;
  platform: MusicPlatform;
  url: string;
  embedUrl?: string;
  key?: string;
  tempo?: number;
  genre?: string;
}

export interface MusicSearchResult {
  tracks: MusicTrack[];
  query: MusicSearchQuery;
  totalResults: number;
  nextPageToken?: string;
}

// Musical keys for search
export const MUSICAL_KEYS = [
  'C', 'C#', 'Db', 'D', 'D#', 'Eb', 'E', 'F', 'F#', 'Gb', 'G', 'G#', 'Ab', 'A', 'A#', 'Bb', 'B',
  'Am', 'A#m', 'Bbm', 'Bm', 'Cm', 'C#m', 'Dbm', 'Dm', 'D#m', 'Ebm', 'Em', 'Fm', 'F#m', 'Gbm', 'Gm', 'G#m', 'Abm'
];

// Common genres for musicians
export const MUSIC_GENRES = [
  'blues', 'rock', 'jazz', 'metal', 'funk', 'pop', 'country', 'reggae',
  'soul', 'r&b', 'folk', 'classical', 'latin', 'fusion', 'progressive',
  'punk', 'grunge', 'indie', 'acoustic', 'electronic'
];

/**
 * Build a smart search query for backing tracks
 */
function buildBackingTrackQuery(params: MusicSearchQuery): string {
  const parts: string[] = [];

  // Add the base query
  if (params.query) {
    parts.push(params.query);
  }

  // Add key if specified
  if (params.key) {
    parts.push(`${params.key} key`);
  }

  // Add genre
  if (params.genre) {
    parts.push(params.genre);
  }

  // Add tempo
  if (params.tempo) {
    parts.push(`${params.tempo} bpm`);
  }

  // Add category-specific terms
  switch (params.category) {
    case 'backing-track':
      parts.push('backing track');
      break;
    case 'drum-track':
      parts.push('drum track only drums');
      break;
    case 'bass-track':
      parts.push('bass track isolated bass');
      break;
    case 'loop':
      parts.push('loop jam');
      break;
    case 'metronome':
      parts.push('metronome click track');
      break;
  }

  return parts.join(' ');
}

/**
 * Parse natural language music search
 */
export function parseNaturalMusicQuery(text: string): MusicSearchQuery {
  const query: MusicSearchQuery = {
    query: text,
    category: 'backing-track',
    platform: 'youtube',
  };

  const lowerText = text.toLowerCase();

  // Extract key
  for (const key of MUSICAL_KEYS) {
    const keyPatterns = [
      new RegExp(`\\b${key.toLowerCase()}\\s*(major|minor)?\\b`, 'i'),
      new RegExp(`\\bin\\s+${key.toLowerCase()}\\b`, 'i'),
      new RegExp(`\\bkey\\s+of\\s+${key.toLowerCase()}\\b`, 'i'),
    ];

    for (const pattern of keyPatterns) {
      if (pattern.test(lowerText)) {
        query.key = key;
        break;
      }
    }
    if (query.key) break;
  }

  // Extract genre
  for (const genre of MUSIC_GENRES) {
    if (lowerText.includes(genre)) {
      query.genre = genre;
      break;
    }
  }

  // Extract tempo
  const tempoMatch = lowerText.match(/(\d{2,3})\s*(bpm|beats)/);
  if (tempoMatch) {
    query.tempo = parseInt(tempoMatch[1], 10);
  }

  // Determine category
  if (lowerText.includes('drum') || lowerText.includes('drums only')) {
    query.category = 'drum-track';
  } else if (lowerText.includes('bass') && (lowerText.includes('only') || lowerText.includes('track'))) {
    query.category = 'bass-track';
  } else if (lowerText.includes('loop')) {
    query.category = 'loop';
  } else if (lowerText.includes('song') || lowerText.includes('original')) {
    query.category = 'song';
  } else if (lowerText.includes('click') || lowerText.includes('metronome')) {
    query.category = 'metronome';
  }

  return query;
}

/**
 * Search YouTube for music (using embed-friendly approach)
 */
async function searchYouTube(params: MusicSearchQuery): Promise<MusicTrack[]> {
  const searchQuery = buildBackingTrackQuery(params);

  // For demo purposes, we'll generate mock results based on the query
  // In production, you'd use the YouTube Data API v3
  const mockTracks: MusicTrack[] = generateMockResults(searchQuery, params, 'youtube');

  return mockTracks;
}

/**
 * Generate mock search results for demo
 * In production, replace with actual API calls
 */
function generateMockResults(searchQuery: string, params: MusicSearchQuery, platform: MusicPlatform): MusicTrack[] {
  const categoryTitles: Record<MusicCategory, string[]> = {
    'backing-track': ['Backing Track', 'Jam Track', 'Play Along', 'Minus One'],
    'drum-track': ['Drum Track', 'Drums Only', 'Drum Backing'],
    'bass-track': ['Bass Track', 'Bass Line', 'Isolated Bass'],
    'loop': ['Loop', 'Jam Loop', '4 Bar Loop', '8 Bar Loop'],
    'song': ['', 'Original', 'Full Song'],
    'metronome': ['Click Track', 'Metronome', 'BPM'],
    'any': [''],
  };

  const artists = [
    'JamTrackCentral', 'BackingTrackHQ', 'GuitarBackingTrack',
    'ElevatedJam', 'QuistJam', 'JTC Guitar'
  ];

  const titles = categoryTitles[params.category || 'backing-track'];
  const tracks: MusicTrack[] = [];

  // Generate 5-10 mock results
  const numResults = 5 + Math.floor(Math.random() * 5);

  for (let i = 0; i < numResults; i++) {
    const title = titles[i % titles.length];
    const keyStr = params.key ? `${params.key} ` : '';
    const genreStr = params.genre ? `${params.genre.charAt(0).toUpperCase() + params.genre.slice(1)} ` : '';
    const tempoStr = params.tempo ? `${params.tempo} BPM ` : '';

    const videoId = `demo_${Date.now()}_${i}`;

    tracks.push({
      id: videoId,
      title: `${genreStr}${keyStr}${title} ${tempoStr}#${i + 1}`.trim(),
      artist: artists[i % artists.length],
      duration: 180 + Math.floor(Math.random() * 300), // 3-8 minutes
      thumbnail: `https://i.ytimg.com/vi/${videoId}/mqdefault.jpg`,
      platform,
      url: `https://www.youtube.com/watch?v=${videoId}`,
      embedUrl: `https://www.youtube.com/embed/${videoId}?autoplay=1`,
      key: params.key,
      tempo: params.tempo,
      genre: params.genre,
    });
  }

  return tracks;
}

/**
 * Main search function
 */
export async function searchMusic(params: MusicSearchQuery): Promise<MusicSearchResult> {
  const platform = params.platform || 'youtube';
  let tracks: MusicTrack[] = [];

  switch (platform) {
    case 'youtube':
      tracks = await searchYouTube(params);
      break;
    case 'all':
      // Search all platforms and combine
      tracks = await searchYouTube(params);
      // Add other platforms here
      break;
    default:
      tracks = await searchYouTube(params);
  }

  return {
    tracks,
    query: params,
    totalResults: tracks.length,
  };
}

/**
 * Get suggested searches for jamming
 */
export function getSuggestedSearches(): MusicSearchQuery[] {
  return [
    { query: 'blues', key: 'A', genre: 'blues', category: 'backing-track', platform: 'youtube' },
    { query: 'rock', key: 'E', genre: 'rock', category: 'backing-track', platform: 'youtube' },
    { query: 'jazz', key: 'Dm', genre: 'jazz', category: 'backing-track', platform: 'youtube' },
    { query: 'funk', key: 'E', genre: 'funk', category: 'backing-track', platform: 'youtube' },
    { query: 'metal', key: 'Em', genre: 'metal', category: 'backing-track', platform: 'youtube' },
    { query: 'acoustic', genre: 'acoustic', category: 'backing-track', platform: 'youtube' },
  ];
}

// Current playback state
interface PlaybackState {
  currentTrack: MusicTrack | null;
  isPlaying: boolean;
  volume: number;
  position: number;
}

const playbackState: PlaybackState = {
  currentTrack: null,
  isPlaying: false,
  volume: 0.8,
  position: 0,
};

type PlaybackListener = (state: PlaybackState) => void;
const playbackListeners = new Set<PlaybackListener>();

/**
 * Subscribe to playback state changes
 */
export function subscribeToPlayback(listener: PlaybackListener): () => void {
  playbackListeners.add(listener);
  listener(playbackState);
  return () => playbackListeners.delete(listener);
}

function notifyPlaybackListeners() {
  playbackListeners.forEach(l => l({ ...playbackState }));
}

/**
 * Play a track
 */
export function playTrack(track: MusicTrack): void {
  playbackState.currentTrack = track;
  playbackState.isPlaying = true;
  playbackState.position = 0;
  notifyPlaybackListeners();
}

/**
 * Pause playback
 */
export function pausePlayback(): void {
  playbackState.isPlaying = false;
  notifyPlaybackListeners();
}

/**
 * Resume playback
 */
export function resumePlayback(): void {
  if (playbackState.currentTrack) {
    playbackState.isPlaying = true;
    notifyPlaybackListeners();
  }
}

/**
 * Stop playback
 */
export function stopPlayback(): void {
  playbackState.isPlaying = false;
  playbackState.position = 0;
  notifyPlaybackListeners();
}

/**
 * Get current playback state
 */
export function getPlaybackState(): PlaybackState {
  return { ...playbackState };
}
