/**
 * Interactive Fretboard Component
 * Allows clicking to add/edit notes on guitar strings
 */

import { makeStyles, shorthands, tokens } from '@fluentui/react-components';
import { useState } from 'react';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px',
  },
  title: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
  },
  tuningInfo: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
  },
  fretboard: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('4px'),
    position: 'relative',
  },
  string: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('2px'),
    position: 'relative',
  },
  stringLabel: {
    width: '40px',
    fontSize: '14px',
    fontWeight: 600,
    textAlign: 'right',
    color: tokens.colorNeutralForeground2,
  },
  stringLine: {
    flex: 1,
    height: '2px',
    backgroundColor: tokens.colorNeutralStroke2,
    position: 'relative',
    display: 'flex',
    alignItems: 'center',
  },
  fretContainer: {
    display: 'flex',
    flex: 1,
    position: 'relative',
  },
  fret: {
    flex: 1,
    height: '40px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    cursor: 'pointer',
    position: 'relative',
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
    },
    '::after': {
      content: '""',
      position: 'absolute',
      right: '0',
      top: '0',
      bottom: '0',
      width: '2px',
      backgroundColor: tokens.colorNeutralStroke3,
    },
  },
  fretMarker: {
    position: 'absolute',
    bottom: '-20px',
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    left: '50%',
    transform: 'translateX(-50%)',
  },
  note: {
    width: '32px',
    height: '32px',
    ...shorthands.borderRadius('50%'),
    backgroundColor: tokens.colorBrandBackground,
    color: '#ffffff',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    fontSize: '14px',
    fontWeight: 600,
    cursor: 'pointer',
    position: 'absolute',
    zIndex: 2,
    boxShadow: tokens.shadow8,
    ':hover': {
      backgroundColor: tokens.colorBrandBackgroundHover,
      transform: 'scale(1.1)',
    },
  },
  selectedNote: {
    ...shorthands.border('3px', 'solid', tokens.colorPaletteYellowBorder1),
    boxShadow: `0 0 0 2px ${tokens.colorPaletteYellowBackground2}`,
  },
  fretNumbers: {
    display: 'flex',
    marginLeft: '40px',
    marginTop: '8px',
    ...shorthands.gap('2px'),
  },
  fretNumber: {
    flex: 1,
    textAlign: 'center',
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    fontWeight: 600,
  },
});

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
  const styles = useStyles();
  const [hoveredFret, setHoveredFret] = useState<{ string: number; fret: number } | null>(null);

  // Fret markers (dots) for visual reference
  const fretMarkers = [3, 5, 7, 9, 12, 15, 17, 19, 21, 24];

  const handleFretClick = (stringNum: number, fretNum: number) => {
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
        className={`${styles.note} ${isSelected ? styles.selectedNote : ''}`}
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
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>Fretboard</div>
        <div className={styles.tuningInfo}>
          Tuning: {tuning.slice().reverse().join('-')}
        </div>
      </div>

      <div className={styles.fretboard}>
        {/* Render strings from high to low (1-6) */}
        {Array.from({ length: numStrings }, (_, i) => {
          const stringNum = numStrings - i; // High E = 1, Low E = 6
          const stringNote = tuning[tuning.length - stringNum] || 'E';

          return (
            <div key={`string-${stringNum}`} className={styles.string}>
              <div className={styles.stringLabel}>{stringNote}</div>
              <div className={styles.fretContainer}>
                {Array.from({ length: numFrets + 1 }, (_, fretNum) => (
                  <div
                    key={`fret-${fretNum}`}
                    className={styles.fret}
                    onClick={() => handleFretClick(stringNum, fretNum)}
                    onMouseEnter={() => setHoveredFret({ string: stringNum, fret: fretNum })}
                    onMouseLeave={() => setHoveredFret(null)}
                    style={{
                      backgroundColor:
                        hoveredFret?.string === stringNum && hoveredFret?.fret === fretNum
                          ? tokens.colorNeutralBackground1Hover
                          : 'transparent',
                    }}
                  >
                    {renderNote(stringNum, fretNum)}
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>

      {/* Fret numbers */}
      <div className={styles.fretNumbers}>
        {Array.from({ length: numFrets + 1 }, (_, i) => (
          <div key={`fret-num-${i}`} className={styles.fretNumber}>
            {i === 0 ? 'O' : i}
            {fretMarkers.includes(i) && <div style={{ marginTop: '2px' }}>•</div>}
          </div>
        ))}
      </div>
    </div>
  );
}
