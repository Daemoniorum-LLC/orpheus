/**
 * Playhead - Manages playback position and state
 */

/**
 * Playhead represents the current playback position
 * It handles play/pause/stop and maintains accurate timing
 */
export class Playhead {
  private position: number = 0; // Current position in seconds
  private playing: boolean = false;
  private startTime: number = 0;
  private pausedTime: number = 0;
  private callbacks: Array<() => void> = [];

  /**
   * Starts playback
   */
  play(): void {
    if (this.playing) return;

    this.playing = true;
    this.startTime = performance.now() - (this.pausedTime * 1000);
    this.notifyCallbacks();
  }

  /**
   * Pauses playback
   */
  pause(): void {
    if (!this.playing) return;

    this.playing = false;
    this.pausedTime = (performance.now() - this.startTime) / 1000;
    this.position = this.pausedTime;
    this.notifyCallbacks();
  }

  /**
   * Stops playback
   */
  stop(): void {
    this.playing = false;
    this.position = 0;
    this.pausedTime = 0;
    this.startTime = 0;
    this.notifyCallbacks();
  }

  /**
   * Sets the playback position
   */
  setPosition(seconds: number): void {
    this.position = seconds;
    this.pausedTime = seconds;

    if (this.playing) {
      this.startTime = performance.now() - (seconds * 1000);
    }

    this.notifyCallbacks();
  }

  /**
   * Gets the current playback position
   */
  getPosition(): number {
    if (this.playing) {
      return (performance.now() - this.startTime) / 1000;
    }
    return this.position;
  }

  /**
   * Checks if currently playing
   */
  isPlaying(): boolean {
    return this.playing;
  }

  /**
   * Advances the playhead by a delta (used in non-real-time scenarios)
   */
  advance(deltaSeconds: number): void {
    if (!this.playing) return;

    this.position += deltaSeconds;
    this.pausedTime = this.position;
    this.notifyCallbacks();
  }

  /**
   * Subscribes to position changes
   */
  onPositionChange(callback: () => void): () => void {
    this.callbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = this.callbacks.indexOf(callback);
      if (index !== -1) {
        this.callbacks.splice(index, 1);
      }
    };
  }

  private notifyCallbacks(): void {
    for (const callback of this.callbacks) {
      callback();
    }
  }
}
