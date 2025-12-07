/**
 * Keyboard Shortcuts Hook
 * Global keyboard shortcut handling for Maestro AI
 */

import { useEffect } from 'react';
import { useAppStore, type AppMode } from '../store/app-store';
import { getPlaybackCoordinator } from '../services/playback-coordinator';
import { saveProject } from '../services/project-save';
import { showInfo } from '../services/toast';
import { logger } from '../utils/logger';

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  description: string;
  action: () => void;
}

/**
 * Hook to enable global keyboard shortcuts
 */
export function useKeyboardShortcuts() {
  const { mode, setMode, aiAssistantOpen, setAIAssistantOpen, project, setProjectModified, undo, redo, canUndo, canRedo } = useAppStore();
  const coordinator = getPlaybackCoordinator();

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Skip if user is typing in an input field
      const target = event.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return;
      }

      // Undo/Redo
      if (event.ctrlKey && event.shiftKey && event.key.toLowerCase() === 'z') {
        event.preventDefault();
        if (canRedo()) {
          logger.shortcuts.debug('Redo');
          redo();
        }
        return;
      }

      if (event.ctrlKey && !event.shiftKey && event.key.toLowerCase() === 'z') {
        event.preventDefault();
        if (canUndo()) {
          logger.shortcuts.debug('Undo');
          undo();
        }
        return;
      }

      // File operations (Ctrl+S, Ctrl+O)
      if (event.ctrlKey && !event.shiftKey && !event.altKey) {
        if (event.key.toLowerCase() === 's') {
          event.preventDefault();
          if (project) {
            logger.shortcuts.debug('Save project');
            saveProject(project);
            setProjectModified(false); // Clear modified flag after save
          } else {
            showInfo('No project to save. Create or open a project first.', 3000);
          }
          return;
        }

        if (event.key.toLowerCase() === 'o') {
          event.preventDefault();
          logger.shortcuts.debug('Open file');
          // Trigger file picker by simulating click on Open button
          const openButton = document.querySelector('[aria-label*="Open"]') as HTMLElement;
          if (openButton) {
            openButton.click();
          }
          return;
        }
      }

      // Mode switching (Ctrl+1-6)
      if (event.ctrlKey && !event.shiftKey && !event.altKey) {
        const modeMap: Record<string, AppMode> = {
          '1': 'compose',
          '2': 'record',
          '3': 'mix',
          '4': 'master',
          '5': 'practice',
          '6': 'distribute',
        };

        if (modeMap[event.key]) {
          event.preventDefault();
          logger.shortcuts.debug('Switching to mode:', modeMap[event.key]);
          setMode(modeMap[event.key]);
          return;
        }

        // AI Assistant toggle (Ctrl+.)
        if (event.key === '.') {
          event.preventDefault();
          setAIAssistantOpen(!aiAssistantOpen);
          logger.shortcuts.debug('AI Assistant toggled via Ctrl+.');
          return;
        }
      }

      // Playback controls
      if (!event.ctrlKey && !event.altKey) {
        switch (event.key) {
          case ' ': // Spacebar - Play/Pause
            event.preventDefault();
            const state = coordinator.getState();
            if (state.isPlaying) {
              coordinator.pause();
            } else {
              coordinator.play();
            }
            logger.shortcuts.debug('Play/Pause toggled');
            return;

          case 'Home': // Home - Jump to start
            event.preventDefault();
            coordinator.stop();
            logger.shortcuts.debug('Jump to start');
            return;

          case 'Escape': // Escape - Close AI Assistant
            if (aiAssistantOpen) {
              event.preventDefault();
              setAIAssistantOpen(false);
              logger.shortcuts.debug('AI Assistant closed');
            }
            return;
        }
      }

      // Help (Ctrl+/)
      if (event.ctrlKey && event.key === '/') {
        event.preventDefault();
        logger.shortcuts.debug('Help requested (not yet implemented)');
        // TODO: Show help overlay
        return;
      }
    };

    // Add event listener
    window.addEventListener('keydown', handleKeyDown);

    // Cleanup
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [mode, setMode, aiAssistantOpen, setAIAssistantOpen, project, setProjectModified, undo, redo, canUndo, canRedo, coordinator]);
}

/**
 * Get list of all available shortcuts for documentation
 */
export function getShortcuts(): KeyboardShortcut[] {
  return [
    // Edit operations
    {
      key: 'Z',
      ctrl: true,
      description: 'Undo',
      action: () => {},
    },
    {
      key: 'Z',
      ctrl: true,
      shift: true,
      description: 'Redo',
      action: () => {},
    },

    // File operations
    {
      key: 'S',
      ctrl: true,
      description: 'Save Project',
      action: () => {},
    },
    {
      key: 'O',
      ctrl: true,
      description: 'Open File',
      action: () => {},
    },

    // Mode switching
    {
      key: '1',
      ctrl: true,
      description: 'Switch to Compose Mode',
      action: () => {},
    },
    {
      key: '2',
      ctrl: true,
      description: 'Switch to Record Mode',
      action: () => {},
    },
    {
      key: '3',
      ctrl: true,
      description: 'Switch to Mix Mode',
      action: () => {},
    },
    {
      key: '4',
      ctrl: true,
      description: 'Switch to Master Mode',
      action: () => {},
    },
    {
      key: '5',
      ctrl: true,
      description: 'Switch to Practice Mode',
      action: () => {},
    },
    {
      key: '6',
      ctrl: true,
      description: 'Switch to Distribute Mode',
      action: () => {},
    },

    // Playback
    {
      key: 'Space',
      description: 'Play / Pause',
      action: () => {},
    },
    {
      key: 'Home',
      description: 'Jump to Start',
      action: () => {},
    },

    // UI
    {
      key: '.',
      ctrl: true,
      description: 'Toggle AI Assistant',
      action: () => {},
    },
    {
      key: 'Escape',
      description: 'Close AI Assistant',
      action: () => {},
    },

    // Help
    {
      key: '/',
      ctrl: true,
      description: 'Show Keyboard Shortcuts',
      action: () => {},
    },
  ];
}

/**
 * Format shortcut for display
 */
export function formatShortcut(shortcut: KeyboardShortcut): string {
  const parts: string[] = [];

  if (shortcut.ctrl) parts.push('Ctrl');
  if (shortcut.shift) parts.push('Shift');
  if (shortcut.alt) parts.push('Alt');
  parts.push(shortcut.key);

  return parts.join('+');
}
