/**
 * Tab Editor Component
 * Complete tablature editing interface with fretboard, techniques, and chord library
 */

import {
  makeStyles,
  shorthands,
  tokens,
  Button,
  Select,
  Label,
  Slider,
  Tooltip,
} from '@fluentui/react-components';
import {
  ChevronLeftRegular,
  ChevronRightRegular,
  AddRegular,
  DeleteRegular,
  LibraryRegular,
  ArrowUploadRegular,
  PlayRegular,
  PauseRegular,
  StopRegular,
} from '@fluentui/react-icons';
import { useState, useEffect, useCallback } from 'react';
import { Fretboard, type Note as FretboardNote } from './Fretboard';
import { TabStaff } from './TabStaff';
import { TechniquePicker } from './TechniquePicker';
import { ChordLibraryDialog } from './ChordLibraryDialog';
import { GPImportDialog } from './GPImportDialog';
import { useTabPlayback } from '../hooks/useTabPlayback';
import type { MaestroProject } from '@orpheus/shared-types';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    ...shorthands.gap('16px'),
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('8px'),
  },
  title: {
    fontSize: '18px',
    fontWeight: tokens.fontWeightSemibold,
  },
  measureNav: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('12px'),
  },
  measureInfo: {
    fontSize: '14px',
    fontWeight: 600,
    minWidth: '100px',
    textAlign: 'center',
  },
  editorLayout: {
    display: 'grid',
    gridTemplateColumns: '2fr 1fr',
    ...shorthands.gap('16px'),
    flex: 1,
    minHeight: 0,
  },
  fretboardSection: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
  },
  propertiesPanel: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('8px'),
    height: 'fit-content',
  },
  propertySection: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  propertyLabel: {
    fontSize: '14px',
    fontWeight: 600,
    color: tokens.colorNeutralForeground2,
  },
  selectedNoteInfo: {
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorBrandBackground,
    color: '#ffffff',
    ...shorthands.borderRadius('8px'),
    fontSize: '14px',
  },
  noteList: {
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('8px'),
    maxHeight: '300px',
    overflowY: 'auto',
  },
  noteItem: {
    ...shorthands.padding('8px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke2),
    cursor: 'pointer',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
    },
  },
  noteItemSelected: {
    backgroundColor: tokens.colorBrandBackground,
    color: '#ffffff',
  },
  actionButtons: {
    display: 'flex',
    ...shorthands.gap('8px'),
    flexWrap: 'wrap',
  },
});

const DURATIONS = [
  { value: 'whole', label: 'Whole Note' },
  { value: 'half', label: 'Half Note' },
  { value: 'quarter', label: 'Quarter Note' },
  { value: 'eighth', label: 'Eighth Note' },
  { value: 'sixteenth', label: '16th Note' },
  { value: 'thirty-second', label: '32nd Note' },
];

export interface TabEditorProps {
  project: MaestroProject;
  onProjectChange: (project: MaestroProject) => void;
}

