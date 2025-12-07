/**
 * MIDI clock for synchronization
 */

export class MIDIClock {
  private tempo: number;
  private division: number;
  private running: boolean = false;
  private startTime: number = 0;
  private pausedTime: number = 0;
  private callbacks: Array<(tick: number) => void> = [];

  constructor(tempo: number = 120, division: number = 480) {
    this.tempo = tempo;
    this.division = division;
  }

  /**
   * Starts the MIDI clock
   */
  start(): void {
    if (this.running) return;

    this.running = true;
    this.startTime = performance.now() - this.pausedTime;
    this.tick();
  }

  /**
   * Stops the MIDI clock
   */
  stop(): void {
    this.running = false;
    this.pausedTime = 0;
  }

  /**
   * Pauses the MIDI clock
   */
  pause(): void {
    if (!this.running) return;

    this.running = false;
    this.pausedTime = performance.now() - this.startTime;
  }

  /**
   * Sets the tempo (BPM)
   */
  setTempo(bpm: number): void {
    this.tempo = bpm;
  }

  /**
   * Gets the current tempo
   */
  getTempo(): number {
    return this.tempo;
  }

  /**
   * Gets the current tick
   */
  getCurrentTick(): number {
    if (!this.running) return 0;

    const elapsed = performance.now() - this.startTime;
    const ticksPerSecond = (this.tempo / 60) * this.division;
    return Math.floor((elapsed / 1000) * ticksPerSecond);
  }

  /**
   * Registers a callback to be called on each tick
   */
  onTick(callback: (tick: number) => void): () => void {
    this.callbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = this.callbacks.indexOf(callback);
      if (index !== -1) {
        this.callbacks.splice(index, 1);
      }
    };
  }

  private tick(): void {
    if (!this.running) return;

    const currentTick = this.getCurrentTick();

    // Call all registered callbacks
    for (const callback of this.callbacks) {
      callback(currentTick);
    }

    // Schedule next tick
    requestAnimationFrame(() => this.tick());
  }
}

/**
 * Converts ticks to milliseconds
 */
export function ticksToMilliseconds(ticks: number, tempo: number, division: number): number {
  const beatsPerSecond = tempo / 60;
  const ticksPerSecond = beatsPerSecond * division;
  return (ticks / ticksPerSecond) * 1000;
}

/**
 * Converts milliseconds to ticks
 */
export function millisecondsToTicks(ms: number, tempo: number, division: number): number {
  const beatsPerSecond = tempo / 60;
  const ticksPerSecond = beatsPerSecond * division;
  return (ms / 1000) * ticksPerSecond;
}
