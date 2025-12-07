/**
 * Audio Recorder Service
 * Web Audio API-based multi-track audio recording
 */

import * as Tone from 'tone';

export interface RecordingTrack {
  id: string;
  name: string;
  audioBuffer: AudioBuffer | null;
  blob: Blob | null;
  duration: number;
  startTime: number;
}

export interface RecorderState {
  isRecording: boolean;
  isPaused: boolean;
  currentTime: number;
  tracks: RecordingTrack[];
  inputLevel?: number; // 0-1 normalized input level
}

export interface AudioDevice {
  deviceId: string;
  label: string;
  kind: string;
}

/**
 * Multi-track audio recorder
 */
export class AudioRecorder {
  private mediaRecorder: MediaRecorder | null = null;
  private mediaStream: MediaStream | null = null;
  private chunks: Blob[] = [];
  private recorder: Tone.Recorder | null = null;
  private isRecording = false;
  private isPaused = false;
  private startTime: number = 0;
  private pauseTime: number = 0;
  private tracks: RecordingTrack[] = [];
  private stateCallbacks: Array<(state: RecorderState) => void> = [];
  private analyserNode: AnalyserNode | null = null;
  private inputLevel: number = 0;
  private levelMonitoringActive = false;
  private levelMonitoringFrame: number | null = null;

  /**
   * Initialize audio recording
   */
  async initialize(deviceId?: string): Promise<void> {
    try {
      // Stop existing stream if reinitializing
      if (this.mediaStream) {
        this.mediaStream.getTracks().forEach((track) => track.stop());
      }

      // Request microphone access
      const constraints: MediaStreamConstraints = {
        audio: deviceId
          ? {
              deviceId: { exact: deviceId },
              echoCancellation: false,
              noiseSuppression: false,
              autoGainControl: false,
              sampleRate: 48000,
            }
          : {
              echoCancellation: false,
              noiseSuppression: false,
              autoGainControl: false,
              sampleRate: 48000,
            },
      };

      this.mediaStream = await navigator.mediaDevices.getUserMedia(constraints);

      console.log('[AudioRecorder] Microphone access granted');

      // Create Tone.js recorder
      this.recorder = new Tone.Recorder();

      // Create analyser node for input level monitoring
      this.analyserNode = Tone.context.createAnalyser();
      this.analyserNode.fftSize = 256;
      this.analyserNode.smoothingTimeConstant = 0.8;

      // Connect microphone to Tone.js
      const micNode = Tone.context.createMediaStreamSource(this.mediaStream);
      const toneNode = Tone.context.createGain();
      micNode.connect(toneNode);
      micNode.connect(this.analyserNode); // Also connect to analyser

      // Connect to Tone.js graph
      const player = new Tone.Player().toDestination();
      player.connect(this.recorder);

      console.log('[AudioRecorder] Initialized successfully');
    } catch (error) {
      console.error('[AudioRecorder] Failed to initialize:', error);
      throw new Error('Failed to access microphone. Please grant permission.');
    }
  }

  /**
   * Enumerate available audio input devices
   */
  async enumerateDevices(): Promise<AudioDevice[]> {
    try {
      const devices = await navigator.mediaDevices.enumerateDevices();
      const audioInputs = devices
        .filter((device) => device.kind === 'audioinput')
        .map((device) => ({
          deviceId: device.deviceId,
          label: device.label || `Microphone ${device.deviceId.slice(0, 8)}`,
          kind: device.kind,
        }));

      console.log('[AudioRecorder] Found audio input devices:', audioInputs);
      return audioInputs;
    } catch (error) {
      console.error('[AudioRecorder] Failed to enumerate devices:', error);
      return [];
    }
  }

  /**
   * Select a specific audio input device
   */
  async selectDevice(deviceId: string): Promise<void> {
    console.log('[AudioRecorder] Selecting device:', deviceId);
    await this.initialize(deviceId);
  }

  /**
   * Start monitoring input levels
   */
  startLevelMonitoring(): void {
    if (this.levelMonitoringActive || !this.analyserNode) {
      return;
    }

    this.levelMonitoringActive = true;

    const bufferLength = this.analyserNode.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);

    const updateLevel = () => {
      if (!this.levelMonitoringActive || !this.analyserNode) {
        return;
      }

      this.analyserNode.getByteTimeDomainData(dataArray);

      // Calculate RMS (Root Mean Square) for more accurate level
      let sum = 0;
      for (let i = 0; i < bufferLength; i++) {
        const normalized = (dataArray[i] - 128) / 128;
        sum += normalized * normalized;
      }
      const rms = Math.sqrt(sum / bufferLength);

      this.inputLevel = Math.min(1, rms * 2); // Scale and clamp to 0-1
      this.notifyStateChange();

      this.levelMonitoringFrame = requestAnimationFrame(updateLevel);
    };