export function TabEditor({ project, onProjectChange }: TabEditorProps) {
  const styles = useStyles();
  const [currentMeasure, setCurrentMeasure] = useState(0);
  const [currentTrack, setCurrentTrack] = useState(0);
  const [selectedNoteIndex, setSelectedNoteIndex] = useState<number | null>(null);
  const [chordLibraryOpen, setChordLibraryOpen] = useState(false);
  const [gpImportOpen, setGpImportOpen] = useState(false);
  const [notes, setNotes] = useState<FretboardNote[]>([]);

  // Initialize playback hook
  const playback = useTabPlayback({
    tempo: project.project.metadata.tempo || 120,
    onBeatChange: (beat) => setSelectedNoteIndex(beat),
    onPlaybackEnd: () => setSelectedNoteIndex(null),
  });

  // Load notes into playback when they change
  useEffect(() => {
    if (notes.length > 0) {
      playback.loadNotes(notes);
    }
  }, [notes]);

  // Handle GP file import
  const handleGPImport = useCallback((importedProject: MaestroProject) => {
    onProjectChange(importedProject);
    setCurrentMeasure(0);
    setCurrentTrack(0);
  }, [onProjectChange]);

  // Extract current measure/track data
  const tracks = project.project.composition.tracks || [];
  const track = tracks[currentTrack];
  const measure = track?.measures?.[currentMeasure];

  // Load notes from current measure/track
  useEffect(() => {
    if (!measure || !track) {
      setNotes([]);
      return;
    }

    // Get notes from first voice of first beat (simplified for now)
    const voice = measure.voices?.[0];
    if (!voice) {
      setNotes([]);
      return;
    }

    // Collect all notes from all beats in the voice
    const allNotes: FretboardNote[] = [];
    voice.beats?.forEach((beat: any) => {
      beat.notes?.forEach((note: any) => {
        allNotes.push({
          string: note.string,
          fret: note.fret,
          duration: 'quarter', // Simplified
          velocity: note.velocity || 0.8,
          techniques: [], // Simplified - would need to convert NoteTechniques to string[]
        });
      });
    });

    setNotes(allNotes);
    setSelectedNoteIndex(null);
  }, [currentMeasure, currentTrack, measure, track]);

  const handleFretClick = (string: number, fret: number) => {
    // Add new note
    const newNote: FretboardNote = {
      string,
      fret,
      duration: 'quarter',
      velocity: 0.8,
      techniques: [],
    };

    const updatedNotes = [...notes, newNote];
    setNotes(updatedNotes);
    setSelectedNoteIndex(updatedNotes.length - 1);

    // Update project
    updateProjectNotes(updatedNotes);
  };

  const handleNoteClick = (noteIndex: number) => {
    setSelectedNoteIndex(noteIndex);
  };

  const handleNoteDelete = (noteIndex: number) => {
    const updatedNotes = notes.filter((_, i) => i !== noteIndex);
    setNotes(updatedNotes);
    setSelectedNoteIndex(null);

    // Update project
    updateProjectNotes(updatedNotes);
  };

  const handleNotePropertyChange = (property: string, value: any) => {
    if (selectedNoteIndex === null) return;

    const updatedNotes = notes.map((note, i) =>
      i === selectedNoteIndex ? { ...note, [property]: value } : note
    );

    setNotes(updatedNotes);
    updateProjectNotes(updatedNotes);
  };

  const handleChordInsert = (chord: any) => {
    // Insert chord notes
    const chordNotes: FretboardNote[] = chord.frets
      .map((fret: number, stringIndex: number) => {
        if (fret === 0 && chord.fingering[stringIndex] === 'X') return null; // Muted string
        return {
          string: 6 - stringIndex, // Reverse to match guitar strings (1=high E, 6=low E)
          fret,
          duration: 'quarter',
          velocity: 0.8,
          techniques: [],
        };
      })
      .filter((n: any) => n !== null);

    const updatedNotes = [...notes, ...chordNotes];
    setNotes(updatedNotes);
    updateProjectNotes(updatedNotes);
  };

  const updateProjectNotes = (updatedNotes: FretboardNote[]) => {
    if (!track) return;

    // Convert fretboard notes to project format (simplified - single beat per note)
    const beats = updatedNotes.map((note) => ({
      startTime: 0,
      duration: 1,
      notes: [{
        string: note.string,
        fret: note.fret,
        velocity: note.velocity || 0.8,
        techniques: {}, // Simplified - would need proper technique conversion
      }],
    }));

    // Update project
    const updatedProject = { ...project };
    const updatedTracks = [...(updatedProject.project.composition.tracks || [])];
    const trackIndex = updatedTracks.findIndex((t) => t.id === track.id);

    if (trackIndex < 0) return;

    const updatedTrack = { ...updatedTracks[trackIndex] };
    const updatedMeasures = [...(updatedTrack.measures || [])];

    if (!updatedMeasures[currentMeasure]) return;

    const updatedMeasure = { ...updatedMeasures[currentMeasure] };
    updatedMeasure.voices = [{
      voiceIndex: 0,
      beats,
    }];

    updatedMeasures[currentMeasure] = updatedMeasure;
    updatedTrack.measures = updatedMeasures;
    updatedTracks[trackIndex] = updatedTrack;
    updatedProject.project.composition.tracks = updatedTracks;

    onProjectChange(updatedProject);
  };

  const selectedNote = selectedNoteIndex !== null ? notes[selectedNoteIndex] : null;

  return (
    <div className={styles.container}>
      {/* Header with measure navigation */}
      <div className={styles.header}>
        <div className={styles.title}>🎼 Tab Editor</div>
        <div className={styles.measureNav}>
          {/* Playback controls */}
          <Tooltip content="Play/Pause" relationship="label">
            <Button
              icon={playback.isPlaying ? <PauseRegular /> : <PlayRegular />}
              appearance="primary"
              onClick={() => {
                if (playback.isPlaying) {
                  playback.pause();
                } else if (playback.isPaused) {
                  playback.resume();
                } else {
                  playback.play();
                }
              }}
              disabled={notes.length === 0}
            />
          </Tooltip>
          <Tooltip content="Stop" relationship="label">
            <Button
              icon={<StopRegular />}
              appearance="subtle"
              onClick={playback.stop}
              disabled={!playback.isPlaying && !playback.isPaused}
            />
          </Tooltip>

          <div style={{ width: '1px', height: '24px', backgroundColor: tokens.colorNeutralStroke2, margin: '0 8px' }} />

          <Button
            icon={<ChevronLeftRegular />}
            appearance="subtle"
            onClick={() => setCurrentMeasure(Math.max(0, currentMeasure - 1))}
            disabled={currentMeasure === 0}
          />
          <div className={styles.measureInfo}>
            Measure {currentMeasure + 1} of {track?.measures?.length || 0}
          </div>
          <Button
            icon={<ChevronRightRegular />}
            appearance="subtle"
            onClick={() => setCurrentMeasure(Math.min((track?.measures?.length || 0) - 1, currentMeasure + 1))}
            disabled={!track || currentMeasure >= (track.measures?.length || 0) - 1}
          />

          <div style={{ width: '1px', height: '24px', backgroundColor: tokens.colorNeutralStroke2, margin: '0 8px' }} />

          <Tooltip content="Import Guitar Pro file" relationship="label">
            <Button
              icon={<ArrowUploadRegular />}
              appearance="secondary"
              onClick={() => setGpImportOpen(true)}
            >
              Import GP
            </Button>
          </Tooltip>
          <Button
            icon={<AddRegular />}
            appearance="primary"
            size="small"
            disabled
            title="Add measure (coming soon)"
          >
            Add Measure
          </Button>
        </div>
      </div>

      {/* Main editor layout */}
      <div className={styles.editorLayout}>
        {/* Left: Fretboard */}
        <div className={styles.fretboardSection}>
          <Fretboard
            numStrings={6}
            numFrets={12}
            tuning={['E', 'A', 'D', 'G', 'B', 'E']} // Default standard tuning for now
            notes={notes}
            selectedNoteIndex={selectedNoteIndex || undefined}
            onFretClick={handleFretClick}
            onNoteClick={handleNoteClick}
            onNoteDelete={handleNoteDelete}
          />

          <div className={styles.actionButtons}>
            <Button
              icon={<LibraryRegular />}
              appearance="secondary"
              onClick={() => setChordLibraryOpen(true)}
            >
              Insert Chord
            </Button>
            <Button
              icon={<DeleteRegular />}
              appearance="secondary"
              onClick={() => {
                setNotes([]);
                updateProjectNotes([]);
              }}
              disabled={notes.length === 0}
            >
              Clear All Notes
            </Button>
          </div>

          {/* Tab Staff Notation View */}
          <TabStaff
            notes={notes}
            numStrings={6}
            tuning={['E', 'A', 'D', 'G', 'B', 'E']}
            selectedNoteIndex={selectedNoteIndex ?? undefined}
            measureNumber={currentMeasure + 1}
            beatsPerMeasure={4}
            onNoteClick={handleNoteClick}
          />
        </div>

        {/* Right: Properties panel */}
        <div className={styles.propertiesPanel}>
          {/* Track selector */}
          <div className={styles.propertySection}>
            <Label className={styles.propertyLabel}>Track</Label>
            <Select
              value={String(currentTrack)}
              onChange={(_, data) => setCurrentTrack(Number(data.value))}
            >
              {tracks.map((t: any, i: number) => (
                <option key={t.id} value={String(i)}>
                  {t.name || `Track ${i + 1}`}
                </option>
              ))}
            </Select>
          </div>

          {/* Selected note info */}
          {selectedNote ? (
            <>
              <div className={styles.selectedNoteInfo}>
                📍 Note Selected: String {selectedNote.string}, Fret {selectedNote.fret}
              </div>

              {/* Duration */}
              <div className={styles.propertySection}>
                <Label className={styles.propertyLabel}>Duration</Label>
                <Select
                  value={selectedNote.duration || 'quarter'}
                  onChange={(_, data) => handleNotePropertyChange('duration', data.value)}
                >
                  {DURATIONS.map((d) => (
                    <option key={d.value} value={d.value}>
                      {d.label}
                    </option>
                  ))}
                </Select>
              </div>

              {/* Velocity */}
              <div className={styles.propertySection}>
                <Label className={styles.propertyLabel}>
                  Velocity: {Math.round((selectedNote.velocity || 0.8) * 100)}%
                </Label>
                <Slider
                  min={0}
                  max={1}
                  step={0.05}
                  value={selectedNote.velocity || 0.8}
                  onChange={(_, data) => handleNotePropertyChange('velocity', data.value)}
                />
              </div>

              {/* Techniques */}
              <div className={styles.propertySection}>
                <Label className={styles.propertyLabel}>Techniques</Label>
                <TechniquePicker
                  selectedTechniques={selectedNote.techniques || []}
                  onTechniquesChange={(techniques) =>
                    handleNotePropertyChange('techniques', techniques)
                  }
                />
              </div>

              <Button
                icon={<DeleteRegular />}
                appearance="secondary"
                onClick={() => selectedNoteIndex !== null && handleNoteDelete(selectedNoteIndex)}
              >
                Delete Note
              </Button>
            </>
          ) : (
            <div style={{ textAlign: 'center', padding: '20px', color: tokens.colorNeutralForeground3 }}>
              Click on the fretboard to add notes
            </div>
          )}

          {/* Notes list */}
          {notes.length > 0 && (
            <div className={styles.propertySection}>
              <Label className={styles.propertyLabel}>
                Notes in Measure ({notes.length})
              </Label>
              <div className={styles.noteList}>
                {notes.map((note, i) => (
                  <div
                    key={i}
                    className={`${styles.noteItem} ${
                      selectedNoteIndex === i ? styles.noteItemSelected : ''
                    }`}
                    onClick={() => setSelectedNoteIndex(i)}
                  >
                    <div>
                      String {note.string}, Fret {note.fret}
                    </div>
                    <div style={{ fontSize: '11px', opacity: 0.8 }}>
                      {note.duration || 'quarter'}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Chord Library Dialog */}
      <ChordLibraryDialog
        open={chordLibraryOpen}
        onOpenChange={setChordLibraryOpen}
        onChordSelect={handleChordInsert}
      />

      {/* GP Import Dialog */}
      <GPImportDialog
        open={gpImportOpen}
        onOpenChange={setGpImportOpen}
        onImport={handleGPImport}
      />
    </div>
  );
}
