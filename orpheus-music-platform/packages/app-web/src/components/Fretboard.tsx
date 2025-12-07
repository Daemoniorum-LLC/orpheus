/**
 * Interactive Fretboard Component
 * Allows clicking and keyboard navigation to add/edit notes on guitar strings
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { cn } from '../lib/utils';

export interface Note {
  string: number; // 1-6 (1 = high E)
  fret: number;   // 0-24
  duration?: string;
  velocity?: number;
  techniques?: string[];
}

export interface FretboardProps {
  /** Number of strings (default 6) */
  numStrings?: number;
  /** Number of frets to display (default 12) */
  numFrets?: number;
  /** Tuning for each string (low to high) */
  tuning?: string[];
  /** Currently placed notes */
  notes?: Note[];
  /** Selected note index */
  selectedNoteIndex?: number;
  /** Callback when fret is clicked */
  onFretClick?: (string: number, fret: number) => void;
  /** Callback when note is clicked */
  onNoteClick?: (noteIndex: number) => void;
  /** Callback when note is deleted */
  onNoteDelete?: (noteIndex: number) => void;
}

export function Fretboard({
  numStrings = 6,
  numFrets = 12,
  tuning = ['E', 'A', 'D', 'G', 'B', 'E'],
  notes = [],
  selectedNoteIndex,
  onFretClick,
  onNoteClick,
  onNoteDelete,
}: FretboardProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [hoveredFret, setHoveredFret] = useState<{ string: number; fret: number } | null>(null);
  const [cursorPosition, setCursorPosition] = useState<{ string: number; fret: number }>({ string: 1, fret: 0 });
  const [isFocused, setIsFocused] = useState(false);
  const [pendingFret, setPendingFret] = useState<string>(''); // For multi-digit fret entry

  // Fret markers (dots) for visual reference
  const fretMarkers = [3, 5, 7, 9, 12, 15, 17, 19, 21, 24];

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (!isFocused) return;

    switch (e.key) {
      case 'ArrowUp':
        e.preventDefault();
        setCursorPosition(prev => ({
          ...prev,
          string: Math.max(1, prev.string - 1)
        }));
        setPendingFret('');
        break;
      case 'ArrowDown':
        e.preventDefault();
        setCursorPosition(prev => ({
          ...prev,
          string: Math.min(numStrings, prev.string + 1)
        }));
        setPendingFret('');
        break;
      case 'ArrowLeft':
        e.preventDefault();
        setCursorPosition(prev => ({
          ...prev,
          fret: Math.max(0, prev.fret - 1)
        }));
        setPendingFret('');
        break;
      case 'ArrowRight':
        e.preventDefault();
        setCursorPosition(prev => ({
          ...prev,
          fret: Math.min(numFrets, prev.fret + 1)
        }));
        setPendingFret('');
        break;
      case 'Enter':
      case ' ':
        e.preventDefault();
        handleFretClick(cursorPosition.string, cursorPosition.fret);
        setPendingFret('');
        break;
      case 'Delete':
      case 'Backspace':
        e.preventDefault();
        // Delete note at cursor if exists
        const noteAtCursor = notes.findIndex(
          n => n.string === cursorPosition.string && n.fret === cursorPosition.fret
        );
        if (noteAtCursor >= 0) {
          onNoteDelete?.(noteAtCursor);
        }
        setPendingFret('');
        break;
      case '0': case '1': case '2': case '3': case '4':
      case '5': case '6': case '7': case '8': case '9':
        e.preventDefault();
        // Allow multi-digit fret entry (e.g., "1" then "2" = fret 12)
        const newPending = pendingFret + e.key;
        const fretNum = parseInt(newPending, 10);
        if (fretNum <= numFrets) {
          setPendingFret(newPending);
          // Add note at this fret after a short delay or immediately for single digit
          if (fretNum >= 10 || newPending.length >= 2) {
            onFretClick?.(cursorPosition.string, fretNum);
            setPendingFret('');
          } else {
            // Wait briefly for potential second digit
            setTimeout(() => {
              if (pendingFret === newPending.charAt(0)) {
                onFretClick?.(cursorPosition.string, parseInt(newPending, 10));
                setPendingFret('');
              }
            }, 300);
          }
        }
        break;
      case 'Escape':
        e.preventDefault();
        setPendingFret('');
        containerRef.current?.blur();
        break;
    }
  }, [isFocused, cursorPosition, numStrings, numFrets, notes, onFretClick, onNoteDelete, pendingFret]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  const handleFretClick = (stringNum: number, fretNum: number) => {
    // Update cursor position
    setCursorPosition({ string: stringNum, fret: fretNum });

    // Check if there's already a note here
    const existingNoteIndex = notes.findIndex(
      (n) => n.string === stringNum && n.fret === fretNum
    );

    if (existingNoteIndex >= 0) {
      // Select or delete existing note
      if (selectedNoteIndex === existingNoteIndex) {
        onNoteDelete?.(existingNoteIndex);
      } else {
        onNoteClick?.(existingNoteIndex);
      }
    } else {
      // Add new note
      onFretClick?.(stringNum, fretNum);
    }
  };

  const renderNote = (stringNum: number, fretNum: number) => {
    const noteIndex = notes.findIndex(
      (n) => n.string === stringNum && n.fret === fretNum
    );

    if (noteIndex < 0) return null;

    const isSelected = selectedNoteIndex === noteIndex;

    return (
      <div
        key={`note-${stringNum}-${fretNum}`}
        className={cn(
          'w-8 h-8 rounded-full bg-primary text-primary-foreground',
          'flex items-center justify-center text-sm font-semibold',
          'cursor-pointer absolute z-[2] shadow-md',
          'hover:bg-primary/80 hover:scale-110',
          isSelected && 'border-[3px] border-yellow-400 shadow-[0_0_0_2px_rgba(250,204,21,0.3)]'
        )}
        onClick={(e) => {
          e.stopPropagation();
          onNoteClick?.(noteIndex);
        }}
        title={`String ${stringNum}, Fret ${fretNum}${isSelected ? ' (selected)' : ''}`}
      >
        {fretNum}
      </div>
    );
  };

  return (
    <div
      ref={containerRef}
      className={cn(
        'flex flex-col gap-2 p-4 bg-background rounded-lg border border-border outline-none',
        'focus:border-primary focus:shadow-[0_0_0_2px_hsl(var(--primary)/0.2)]'
      )}
      tabIndex={0}
      onFocus={() => setIsFocused(true)}
      onBlur={() => setIsFocused(false)}
    >
      <div className="flex justify-between items-center mb-2">
        <div className="text-base font-semibold">Fretboard</div>
        <div className="flex items-center gap-3">
          <div className="text-xs text-muted-foreground">
            Tuning: {tuning.slice().reverse().join('-')}
          </div>
          {isFocused && (
            <div className="text-[11px] text-muted-foreground bg-muted px-2 py-1 rounded">
              Navigate | 0-9 Add Note | Del Remove | Esc Exit
            </div>
          )}
        </div>
      </div>

      <div className="flex flex-col gap-1 relative">
        {/* Render strings from high to low (1-6) */}
        {Array.from({ length: numStrings }, (_, i) => {
          const stringNum = numStrings - i; // High E = 1, Low E = 6
          const stringNote = tuning[tuning.length - stringNum] || 'E';

          return (
            <div key={`string-${stringNum}`} className="flex items-center gap-0.5 relative">
              <div className="w-10 text-sm font-semibold text-right text-muted-foreground">
                {stringNote}
              </div>
              <div className="flex flex-1 relative">
                {Array.from({ length: numFrets + 1 }, (_, fretNum) => {
                  const isCursor = isFocused && cursorPosition.string === stringNum && cursorPosition.fret === fretNum;
                  const isHovered = hoveredFret?.string === stringNum && hoveredFret?.fret === fretNum;

                  return (
                    <div
                      key={`fret-${fretNum}`}
                      className={cn(
                        'flex-1 h-10 flex items-center justify-center cursor-pointer relative',
                        'border-r-2 border-border/50 last:border-r-0',
                        'hover:bg-muted',
                        isCursor && 'bg-green-500/20 before:absolute before:inset-1 before:border-2 before:border-dashed before:border-green-500 before:rounded'
                      )}
                      onClick={() => handleFretClick(stringNum, fretNum)}
                      onMouseEnter={() => setHoveredFret({ string: stringNum, fret: fretNum })}
                      onMouseLeave={() => setHoveredFret(null)}
                    >
                      {renderNote(stringNum, fretNum)}
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>

      {/* Fret numbers */}
      <div className="flex ml-10 mt-2 gap-0.5">
        {Array.from({ length: numFrets + 1 }, (_, i) => (
          <div key={`fret-num-${i}`} className="flex-1 text-center text-[11px] text-muted-foreground font-semibold">
            {i === 0 ? 'O' : i}
            {fretMarkers.includes(i) && <div className="mt-0.5"></div>}
          </div>
        ))}
      </div>

      {/* Pending fret input indicator */}
      {pendingFret && (
        <div className="absolute bottom-4 right-4 px-4 py-2 bg-primary text-primary-foreground rounded font-semibold text-lg">
          Fret: {pendingFret}_
        </div>
      )}
    </div>
  );
}
