/**
 * Tuner Component - Professional chromatic tuner for instrument tuning
 */

import { useState, useEffect, useCallback } from 'react';
import {
  makeStyles,
  shorthands,
  tokens,
  Button,
  Dropdown,
  Option,
  Label,
  Slider,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogContent,
  DialogActions,
  Tooltip,
} from '@fluentui/react-components';
import {
  MusicNote224Regular,
  Settings24Regular,
  Dismiss24Regular,
  MicOn24Regular,
  MicOff24Regular,
} from '@fluentui/react-icons';
import {
  getTunerService,
  TunerState,
  TunerNote,
  TUNING_PRESETS,
  TuningPreset,
  noteToFrequency,
} from '../services/tuner-service';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    ...shorthands.padding('24px'),
    ...shorthands.gap('16px'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('12px'),
    minWidth: '320px',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    width: '100%',
  },
  title: {
    fontSize: '18px',
    fontWeight: tokens.fontWeightSemibold,
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  noteDisplay: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    minHeight: '120px',
    justifyContent: 'center',
  },
  noteName: {
    fontSize: '72px',
    fontWeight: tokens.fontWeightBold,
    lineHeight: '1',
    fontFamily: 'monospace',
  },
  noteOctave: {
    fontSize: '24px',
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground2,
  },
  noteFrequency: {
    fontSize: '14px',
    color: tokens.colorNeutralForeground3,
  },
  noSignal: {
    fontSize: '24px',
    color: tokens.colorNeutralForeground4,
    textAlign: 'center',
  },
  centsDisplay: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    ...shorthands.gap('4px'),
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
  },
  centsValue: {
    fontFamily: 'monospace',
    minWidth: '60px',
    textAlign: 'center',
  },
  meterContainer: {
    width: '100%',
    maxWidth: '280px',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  meterBackground: {
    width: '100%',
    height: '16px',
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('8px'),
    position: 'relative',
    overflow: 'hidden',
  },
  meterCenter: {
    position: 'absolute',
    left: '50%',
    top: '0',
    bottom: '0',
    width: '2px',
    backgroundColor: tokens.colorNeutralForeground1,
    transform: 'translateX(-50%)',
    zIndex: 2,
  },
  meterIndicator: {
    position: 'absolute',
    top: '2px',
    bottom: '2px',
    width: '8px',
    ...shorthands.borderRadius('4px'),
    ...shorthands.transition('left', '50ms', 'ease-out'),
    zIndex: 1,
  },
  meterLabels: {
    display: 'flex',
    justifyContent: 'space-between',
    width: '100%',
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
  },
  inputLevelContainer: {
    width: '100%',
    maxWidth: '280px',
  },
  inputLevelLabel: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
    marginBottom: '4px',
    display: 'block',
  },
  inputLevelBar: {
    width: '100%',
    height: '8px',
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('4px'),
    overflow: 'hidden',
  },
  inputLevelFill: {
    height: '100%',
    ...shorthands.transition('width', '50ms', 'ease-out'),
    backgroundColor: tokens.colorPaletteGreenBackground3,
  },
  tuningPreset: {
    width: '100%',
    maxWidth: '280px',
  },
  stringButtons: {
    display: 'flex',
    justifyContent: 'center',
    ...shorthands.gap('8px'),
    flexWrap: 'wrap',
  },
  stringButton: {
    minWidth: '48px',
    fontFamily: 'monospace',
  },
  activeString: {
    backgroundColor: tokens.colorBrandBackground,
    color: tokens.colorNeutralForegroundOnBrand,
  },
  controls: {
    display: 'flex',
    ...shorthands.gap('8px'),
    marginTop: '8px',
  },
  settingsRow: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
  },
  settingsLabel: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
  },
  settingsValue: {
    fontSize: '14px',
    fontFamily: 'monospace',
  },
  inTune: {
    color: tokens.colorPaletteGreenForeground1,
  },
  flat: {
    color: tokens.colorPaletteRedForeground1,
  },
  sharp: {
    color: tokens.colorPaletteYellowForeground1,
  },
});

interface TunerProps {
  onClose?: () => void;
  compact?: boolean;
}