    updateLevel();
    console.log('[AudioRecorder] Started level monitoring');
  }

  /**
   * Stop monitoring input levels
   */
  stopLevelMonitoring(): void {
    this.levelMonitoringActive = false;
    if (this.levelMonitoringFrame !== null) {
      cancelAnimationFrame(this.levelMonitoringFrame);
      this.levelMonitoringFrame = null;
    }
    this.inputLevel = 0;
    console.log('[AudioRecorder] Stopped level monitoring');
  }

  /**
   * Get current input level (0-1)
   */
  getCurrentInputLevel(): number {
    return this.inputLevel;
  }

  /**
   * Start recording
   */
  async startRecording(): Promise<void> {
    if (!this.recorder) {
      throw new Error('Recorder not initialized. Call initialize() first.');
    }

    if (this.isRecording) {
      console.warn('[AudioRecorder] Already recording');
      return;
    }

    console.log('[AudioRecorder] Starting recording');

    this.chunks = [];
    this.isRecording = true;
    this.isPaused = false;
    this.startTime = performance.now();

    // Start Tone.js recorder
    this.recorder.start();

    // Also use MediaRecorder for backup
    if (this.mediaStream) {
      this.mediaRecorder = new MediaRecorder(this.mediaStream, {
        mimeType: 'audio/webm;codecs=opus',
      });

      this.mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          this.chunks.push(event.data);
        }
      };

      this.mediaRecorder.start(100); // Collect data every 100ms
    }

    this.notifyStateChange();
  }

  /**
   * Pause recording
   */
  pauseRecording(): void {
    if (!this.isRecording || this.isPaused) return;

    console.log('[AudioRecorder] Pausing recording');
    this.isPaused = true;
    this.pauseTime = performance.now();

    if (this.mediaRecorder && this.mediaRecorder.state === 'recording') {
      this.mediaRecorder.pause();
    }

    this.notifyStateChange();
  }

  /**
   * Resume recording
   */
  resumeRecording(): void {
    if (!this.isPaused) return;

    console.log('[AudioRecorder] Resuming recording');
    this.isPaused = false;

    if (this.mediaRecorder && this.mediaRecorder.state === 'paused') {
      this.mediaRecorder.resume();
    }

    this.notifyStateChange();
  }

  /**
   * Stop recording and save track
   */
  async stopRecording(trackName?: string): Promise<RecordingTrack> {
    if (!this.isRecording || !this.recorder) {
      throw new Error('Not currently recording');
    }

    console.log('[AudioRecorder] Stopping recording');

    // Stop Tone.js recorder
    const recording = await this.recorder.stop();

    // Stop MediaRecorder
    if (this.mediaRecorder && this.mediaRecorder.state !== 'inactive') {
      this.mediaRecorder.stop();
    }

    this.isRecording = false;
    this.isPaused = false;

    const duration = (performance.now() - this.startTime) / 1000;

    // Create blob from chunks
    const blob = new Blob(this.chunks, { type: 'audio/webm' });

    // Convert to AudioBuffer
    const arrayBuffer = await blob.arrayBuffer();
    const audioBuffer = await Tone.context.decodeAudioData(arrayBuffer);

    // Create track
    const track: RecordingTrack = {
      id: `track-${Date.now()}`,
      name: trackName || `Recording ${this.tracks.length + 1}`,
      audioBuffer,
      blob: recording, // Use Tone.js recording
      duration,
      startTime: this.startTime,
    };

    this.tracks.push(track);
    this.notifyStateChange();

    console.log(`[AudioRecorder] Saved track: ${track.name} (${duration.toFixed(2)}s)`);

    return track;
  }

  /**
   * Get current recording time
   */
  getCurrentTime(): number {
    if (!this.isRecording) return 0;
    if (this.isPaused) {
      return (this.pauseTime - this.startTime) / 1000;
    }
    return (performance.now() - this.startTime) / 1000;
  }

  /**
   * Get all recorded tracks
   */
  getTracks(): RecordingTrack[] {
    return [...this.tracks];
  }

  /**
   * Delete a track
   */
  deleteTrack(trackId: string): void {
    this.tracks = this.tracks.filter((t) => t.id !== trackId);
    this.notifyStateChange();
  }

  /**
   * Clear all tracks
   */
  clearTracks(): void {
    this.tracks = [];
    this.notifyStateChange();
  }

  /**
   * Export track as WAV
   */
  async exportTrack(trackId: string): Promise<Blob> {
    const track = this.tracks.find((t) => t.id === trackId);
    if (!track || !track.blob) {
      throw new Error('Track not found or has no audio data');
    }

    return track.blob;
  }

  /**
   * Get current state
   */
  getState(): RecorderState {
    return {
      isRecording: this.isRecording,
      isPaused: this.isPaused,
      currentTime: this.getCurrentTime(),
      tracks: [...this.tracks],
      inputLevel: this.inputLevel,
    };
  }

  /**
   * Subscribe to state changes
   */
  onStateChange(callback: (state: RecorderState) => void): () => void {
    this.stateCallbacks.push(callback);
    return () => {
      const index = this.stateCallbacks.indexOf(callback);
      if (index !== -1) {
        this.stateCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Cleanup resources
   */
  dispose(): void {
    if (this.isRecording) {
      this.stopRecording().catch(console.error);
    }

    this.stopLevelMonitoring();

    if (this.mediaRecorder) {
      this.mediaRecorder.stop();
      this.mediaRecorder = null;
    }

    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach((track) => track.stop());
      this.mediaStream = null;
    }

    if (this.recorder) {
      this.recorder.dispose();
      this.recorder = null;
    }

    if (this.analyserNode) {
      this.analyserNode = null;
    }

    this.tracks = [];
    this.stateCallbacks = [];
  }

  /**
   * Notify state change callbacks
   */
  private notifyStateChange(): void {
    const state = this.getState();
    for (const callback of this.stateCallbacks) {
      callback(state);
    }
  }
}

// Singleton instance
let recorderInstance: AudioRecorder | null = null;

/**
 * Get the global audio recorder instance
 */
export function getAudioRecorder(): AudioRecorder {
  if (!recorderInstance) {
    recorderInstance = new AudioRecorder();
  }
  return recorderInstance;
}

/**
 * Reset the recorder (for testing)
 */
export function resetAudioRecorder(): void {
  if (recorderInstance) {
    recorderInstance.dispose();
    recorderInstance = null;
  }
}
