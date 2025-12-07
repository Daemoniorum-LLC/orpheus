/**
 * Voice Command Button - Mic button with visual feedback for voice control
 */

import { Button, Tooltip, TooltipTrigger, TooltipContent } from '@persona-framework/ui';
import { Mic, MicOff, Loader2 } from 'lucide-react';
import { useVoiceCommandState } from '../hooks/useVoiceCommands';
import { cn } from '../lib/utils';
import { loadPreferences } from './PreferencesDialog';
import { useState, useEffect } from 'react';

interface VoiceCommandButtonProps {
  className?: string;
  showLabel?: boolean;
  continuous?: boolean;
}

export function VoiceCommandButton({
  className,
  showLabel = false,
  continuous = false,
}: VoiceCommandButtonProps) {
  const {
    state,
    isSupported,
    isListening,
    toggleListening,
  } = useVoiceCommandState();

  // Check if voice is enabled in preferences
  const [voiceEnabled, setVoiceEnabled] = useState(() => loadPreferences().voiceEnabled);

  // Re-check preferences periodically (in case user changes them)
  useEffect(() => {
    const interval = setInterval(() => {
      setVoiceEnabled(loadPreferences().voiceEnabled);
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  if (!isSupported) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="sm"
            disabled
            className={cn('opacity-50', className)}
          >
            <MicOff className="h-4 w-4" />
            {showLabel && <span className="ml-2">Voice N/A</span>}
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          Voice commands not supported in this browser
        </TooltipContent>
      </Tooltip>
    );
  }

  if (!voiceEnabled) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="sm"
            className={cn('opacity-50', className)}
          >
            <MicOff className="h-4 w-4" />
            {showLabel && <span className="ml-2">Voice Off</span>}
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          Voice commands disabled - enable in Preferences
        </TooltipContent>
      </Tooltip>
    );
  }

  const getIcon = () => {
    if (state === 'processing') {
      return <Loader2 className="h-4 w-4 animate-spin" />;
    }
    if (isListening) {
      return <Mic className="h-4 w-4 animate-pulse" />;
    }
    return <Mic className="h-4 w-4" />;
  };

  const getTooltip = () => {
    if (isListening) {
      return continuous
        ? 'Voice active - say "stop listening" to disable'
        : 'Listening... Click to cancel';
    }
    return 'Voice Commands (Click or say "Hey Orpheus")';
  };

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant={isListening ? 'default' : 'ghost'}
          size="sm"
          onClick={() => toggleListening(continuous)}
          className={cn(
            'transition-all',
            isListening && 'bg-red-500 hover:bg-red-600 text-white animate-pulse',
            state === 'success' && 'bg-green-500 hover:bg-green-600 text-white',
            className
          )}
          aria-label={isListening ? 'Stop voice commands' : 'Start voice commands'}
          aria-pressed={isListening}
        >
          {getIcon()}
          {showLabel && (
            <span className="ml-2">
              {isListening ? 'Listening...' : 'Voice'}
            </span>
          )}
        </Button>
      </TooltipTrigger>
      <TooltipContent>{getTooltip()}</TooltipContent>
    </Tooltip>
  );
}
