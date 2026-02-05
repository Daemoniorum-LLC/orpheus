/**
 * alphaTab Integration Hook
 * Renders Guitar Pro tablature in the browser
 */

import { useEffect, useRef, useState } from 'react';
import * as alphaTab from '@coderline/alphatab';
import type { MaestroProject } from '@maestro-ai/shared-types';
import { convertMaestroToAlphaTabTEX, getProjectHash } from '../services/alphatab-converter';

export interface AlphaTabOptions {
  /** Display mode */
  display?: 'page' | 'score';
  /** Enable playback controls */
  enablePlayer?: boolean;
  /** Zoom level */
  zoom?: number;
  /** Layout mode */
  layoutMode?: 'page' | 'horizontal';
}

/**
 * Hook to integrate alphaTab for tablature rendering
 */
export function useAlphaTab(
  project: MaestroProject | null,
  options: AlphaTabOptions = {},
  rawFileBuffer?: ArrayBuffer | null
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const apiRef = useRef<alphaTab.AlphaTabApi | null>(null);
  const [projectHash, setProjectHash] = useState<string>('');
  const initializedRef = useRef(false);

  // Initialize alphaTab on first render
  useEffect(() => {
    if (!containerRef.current || !project || initializedRef.current) {
      return;
    }

    // Initialize alphaTab
    const settings: alphaTab.Settings = new alphaTab.Settings();

    // Configure display
    settings.core.engine = 'svg';
    settings.core.logLevel = alphaTab.LogLevel.Warning;

    // Configure player
    settings.player.enablePlayer = options.enablePlayer ?? true;
    settings.player.enableCursor = true;
    settings.player.enableUserInteraction = true;

    // Configure display settings
    settings.display.scale = options.zoom ?? 1.0;
    settings.display.layoutMode =
      options.layoutMode === 'horizontal'
        ? alphaTab.LayoutMode.Horizontal
        : alphaTab.LayoutMode.Page;

    // Create alphaTab API
    try {
      const api = new alphaTab.AlphaTabApi(containerRef.current, settings);
      apiRef.current = api;
      initializedRef.current = true;

      // Load score from raw Guitar Pro file or convert from Maestro format
      if (rawFileBuffer) {
        console.log('[alphaTab] Loading native Guitar Pro file');
        const uint8Array = new Uint8Array(rawFileBuffer);
        api.load(uint8Array);
      } else {
        console.log('[alphaTab] Converting Maestro project to TEX notation');
        const score = convertMaestroToAlphaTabTEX(project);
        if (score) {
          api.tex(score);
        }
      }

      // Event listeners
      api.renderFinished.on(() => {
        console.log('[alphaTab] Rendering complete');
      });

      api.playerReady.on(() => {
        console.log('[alphaTab] Player ready');
      });

      api.error.on((error) => {
        console.error('[alphaTab] Error:', error);
      });

      // Set initial project hash
      setProjectHash(getProjectHash(project));

    } catch (error) {
      console.error('[alphaTab] Initialization error:', error);
    }

    // Cleanup
    return () => {
      if (apiRef.current) {
        apiRef.current.destroy();
        apiRef.current = null;
        initializedRef.current = false;
      }
    };
  }, [rawFileBuffer, options.enablePlayer, options.zoom, options.layoutMode]);

  // Track if project has been edited (hash changed after initial load)
  const hasBeenEditedRef = useRef(false);

  // Update alphaTab when project changes (e.g., after editing)
  useEffect(() => {
    if (!project || !apiRef.current || !initializedRef.current) return;

    const newHash = getProjectHash(project);
    if (newHash !== projectHash && newHash !== '') {
      console.log('[alphaTab] Project changed, refreshing view');
      setProjectHash(newHash);

      // Mark as edited once hash changes (not first load)
      if (projectHash !== '') {
        hasBeenEditedRef.current = true;
      }

      // Refresh from TEX if:
      // 1. No raw file buffer (always use TEX), OR
      // 2. Project has been edited (edits override raw file)
      if (!rawFileBuffer || hasBeenEditedRef.current) {
        try {
          const score = convertMaestroToAlphaTabTEX(project);
          if (score) {
            apiRef.current.tex(score);
            console.log('[alphaTab] View refreshed with updated notation from TEX');
          }
        } catch (error) {
          console.error('[alphaTab] Error refreshing view:', error);
        }
      }
    }
  }, [project, projectHash, rawFileBuffer]);

  return {
    containerRef,
    api: apiRef.current,
  };
}

/**
 * Playback controls for alphaTab
 */
export function useAlphaTabPlayback(api: alphaTab.AlphaTabApi | null) {
  const play = () => {
    if (api?.playerReady) {
      api.play();
    }
  };

  const pause = () => {
    if (api) {
      api.pause();
    }
  };

  const stop = () => {
    if (api) {
      api.stop();
    }
  };

  const setPlaybackSpeed = (speed: number) => {
    if (api) {
      api.playbackSpeed = speed;
    }
  };

  const seekToTime = (timeInMillis: number) => {
    if (api) {
      api.tickPosition = timeInMillis;
    }
  };

  return {
    play,
    pause,
    stop,
    setPlaybackSpeed,
    seekToTime,
  };
}
