/**
 * Tab Staff Component
 * Displays notes in traditional ASCII tablature format
 */

import type { Note } from './Fretboard';
import { cn } from '../lib/utils';

interface Beat {
  notes: Note[];
  position: number; // Beat position in measure (0-based)
}

export interface TabStaffProps {
  /** Notes to display */
  notes: Note[];
  /** Number of strings (default 6) */
  numStrings?: number;
  /** Tuning labels (low to high) */
  tuning?: string[];
  /** Currently selected note index */
  selectedNoteIndex?: number;
  /** Measure number for display */
  measureNumber?: number;
  /** Beats per measure */
  beatsPerMeasure?: number;
  /** Callback when note is clicked */
  onNoteClick?: (noteIndex: number) => void;
}

export function TabStaff({
  notes,
  numStrings = 6,
  tuning = ['E', 'A', 'D', 'G', 'B', 'E'],
  selectedNoteIndex,
  measureNumber = 1,
  beatsPerMeasure = 4,
  onNoteClick,
}: TabStaffProps) {
  // Group notes into beats (simplified - one note per beat position)
  // In a real implementation, this would use actual timing data
  const beats: Beat[] = [];
  const notesPerBeat = Math.ceil(notes.length / beatsPerMeasure) || 1;

  for (let i = 0; i < Math.max(beatsPerMeasure, Math.ceil(notes.length / notesPerBeat)); i++) {
    const beatNotes = notes.slice(i * notesPerBeat, (i + 1) * notesPerBeat);
    beats.push({
      notes: beatNotes,
      position: i,
    });
  }

  // Ensure we have at least beatsPerMeasure beats
  while (beats.length < beatsPerMeasure) {
    beats.push({ notes: [], position: beats.length });
  }

  const renderBeatCell = (stringNum: number, beat: Beat, beatIndex: number) => {
    // Find note on this string in this beat
    const noteIndex = notes.findIndex(
      (n, idx) =>
        n.string === stringNum &&
        beat.notes.includes(n)
    );
    const note = noteIndex >= 0 ? notes[noteIndex] : null;
    const isSelected = noteIndex >= 0 && selectedNoteIndex === noteIndex;

    return (
      <div
        key={`beat-${beatIndex}-string-${stringNum}`}
        className="w-10 h-6 flex items-center justify-center relative border-r border-border last:border-r-0"
        onClick={() => noteIndex >= 0 && onNoteClick?.(noteIndex)}
        style={{ cursor: noteIndex >= 0 ? 'pointer' : 'default' }}
      >
        {note ? (
          <span
            className={cn(
              'text-sm font-semibold px-1 py-0.5 rounded min-w-[16px] text-center',
              isSelected
                ? 'bg-primary text-primary-foreground'
                : 'bg-background text-primary'
            )}
          >
            {note.fret}
          </span>
        ) : (
          <span className="text-muted-foreground/50">—</span>
        )}
      </div>
    );
  };

  return (
    <div className="p-4 bg-background rounded-lg border border-border font-mono text-sm overflow-x-auto">
      <div className="flex justify-between items-center mb-3">
        <div className="text-base font-semibold">📋 Tab Notation</div>
        <div className="text-xs text-muted-foreground">
          Measure {measureNumber} • {notes.length} note{notes.length !== 1 ? 's' : ''}
        </div>
      </div>

      <div className="flex flex-col gap-0.5">
        {/* Render strings from high E (1) to low E (6) */}
        {Array.from({ length: numStrings }, (_, i) => {
          const stringNum = i + 1; // 1 = high E, 6 = low E
          const stringLabel = tuning[tuning.length - stringNum] || 'E';

          return (
            <div key={`string-${stringNum}`} className="flex items-center h-6">
              <div className="w-6 font-semibold text-muted-foreground text-right pr-2">
                {stringLabel}
              </div>
              <div className="flex items-center flex-1 bg-muted rounded-sm relative min-w-[400px]">
                {beats.map((beat, beatIndex) => renderBeatCell(stringNum, beat, beatIndex))}
              </div>
            </div>
          );
        })}
      </div>

      <div className="mt-3 text-[11px] text-muted-foreground flex gap-4 flex-wrap">
        <div className="flex items-center gap-1">
          <span className="font-semibold">—</span>
          <span>Empty beat</span>
        </div>
        <div className="flex items-center gap-1">
          <span className="text-[11px] px-1 py-0.5 bg-background text-primary font-semibold rounded">5</span>
          <span>Fret number</span>
        </div>
        <div className="flex items-center gap-1">
          <span className="text-[11px] px-1 py-0.5 bg-primary text-primary-foreground font-semibold rounded">5</span>
          <span>Selected</span>
        </div>
      </div>
    </div>
  );
}
