/**
 * Speed Trainer Component
 * Progressive speed building with loop sections and auto-increment
 */

import { makeStyles, shorthands, tokens, Button, Slider, Label, Switch, Input } from '@fluentui/react-components';
import { Play24Regular, Stop24Regular, Add24Regular, Subtract24Regular, ArrowRepeatAll24Regular } from '@fluentui/react-icons';
import { useState, useEffect } from 'react';
import { MessageDialog, type MessageType } from './MessageDialog';
import { ConfirmDialog } from './ConfirmDialog';

const useStyles = makeStyles({
  container: {
    ...shorthands.padding('20px'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    backgroundColor: tokens.colorNeutralBackground2,
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('20px'),
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
  },
  speedDisplay: {
    textAlign: 'center',
    ...shorthands.padding('24px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('8px'),
  },
  speedValue: {
    fontSize: '48px',
    fontWeight: 700,
    fontFamily: 'monospace',
    color: tokens.colorBrandForeground1,
    lineHeight: '1',
  },
  speedLabel: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
    marginTop: '8px',
    textTransform: 'uppercase',
  },
  targetSpeed: {
    fontSize: '14px',
    color: tokens.colorNeutralForeground2,
    marginTop: '4px',
  },
  controls: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    ...shorthands.gap('16px'),
  },
  control: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
  label: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground2,
    fontWeight: tokens.fontWeightSemibold,
  },
  value: {
    fontSize: '12px',
    fontFamily: 'monospace',
    textAlign: 'center',
    color: tokens.colorBrandForeground1,
  },
  buttonGroup: {
    display: 'flex',
    ...shorthands.gap('12px'),
    justifyContent: 'center',
    flexWrap: 'wrap',
  },
  speedButtons: {
    display: 'flex',
    ...shorthands.gap('8px'),
    justifyContent: 'center',
  },
  loopSection: {
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('6px'),
  },
  loopHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  loopTitle: {
    fontSize: '12px',
    fontWeight: tokens.fontWeightSemibold,
  },
  loopInputs: {
    display: 'flex',
    ...shorthands.gap('12px'),
    alignItems: 'center',
  },
  progress: {
    ...shorthands.padding('12px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('6px'),
    textAlign: 'center',
  },
  progressText: {
    fontSize: '14px',
    color: tokens.colorNeutralForeground2,
  },
});

export interface SpeedTrainerProps {
  /** Initial BPM */
  initialBPM?: number;
  /** Target BPM */
  targetBPM?: number;
  /** Increment step */
  incrementStep?: number;
  /** Number of successful reps before increment */
  repsBeforeIncrement?: number;
  /** Callback when speed changes */
  onSpeedChange?: (bpm: number) => void;
}

