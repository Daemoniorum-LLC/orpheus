/**
 * Voice Command Overlay - Shows visual feedback for voice recognition state
 * Displays what's being heard and recognized commands
 */

import { useEffect, useState } from 'react';
import { Mic, Check, AlertCircle, Volume2 } from 'lucide-react';
import { useVoiceCommandState } from '../hooks/useVoiceCommands';
import { cn } from '../lib/utils';

export function VoiceCommandOverlay() {
  const {
    state,
    transcript,
    matchedCommand,
    error,
    isSupported,
  } = useVoiceCommandState();

  const [visible, setVisible] = useState(false);
  const [displayText, setDisplayText] = useState('');

  // Show/hide overlay based on state
  useEffect(() => {
    if (state === 'idle') {
      // Delay hiding to show final state
      const timeout = setTimeout(() => setVisible(false), 300);
      return () => clearTimeout(timeout);
    } else {
      setVisible(true);
    }
  }, [state]);

  // Update display text
  useEffect(() => {
    if (state === 'listening') {
      setDisplayText(transcript || 'Listening...');
    } else if (state === 'processing') {
      setDisplayText(`Processing: "${transcript}"`);
    } else if (state === 'success' && matchedCommand) {
      setDisplayText(`✓ ${matchedCommand.description}`);
    } else if (state === 'error') {
      setDisplayText(error || 'Command not recognized');
    }
  }, [state, transcript, matchedCommand, error]);

  if (!isSupported || !visible) {
    return null;
  }

  return (
    <div
      className={cn(
        'fixed top-4 left-1/2 -translate-x-1/2 z-[10001]',
        'pointer-events-none transition-all duration-300',
        visible ? 'opacity-100 translate-y-0' : 'opacity-0 -translate-y-4'
      )}
      role="status"
      aria-live="polite"
      aria-label="Voice command status"
    >
      <div
        className={cn(
          'flex items-center gap-3 px-6 py-4 rounded-2xl shadow-2xl backdrop-blur-md',
          'border min-w-[300px] max-w-[500px]',
          state === 'listening' && 'bg-blue-500/90 border-blue-400 text-white',
          state === 'processing' && 'bg-yellow-500/90 border-yellow-400 text-white',
          state === 'success' && 'bg-green-500/90 border-green-400 text-white',
          state === 'error' && 'bg-red-500/90 border-red-400 text-white'
        )}
      >
        {/* Icon */}
        <div className="flex-shrink-0">
          {state === 'listening' && (
            <div className="relative">
              <Mic className="h-6 w-6" />
              {/* Pulsing rings animation */}
              <div className="absolute inset-0 animate-ping opacity-75">
                <Mic className="h-6 w-6" />
              </div>
            </div>
          )}
          {state === 'processing' && (
            <Volume2 className="h-6 w-6 animate-pulse" />
          )}
          {state === 'success' && (
            <Check className="h-6 w-6 animate-success-scale" />
          )}
          {state === 'error' && (
            <AlertCircle className="h-6 w-6" />
          )}
        </div>

        {/* Text */}
        <div className="flex-1 min-w-0">
          <div className="text-sm font-medium truncate">
            {displayText}
          </div>
          {state === 'listening' && !transcript && (
            <div className="text-xs opacity-75 mt-0.5">
              Say "Orpheus" + command (e.g., "Orpheus play")
            </div>
          )}
        </div>

        {/* Sound wave visualization for listening state */}
        {state === 'listening' && (
          <div className="flex items-center gap-0.5 h-6">
            {[...Array(5)].map((_, i) => (
              <div
                key={i}
                className="w-1 bg-white/80 rounded-full animate-soundwave"
                style={{
                  animationDelay: `${i * 0.1}s`,
                  height: `${Math.random() * 16 + 8}px`,
                }}
              />
            ))}
          </div>
        )}
      </div>

      {/* Quick commands hint when listening */}
      {state === 'listening' && !transcript && (
        <div className="mt-2 flex justify-center gap-2 flex-wrap">
          {['play', 'pause', 'save', 'compose', 'mix'].map(cmd => (
            <span
              key={cmd}
              className="px-2 py-1 bg-black/50 text-white/80 text-xs rounded-full backdrop-blur-sm"
            >
              {cmd}
            </span>
          ))}
        </div>
      )}

      {/* Soundwave animation styles */}
      <style>{`
        @keyframes soundwave {
          0%, 100% {
            height: 8px;
          }
          50% {
            height: 24px;
          }
        }
        .animate-soundwave {
          animation: soundwave 0.5s ease-in-out infinite;
        }
      `}</style>
    </div>
  );
}
