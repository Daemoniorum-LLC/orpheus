/**
 * Technique Picker Component
 * Allows selecting playing techniques for notes
 */

import { Button } from '@persona-framework/ui';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@persona-framework/ui';
import { Music } from 'lucide-react';
import { cn } from '../lib/utils';

export interface Technique {
  id: string;
  name: string;
  symbol?: string;
  description?: string;
}

export const AVAILABLE_TECHNIQUES: Record<string, Technique[]> = {
  'Articulation': [
    { id: 'hammer-on', name: 'Hammer-on', symbol: 'H', description: 'Legato ascending' },
    { id: 'pull-off', name: 'Pull-off', symbol: 'P', description: 'Legato descending' },
    { id: 'slide', name: 'Slide', symbol: 'S', description: 'Slide between notes' },
    { id: 'bend', name: 'Bend', symbol: 'B', description: 'Pitch bend' },
    { id: 'vibrato', name: 'Vibrato', symbol: '~', description: 'Vibrato effect' },
    { id: 'trill', name: 'Trill', symbol: 'tr', description: 'Rapid alternation' },
  ],
  'Dynamics': [
    { id: 'accent', name: 'Accent', symbol: '>', description: 'Emphasized note' },
    { id: 'staccato', name: 'Staccato', symbol: '.', description: 'Short, detached' },
    { id: 'palm-mute', name: 'Palm Mute', symbol: 'PM', description: 'Muted with palm' },
    { id: 'ghost', name: 'Ghost Note', symbol: '()', description: 'Very quiet note' },
    { id: 'dead-note', name: 'Dead Note', symbol: 'X', description: 'Completely muted' },
  ],
  'Picking': [
    { id: 'down-stroke', name: 'Down Stroke', symbol: '⌄', description: 'Pick downward' },
    { id: 'up-stroke', name: 'Up Stroke', symbol: '^', description: 'Pick upward' },
    { id: 'tremolo', name: 'Tremolo Picking', symbol: 'T', description: 'Rapid picking' },
    { id: 'sweep', name: 'Sweep Picking', symbol: 'SW', description: 'Sweep across strings' },
  ],
  'Special': [
    { id: 'harmonic', name: 'Harmonic', symbol: '<>', description: 'Natural harmonic' },
    { id: 'tap', name: 'Tap', symbol: 'T', description: 'Finger tap' },
    { id: 'slap', name: 'Slap', symbol: 'SL', description: 'Slap bass technique' },
    { id: 'pop', name: 'Pop', symbol: 'PO', description: 'Pop bass technique' },
  ],
};

export interface TechniquePickerProps {
  /** Currently selected techniques */
  selectedTechniques?: string[];
  /** Callback when techniques change */
  onTechniquesChange?: (techniques: string[]) => void;
  /** Show as compact button */
  compact?: boolean;
}

export function TechniquePicker({
  selectedTechniques = [],
  onTechniquesChange,
  compact = false,
}: TechniquePickerProps) {
  const handleToggleTechnique = (techniqueId: string) => {
    const isSelected = selectedTechniques.includes(techniqueId);
    const newTechniques = isSelected
      ? selectedTechniques.filter((t) => t !== techniqueId)
      : [...selectedTechniques, techniqueId];

    onTechniquesChange?.(newTechniques);
  };

  const getTechniqueName = (id: string): string => {
    for (const category of Object.values(AVAILABLE_TECHNIQUES)) {
      const technique = category.find((t) => t.id === id);
      if (technique) return technique.name;
    }
    return id;
  };

  return (
    <div className="flex flex-col gap-2">
      {/* Selected techniques display */}
      {!compact && (
        <div className="flex flex-wrap gap-1 p-2 bg-muted rounded min-h-[40px]">
          {selectedTechniques.length === 0 ? (
            <div className="text-xs text-muted-foreground italic">No techniques selected</div>
          ) : (
            selectedTechniques.map((techId) => (
              <div
                key={techId}
                className="px-2 py-1 rounded bg-primary text-primary-foreground text-xs cursor-pointer hover:bg-primary/80"
                onClick={() => handleToggleTechnique(techId)}
                title="Click to remove"
              >
                {getTechniqueName(techId)}
              </div>
            ))
          )}
        </div>
      )}

      {/* Technique picker menu */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="secondary" size="sm">
            <Music className="h-4 w-4 mr-2" />
            {compact
              ? `${selectedTechniques.length} technique${selectedTechniques.length !== 1 ? 's' : ''}`
              : 'Add Technique'}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent className="w-64">
          {Object.entries(AVAILABLE_TECHNIQUES).map(([category, techniques]) => (
            <DropdownMenuGroup key={category}>
              <DropdownMenuLabel>{category}</DropdownMenuLabel>
              {techniques.map((technique) => (
                <DropdownMenuItem
                  key={technique.id}
                  onClick={() => handleToggleTechnique(technique.id)}
                >
                  <div className="flex items-center gap-2 w-full">
                    <input
                      type="checkbox"
                      checked={selectedTechniques.includes(technique.id)}
                      readOnly
                      className="m-0"
                    />
                    <div className="flex-1">
                      <div className="font-semibold">
                        {technique.name}
                        {technique.symbol && (
                          <span className="ml-2 text-muted-foreground">
                            ({technique.symbol})
                          </span>
                        )}
                      </div>
                      {technique.description && (
                        <div className="text-[11px] text-muted-foreground">
                          {technique.description}
                        </div>
                      )}
                    </div>
                  </div>
                </DropdownMenuItem>
              ))}
            </DropdownMenuGroup>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
