/**
 * Undo/Redo Hook
 * Manages history state for reversible operations
 */

import { useState, useCallback, useRef, useEffect } from 'react';

export interface UndoRedoState<T> {
  current: T;
  canUndo: boolean;
  canRedo: boolean;
  undoCount: number;
  redoCount: number;
}

export interface UndoRedoActions<T> {
  /** Push a new state to history */
  push: (state: T) => void;
  /** Undo the last action */
  undo: () => T | null;
  /** Redo the last undone action */
  redo: () => T | null;
  /** Reset history to initial state */
  reset: (initialState: T) => void;
  /** Clear all history but keep current state */
  clearHistory: () => void;
}

export interface UseUndoRedoOptions {
  /** Maximum history size (default: 50) */
  maxHistorySize?: number;
  /** Debounce time in ms for push operations (default: 0 - no debounce) */
  debounceMs?: number;
}

/**
 * Hook for managing undo/redo state
 */
export function useUndoRedo<T>(
  initialState: T,
  options: UseUndoRedoOptions = {}
): [UndoRedoState<T>, UndoRedoActions<T>] {
  const { maxHistorySize = 50, debounceMs = 0 } = options;

  // History stacks
  const [history, setHistory] = useState<T[]>([initialState]);
  const [currentIndex, setCurrentIndex] = useState(0);

  // Debounce tracking
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingStateRef = useRef<T | null>(null);

  // Cleanup debounce timer on unmount
  useEffect(() => {
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, []);

  // Push a new state to history
  const push = useCallback((state: T) => {
    const executePush = () => {
      setHistory((prevHistory) => {
        setCurrentIndex((prevIndex) => {
          // Truncate any future states (redo stack)
          const newHistory = prevHistory.slice(0, prevIndex + 1);

          // Add new state
          newHistory.push(state);

          // Limit history size
          if (newHistory.length > maxHistorySize) {
            // Remove oldest entry and adjust index
            newHistory.shift();
            return newHistory.length - 1;
          }

          return newHistory.length - 1;
        });

        // Return updated history
        const truncated = prevHistory.slice(0, currentIndex + 1);
        truncated.push(state);
        if (truncated.length > maxHistorySize) {
          truncated.shift();
        }
        return truncated;
      });

      pendingStateRef.current = null;
    };

    if (debounceMs > 0) {
      pendingStateRef.current = state;
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
      debounceTimerRef.current = setTimeout(executePush, debounceMs);
    } else {
      executePush();
    }
  }, [currentIndex, maxHistorySize, debounceMs]);

  // Undo the last action
  const undo = useCallback((): T | null => {
    // Flush any pending debounced push
    if (debounceTimerRef.current && pendingStateRef.current) {
      clearTimeout(debounceTimerRef.current);
      debounceTimerRef.current = null;
    }

    if (currentIndex <= 0) {
      return null;
    }

    const newIndex = currentIndex - 1;
    setCurrentIndex(newIndex);
    return history[newIndex];
  }, [currentIndex, history]);

  // Redo the last undone action
  const redo = useCallback((): T | null => {
    if (currentIndex >= history.length - 1) {
      return null;
    }

    const newIndex = currentIndex + 1;
    setCurrentIndex(newIndex);
    return history[newIndex];
  }, [currentIndex, history]);

  // Reset history to initial state
  const reset = useCallback((newInitialState: T) => {
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
      debounceTimerRef.current = null;
    }
    pendingStateRef.current = null;
    setHistory([newInitialState]);
    setCurrentIndex(0);
  }, []);

  // Clear history but keep current state
  const clearHistory = useCallback(() => {
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
      debounceTimerRef.current = null;
    }
    pendingStateRef.current = null;
    setHistory([history[currentIndex]]);
    setCurrentIndex(0);
  }, [history, currentIndex]);

  const state: UndoRedoState<T> = {
    current: history[currentIndex],
    canUndo: currentIndex > 0,
    canRedo: currentIndex < history.length - 1,
    undoCount: currentIndex,
    redoCount: history.length - 1 - currentIndex,
  };

  const actions: UndoRedoActions<T> = {
    push,
    undo,
    redo,
    reset,
    clearHistory,
  };

  return [state, actions];
}

/**
 * Keyboard shortcuts for undo/redo
 */
export function useUndoRedoKeyboard(
  undo: () => void,
  redo: () => void,
  canUndo: boolean,
  canRedo: boolean
) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check if user is in an input field
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        // Let native undo/redo work in text inputs
        return;
      }

      // Undo: Ctrl+Z or Cmd+Z
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        if (canUndo) {
          e.preventDefault();
          undo();
        }
        return;
      }

      // Redo: Ctrl+Shift+Z or Cmd+Shift+Z or Ctrl+Y
      if (
        ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'z') ||
        (e.ctrlKey && e.key === 'y')
      ) {
        if (canRedo) {
          e.preventDefault();
          redo();
        }
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [undo, redo, canUndo, canRedo]);
}