export function Tuner({ onClose, compact = false }: TunerProps) {
  const styles = useStyles();
  const tunerService = getTunerService();

  const [tunerState, setTunerState] = useState<TunerState>({
    isActive: false,
    currentNote: null,
    inputLevel: 0,
    referencePitch: 440,
  });
  const [selectedPreset, setSelectedPreset] = useState<TuningPreset>(TUNING_PRESETS[0]);
  const [targetNote, setTargetNote] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [referencePitch, setReferencePitch] = useState(440);

  // Subscribe to tuner state updates
  useEffect(() => {
    const unsubscribe = tunerService.subscribe(setTunerState);
    return () => {
      unsubscribe();
    };
  }, [tunerService]);

  // Toggle tuner
  const toggleTuner = useCallback(async () => {
    if (tunerState.isActive) {
      tunerService.stop();
    } else {
      try {
        await tunerService.start();
      } catch (error) {
        console.error('Failed to start tuner:', error);
      }
    }
  }, [tunerState.isActive, tunerService]);

  // Update reference pitch
  const handleReferencePitchChange = useCallback(
    (value: number) => {
      setReferencePitch(value);
      tunerService.setReferencePitch(value);
    },
    [tunerService]
  );

  // Get cents indicator color and position
  const getCentsInfo = (note: TunerNote | null) => {
    if (!note) {
      return { color: 'transparent', position: 50, status: '' };
    }

    const position = 50 + (note.cents / 50) * 50; // Map -50 to 50 cents to 0% to 100%

    if (note.inTune) {
      return { color: tokens.colorPaletteGreenBackground3, position, status: 'In Tune' };
    } else if (note.cents < 0) {
      return { color: tokens.colorPaletteRedBackground3, position, status: 'Flat' };
    } else {
      return { color: tokens.colorPaletteYellowBackground3, position, status: 'Sharp' };
    }
  };

  // Get the closest target string for the current note
  const getClosestString = (note: TunerNote | null): string | null => {
    if (!note) return null;

    const noteWithOctave = `${note.name}${note.octave}`;
    const targetFreq = noteToFrequency(noteWithOctave, referencePitch);

    // Find closest string in preset
    let closestString: string | null = null;
    let closestDiff = Infinity;

    for (const stringNote of selectedPreset.notes) {
      const stringFreq = noteToFrequency(stringNote, referencePitch);
      const diff = Math.abs(Math.log2(targetFreq / stringFreq) * 12);

      if (diff < closestDiff && diff < 2) {
        // Within 2 semitones
        closestDiff = diff;
        closestString = stringNote;
      }
    }

    return closestString;
  };

  const centsInfo = getCentsInfo(tunerState.currentNote);
  const closestString = getClosestString(tunerState.currentNote);

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.title}>
          <MusicNote224Regular />
          Tuner
        </div>
        <div style={{ display: 'flex', gap: '8px' }}>
          <Dialog open={settingsOpen} onOpenChange={(_, data) => setSettingsOpen(data.open)}>
            <DialogTrigger disableButtonEnhancement>
              <Tooltip content="Tuner Settings" relationship="label">
                <Button icon={<Settings24Regular />} appearance="subtle" size="small" />
              </Tooltip>
            </DialogTrigger>
            <DialogSurface>
              <DialogBody>
                <DialogTitle>Tuner Settings</DialogTitle>
                <DialogContent>
                  <div className={styles.settingsRow}>
                    <span className={styles.settingsLabel}>Reference Pitch (A4)</span>
                    <span className={styles.settingsValue}>{referencePitch} Hz</span>
                  </div>
                  <Slider
                    min={400}
                    max={480}
                    step={1}
                    value={referencePitch}
                    onChange={(_, data) => handleReferencePitchChange(data.value)}
                  />
                  <div style={{ marginTop: '16px' }}>
                    <Button
                      appearance="subtle"
                      size="small"
                      onClick={() => handleReferencePitchChange(440)}
                    >
                      Reset to 440 Hz
                    </Button>
                    <Button
                      appearance="subtle"
                      size="small"
                      onClick={() => handleReferencePitchChange(432)}
                      style={{ marginLeft: '8px' }}
                    >
                      432 Hz
                    </Button>
                    <Button
                      appearance="subtle"
                      size="small"
                      onClick={() => handleReferencePitchChange(442)}
                      style={{ marginLeft: '8px' }}
                    >
                      442 Hz (Orchestra)
                    </Button>
                  </div>
                </DialogContent>
                <DialogActions>
                  <Button appearance="primary" onClick={() => setSettingsOpen(false)}>
                    Done
                  </Button>
                </DialogActions>
              </DialogBody>
            </DialogSurface>
          </Dialog>
          {onClose && (
            <Tooltip content="Close Tuner" relationship="label">
              <Button icon={<Dismiss24Regular />} appearance="subtle" size="small" onClick={onClose} />
            </Tooltip>
          )}
        </div>
      </div>

      {/* Note Display */}
      <div className={styles.noteDisplay}>
        {tunerState.isActive && tunerState.currentNote ? (
          <>
            <div
              className={`${styles.noteName} ${
                tunerState.currentNote.inTune
                  ? styles.inTune
                  : tunerState.currentNote.cents < 0
                  ? styles.flat
                  : styles.sharp
              }`}
            >
              {tunerState.currentNote.name}
              <span className={styles.noteOctave}>{tunerState.currentNote.octave}</span>
            </div>
            <div className={styles.noteFrequency}>
              {tunerState.currentNote.frequency.toFixed(1)} Hz
            </div>
          </>
        ) : (
          <div className={styles.noSignal}>
            {tunerState.isActive ? 'Play a note...' : 'Tuner inactive'}
          </div>
        )}
      </div>

      {/* Cents Meter */}
      <div className={styles.meterContainer}>
        <div className={styles.centsDisplay}>
          <span>{centsInfo.status || '—'}</span>
          {tunerState.currentNote && (
            <span
              className={`${styles.centsValue} ${
                tunerState.currentNote.inTune
                  ? styles.inTune
                  : tunerState.currentNote.cents < 0
                  ? styles.flat
                  : styles.sharp
              }`}
            >
              {tunerState.currentNote.cents > 0 ? '+' : ''}
              {tunerState.currentNote.cents} cents
            </span>
          )}
        </div>
        <div className={styles.meterBackground}>
          <div className={styles.meterCenter} />
          {tunerState.currentNote && (
            <div
              className={styles.meterIndicator}
              style={{
                left: `calc(${centsInfo.position}% - 4px)`,
                backgroundColor: centsInfo.color,
              }}
            />
          )}
        </div>
        <div className={styles.meterLabels}>
          <span>-50</span>
          <span>-25</span>
          <span>0</span>
          <span>+25</span>
          <span>+50</span>
        </div>
      </div>

      {/* Input Level */}
      <div className={styles.inputLevelContainer}>
        <span className={styles.inputLevelLabel}>Input Level</span>
        <div className={styles.inputLevelBar}>
          <div
            className={styles.inputLevelFill}
            style={{
              width: `${tunerState.inputLevel * 100}%`,
              backgroundColor:
                tunerState.inputLevel > 0.9
                  ? tokens.colorPaletteRedBackground3
                  : tunerState.inputLevel > 0.7
                  ? tokens.colorPaletteYellowBackground3
                  : tokens.colorPaletteGreenBackground3,
            }}
          />
        </div>
      </div>

      {/* Tuning Preset */}
      {!compact && (
        <div className={styles.tuningPreset}>
          <Label htmlFor="tuning-preset">Tuning</Label>
          <Dropdown
            id="tuning-preset"
            value={selectedPreset.name}
            onOptionSelect={(_, data) => {
              const preset = TUNING_PRESETS.find((p) => p.name === data.optionValue);
              if (preset) setSelectedPreset(preset);
            }}
          >
            {TUNING_PRESETS.map((preset) => (
              <Option key={preset.name} value={preset.name}>
                {preset.name}
              </Option>
            ))}
          </Dropdown>
        </div>
      )}

      {/* String Buttons */}
      <div className={styles.stringButtons}>
        {selectedPreset.notes.map((note, index) => (
          <Button
            key={index}
            className={`${styles.stringButton} ${closestString === note ? styles.activeString : ''}`}
            appearance={targetNote === note ? 'primary' : 'secondary'}
            size="small"
            onClick={() => setTargetNote(targetNote === note ? null : note)}
          >
            {note}
          </Button>
        ))}
      </div>

      {/* Controls */}
      <div className={styles.controls}>
        <Button
          icon={tunerState.isActive ? <MicOff24Regular /> : <MicOn24Regular />}
          appearance={tunerState.isActive ? 'secondary' : 'primary'}
          onClick={toggleTuner}
        >
          {tunerState.isActive ? 'Stop' : 'Start Tuner'}
        </Button>
      </div>
    </div>
  );
}

/**
 * Tuner Dialog - Modal wrapper for tuner
 */
interface TunerDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function TunerDialog({ open, onOpenChange }: TunerDialogProps) {
  const tunerService = getTunerService();

  // Stop tuner when dialog closes
  useEffect(() => {
    if (!open) {
      tunerService.stop();
    }
  }, [open, tunerService]);

  return (
    <Dialog open={open} onOpenChange={(_, data) => onOpenChange(data.open)}>
      <DialogSurface>
        <Tuner onClose={() => onOpenChange(false)} />
      </DialogSurface>
    </Dialog>
  );
}