export function SpeedTrainer({
  initialBPM = 60,
  targetBPM = 120,
  incrementStep = 5,
  repsBeforeIncrement = 3,
  onSpeedChange,
}: SpeedTrainerProps) {
  const styles = useStyles();
  const [currentSpeed, setCurrentSpeed] = useState(initialBPM);
  const [targetSpeed, setTargetSpeed] = useState(targetBPM);
  const [increment, setIncrement] = useState(incrementStep);
  const [repsRequired, setRepsRequired] = useState(repsBeforeIncrement);
  const [successfulReps, setSuccessfulReps] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [autoIncrement, setAutoIncrement] = useState(false);

  // Message dialog state
  const [messageDialog, setMessageDialog] = useState<{
    open: boolean;
    title: string;
    message: string;
    type: MessageType;
  }>({
    open: false,
    title: '',
    message: '',
    type: 'info',
  });

  // Confirm dialog state
  const [resetConfirmOpen, setResetConfirmOpen] = useState(false);
  const [loopEnabled, setLoopEnabled] = useState(false);
  const [loopStart, setLoopStart] = useState('0');
  const [loopEnd, setLoopEnd] = useState('4');

  useEffect(() => {
    onSpeedChange?.(currentSpeed);
  }, [currentSpeed, onSpeedChange]);

  const handleSpeedChange = (delta: number) => {
    setCurrentSpeed(Math.max(40, Math.min(240, currentSpeed + delta)));
  };

  const handleSuccessfulRep = () => {
    const newReps = successfulReps + 1;
    setSuccessfulReps(newReps);

    if (autoIncrement && newReps >= repsRequired) {
      // Auto-increment speed
      const newSpeed = Math.min(targetSpeed, currentSpeed + increment);
      setCurrentSpeed(newSpeed);
      setSuccessfulReps(0);

      if (newSpeed >= targetSpeed) {
        setMessageDialog({
          open: true,
          title: 'Target Speed Reached!',
          message: `Congratulations! You've reached your target speed of ${targetSpeed} BPM!`,
          type: 'success',
        });
        setAutoIncrement(false);
      }
    }
  };

  const handleFailedRep = () => {
    // Drop back 10 BPM on failure
    setCurrentSpeed(Math.max(40, currentSpeed - 10));
    setSuccessfulReps(0);
  };

  const handleResetClick = () => {
    setResetConfirmOpen(true);
  };

  const confirmReset = () => {
    setCurrentSpeed(initialBPM);
    setSuccessfulReps(0);
  };

  const progressPercentage = ((currentSpeed - initialBPM) / (targetSpeed - initialBPM)) * 100;

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.title}>Speed Trainer</div>
        <Switch
          checked={autoIncrement}
          onChange={(_, data) => setAutoIncrement(data.checked)}
          label={autoIncrement ? 'Auto-Increment: ON' : 'Auto-Increment: OFF'}
        />
      </div>

      {/* Speed Display */}
      <div className={styles.speedDisplay}>
        <div className={styles.speedValue}>{currentSpeed}</div>
        <div className={styles.speedLabel}>BPM</div>
        <div className={styles.targetSpeed}>Target: {targetSpeed} BPM</div>
      </div>

      {/* Speed Control Buttons */}
      <div className={styles.speedButtons}>
        <Button
          icon={<Subtract24Regular />}
          onClick={() => handleSpeedChange(-increment)}
          size="large"
        >
          -{increment}
        </Button>
        <Button
          icon={<Subtract24Regular />}
          onClick={() => handleSpeedChange(-1)}
        >
          -1
        </Button>
        <Button
          appearance="primary"
          onClick={() => setCurrentSpeed(targetSpeed)}
        >
          Jump to Target
        </Button>
        <Button
          icon={<Add24Regular />}
          onClick={() => handleSpeedChange(1)}
        >
          +1
        </Button>
        <Button
          icon={<Add24Regular />}
          onClick={() => handleSpeedChange(increment)}
          size="large"
        >
          +{increment}
        </Button>
      </div>

      {/* Controls */}
      <div className={styles.controls}>
        <div className={styles.control}>
          <Label className={styles.label}>INCREMENT STEP</Label>
          <Slider
            min={1}
            max={20}
            step={1}
            value={increment}
            onChange={(_, data) => setIncrement(data.value)}
          />
          <div className={styles.value}>{increment} BPM</div>
        </div>

        <div className={styles.control}>
          <Label className={styles.label}>REPS BEFORE INCREMENT</Label>
          <Slider
            min={1}
            max={10}
            step={1}
            value={repsRequired}
            onChange={(_, data) => setRepsRequired(data.value)}
          />
          <div className={styles.value}>{repsRequired} reps</div>
        </div>

        <div className={styles.control}>
          <Label className={styles.label}>TARGET SPEED</Label>
          <Slider
            min={currentSpeed}
            max={240}
            step={5}
            value={targetSpeed}
            onChange={(_, data) => setTargetSpeed(data.value)}
          />
          <div className={styles.value}>{targetSpeed} BPM</div>
        </div>
      </div>

      {/* Loop Section */}
      <div className={styles.loopSection}>
        <div className={styles.loopHeader}>
          <div className={styles.loopTitle}>Loop Section</div>
          <Switch
            checked={loopEnabled}
            onChange={(_, data) => setLoopEnabled(data.checked)}
            label={loopEnabled ? 'ON' : 'OFF'}
          />
        </div>
        {loopEnabled && (
          <div className={styles.loopInputs}>
            <Label>Start (measure):</Label>
            <Input
              type="number"
              value={loopStart}
              onChange={(e) => setLoopStart(e.target.value)}
              style={{ width: '80px' }}
            />
            <Label>End (measure):</Label>
            <Input
              type="number"
              value={loopEnd}
              onChange={(e) => setLoopEnd(e.target.value)}
              style={{ width: '80px' }}
            />
            <Button icon={<ArrowRepeatAll24Regular />} appearance="subtle">
              Set Loop
            </Button>
          </div>
        )}
      </div>

      {/* Progress */}
      {autoIncrement && (
        <div className={styles.progress}>
          <div className={styles.progressText}>
            Progress: {successfulReps}/{repsRequired} successful reps
          </div>
          <div className={styles.progressText}>
            {progressPercentage.toFixed(0)}% to target ({currentSpeed}/{targetSpeed} BPM)
          </div>
        </div>
      )}

      {/* Action Buttons */}
      <div className={styles.buttonGroup}>
        <Button
          icon={isPlaying ? <Stop24Regular /> : <Play24Regular />}
          appearance="primary"
          onClick={() => setIsPlaying(!isPlaying)}
          size="large"
        >
          {isPlaying ? 'Stop' : 'Start'} Practice
        </Button>
        {autoIncrement && (
          <>
            <Button
              appearance="primary"
              onClick={handleSuccessfulRep}
            >
              ✓ Successful Rep
            </Button>
            <Button
              appearance="secondary"
              onClick={handleFailedRep}
            >
              ✗ Failed Rep
            </Button>
          </>
        )}
        <Button
          appearance="subtle"
          onClick={handleResetClick}
        >
          Reset Progress
        </Button>
      </div>

      <MessageDialog
        open={messageDialog.open}
        onClose={() => setMessageDialog({ ...messageDialog, open: false })}
        title={messageDialog.title}
        message={messageDialog.message}
        type={messageDialog.type}
      />

      <ConfirmDialog
        open={resetConfirmOpen}
        onConfirm={confirmReset}
        onCancel={() => setResetConfirmOpen(false)}
        title="Reset Progress?"
        message="Are you sure you want to reset your progress? This will set your speed back to the starting BPM and clear all successful reps."
        confirmText="Reset"
        cancelText="Cancel"
        type="warning"
      />
    </div>
  );
}
