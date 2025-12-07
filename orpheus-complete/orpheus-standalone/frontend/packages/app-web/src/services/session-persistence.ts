/**
 * Session Persistence Service
 * Automatically saves and restores UI state across sessions
 */

import type { AppMode } from '../store/app-store';

interface SessionState {
  mode: AppMode;
  sidebarOpen: boolean;
  aiAssistantOpen: boolean;
  zoomLevel: number;
  selectedTrackId: string | null;
  timestamp: string;
}

const SESSION_STORAGE_KEY = 'maestro-session-state';

/**
 * Save current session state
 */
export function saveSessionState(state: Partial<SessionState>): void {
  try {
    const currentState = loadSessionState();
    const newState: SessionState = {
      ...currentState,
      ...state,
      timestamp: new Date().toISOString(),
    };

    sessionStorage.setItem(SESSION_STORAGE_KEY, JSON.stringify(newState));
    console.log('[SessionPersistence] Session state saved');
  } catch (error) {
    console.error('[SessionPersistence] Failed to save session state:', error);
  }
}

/**
 * Load session state
 */
export function loadSessionState(): SessionState {
  try {
    const stored = sessionStorage.getItem(SESSION_STORAGE_KEY);
    if (!stored) {
      return getDefaultSessionState();
    }

    const state = JSON.parse(stored) as SessionState;
    console.log('[SessionPersistence] Session state loaded');
    return state;
  } catch (error) {
    console.error('[SessionPersistence] Failed to load session state:', error);
    return getDefaultSessionState();
  }
}

/**
 * Clear session state
 */
export function clearSessionState(): void {
  try {
    sessionStorage.removeItem(SESSION_STORAGE_KEY);
    console.log('[SessionPersistence] Session state cleared');
  } catch (error) {
    console.error('[SessionPersistence] Failed to clear session state:', error);
  }
}

/**
 * Get default session state
 */
function getDefaultSessionState(): SessionState {
  return {
    mode: 'compose',
    sidebarOpen: true,
    aiAssistantOpen: false,
    zoomLevel: 1.0,
    selectedTrackId: null,
    timestamp: new Date().toISOString(),
  };
}

/**
 * Check if session state is stale (older than 24 hours)
 */
export function isSessionStateStale(state: SessionState): boolean {
  const timestamp = new Date(state.timestamp);
  const now = new Date();
  const hoursSince = (now.getTime() - timestamp.getTime()) / (1000 * 60 * 60);
  return hoursSince > 24;
}
