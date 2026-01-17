/**
 * Maestro AI Global State Management
 * Using Zustand for simple, performant state
 */

import { create } from 'zustand';
import type { MaestroProject } from '@orpheus/shared-types';

/**
 * The 6 modes of Maestro AI
 */
export type AppMode = 'compose' | 'record' | 'mix' | 'master' | 'practice' | 'distribute';

// Reduced history size to limit memory usage
const MAX_HISTORY_SIZE = 20;

// Debounce time for coalescing rapid updates (e.g., slider dragging)
const HISTORY_DEBOUNCE_MS = 500;

/**
 * Represents a history entry with optional compressed representation
 * Uses lazy serialization - only serializes when memory pressure requires it
 */
interface HistoryEntry {
  // The actual project data (may be null if compressed)
  project: MaestroProject | null;
  // Compressed JSON string (used for older entries to save memory)
  compressed: string | null;
  // Timestamp for age-based compression
  timestamp: number;
}

/**
 * Compresses older history entries to save memory.
 * Keeps recent entries as objects, older ones as JSON strings.
 */
function compressOldEntries(entries: HistoryEntry[], currentIndex: number): HistoryEntry[] {
  const KEEP_UNCOMPRESSED = 5; // Keep last 5 entries uncompressed for fast undo

  return entries.map((entry, index) => {
    const distanceFromCurrent = Math.abs(index - currentIndex);

    // Keep entries close to current position uncompressed
    if (distanceFromCurrent <= KEEP_UNCOMPRESSED) {
      // Decompress if needed
      if (entry.compressed && !entry.project) {
        return {
          project: JSON.parse(entry.compressed),
          compressed: null,
          timestamp: entry.timestamp,
        };
      }
      return entry;
    }

    // Compress older entries
    if (entry.project && !entry.compressed) {
      return {
        project: null,
        compressed: JSON.stringify(entry.project),
        timestamp: entry.timestamp,
      };
    }

    return entry;
  });
}

/**
 * Gets the project from a history entry, decompressing if necessary
 */
function getProjectFromEntry(entry: HistoryEntry): MaestroProject {
  if (entry.project) {
    return entry.project;
  }
  if (entry.compressed) {
    return JSON.parse(entry.compressed);
  }
  throw new Error('Invalid history entry: no project data');
}

// Debounce timer for history updates
let historyDebounceTimer: ReturnType<typeof setTimeout> | null = null;
let pendingProject: MaestroProject | null = null;

/**
 * Global application state
 */
export interface AppState {
  // Current mode
  mode: AppMode;
  setMode: (mode: AppMode) => void;

  // Project
  project: MaestroProject | null;
  setProject: (project: MaestroProject | null) => void;
  updateProject: (project: MaestroProject) => void; // Updates with history tracking
  projectModified: boolean;
  setProjectModified: (modified: boolean) => void;
  rawFileBuffer: ArrayBuffer | null; // For alphaTab native rendering
  setRawFileBuffer: (buffer: ArrayBuffer | null) => void;

  // Undo/Redo (uses HistoryEntry for memory efficiency)
  history: HistoryEntry[];
  historyIndex: number;
  undo: () => void;
  redo: () => void;
  canUndo: () => boolean;
  canRedo: () => boolean;

  // Immediate update without debouncing (for final state after interactions)
  commitProject: (project: MaestroProject) => void;

  // Playback
  isPlaying: boolean;
  setIsPlaying: (playing: boolean) => void;
  currentTime: number; // in seconds
  setCurrentTime: (time: number) => void;

  // UI state
  sidebarOpen: boolean;
  setSidebarOpen: (open: boolean) => void;
  aiAssistantOpen: boolean;
  setAIAssistantOpen: (open: boolean) => void;

  // Selected track
  selectedTrackId: string | null;
  setSelectedTrackId: (id: string | null) => void;

  // Zoom level (for notation/waveform)
  zoomLevel: number;
  setZoomLevel: (level: number) => void;

  // Loading state
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
  loadingMessage: string;
  setLoadingMessage: (message: string) => void;
  loadingProgress: number; // 0-100, -1 for indeterminate
  setLoadingProgress: (progress: number) => void;
  loadingCancellable: boolean;
  setLoadingCancellable: (cancellable: boolean) => void;
  cancelLoading: (() => void) | null;
  setCancelLoading: (callback: (() => void) | null) => void;
}

/**
 * Create the global store
 */
