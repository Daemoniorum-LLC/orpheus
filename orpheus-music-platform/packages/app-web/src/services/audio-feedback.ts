/**
 * Audio Feedback Service - Subtle audio cues for hands-free operation
 * Plays tones so musicians don't need to look at the screen
 */

type FeedbackType =
  | 'listening'      // Started listening for voice
  | 'success'        // Command recognized and executed
  | 'error'          // Command not understood
  | 'confirm'        // Action completed (save, etc.)
  | 'metronome'      // Metronome click
  | 'countIn'        // Count-in beat
  | 'recordStart'    // Recording started
  | 'recordStop';    // Recording stopped

interface AudioFeedbackConfig {
  enabled: boolean;
  volume: number;  // 0-1
}

class AudioFeedbackService {
  private audioContext: AudioContext | null = null;
  private config: AudioFeedbackConfig = {
    enabled: true,
    volume: 0.3,
  };

  private getContext(): AudioContext {
    if (!this.audioContext) {
      this.audioContext = new AudioContext();
    }
    return this.audioContext;
  }

  /**
   * Play a tone with specified frequency and duration
   */
  private playTone(
    frequency: number,
    duration: number,
    type: OscillatorType = 'sine',
    volumeMultiplier: number = 1
  ): void {
    if (!this.config.enabled) return;

    try {
      const ctx = this.getContext();
      const oscillator = ctx.createOscillator();
      const gainNode = ctx.createGain();

      oscillator.type = type;
      oscillator.frequency.setValueAtTime(frequency, ctx.currentTime);

      // Envelope for smooth sound
      const volume = this.config.volume * volumeMultiplier;
      gainNode.gain.setValueAtTime(0, ctx.currentTime);
      gainNode.gain.linearRampToValueAtTime(volume, ctx.currentTime + 0.01);
      gainNode.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + duration);

      oscillator.connect(gainNode);
      gainNode.connect(ctx.destination);

      oscillator.start(ctx.currentTime);
      oscillator.stop(ctx.currentTime + duration);
    } catch (e) {
      // Audio context might not be available
      console.debug('Audio feedback unavailable:', e);
    }
  }

  /**
   * Play a chord (multiple frequencies)
   */
  private playChord(frequencies: number[], duration: number): void {
    frequencies.forEach((freq, i) => {
      setTimeout(() => {
        this.playTone(freq, duration, 'sine', 0.5);
      }, i * 30); // Slight arpeggio effect
    });
  }

  /**
   * Play feedback sound for a specific event type
   */
  play(type: FeedbackType): void {
    if (!this.config.enabled) return;

    switch (type) {
      case 'listening':
        // Ascending two-tone: "I'm listening"
        this.playTone(440, 0.1);  // A4
        setTimeout(() => this.playTone(554, 0.1), 100);  // C#5
        break;

      case 'success':
        // Pleasant major chord arpeggio: "Got it!"
        this.playChord([523, 659, 784], 0.2);  // C5, E5, G5
        break;

      case 'error':
        // Descending minor tone: "Didn't catch that"
        this.playTone(330, 0.15, 'triangle');  // E4
        setTimeout(() => this.playTone(294, 0.2, 'triangle'), 150);  // D4
        break;

      case 'confirm':
        // Single confident tone: "Done"
        this.playTone(880, 0.15);  // A5
        break;

      case 'metronome':
        // Sharp click
        this.playTone(1000, 0.03, 'square', 0.4);
        break;

      case 'countIn':
        // Higher pitched click for count-in (stands out from metronome)
        this.playTone(1500, 0.05, 'square', 0.5);
        break;

      case 'recordStart':
        // Three ascending tones: "Recording!"
        this.playTone(440, 0.1);
        setTimeout(() => this.playTone(554, 0.1), 120);
        setTimeout(() => this.playTone(659, 0.15), 240);
        break;

      case 'recordStop':
        // Two descending tones: "Stopped"
        this.playTone(659, 0.1);
        setTimeout(() => this.playTone(440, 0.15), 120);
        break;
    }
  }

  /**
   * Play count-in sequence (e.g., 4 beats before recording)
   */
  playCountIn(beats: number, bpm: number, onComplete: () => void): void {
    if (!this.config.enabled) {
      setTimeout(onComplete, (beats * 60 * 1000) / bpm);
      return;
    }

    const beatDuration = 60000 / bpm;
    let beat = 0;

    const tick = () => {
      if (beat < beats) {
        // First beat is accented
        if (beat === 0) {
          this.playTone(1800, 0.08, 'square', 0.6);
        } else {
          this.play('countIn');
        }
        beat++;
        setTimeout(tick, beatDuration);
      } else {
        onComplete();
      }
    };

    tick();
  }

  /**
   * Speak text using speech synthesis
   */
  speak(text: string, interrupt: boolean = true): void {
    if (!this.config.enabled) return;
    if (!('speechSynthesis' in window)) return;

    if (interrupt) {
      speechSynthesis.cancel();
    }

    const utterance = new SpeechSynthesisUtterance(text);
    utterance.rate = 1.1;  // Slightly faster
    utterance.pitch = 1.0;
    utterance.volume = this.config.volume;

    speechSynthesis.speak(utterance);
  }

  /**
   * Quick spoken confirmation
   */
  sayConfirmation(action: string): void {
    // Only speak for important actions, use tones for common ones
    const spokenActions = ['saved', 'recording', 'stopped'];
    if (spokenActions.some(a => action.toLowerCase().includes(a))) {
      this.speak(action);
    }
  }

  /**
   * Configure audio feedback
   */
  configure(config: Partial<AudioFeedbackConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfig(): AudioFeedbackConfig {
    return { ...this.config };
  }

  /**
   * Enable/disable audio feedback
   */
  setEnabled(enabled: boolean): void {
    this.config.enabled = enabled;
  }

  /**
   * Set volume (0-1)
   */
  setVolume(volume: number): void {
    this.config.volume = Math.max(0, Math.min(1, volume));
  }
}

// Singleton
let instance: AudioFeedbackService | null = null;

export function getAudioFeedback(): AudioFeedbackService {
  if (!instance) {
    instance = new AudioFeedbackService();
  }
  return instance;
}

export type { FeedbackType, AudioFeedbackConfig };
