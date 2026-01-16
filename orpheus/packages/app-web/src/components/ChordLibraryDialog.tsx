/**
 * Chord Library Dialog
 * Browse and insert chords from the 100+ chord library
 */

import {
  Dialog,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Button,
  Input,
  makeStyles,
  shorthands,
  tokens,
} from '@fluentui/react-components';
import { Search24Regular, Dismiss24Regular } from '@fluentui/react-icons';
import { useState, useMemo } from 'react';

const useStyles = makeStyles({
  dialogSurface: {
    maxWidth: '800px',
    width: '90vw',
  },
  searchContainer: {
    marginBottom: '16px',
  },
  chordGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
    ...shorthands.gap('12px'),
    maxHeight: '500px',
    overflowY: 'auto',
    ...shorthands.padding('8px'),
  },
  chordCard: {
    ...shorthands.padding('12px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke2),
    ...shorthands.borderRadius('8px'),
    cursor: 'pointer',
    transition: 'all 0.2s',
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
      ...shorthands.borderColor(tokens.colorBrandStroke1),
      transform: 'translateY(-2px)',
      boxShadow: tokens.shadow8,
    },
  },
  chordName: {
    fontSize: '16px',
    fontWeight: 600,
    marginBottom: '8px',
  },
  chordFingering: {
    display: 'flex',
    ...shorthands.gap('4px'),
    fontSize: '12px',
    fontFamily: 'monospace',
    color: tokens.colorNeutralForeground2,
    marginBottom: '4px',
  },
  chordCategory: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    textTransform: 'uppercase',
    fontWeight: 600,
  },
  fretDot: {
    width: '20px',
    textAlign: 'center',
  },
  categoryFilter: {
    display: 'flex',
    flexWrap: 'wrap',
    ...shorthands.gap('8px'),
    marginBottom: '16px',
  },
  categoryChip: {
    ...shorthands.padding('4px', '12px'),
    ...shorthands.borderRadius('16px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke2),
    cursor: 'pointer',
    fontSize: '12px',
    transition: 'all 0.2s',
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
    },
  },
  categoryChipActive: {
    backgroundColor: tokens.colorBrandBackground,
    color: '#ffffff',
    ...shorthands.borderColor(tokens.colorBrandStroke1),
  },
});

// Sample chord library (in production, import from @orpheus/music-theory)
const CHORD_LIBRARY = [
  // Major Chords
  { name: 'C Major', root: 'C', type: 'major', category: 'Major', fingering: ['X', '3', '2', '0', '1', '0'], frets: [0, 3, 2, 0, 1, 0] },
  { name: 'D Major', root: 'D', type: 'major', category: 'Major', fingering: ['X', 'X', '0', '2', '3', '2'], frets: [0, 0, 0, 2, 3, 2] },
  { name: 'E Major', root: 'E', type: 'major', category: 'Major', fingering: ['0', '2', '2', '1', '0', '0'], frets: [0, 2, 2, 1, 0, 0] },
  { name: 'F Major', root: 'F', type: 'major', category: 'Major', fingering: ['1', '3', '3', '2', '1', '1'], frets: [1, 3, 3, 2, 1, 1] },
  { name: 'G Major', root: 'G', type: 'major', category: 'Major', fingering: ['3', '2', '0', '0', '0', '3'], frets: [3, 2, 0, 0, 0, 3] },
  { name: 'A Major', root: 'A', type: 'major', category: 'Major', fingering: ['X', '0', '2', '2', '2', '0'], frets: [0, 0, 2, 2, 2, 0] },
  { name: 'B Major', root: 'B', type: 'major', category: 'Major', fingering: ['X', '2', '4', '4', '4', '2'], frets: [0, 2, 4, 4, 4, 2] },

  // Minor Chords
  { name: 'Am', root: 'A', type: 'minor', category: 'Minor', fingering: ['X', '0', '2', '2', '1', '0'], frets: [0, 0, 2, 2, 1, 0] },
  { name: 'Dm', root: 'D', type: 'minor', category: 'Minor', fingering: ['X', 'X', '0', '2', '3', '1'], frets: [0, 0, 0, 2, 3, 1] },
  { name: 'Em', root: 'E', type: 'minor', category: 'Minor', fingering: ['0', '2', '2', '0', '0', '0'], frets: [0, 2, 2, 0, 0, 0] },
  { name: 'Fm', root: 'F', type: 'minor', category: 'Minor', fingering: ['1', '3', '3', '1', '1', '1'], frets: [1, 3, 3, 1, 1, 1] },

  // Dominant 7th
  { name: 'C7', root: 'C', type: 'dominant7', category: '7th', fingering: ['X', '3', '2', '3', '1', '0'], frets: [0, 3, 2, 3, 1, 0] },
  { name: 'D7', root: 'D', type: 'dominant7', category: '7th', fingering: ['X', 'X', '0', '2', '1', '2'], frets: [0, 0, 0, 2, 1, 2] },
  { name: 'E7', root: 'E', type: 'dominant7', category: '7th', fingering: ['0', '2', '0', '1', '0', '0'], frets: [0, 2, 0, 1, 0, 0] },
  { name: 'G7', root: 'G', type: 'dominant7', category: '7th', fingering: ['3', '2', '0', '0', '0', '1'], frets: [3, 2, 0, 0, 0, 1] },
  { name: 'A7', root: 'A', type: 'dominant7', category: '7th', fingering: ['X', '0', '2', '0', '2', '0'], frets: [0, 0, 2, 0, 2, 0] },

  // Sus and Add chords
  { name: 'Dsus4', root: 'D', type: 'sus4', category: 'Sus/Add', fingering: ['X', 'X', '0', '2', '3', '3'], frets: [0, 0, 0, 2, 3, 3] },
  { name: 'Esus4', root: 'E', type: 'sus4', category: 'Sus/Add', fingering: ['0', '2', '2', '2', '0', '0'], frets: [0, 2, 2, 2, 0, 0] },
  { name: 'Asus2', root: 'A', type: 'sus2', category: 'Sus/Add', fingering: ['X', '0', '2', '2', '0', '0'], frets: [0, 0, 2, 2, 0, 0] },
  { name: 'Cadd9', root: 'C', type: 'add9', category: 'Sus/Add', fingering: ['X', '3', '2', '0', '3', '0'], frets: [0, 3, 2, 0, 3, 0] },

  // Power Chords
  { name: 'E5', root: 'E', type: 'power', category: 'Power', fingering: ['0', '2', '2', 'X', 'X', 'X'], frets: [0, 2, 2, 0, 0, 0] },
  { name: 'A5', root: 'A', type: 'power', category: 'Power', fingering: ['X', '0', '2', '2', 'X', 'X'], frets: [0, 0, 2, 2, 0, 0] },
  { name: 'D5', root: 'D', type: 'power', category: 'Power', fingering: ['X', 'X', '0', '2', '3', 'X'], frets: [0, 0, 0, 2, 3, 0] },
];

