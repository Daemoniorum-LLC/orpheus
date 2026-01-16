/**
 * Mode Selector - Switch between the 6 modes of Orpheus
 * Accessible with keyboard navigation (Arrow keys, Enter, Space)
 */

import { Button, Tooltip, TooltipTrigger, TooltipContent } from '@persona-framework/ui';
import {
  Music,
  Mic,
  SlidersHorizontal,
  Sparkles,
  GraduationCap,
  Globe,
} from 'lucide-react';
import { useAppStore, useProject, type AppMode } from '../store/app-store';
import { useRef, useCallback } from 'react';
import { preloadModes } from '../App';
import { cn } from '../lib/utils';

interface ModeInfo {
  id: AppMode;
  label: string;
  emoji: string;
  icon: React.ReactElement;
  description: string;
  shortcut: string;
  requiresProject?: boolean;
}

const modes: ModeInfo[] = [
  {
    id: 'compose',
    label: 'Compose',
    emoji: '🎼',
    icon: <Music className="h-4 w-4" />,
    description: 'Tablature and notation editing (Cadenza AI)',
    shortcut: 'Ctrl+1',
    requiresProject: false,
  },
  {
    id: 'record',
    label: 'Record',
    emoji: '🎙️',
    icon: <Mic className="h-4 w-4" />,
    description: 'Multi-track audio recording (Nexus DAW)',
    shortcut: 'Ctrl+2',
    requiresProject: false,
  },
  {
    id: 'mix',
    label: 'Mix',
    emoji: '🎚️',
    icon: <SlidersHorizontal className="h-4 w-4" />,
    description: 'Professional mixing console (Nexus DAW)',
    shortcut: 'Ctrl+3',
    requiresProject: true,
  },
  {
    id: 'master',
    label: 'Master',
    emoji: '✨',
    icon: <Sparkles className="h-4 w-4" />,
    description: 'AI-powered mastering (Nexus DAW)',
    shortcut: 'Ctrl+4',
    requiresProject: true,
  },
  {
    id: 'practice',
    label: 'Practice',
    emoji: '🎸',
    icon: <GraduationCap className="h-4 w-4" />,
    description: 'Speed trainer and learning tools',
    shortcut: 'Ctrl+5',
    requiresProject: false,
  },
  {
    id: 'distribute',
    label: 'Distribute',
    emoji: '🌍',
    icon: <Globe className="h-4 w-4" />,
    description: 'Music distribution to streaming platforms',
    shortcut: 'Ctrl+6',
    requiresProject: true,
  },
];

export function ModeSelector() {
  const { mode, setMode } = useAppStore();
  const project = useProject();
  const buttonRefs = useRef<(HTMLButtonElement | null)[]>([]);

  const handleModeChange = (newMode: AppMode) => {
    setMode(newMode);
  };

  // Preload mode on hover for instant switching
  const handleModeHover = useCallback((modeId: AppMode) => {
    const preloader = preloadModes[modeId as keyof typeof preloadModes];
    if (typeof preloader === 'function') {
      preloader().catch(() => {
        // Silently ignore preload failures
      });
    }
  }, []);

  // Get the index of a mode, skipping disabled ones for navigation
  const getEnabledModeIndices = useCallback(() => {
    return modes
      .map((m, i) => ({ mode: m, index: i }))
      .filter(({ mode: m }) => !(m.requiresProject && !project))
      .map(({ index }) => index);
  }, [project]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      const enabledIndices = getEnabledModeIndices();
      const currentEnabledIndex = enabledIndices.indexOf(currentIndex);

      let nextIndex: number | null = null;

      switch (e.key) {
        case 'ArrowRight':
        case 'ArrowDown':
          e.preventDefault();
          // Move to next enabled mode
          if (currentEnabledIndex < enabledIndices.length - 1) {
            nextIndex = enabledIndices[currentEnabledIndex + 1];
          } else {
            // Wrap to first
            nextIndex = enabledIndices[0];
          }
          break;

        case 'ArrowLeft':
        case 'ArrowUp':
          e.preventDefault();
          // Move to previous enabled mode
          if (currentEnabledIndex > 0) {
            nextIndex = enabledIndices[currentEnabledIndex - 1];
          } else {
            // Wrap to last
            nextIndex = enabledIndices[enabledIndices.length - 1];
          }
          break;

        case 'Home':
          e.preventDefault();
          nextIndex = enabledIndices[0];
          break;

        case 'End':
          e.preventDefault();
          nextIndex = enabledIndices[enabledIndices.length - 1];
          break;
      }

      if (nextIndex !== null && buttonRefs.current[nextIndex]) {
        buttonRefs.current[nextIndex]?.focus();
      }
    },
    [getEnabledModeIndices]
  );

  const currentModeIndex = modes.findIndex((m) => m.id === mode);

  return (
    <div
      className="flex gap-1.5 sm:gap-2 p-2 sm:p-3 bg-secondary border-b border-border overflow-x-auto scrollbar-thin"
      role="tablist"
      aria-label="Production modes"
    >
      {modes.map((modeInfo, index) => {
        const isDisabled = modeInfo.requiresProject && !project;
        const isActive = mode === modeInfo.id;
        const tooltipContent = isDisabled
          ? `${modeInfo.description} - Requires a project to be loaded`
          : `${modeInfo.description} (${modeInfo.shortcut})`;

        return (
          <Tooltip key={modeInfo.id}>
            <TooltipTrigger asChild>
              <Button
                ref={(el) => {
                  buttonRefs.current[index] = el;
                }}
                role="tab"
                aria-selected={isActive}
                aria-controls={`${modeInfo.id}-panel`}
                tabIndex={isActive ? 0 : -1}
                variant={isActive ? 'default' : 'secondary'}
                size="lg"
                onClick={() => handleModeChange(modeInfo.id)}
                onKeyDown={(e) => handleKeyDown(e, index)}
                onMouseEnter={() => handleModeHover(modeInfo.id)}
                onFocus={() => handleModeHover(modeInfo.id)}
                disabled={isDisabled}
                className={cn(
                  "min-w-0 sm:min-w-[110px] h-9 sm:h-11 font-semibold gap-1 sm:gap-2 px-2 sm:px-4 flex-shrink-0 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                  isDisabled && "cursor-not-allowed opacity-50"
                )}
              >
                <span aria-hidden="true">{modeInfo.emoji}</span>
                <span className="hidden xs:inline sm:inline">{modeInfo.label}</span>
                {isDisabled && (
                  <span className="text-[10px] text-destructive ml-0.5 sm:ml-1" aria-hidden="true">
                    ⚠
                  </span>
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent>{tooltipContent}</TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
}
