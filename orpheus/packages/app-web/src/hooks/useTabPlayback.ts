/**
 * Tab Playback Hook
 * Uses Tone.js for MIDI synthesis and playback of tablature
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import * as Tone from 'tone';
import type { Note as FretboardNote } from '../components/Fretboard';

// Standard guitar tuning in MIDI notes (low E to high E)
const STANDARD_TUNING = [40, 45, 50, 55, 59, 64]; // E2, A2, D3, G3, B3, E4

// Note duration mapping
const DURATION_MAP: Record<string, string> = {
  whole: '1n',
  half: '2n',
  quarter: '4n',
  eighth: '8n',
  sixteenth: '16n',
  'thirty-second': '32n',
};

interface PlaybackState {
  isPlaying: boolean;
  isPaused: boolean;
  currentBeat: number;
  currentTime: number;
  duration: number;
  tempo: number;
}

interface UseTabPlaybackOptions {
  tempo?: number;
  tuning?: number[];
  onBeatChange?: (beat: number) => void;
  onPlaybackEnd?: () => void;
}

export function useTabPlayback(options: UseTabPlaybackOptions = {}) {
  const {
    tempo = 120,
    tuning = STANDARD_TUNING,
    onBeatChange,
    onPlaybackEnd,
  } = options;

  const [state, setState] = useState<PlaybackState>({
    isPlaying: false,
    isPaused: false,
    currentBeat: 0,
    currentTime: 0,
    duration: 0,
    tempo,
  });

  const synthRef = useRef<Tone.PolySynth | null>(null);
  const partRef = useRef<Tone.Part | null>(null);
  const sequenceRef = useRef<Array<{ time: number; notes: FretboardNote[] }>>([]);
  const animationFrameRef = useRef<number | null>(null);

  // Initialize synth
  useEffect(() => {
    // Create a polyphonic synth with a pluck-like sound
    const synth = new Tone.PolySynth(Tone.Synth, {
      oscillator: {
        type: 'fmtriangle',
        modulationType: 'square',
        modulationIndex: 2,
        harmonicity: 1,
      },
      envelope: {
        attack: 0.01,
        decay: 0.2,
        sustain: 0.2,
        release: 0.8,
      },
    }).toDestination();

    // Add some effects for a more realistic guitar sound
    const reverb = new Tone.Reverb({
      decay: 1.5,
      wet: 0.2,
    }).toDestination();

    const chorus = new Tone.Chorus({
      frequency: 1.5,
      delayTime: 3.5,
      depth: 0.3,
      wet: 0.15,
    }).toDestination();

    synth.connect(reverb);
    synth.connect(chorus);

    synthRef.current = synth;

    return () => {
      synth.dispose();
      reverb.dispose();
      chorus.dispose();
    };
  }, []);

  // Update tempo
  useEffect(() => {
    Tone.Transport.bpm.value = tempo;
    setState(prev => ({ ...prev, tempo }));
  }, [tempo]);

  // Convert fret position to frequency
  const fretToFrequency = useCallback((string: number, fret: number): number => {
    const stringIndex = string - 1; // Convert 1-based to 0-based
    if (stringIndex < 0 || stringIndex >= tuning.length) return 0;

    const midiNote = tuning[tuning.length - 1 - stringIndex] + fret; // Reverse for guitar convention
    return Tone.Frequency(midiNote, 'midi').toFrequency();
  }, [tuning]);

  // Load notes into playback sequence
  const loadNotes = useCallback((notes: FretboardNote[], beatsPerMeasure = 4) => {
    // Group notes by their position (simplified - assumes sequential)
    const sequence: Array<{ time: number; notes: FretboardNote[] }> = [];

    let currentTime = 0;
    notes.forEach((note, index) => {
      // For simplicity, space notes evenly (would be more complex with real timing)
      const duration = DURATION_MAP[note.duration || 'quarter'] || '4n';
      const durationSeconds = Tone.Time(duration).toSeconds();

      // Check if we should add to existing beat or create new one
      const existingBeat = sequence.find(s => Math.abs(s.time - currentTime) < 0.01);
      if (existingBeat) {
        existingBeat.notes.push(note);
      } else {
        sequence.push({ time: currentTime, notes: [note] });
      }

      currentTime += durationSeconds;
    });

    sequenceRef.current = sequence;

    // Calculate total duration
    const totalDuration = currentTime;
    setState(prev => ({
      ...prev,
      duration: totalDuration,
      currentBeat: 0,
      currentTime: 0,
    }));
  }, []);

  // Play single note
  const playNote = useCallback((note: FretboardNote) => {
    if (!synthRef.current) return;

    const frequency = fretToFrequency(note.string, note.fret);
    if (frequency === 0) return;

    const duration = DURATION_MAP[note.duration || 'quarter'] || '4n';
    const velocity = note.velocity || 0.8;

    synthRef.current.triggerAttackRelease(frequency, duration, undefined, velocity);
  }, [fretToFrequency]);

  // Play chord (multiple notes)
  const playChord = useCallback((notes: FretboardNote[]) => {
    if (!synthRef.current) return;

    const frequencies = notes
      .map(note => fretToFrequency(note.string, note.fret))
      .filter(f => f > 0);

    if (frequencies.length === 0) return;

    const duration = DURATION_MAP[notes[0]?.duration || 'quarter'] || '4n';
    const velocity = notes[0]?.velocity || 0.8;

    synthRef.current.triggerAttackRelease(frequencies, duration, undefined, velocity);
  }, [fretToFrequency]);

  // Start playback
  const play = useCallback(async () => {
    if (sequenceRef.current.length === 0) return;

    // Ensure audio context is started
    await Tone.start();

    // Stop any existing playback
    if (partRef.current) {
      partRef.current.dispose();
    }

    // Create new part for playback
    const part = new Tone.Part((time, event: { notes: FretboardNote[]; index: number }) => {
      if (!synthRef.current) return;

      const frequencies = event.notes
        .map(note => fretToFrequency(note.string, note.fret))
        .filter(f => f > 0);

      if (frequencies.length > 0) {
        const duration = DURATION_MAP[event.notes[0]?.duration || 'quarter'] || '4n';
        const velocity = event.notes[0]?.velocity || 0.8;
        synthRef.current.triggerAttackRelease(frequencies, duration, time, velocity);
      }

      // Update current beat on main thread
      Tone.Draw.schedule(() => {
        setState(prev => ({ ...prev, currentBeat: event.index }));
        onBeatChange?.(event.index);
      }, time);
    }, sequenceRef.current.map((s, i) => ({ time: s.time, notes: s.notes, index: i })));

    part.start(0);
    partRef.current = part;

    // Start transport
    Tone.Transport.start();

    // Update playing state
    setState(prev => ({ ...prev, isPlaying: true, isPaused: false }));

    // Track current time with animation frame
    const updateTime = () => {
      const currentTime = Tone.Transport.seconds;
      setState(prev => ({ ...prev, currentTime }));

      if (Tone.Transport.state === 'started') {
        animationFrameRef.current = requestAnimationFrame(updateTime);
      }
    };
    animationFrameRef.current = requestAnimationFrame(updateTime);

    // Schedule end callback
    const endTime = sequenceRef.current[sequenceRef.current.length - 1]?.time || 0;
    Tone.Transport.scheduleOnce(() => {
      stop();
      onPlaybackEnd?.();
    }, endTime + 1); // Add small buffer
  }, [fretToFrequency, onBeatChange, onPlaybackEnd]);

  // Pause playback
  const pause = useCallback(() => {
    Tone.Transport.pause();
    setState(prev => ({ ...prev, isPlaying: false, isPaused: true }));

    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
    }
  }, []);

  // Resume playback
  const resume = useCallback(() => {
    Tone.Transport.start();
    setState(prev => ({ ...prev, isPlaying: true, isPaused: false }));

    const updateTime = () => {
      const currentTime = Tone.Transport.seconds;
      setState(prev => ({ ...prev, currentTime }));

      if (Tone.Transport.state === 'started') {
        animationFrameRef.current = requestAnimationFrame(updateTime);
      }
    };
    animationFrameRef.current = requestAnimationFrame(updateTime);
  }, []);

  // Stop playback
  const stop = useCallback(() => {
    Tone.Transport.stop();
    Tone.Transport.seconds = 0;

    if (partRef.current) {
      partRef.current.dispose();
      partRef.current = null;
    }

    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
    }

    setState(prev => ({
      ...prev,
      isPlaying: false,
      isPaused: false,
      currentBeat: 0,
      currentTime: 0,
    }));
  }, []);

  // Seek to position
  const seek = useCallback((time: number) => {
    Tone.Transport.seconds = time;
    setState(prev => ({ ...prev, currentTime: time }));
  }, []);

  // Set volume (0-1)
  const setVolume = useCallback((volume: number) => {
    if (synthRef.current) {
      synthRef.current.volume.value = Tone.gainToDb(volume);
    }
  }, []);

  // Cleanup
  useEffect(() => {
    return () => {
      if (partRef.current) {
        partRef.current.dispose();
      }
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      Tone.Transport.stop();
      Tone.Transport.cancel();
    };
  }, []);

  return {
    ...state,
    loadNotes,
    playNote,
    playChord,
    play,
    pause,
    resume,
    stop,
    seek,
    setVolume,
  };
}