export const useAppStore = create<AppState>((set, get) => ({
  // Mode
  mode: 'compose',
  setMode: (mode) => set({ mode }),

  // Project
  project: null,
  setProject: (project) => {
    // Clear any pending debounced updates
    if (historyDebounceTimer) {
      clearTimeout(historyDebounceTimer);
      historyDebounceTimer = null;
      pendingProject = null;
    }

    const entry: HistoryEntry | null = project ? {
      project,
      compressed: null,
      timestamp: Date.now(),
    } : null;

    set({
      project,
      projectModified: false,
      history: entry ? [entry] : [],
      historyIndex: entry ? 0 : -1
    });
  },

  /**
   * Updates project with debouncing to coalesce rapid changes.
   * This prevents creating many history entries when dragging sliders, etc.
   */
  updateProject: (project) => {
    // Update current project immediately for responsive UI
    set({ project, projectModified: true });

    // Store the latest project for debounced history commit
    pendingProject = project;

    // Clear existing timer
    if (historyDebounceTimer) {
      clearTimeout(historyDebounceTimer);
    }

    // Schedule history commit after debounce period
    historyDebounceTimer = setTimeout(() => {
      if (pendingProject) {
        get().commitProject(pendingProject);
        pendingProject = null;
      }
      historyDebounceTimer = null;
    }, HISTORY_DEBOUNCE_MS);
  },

  /**
   * Immediately commits a project state to history (no debouncing).
   * Use this for discrete actions like "delete track", "add note", etc.
   */
  commitProject: (project) => {
    const state = get();

    // Clear debounce timer since we're committing now
    if (historyDebounceTimer) {
      clearTimeout(historyDebounceTimer);
      historyDebounceTimer = null;
      pendingProject = null;
    }

    // Create new history by removing everything after current index
    let newHistory = state.history.slice(0, state.historyIndex + 1);

    // Add new entry
    const newEntry: HistoryEntry = {
      project,
      compressed: null,
      timestamp: Date.now(),
    };
    newHistory.push(newEntry);

    // Limit history size
    if (newHistory.length > MAX_HISTORY_SIZE) {
      newHistory.shift();
    }

    // Compress older entries to save memory
    const newIndex = newHistory.length - 1;
    newHistory = compressOldEntries(newHistory, newIndex);

    set({
      project,
      history: newHistory,
      historyIndex: newIndex,
      projectModified: true
    });
  },
  projectModified: false,
  setProjectModified: (modified) => set({ projectModified: modified }),
  rawFileBuffer: null,
  setRawFileBuffer: (buffer) => set({ rawFileBuffer: buffer }),

  // Undo/Redo
  history: [],
  historyIndex: -1,
  undo: () => {
    const state = get();

    // Cancel any pending debounced update first
    if (historyDebounceTimer) {
      clearTimeout(historyDebounceTimer);
      historyDebounceTimer = null;
      pendingProject = null;
    }

    if (state.historyIndex > 0) {
      const newIndex = state.historyIndex - 1;
      const project = getProjectFromEntry(state.history[newIndex]);

      // Recompress entries based on new position
      const updatedHistory = compressOldEntries(state.history, newIndex);

      set({
        project,
        history: updatedHistory,
        historyIndex: newIndex,
        projectModified: true
      });
    }
  },
  redo: () => {
    const state = get();

    // Cancel any pending debounced update first
    if (historyDebounceTimer) {
      clearTimeout(historyDebounceTimer);
      historyDebounceTimer = null;
      pendingProject = null;
    }

    if (state.historyIndex < state.history.length - 1) {
      const newIndex = state.historyIndex + 1;
      const project = getProjectFromEntry(state.history[newIndex]);

      // Recompress entries based on new position
      const updatedHistory = compressOldEntries(state.history, newIndex);

      set({
        project,
        history: updatedHistory,
        historyIndex: newIndex,
        projectModified: true
      });
    }
  },
  canUndo: () => {
    const state = get();
    return state.historyIndex > 0;
  },
  canRedo: () => {
    const state = get();
    return state.historyIndex < state.history.length - 1;
  },

  // Playback
  isPlaying: false,
  setIsPlaying: (playing) => set({ isPlaying: playing }),
  currentTime: 0,
  setCurrentTime: (time) => set({ currentTime: time }),

  // UI
  sidebarOpen: true,
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
  aiAssistantOpen: false,
  setAIAssistantOpen: (open) => set({ aiAssistantOpen: open }),

  // Selected track
  selectedTrackId: null,
  setSelectedTrackId: (id) => set({ selectedTrackId: id }),

  // Zoom
  zoomLevel: 1.0,
  setZoomLevel: (level) => set({ zoomLevel: level }),

  // Loading
  isLoading: false,
  setIsLoading: (loading) => set({ isLoading: loading }),
  loadingMessage: '',
  setLoadingMessage: (message) => set({ loadingMessage: message }),
  loadingProgress: -1,
  setLoadingProgress: (progress) => set({ loadingProgress: progress }),
  loadingCancellable: false,
  setLoadingCancellable: (cancellable) => set({ loadingCancellable: cancellable }),
  cancelLoading: null,
  setCancelLoading: (callback) => set({ cancelLoading: callback }),
}));

/**
 * Selectors for derived state
 */
export const useCurrentMode = () => useAppStore((state) => state.mode);
export const useProject = () => useAppStore((state) => state.project);
export const useIsPlaying = () => useAppStore((state) => state.isPlaying);
export const useSelectedTrack = () => {
  const project = useProject();
  const selectedId = useAppStore((state) => state.selectedTrackId);

  if (!project || !selectedId) return null;

  return (project.project.composition.tracks || []).find((t: any) => t.id === selectedId) || null;
};
