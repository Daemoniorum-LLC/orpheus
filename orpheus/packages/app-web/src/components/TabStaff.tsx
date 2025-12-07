/**
 * Tab Staff Component
 * Displays notes in traditional ASCII tablature format
 */

import { makeStyles, shorthands, tokens } from '@fluentui/react-components';
import type { Note } from './Fretboard';

const useStyles = makeStyles({
  container: {
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    fontFamily: 'monospace',
    fontSize: '14px',
    overflowX: 'auto',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  title: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    fontFamily: 'inherit',
  },
  measureInfo: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
  },
  staff: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('2px'),
  },
  stringLine: {
    display: 'flex',
    alignItems: 'center',
    height: '24px',
  },
  stringLabel: {
    width: '24px',
    fontWeight: 600,
    color: tokens.colorNeutralForeground2,
    textAlign: 'right',
    paddingRight: '8px',
  },
  tabLine: {
    display: 'flex',
    alignItems: 'center',
    flex: 1,
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('2px'),
    position: 'relative',
    minWidth: '400px',
  },
  beat: {
    width: '40px',
    height: '24px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    position: 'relative',
    ...shorthands.borderRight('1px', 'solid', tokens.colorNeutralStroke2),
    ':last-child': {
      ...shorthands.borderRight('none'),
    },
  },
  fretNumber: {
    fontSize: '14px',
    fontWeight: 600,
    color: tokens.colorBrandForeground1,
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.padding('2px', '4px'),
    ...shorthands.borderRadius('2px'),
    minWidth: '16px',
    textAlign: 'center',
  },
  selectedFret: {
    backgroundColor: tokens.colorBrandBackground,
    color: '#ffffff',
  },
  emptyBeat: {
    color: tokens.colorNeutralForeground4,
  },
  measureBar: {
    width: '2px',
    height: '100%',
    backgroundColor: tokens.colorNeutralStroke1,
  },
  legend: {
    marginTop: '12px',
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    display: 'flex',
    ...shorthands.gap('16px'),
    flexWrap: 'wrap',
  },
  legendItem: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('4px'),
  },
});

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
  const styles = useStyles();

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
        className={styles.beat}
        onClick={() => noteIndex >= 0 && onNoteClick?.(noteIndex)}
        style={{ cursor: noteIndex >= 0 ? 'pointer' : 'default' }}
      >
        {note ? (
          <span className={`${styles.fretNumber} ${isSelected ? styles.selectedFret : ''}`}>
            {note.fret}
          </span>
        ) : (
          <span className={styles.emptyBeat}>—</span>
        )}
      </div>
    );
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>📋 Tab Notation</div>
        <div className={styles.measureInfo}>
          Measure {measureNumber} • {notes.length} note{notes.length !== 1 ? 's' : ''}
        </div>
      </div>

      <div className={styles.staff}>
        {/* Render strings from high E (1) to low E (6) */}
        {Array.from({ length: numStrings }, (_, i) => {
          const stringNum = i + 1; // 1 = high E, 6 = low E
          const stringLabel = tuning[tuning.length - stringNum] || 'E';

          return (
            <div key={`string-${stringNum}`} className={styles.stringLine}>
              <div className={styles.stringLabel}>{stringLabel}</div>
              <div className={styles.tabLine}>
                {beats.map((beat, beatIndex) => renderBeatCell(stringNum, beat, beatIndex))}
              </div>
            </div>
          );
        })}
      </div>

      <div className={styles.legend}>
        <div className={styles.legendItem}>
          <span style={{ fontWeight: 600 }}>—</span>
          <span>Empty beat</span>
        </div>
        <div className={styles.legendItem}>
          <span className={styles.fretNumber} style={{ fontSize: '11px', padding: '1px 3px' }}>5</span>
          <span>Fret number</span>
        </div>
        <div className={styles.legendItem}>
          <span className={`${styles.fretNumber} ${styles.selectedFret}`} style={{ fontSize: '11px', padding: '1px 3px' }}>5</span>
          <span>Selected</span>
        </div>
      </div>
    </div>
  );
}
