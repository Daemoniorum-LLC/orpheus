/**
 * Technique Picker Component
 * Allows selecting playing techniques for notes
 */

import {
  makeStyles,
  shorthands,
  tokens,
  Button,
  Menu,
  MenuTrigger,
  MenuPopover,
  MenuList,
  MenuItem,
  MenuGroup,
  MenuGroupHeader,
} from '@fluentui/react-components';
import { MusicNote224Regular } from '@fluentui/react-icons';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  techniqueList: {
    display: 'flex',
    flexWrap: 'wrap',
    ...shorthands.gap('4px'),
    ...shorthands.padding('8px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('4px'),
    minHeight: '40px',
  },
  techniqueChip: {
    ...shorthands.padding('4px', '8px'),
    ...shorthands.borderRadius('4px'),
    backgroundColor: tokens.colorBrandBackground,
    color: '#ffffff',
    fontSize: '12px',
    cursor: 'pointer',
    ':hover': {
      backgroundColor: tokens.colorBrandBackgroundHover,
    },
  },
  emptyState: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
    fontStyle: 'italic',
  },
});

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
  const styles = useStyles();

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
    <div className={styles.container}>
      {/* Selected techniques display */}
      {!compact && (
        <div className={styles.techniqueList}>
          {selectedTechniques.length === 0 ? (
            <div className={styles.emptyState}>No techniques selected</div>
          ) : (
            selectedTechniques.map((techId) => (
              <div
                key={techId}
                className={styles.techniqueChip}
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
      <Menu>
        <MenuTrigger disableButtonEnhancement>
          <Button icon={<MusicNote224Regular />} appearance="secondary" size="small">
            {compact
              ? `${selectedTechniques.length} technique${selectedTechniques.length !== 1 ? 's' : ''}`
              : 'Add Technique'}
          </Button>
        </MenuTrigger>
        <MenuPopover>
          <MenuList>
            {Object.entries(AVAILABLE_TECHNIQUES).map(([category, techniques]) => (
              <MenuGroup key={category}>
                <MenuGroupHeader>{category}</MenuGroupHeader>
                {techniques.map((technique) => (
                  <MenuItem
                    key={technique.id}
                    onClick={() => handleToggleTechnique(technique.id)}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', width: '100%' }}>
                      <input
                        type="checkbox"
                        checked={selectedTechniques.includes(technique.id)}
                        readOnly
                        style={{ margin: 0 }}
                      />
                      <div style={{ flex: 1 }}>
                        <div style={{ fontWeight: 600 }}>
                          {technique.name}
                          {technique.symbol && (
                            <span style={{ marginLeft: '8px', color: tokens.colorNeutralForeground3 }}>
                              ({technique.symbol})
                            </span>
                          )}
                        </div>
                        {technique.description && (
                          <div style={{ fontSize: '11px', color: tokens.colorNeutralForeground3 }}>
                            {technique.description}
                          </div>
                        )}
                      </div>
                    </div>
                  </MenuItem>
                ))}
              </MenuGroup>
            ))}
          </MenuList>
        </MenuPopover>
      </Menu>
    </div>
  );
}