const CATEGORIES = ['All', 'Major', 'Minor', '7th', 'Sus/Add', 'Power'];

export interface ChordLibraryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onChordSelect?: (chord: typeof CHORD_LIBRARY[0]) => void;
}

export function ChordLibraryDialog({
  open,
  onOpenChange,
  onChordSelect,
}: ChordLibraryDialogProps) {
  const styles = useStyles();
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('All');

  const filteredChords = useMemo(() => {
    return CHORD_LIBRARY.filter((chord) => {
      const matchesSearch =
        searchQuery === '' ||
        chord.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        chord.root.toLowerCase().includes(searchQuery.toLowerCase());

      const matchesCategory =
        selectedCategory === 'All' || chord.category === selectedCategory;

      return matchesSearch && matchesCategory;
    });
  }, [searchQuery, selectedCategory]);

  const handleChordSelect = (chord: typeof CHORD_LIBRARY[0]) => {
    onChordSelect?.(chord);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={(_, data) => onOpenChange(data.open)}>
      <DialogSurface className={styles.dialogSurface}>
        <DialogBody>
          <DialogTitle>
            🎸 Chord Library
            <Button
              appearance="subtle"
              icon={<Dismiss24Regular />}
              onClick={() => onOpenChange(false)}
              style={{ float: 'right' }}
            />
          </DialogTitle>
          <DialogContent>
            {/* Search */}
            <div className={styles.searchContainer}>
              <Input
                placeholder="Search chords... (e.g., C, Am, D7)"
                value={searchQuery}
                onChange={(_, data) => setSearchQuery(data.value)}
                contentBefore={<Search24Regular />}
                size="large"
              />
            </div>

            {/* Category filter */}
            <div className={styles.categoryFilter}>
              {CATEGORIES.map((category) => (
                <div
                  key={category}
                  className={`${styles.categoryChip} ${
                    selectedCategory === category ? styles.categoryChipActive : ''
                  }`}
                  onClick={() => setSelectedCategory(category)}
                >
                  {category}
                </div>
              ))}
            </div>

            {/* Chord grid */}
            <div className={styles.chordGrid}>
              {filteredChords.map((chord, index) => (
                <div
                  key={`${chord.name}-${index}`}
                  className={styles.chordCard}
                  onClick={() => handleChordSelect(chord)}
                >
                  <div className={styles.chordName}>{chord.name}</div>
                  <div className={styles.chordFingering}>
                    {chord.fingering.map((fret, i) => (
                      <div key={i} className={styles.fretDot}>
                        {fret}
                      </div>
                    ))}
                  </div>
                  <div className={styles.chordCategory}>{chord.category}</div>
                </div>
              ))}
            </div>

            {filteredChords.length === 0 && (
              <div style={{ textAlign: 'center', padding: '40px', color: tokens.colorNeutralForeground3 }}>
                No chords found matching "{searchQuery}"
              </div>
            )}
          </DialogContent>
          <DialogActions>
            <Button appearance="secondary" onClick={() => onOpenChange(false)}>
              Close
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
