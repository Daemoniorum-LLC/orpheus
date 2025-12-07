/**
 * Typing Animation Component
 * Displays an animated typing indicator with bouncing dots
 */

import { cn } from '../lib/utils';

export interface TypingAnimationProps {
  /** Optional text to show alongside the animation */
  text?: string;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
  /** Additional class names */
  className?: string;
}

export function TypingAnimation({
  text = 'Thinking',
  size = 'md',
  className,
}: TypingAnimationProps) {
  const dotSizes = {
    sm: 'w-1.5 h-1.5',
    md: 'w-2 h-2',
    lg: 'w-2.5 h-2.5',
  };

  const textSizes = {
    sm: 'text-xs',
    md: 'text-sm',
    lg: 'text-base',
  };

  return (
    <div className={cn('flex items-center gap-2', className)}>
      {text && (
        <span className={cn('text-muted-foreground', textSizes[size])}>
          {text}
        </span>
      )}
      <div className="flex gap-1 items-center">
        <span
          className={cn(
            'rounded-full bg-primary/60 animate-bounce',
            dotSizes[size]
          )}
          style={{ animationDelay: '0ms', animationDuration: '600ms' }}
        />
        <span
          className={cn(
            'rounded-full bg-primary/60 animate-bounce',
            dotSizes[size]
          )}
          style={{ animationDelay: '150ms', animationDuration: '600ms' }}
        />
        <span
          className={cn(
            'rounded-full bg-primary/60 animate-bounce',
            dotSizes[size]
          )}
          style={{ animationDelay: '300ms', animationDuration: '600ms' }}
        />
      </div>
    </div>
  );
}

/**
 * Streaming text animation - reveals text character by character
 */
export interface StreamingTextProps {
  /** The full text to display */
  text: string;
  /** Speed in ms per character */
  speed?: number;
  /** Called when animation completes */
  onComplete?: () => void;
  /** Additional class names */
  className?: string;
}

import { useState, useEffect } from 'react';

export function StreamingText({
  text,
  speed = 20,
  onComplete,
  className,
}: StreamingTextProps) {
  const [displayedText, setDisplayedText] = useState('');
  const [currentIndex, setCurrentIndex] = useState(0);

  useEffect(() => {
    if (currentIndex < text.length) {
      const timer = setTimeout(() => {
        setDisplayedText(text.slice(0, currentIndex + 1));
        setCurrentIndex(currentIndex + 1);
      }, speed);

      return () => clearTimeout(timer);
    } else if (onComplete && currentIndex === text.length) {
      onComplete();
    }
  }, [currentIndex, text, speed, onComplete]);

  // Reset when text changes
  useEffect(() => {
    setDisplayedText('');
    setCurrentIndex(0);
  }, [text]);

  return (
    <span className={className}>
      {displayedText}
      {currentIndex < text.length && (
        <span className="inline-block w-0.5 h-4 bg-primary animate-pulse ml-0.5" />
      )}
    </span>
  );
}
