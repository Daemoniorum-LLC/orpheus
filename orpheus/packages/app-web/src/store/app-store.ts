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

const MAX_HISTORY_SIZE = 50;

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

  // Undo/Redo
  history: MaestroProject[];
  historyIndex: number;
  undo: () => void;
  redo: () => void;
  canUndo: () => boolean;
  canRedo: () => boolean;

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
  setProject: (project) =>
    set({
      project,
      projectModified: false,
      history: project ? [project] : [],
      historyIndex: project ? 0 : -1
    }),
  updateProject: (project) => {
    const state = get();

    // Create new history by removing everything after current index
    const newHistory = state.history.slice(0, state.historyIndex + 1);

    // Add new state
    newHistory.push(project);

    // Limit history size
    if (newHistory.length > MAX_HISTORY_SIZE) {
      newHistory.shift();
    } else {
      // Increment index only if we didn't remove the first item
      set({
        project,
        history: newHistory,
        historyIndex: newHistory.length - 1,
        projectModified: true
      });
      return;
    }

    set({
      project,
      history: newHistory,
      historyIndex: newHistory.length - 1,
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
    if (state.historyIndex > 0) {
      const newIndex = state.historyIndex - 1;
      set({
        project: state.history[newIndex],
        historyIndex: newIndex,
        projectModified: true
      });
      console.log('[Undo] Undid to index:', newIndex);
    }
  },
  redo: () => {
    const state = get();
    if (state.historyIndex < state.history.length - 1) {
      const newIndex = state.historyIndex + 1;
      set({
        project: state.history[newIndex],
        historyIndex: newIndex,
        projectModified: true
      });
      console.log('[Redo] Redid to index:', newIndex);
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
