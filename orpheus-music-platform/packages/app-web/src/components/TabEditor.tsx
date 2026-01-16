/**
 * Tab Editor Component
 * Complete tablature editing interface with fretboard, techniques, and chord library
 */

import { Button, Label, Slider } from '@persona-framework/ui';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@persona-framework/ui';
import { Tooltip, TooltipContent, TooltipTrigger } from '@persona-framework/ui';
import {
  ChevronLeft,
  ChevronRight,
  Trash2,
  Library,
  Upload,
  Play,
  Pause,
  Square,
} from 'lucide-react';
import { useState, useEffect, useCallback } from 'react';
import { Fretboard, type Note as FretboardNote } from './Fretboard';
import { TabStaff } from './TabStaff';
import { TechniquePicker } from './TechniquePicker';
import { ChordLibraryDialog } from './ChordLibraryDialog';
import { GPImportDialog } from './GPImportDialog';
import { ConfirmDialog } from './ConfirmDialog';
import { useTabPlayback } from '../hooks/useTabPlayback';
import type { MaestroProject } from '@orpheus/shared-types';
import { cn } from '../lib/utils';

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
  const [currentMeasure, setCurrentMeasure] = useState(0);
  const [currentTrack, setCurrentTrack] = useState(0);
  const [selectedNoteIndex, setSelectedNoteIndex] = useState<number | null>(null);
  const [chordLibraryOpen, setChordLibraryOpen] = useState(false);
  const [gpImportOpen, setGpImportOpen] = useState(false);
  const [clearNotesConfirmOpen, setClearNotesConfirmOpen] = useState(false);
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
          techniques: [], // Simplified
        });
      });
    });

    setNotes(allNotes);
    setSelectedNoteIndex(null);
  }, [currentMeasure, currentTrack, measure, track]);

  const handleFretClick = (string: number, fret: number) => {
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
    updateProjectNotes(updatedNotes);
  };

  const handleNoteClick = (noteIndex: number) => {
    setSelectedNoteIndex(noteIndex);
  };

  const handleNoteDelete = (noteIndex: number) => {
    const updatedNotes = notes.filter((_, i) => i !== noteIndex);
    setNotes(updatedNotes);
    setSelectedNoteIndex(null);
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
    const chordNotes: FretboardNote[] = chord.frets
      .map((fret: number, stringIndex: number) => {
        if (fret === 0 && chord.fingering[stringIndex] === 'X') return null;
        return {
          string: 6 - stringIndex,
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

    const beats = updatedNotes.map((note) => ({
      startTime: 0,
      duration: 1,
      notes: [{
        string: note.string,
        fret: note.fret,
        velocity: note.velocity || 0.8,
        techniques: {},
      }],
    }));

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
    <div className="flex flex-col h-full gap-4">
      {/* Header with measure navigation */}
      <div className="flex justify-between items-center p-4 bg-muted rounded-lg">
        <div className="text-lg font-semibold">Tab Editor</div>
        <div className="flex items-center gap-3">
          {/* Playback controls */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
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
              >
                {playback.isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
              </Button>
            </TooltipTrigger>
            <TooltipContent>Play/Pause</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                onClick={playback.stop}
                disabled={!playback.isPlaying && !playback.isPaused}
              >
                <Square className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Stop</TooltipContent>
          </Tooltip>

          <div className="w-px h-6 bg-border mx-2" />

          <Button
            variant="ghost"
            size="sm"
            onClick={() => setCurrentMeasure(Math.max(0, currentMeasure - 1))}
            disabled={currentMeasure === 0}
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <div className="text-sm font-semibold min-w-[100px] text-center">
            Measure {currentMeasure + 1} of {track?.measures?.length || 0}
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setCurrentMeasure(Math.min((track?.measures?.length || 0) - 1, currentMeasure + 1))}
            disabled={!track || currentMeasure >= (track.measures?.length || 0) - 1}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>

          <div className="w-px h-6 bg-border mx-2" />

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => setGpImportOpen(true)}
              >
                <Upload className="h-4 w-4 mr-2" />
                Import GP
              </Button>
            </TooltipTrigger>
            <TooltipContent>Import Guitar Pro file</TooltipContent>
          </Tooltip>
        </div>
      </div>

      {/* Main editor layout */}
      <div className="grid grid-cols-[2fr_1fr] gap-4 flex-1 min-h-0">
        {/* Left: Fretboard */}
        <div className="flex flex-col gap-3">
          <Fretboard
            numStrings={6}
            numFrets={12}
            tuning={['E', 'A', 'D', 'G', 'B', 'E']}
            notes={notes}
            selectedNoteIndex={selectedNoteIndex || undefined}
            onFretClick={handleFretClick}
            onNoteClick={handleNoteClick}
            onNoteDelete={handleNoteDelete}
          />

          <div className="flex gap-2 flex-wrap">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setChordLibraryOpen(true)}
            >
              <Library className="h-4 w-4 mr-2" />
              Insert Chord
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setClearNotesConfirmOpen(true)}
              disabled={notes.length === 0}
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Clear All Notes
            </Button>
          </div>

          <ConfirmDialog
            open={clearNotesConfirmOpen}
            onConfirm={() => {
              setNotes([]);
              updateProjectNotes([]);
              setClearNotesConfirmOpen(false);
            }}
            onCancel={() => setClearNotesConfirmOpen(false)}
            title="Clear All Notes?"
            message={`This will remove all ${notes.length} note${notes.length !== 1 ? 's' : ''} from measure ${currentMeasure + 1}. This action cannot be undone.`}
            confirmText="Clear Notes"
            cancelText="Keep Notes"
            type="warning"
          />

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
        <div className="flex flex-col gap-4 p-4 bg-muted rounded-lg h-fit">
          {/* Track selector */}
          <div className="flex flex-col gap-2">
            <Label className="text-sm font-semibold text-muted-foreground">Track</Label>
            <Select
              value={String(currentTrack)}
              onValueChange={(value) => setCurrentTrack(Number(value))}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select track" />
              </SelectTrigger>
              <SelectContent>
                {tracks.map((t: any, i: number) => (
                  <SelectItem key={t.id} value={String(i)}>
                    {t.name || `Track ${i + 1}`}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Selected note info */}
          {selectedNote ? (
            <>
              <div className="p-3 bg-primary text-primary-foreground rounded-lg text-sm">
                Note Selected: String {selectedNote.string}, Fret {selectedNote.fret}
              </div>

              {/* Duration */}
              <div className="flex flex-col gap-2">
                <Label className="text-sm font-semibold text-muted-foreground">Duration</Label>
                <Select
                  value={selectedNote.duration || 'quarter'}
                  onValueChange={(value) => handleNotePropertyChange('duration', value)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Duration" />
                  </SelectTrigger>
                  <SelectContent>
                    {DURATIONS.map((d) => (
                      <SelectItem key={d.value} value={d.value}>
                        {d.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Velocity */}
              <div className="flex flex-col gap-2">
                <Label className="text-sm font-semibold text-muted-foreground">
                  Velocity: {Math.round((selectedNote.velocity || 0.8) * 100)}%
                </Label>
                <Slider
                  min={0}
                  max={1}
                  step={0.05}
                  value={[selectedNote.velocity || 0.8]}
                  onValueChange={(value) => handleNotePropertyChange('velocity', value[0])}
                />
              </div>

              {/* Techniques */}
              <div className="flex flex-col gap-2">
                <Label className="text-sm font-semibold text-muted-foreground">Techniques</Label>
                <TechniquePicker
                  selectedTechniques={selectedNote.techniques || []}
                  onTechniquesChange={(techniques) =>
                    handleNotePropertyChange('techniques', techniques)
                  }
                />
              </div>

              <Button
                variant="secondary"
                onClick={() => selectedNoteIndex !== null && handleNoteDelete(selectedNoteIndex)}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Delete Note
              </Button>
            </>
          ) : (
            <div className="text-center py-5 text-muted-foreground">
              Click on the fretboard to add notes
            </div>
          )}

          {/* Notes list */}
          {notes.length > 0 && (
            <div className="flex flex-col gap-2">
              <Label className="text-sm font-semibold text-muted-foreground">
                Notes in Measure ({notes.length})
              </Label>
              <div className="p-3 bg-background rounded-lg max-h-[300px] overflow-y-auto">
                {notes.map((note, i) => (
                  <div
                    key={i}
                    className={cn(
                      'p-2 border-b border-border cursor-pointer flex justify-between items-center',
                      'hover:bg-muted',
                      selectedNoteIndex === i && 'bg-primary text-primary-foreground'
                    )}
                    onClick={() => setSelectedNoteIndex(i)}
                  >
                    <div>
                      String {note.string}, Fret {note.fret}
                    </div>
                    <div className="text-[11px] opacity-80">
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
