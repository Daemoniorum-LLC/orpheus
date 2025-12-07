/**
 * Chord Library Dialog
 * Browse and insert chords from the 100+ chord library
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@persona-framework/ui';
import { Button, Input } from '@persona-framework/ui';
import { Search, X } from 'lucide-react';
import { useState, useMemo } from 'react';
import { cn } from '../lib/utils';

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
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-[800px] w-[95vw] sm:w-[90vw] max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center justify-between">
            <span>Chord Library</span>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onOpenChange(false)}
            >
              <X className="h-4 w-4" />
            </Button>
          </DialogTitle>
        </DialogHeader>

        {/* Search */}
        <div className="mb-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search chords... (e.g., C, Am, D7)"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10"
            />
          </div>
        </div>

        {/* Category filter */}
        <div className="flex flex-wrap gap-2 mb-4">
          {CATEGORIES.map((category) => (
            <button
              key={category}
              type="button"
              className={cn(
                'px-3 py-1 rounded-full border cursor-pointer text-xs transition-all',
                'hover:bg-muted hover:-translate-y-0.5 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1',
                selectedCategory === category
                  ? 'bg-primary text-primary-foreground border-primary'
                  : 'border-border'
              )}
              onClick={() => setSelectedCategory(category)}
            >
              {category}
            </button>
          ))}
        </div>

        {/* Chord grid */}
        <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3 max-h-[500px] overflow-y-auto p-2">
          {filteredChords.map((chord, index) => (
            <div
              key={`${chord.name}-${index}`}
              className="p-3 border border-border rounded-lg cursor-pointer transition-all hover:bg-muted hover:border-primary hover:-translate-y-0.5 hover:shadow-md"
              onClick={() => handleChordSelect(chord)}
            >
              <div className="text-base font-semibold mb-2">{chord.name}</div>
              <div className="flex gap-1 text-xs font-mono text-muted-foreground mb-1">
                {chord.fingering.map((fret, i) => (
                  <div key={i} className="w-5 text-center">
                    {fret}
                  </div>
                ))}
              </div>
              <div className="text-[11px] text-muted-foreground uppercase font-semibold">
                {chord.category}
              </div>
            </div>
          ))}
        </div>

        {filteredChords.length === 0 && (
          <div className="text-center py-10 text-muted-foreground">
            No chords found matching "{searchQuery}"
          </div>
        )}

        <DialogFooter>
          <Button variant="secondary" onClick={() => onOpenChange(false)}>
            Close
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
