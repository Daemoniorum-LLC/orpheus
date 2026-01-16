/**
 * Speed Trainer Component
 * Progressive speed building with loop sections and auto-increment
 */

import { Button, Label, Switch, Input, Slider } from '@persona-framework/ui';
import { Play, Square, Plus, Minus, Repeat } from 'lucide-react';
import { useState, useEffect } from 'react';
import { MessageDialog, type MessageType } from './MessageDialog';
import { ConfirmDialog } from './ConfirmDialog';

export interface SpeedTrainerProps {
  initialBPM?: number;
  targetBPM?: number;
  incrementStep?: number;
  repsBeforeIncrement?: number;
  onSpeedChange?: (bpm: number) => void;
}

export function SpeedTrainer({
  initialBPM = 60,
  targetBPM = 120,
  incrementStep = 5,
  repsBeforeIncrement = 3,
  onSpeedChange,
}: SpeedTrainerProps) {
  const [currentSpeed, setCurrentSpeed] = useState(initialBPM);
  const [targetSpeed, setTargetSpeed] = useState(targetBPM);
  const [increment, setIncrement] = useState(incrementStep);
  const [repsRequired, setRepsRequired] = useState(repsBeforeIncrement);
  const [successfulReps, setSuccessfulReps] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [autoIncrement, setAutoIncrement] = useState(false);

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
    <div className="p-5 border border-border rounded-lg bg-secondary flex flex-col gap-5">
      <div className="flex justify-between items-center">
        <div className="text-base font-semibold">Speed Trainer</div>
        <div className="flex items-center gap-2">
          <Switch
            checked={autoIncrement}
            onCheckedChange={setAutoIncrement}
          />
          <Label className="text-sm">
            Auto-Increment: {autoIncrement ? 'ON' : 'OFF'}
          </Label>
        </div>
      </div>

      {/* Speed Display */}
      <div className="text-center p-6 bg-background rounded-lg">
        <div className="text-5xl font-bold font-mono text-primary leading-none">
          {currentSpeed}
        </div>
        <div className="text-xs text-muted-foreground mt-2 uppercase">BPM</div>
        <div className="text-sm text-muted-foreground mt-1">Target: {targetSpeed} BPM</div>
      </div>

      {/* Speed Control Buttons */}
      <div className="flex gap-2 justify-center flex-wrap">
        <Button
          variant="outline"
          size="lg"
          onClick={() => handleSpeedChange(-increment)}
        >
          <Minus className="h-4 w-4 mr-1" />
          -{increment}
        </Button>
        <Button
          variant="outline"
          onClick={() => handleSpeedChange(-1)}
        >
          <Minus className="h-4 w-4 mr-1" />
          -1
        </Button>
        <Button onClick={() => setCurrentSpeed(targetSpeed)}>
          Jump to Target
        </Button>
        <Button
          variant="outline"
          onClick={() => handleSpeedChange(1)}
        >
          <Plus className="h-4 w-4 mr-1" />
          +1
        </Button>
        <Button
          variant="outline"
          size="lg"
          onClick={() => handleSpeedChange(increment)}
        >
          <Plus className="h-4 w-4 mr-1" />
          +{increment}
        </Button>
      </div>

      {/* Controls */}
      <div className="grid grid-cols-2 gap-4">
        <div className="flex flex-col gap-2">
          <Label className="text-[11px] text-muted-foreground font-semibold">INCREMENT STEP</Label>
          <Slider
            min={1}
            max={20}
            step={1}
            value={[increment]}
            onValueChange={(value) => setIncrement(value[0])}
          />
          <div className="text-xs font-mono text-center text-primary">{increment} BPM</div>
        </div>

        <div className="flex flex-col gap-2">
          <Label className="text-[11px] text-muted-foreground font-semibold">REPS BEFORE INCREMENT</Label>
          <Slider
            min={1}
            max={10}
            step={1}
            value={[repsRequired]}
            onValueChange={(value) => setRepsRequired(value[0])}
          />
          <div className="text-xs font-mono text-center text-primary">{repsRequired} reps</div>
        </div>

        <div className="flex flex-col gap-2 col-span-2">
          <Label className="text-[11px] text-muted-foreground font-semibold">TARGET SPEED</Label>
          <Slider
            min={currentSpeed}
            max={240}
            step={5}
            value={[targetSpeed]}
            onValueChange={(value) => setTargetSpeed(value[0])}
          />
          <div className="text-xs font-mono text-center text-primary">{targetSpeed} BPM</div>
        </div>
      </div>

      {/* Loop Section */}
      <div className="p-4 bg-background rounded-md">
        <div className="flex justify-between items-center mb-3">
          <div className="text-xs font-semibold">Loop Section</div>
          <div className="flex items-center gap-2">
            <Switch
              checked={loopEnabled}
              onCheckedChange={setLoopEnabled}
            />
            <Label className="text-xs">{loopEnabled ? 'ON' : 'OFF'}</Label>
          </div>
        </div>
        {loopEnabled && (
          <div className="flex gap-3 items-center">
            <Label className="text-xs">Start (measure):</Label>
            <Input
              type="number"
              value={loopStart}
              onChange={(e) => setLoopStart(e.target.value)}
              className="w-20"
            />
            <Label className="text-xs">End (measure):</Label>
            <Input
              type="number"
              value={loopEnd}
              onChange={(e) => setLoopEnd(e.target.value)}
              className="w-20"
            />
            <Button variant="ghost" size="sm">
              <Repeat className="h-4 w-4 mr-1" />
              Set Loop
            </Button>
          </div>
        )}
      </div>

      {/* Progress */}
      {autoIncrement && (
        <div className="p-3 bg-background rounded-md text-center">
          <div className="text-sm text-muted-foreground">
            Progress: {successfulReps}/{repsRequired} successful reps
          </div>
          <div className="text-sm text-muted-foreground">
            {progressPercentage.toFixed(0)}% to target ({currentSpeed}/{targetSpeed} BPM)
          </div>
        </div>
      )}

      {/* Action Buttons */}
      <div className="flex gap-3 justify-center flex-wrap">
        <Button
          size="lg"
          onClick={() => setIsPlaying(!isPlaying)}
        >
          {isPlaying ? (
            <Square className="h-4 w-4 mr-2" />
          ) : (
            <Play className="h-4 w-4 mr-2" />
          )}
          {isPlaying ? 'Stop' : 'Start'} Practice
        </Button>
        {autoIncrement && (
          <>
            <Button onClick={handleSuccessfulRep}>
              Successful Rep
            </Button>
            <Button variant="outline" onClick={handleFailedRep}>
              Failed Rep
            </Button>
          </>
        )}
        <Button
          variant="ghost"
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
