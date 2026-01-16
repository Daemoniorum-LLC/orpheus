/**
 * Record Mode - Multi-track audio recording (Nexus DAW)
 */

import {
  Button,
  Card,
  CardContent,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@persona-framework/ui';
import {
  Circle,
  Square,
  Pause,
  Play,
  Trash2,
  Download,
  Mic,
} from 'lucide-react';
import { useState, useEffect } from 'react';
import { WaveformVisualizer } from '../components/WaveformVisualizer';
import { Metronome } from '../components/Metronome';
import { InputLevelMeter } from '../components/InputLevelMeter';
import { getAudioRecorder, type RecordingTrack, type RecorderState, type AudioDevice } from '../services/audio-recorder';
import { ConfirmDialog } from '../components/ConfirmDialog';

export function RecordMode() {
  const [recorderState, setRecorderState] = useState<RecorderState>({
    isRecording: false,
    isPaused: false,
    currentTime: 0,
    tracks: [],
  });
  const [trackName, setTrackName] = useState('');
  const [initialized, setInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string>('');

  // Confirm dialog state
  const [deleteConfirm, setDeleteConfirm] = useState<{
    open: boolean;
    trackId: string;
    trackName: string;
  }>({
    open: false,
    trackId: '',
    trackName: '',
  });

  const recorder = getAudioRecorder();

  // Initialize recorder
  useEffect(() => {
    const init = async () => {
      try {
        // Initialize recorder
        await recorder.initialize();

        // Enumerate audio devices
        const devices = await recorder.enumerateDevices();
        setAudioDevices(devices);
        if (devices.length > 0) {
          setSelectedDevice(devices[0].deviceId);
        }

        // Start level monitoring
        recorder.startLevelMonitoring();

        setInitialized(true);
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : 'Initialization failed');
      }
    };

    init();

    // Subscribe to state changes
    const unsubscribe = recorder.onStateChange(setRecorderState);

    // Update time while recording
    const interval = setInterval(() => {
      if (recorder.getState().isRecording && !recorder.getState().isPaused) {
        setRecorderState(recorder.getState());
      }
    }, 100);

    return () => {
      unsubscribe();
      clearInterval(interval);
      recorder.stopLevelMonitoring();
    };
  }, [recorder]);

  const handleDeviceChange = async (deviceId: string) => {
    try {
      setSelectedDevice(deviceId);
      await recorder.selectDevice(deviceId);
      recorder.startLevelMonitoring(); // Restart level monitoring after device change
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to change device');
    }
  };

  const handleStartRecording = async () => {
    try {
      await recorder.startRecording();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start recording');
    }
  };

  const handlePauseResume = () => {
    if (recorderState.isPaused) {
      recorder.resumeRecording();
    } else {
      recorder.pauseRecording();
    }
  };

  const handleStopRecording = async () => {
    try {
      await recorder.stopRecording(trackName || undefined);
      setTrackName('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to stop recording');
    }
  };

  const handleDeleteClick = (track: RecordingTrack) => {
    setDeleteConfirm({
      open: true,
      trackId: track.id,
      trackName: track.name,
    });
  };

  const confirmDeleteTrack = () => {
    recorder.deleteTrack(deleteConfirm.trackId);
  };

  const handleExportTrack = async (track: RecordingTrack) => {
    try {
      const blob = await recorder.exportTrack(track.id);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = track.name + '.webm';
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to export track');
    }
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 100);
    return mins + ':' + secs.toString().padStart(2, '0') + '.' + ms.toString().padStart(2, '0');
  };

  if (error) {
    return (
      <div className="h-full flex flex-col">
        <div className="p-4 border-b border-border flex justify-between items-center">
          <div className="text-xl font-semibold">🎙️ Record - Nexus DAW</div>
        </div>
        <div className="flex-1 flex items-center justify-center flex-col p-10 text-center text-muted-foreground">
          <h3 className="text-destructive mb-2">⚠️ Error</h3>
          <p>{error}</p>
          <p className="mt-4 text-xs">
            Make sure you have granted microphone permission and are using HTTPS.
          </p>
        </div>
      </div>
    );
  }

  if (!initialized) {
    return (
      <div className="h-full flex flex-col">
        <div className="p-4 border-b border-border flex justify-between items-center">
          <div className="text-xl font-semibold">🎙️ Record - Nexus DAW</div>
        </div>
        <div className="flex-1 flex items-center justify-center p-10 text-muted-foreground">
          <p>Initializing audio recorder...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Floating Recording Indicator */}
      {recorderState.isRecording && (
        <div className="fixed top-20 right-5 flex items-center gap-2 px-5 py-3 bg-red-600 text-white rounded-full shadow-lg z-[1000] font-semibold text-sm">
          <div className="w-3 h-3 rounded-full bg-red-400 animate-pulse shadow-[0_0_10px_rgba(255,0,0,0.8)]" />
          <span>● REC</span>
          <span className="font-mono font-bold">
            {formatTime(recorderState.currentTime)}
          </span>
        </div>
      )}

      <div className="p-4 border-b border-border flex justify-between items-center">
        <div className="text-xl font-semibold">🎙️ Record - Nexus DAW</div>
        <div className="flex gap-3 items-center">
          {recorderState.isRecording && (
            <div className="text-2xl font-mono font-bold text-destructive">
              {formatTime(recorderState.currentTime)}
            </div>
          )}
          <Button
            onClick={handleStartRecording}
            disabled={recorderState.isRecording}
            className={!recorderState.isRecording ? 'bg-red-600 hover:bg-red-700' : ''}
          >
            <Circle className="h-4 w-4 mr-2" />
            {recorderState.isRecording ? 'Recording...' : 'Record'}
          </Button>
          {recorderState.isRecording && (
            <>
              <Button variant="outline" onClick={handlePauseResume}>
                {recorderState.isPaused ? (
                  <Play className="h-4 w-4 mr-2" />
                ) : (
                  <Pause className="h-4 w-4 mr-2" />
                )}
                {recorderState.isPaused ? 'Resume' : 'Pause'}
              </Button>
              <Button variant="outline" onClick={handleStopRecording}>
                <Square className="h-4 w-4 mr-2" />
                Stop
              </Button>
            </>
          )}
        </div>
      </div>

      <div className="flex-1 flex flex-col p-6 gap-6 overflow-auto">
        {/* Metronome */}
        <Metronome initialBPM={120} initialTimeSignature={[4, 4]} />

        {/* Audio Input Selection */}
        {audioDevices.length > 0 && (
          <div className="max-w-[400px]">
            <Select
              value={selectedDevice}
              onValueChange={handleDeviceChange}
              disabled={recorderState.isRecording}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select microphone" />
              </SelectTrigger>
              <SelectContent>
                {audioDevices.map((device) => (
                  <SelectItem key={device.deviceId} value={device.deviceId}>
                    {device.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        {/* Input Level Meter */}
        {!recorderState.isRecording && (
          <div className="max-w-[600px]">
            <InputLevelMeter level={recorderState.inputLevel || 0} />
          </div>
        )}

        {/* Waveform Visualizer */}
        <div className={`border rounded-lg p-4 bg-secondary ${recorderState.isRecording ? 'border-destructive shadow-[0_0_20px_rgba(239,68,68,0.3)]' : 'border-border'}`}>
          <div className="flex justify-between items-center mb-4">
            <div className="flex items-center gap-2 font-semibold">
              <Mic className="h-5 w-5" />
              <span>Live Input Monitor</span>
            </div>
            {recorderState.isRecording && (
              <Input
                placeholder="Track name (optional)"
                value={trackName}
                onChange={(e) => setTrackName(e.target.value)}
                className="max-w-[200px]"
              />
            )}
          </div>
          <WaveformVisualizer width={1200} height={150} type="both" color="#667eea" />
        </div>

        {/* Recorded Tracks */}
        <div className="flex flex-col gap-3">
          <div className="font-semibold mb-2">
            Recorded Tracks ({recorderState.tracks.length})
          </div>

          {recorderState.tracks.length === 0 ? (
            <div className="text-center p-10 text-muted-foreground">
              <p>No tracks recorded yet</p>
              <p className="text-xs mt-2">
                Click the Record button above to start recording
              </p>
            </div>
          ) : (
            recorderState.tracks.map((track) => (
              <Card key={track.id}>
                <CardContent className="p-4 flex justify-between items-center">
                  <div className="flex flex-col gap-1">
                    <div className="text-sm font-semibold">{track.name}</div>
                    <div className="text-xs text-muted-foreground">
                      Duration: {formatTime(track.duration)} • ID: {track.id.slice(-8)}
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleExportTrack(track)}
                    >
                      <Download className="h-4 w-4 mr-2" />
                      Export
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleDeleteClick(track)}
                    >
                      <Trash2 className="h-4 w-4 mr-2" />
                      Delete
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))
          )}
        </div>
      </div>

      <ConfirmDialog
        open={deleteConfirm.open}
        onConfirm={confirmDeleteTrack}
        onCancel={() => setDeleteConfirm({ ...deleteConfirm, open: false })}
        title="Delete Recording?"
        message={`Are you sure you want to delete the recording "${deleteConfirm.trackName}"? This action cannot be undone.`}
        confirmText="Delete"
        cancelText="Cancel"
        type="danger"
      />
    </div>
  );
}
