/**
 * Voice Command Service - Hands-free control for Orpheus
 * Uses Web Speech API for recognition with fuzzy matching for robustness
 */

import { logger } from '../utils/logger';
import { getAudioFeedback } from './audio-feedback';

// Command categories for organized grammar
export type VoiceCommandCategory =
  | 'playback'
  | 'navigation'
  | 'project'
  | 'recording'
  | 'mixing'
  | 'system';

export interface VoiceCommand {
  id: string;
  phrases: string[];  // Multiple ways to say the same command
  category: VoiceCommandCategory;
  description: string;
  action: () => void | Promise<void>;
}

export interface VoiceCommandMatch {
  command: VoiceCommand;
  confidence: number;
  matchedPhrase: string;
  transcript: string;
}

export type VoiceState = 'idle' | 'listening' | 'processing' | 'success' | 'error';

export interface VoiceStateChange {
  state: VoiceState;
  transcript?: string;
  matchedCommand?: VoiceCommand;
  error?: string;
}

type VoiceStateListener = (state: VoiceStateChange) => void;

class VoiceCommandService {
  private recognition: SpeechRecognition | null = null;
  private commands: Map<string, VoiceCommand> = new Map();
  private listeners: Set<VoiceStateListener> = new Set();
  private isListening = false;
  private currentState: VoiceState = 'idle';
  private wakeWord: string = 'orpheus';
  private wakeWordEnabled: boolean = true;
  private continuousMode: boolean = false;

  constructor() {
    this.initRecognition();
  }

  private initRecognition(): void {
    // Check for browser support
    const SpeechRecognitionAPI =
      (window as any).SpeechRecognition ||
      (window as any).webkitSpeechRecognition;

    if (!SpeechRecognitionAPI) {
      logger.voice?.warn('Speech recognition not supported in this browser');
      return;
    }

    this.recognition = new SpeechRecognitionAPI();
    this.recognition.continuous = false;  // Single command at a time
    this.recognition.interimResults = true;  // Show partial results
    this.recognition.lang = 'en-US';
    this.recognition.maxAlternatives = 3;  // Get multiple interpretations

    this.recognition.onstart = () => {
      this.setState('listening');
      logger.voice?.info('Voice recognition started');
      // Audio feedback: "I'm listening"
      getAudioFeedback().play('listening');
    };

    this.recognition.onresult = (event: SpeechRecognitionEvent) => {
      const results = event.results[event.results.length - 1];

      if (results.isFinal) {
        const transcript = results[0].transcript.toLowerCase().trim();
        logger.voice?.debug('Final transcript:', transcript);
        this.processTranscript(transcript);
      } else {
        // Interim result - show what's being heard
        const interimTranscript = results[0].transcript;
        this.notifyListeners({
          state: 'listening',
          transcript: interimTranscript,
        });
      }
    };

    this.recognition.onerror = (event: SpeechRecognitionErrorEvent) => {
      logger.voice?.error('Recognition error:', event.error);

      // Don't show error for common non-errors
      if (event.error === 'no-speech' || event.error === 'aborted') {
        this.setState('idle');
        return;
      }

      this.setState('error', undefined, undefined, this.getErrorMessage(event.error));

      // Auto-recover
      setTimeout(() => {
        if (this.isListening) {
          this.setState('idle');
        }
      }, 2000);
    };

    this.recognition.onend = () => {
      // Restart if in continuous mode and still supposed to be listening
      if (this.continuousMode && this.isListening) {
        setTimeout(() => {
          if (this.isListening && this.recognition) {
            try {
              this.recognition.start();
            } catch (e) {
              // Already started, ignore
            }
          }
        }, 100);
      } else {
        this.isListening = false;
        this.setState('idle');
      }
    };
  }

  private getErrorMessage(error: string): string {
    const messages: Record<string, string> = {
      'not-allowed': 'Microphone access denied. Please allow microphone access.',
      'no-speech': 'No speech detected. Try again.',
      'audio-capture': 'No microphone found. Please connect a microphone.',
      'network': 'Network error. Check your connection.',
      'service-not-allowed': 'Speech service not allowed.',
    };
    return messages[error] || `Voice error: ${error}`;
  }

  private setState(
    state: VoiceState,
    transcript?: string,
    matchedCommand?: VoiceCommand,
    error?: string
  ): void {
    this.currentState = state;
    this.notifyListeners({ state, transcript, matchedCommand, error });
  }

  private notifyListeners(change: VoiceStateChange): void {
    this.listeners.forEach(listener => {
      try {
        listener(change);
      } catch (e) {
        logger.voice?.error('Listener error:', e);
      }
    });
  }

  private processTranscript(transcript: string): void {
    this.setState('processing', transcript);

    // Check for wake word if enabled
    if (this.wakeWordEnabled) {
      const hasWakeWord = transcript.includes(this.wakeWord) ||
                          transcript.includes('hey orpheus') ||
                          transcript.includes('okay orpheus') ||
                          transcript.includes('ok orpheus');

      if (!hasWakeWord && !this.continuousMode) {
        // No wake word found - show what was heard but don't execute
        logger.voice?.debug('No wake word detected in:', transcript);
        this.setState('idle');
        return;
      }

      // Remove wake word from transcript for command matching
      transcript = transcript
        .replace(/hey orpheus/g, '')
        .replace(/okay orpheus/g, '')
        .replace(/ok orpheus/g, '')
        .replace(/orpheus/g, '')
        .trim();
    }

    // Find matching command
    const match = this.findBestMatch(transcript);

    if (match && match.confidence > 0.5) {
      logger.voice?.info(`Matched command: ${match.command.id} (${Math.round(match.confidence * 100)}%)`);
      this.setState('success', transcript, match.command);

      // Audio feedback: "Got it!"
      getAudioFeedback().play('success');

      // Execute the command
      try {
        const result = match.command.action();
        if (result instanceof Promise) {
          result.catch(e => logger.voice?.error('Command execution error:', e));
        }
      } catch (e) {
        logger.voice?.error('Command execution error:', e);
      }

      // Reset state after feedback
      setTimeout(() => {
        if (this.currentState === 'success') {
          this.setState('idle');
        }
      }, 1500);
    } else {
      logger.voice?.debug('No matching command for:', transcript);
      this.setState('error', transcript, undefined, `Didn't understand: "${transcript}"`);

      // Audio feedback: "Didn't catch that"
      getAudioFeedback().play('error');

      setTimeout(() => {
        if (this.currentState === 'error') {
          this.setState('idle');
        }
      }, 2000);
    }
  }

  /**
   * Find the best matching command using fuzzy matching
   */
  private findBestMatch(transcript: string): VoiceCommandMatch | null {
    let bestMatch: VoiceCommandMatch | null = null;
    let bestScore = 0;

    for (const command of this.commands.values()) {
      for (const phrase of command.phrases) {
        const score = this.calculateMatchScore(transcript, phrase);

        if (score > bestScore) {
          bestScore = score;
          bestMatch = {
            command,
            confidence: score,
            matchedPhrase: phrase,
            transcript,
          };
        }
      }
    }

    return bestMatch;
  }

  /**
   * Calculate similarity score between transcript and command phrase
   * Uses multiple strategies for robustness
   */
  private calculateMatchScore(transcript: string, phrase: string): number {
    const t = transcript.toLowerCase().trim();
    const p = phrase.toLowerCase().trim();

    // Exact match
    if (t === p) return 1.0;

    // Contains exact phrase
    if (t.includes(p)) return 0.95;

    // Phrase contains transcript (for short commands)
    if (p.includes(t) && t.length > 3) return 0.9;

    // Word-based matching
    const tWords = t.split(/\s+/);
    const pWords = p.split(/\s+/);

    // Count matching words
    let matchingWords = 0;
    for (const tWord of tWords) {
      for (const pWord of pWords) {
        if (tWord === pWord || this.levenshteinDistance(tWord, pWord) <= 1) {
          matchingWords++;
          break;
        }
      }
    }

    const wordScore = matchingWords / Math.max(tWords.length, pWords.length);

    // Levenshtein distance for overall similarity
    const maxLen = Math.max(t.length, p.length);
    const distance = this.levenshteinDistance(t, p);
    const levenScore = 1 - (distance / maxLen);

    // Weighted combination
    return Math.max(wordScore * 0.7 + levenScore * 0.3, levenScore);
  }

  /**
   * Calculate Levenshtein distance between two strings
   */
  private levenshteinDistance(a: string, b: string): number {
    if (a.length === 0) return b.length;
    if (b.length === 0) return a.length;

    const matrix: number[][] = [];

    for (let i = 0; i <= b.length; i++) {
      matrix[i] = [i];
    }
    for (let j = 0; j <= a.length; j++) {
      matrix[0][j] = j;
    }

    for (let i = 1; i <= b.length; i++) {
      for (let j = 1; j <= a.length; j++) {
        if (b.charAt(i - 1) === a.charAt(j - 1)) {
          matrix[i][j] = matrix[i - 1][j - 1];
        } else {
          matrix[i][j] = Math.min(
            matrix[i - 1][j - 1] + 1,
            matrix[i][j - 1] + 1,
            matrix[i - 1][j] + 1
          );
        }
      }
    }

    return matrix[b.length][a.length];
  }

  // Public API

  /**
   * Check if voice commands are supported
   */
  isSupported(): boolean {
    return this.recognition !== null;
  }

  /**
   * Register a voice command
   */
  registerCommand(command: VoiceCommand): void {
    this.commands.set(command.id, command);
    logger.voice?.debug(`Registered command: ${command.id}`);
  }

  /**
   * Register multiple commands at once
   */
  registerCommands(commands: VoiceCommand[]): void {
    commands.forEach(cmd => this.registerCommand(cmd));
  }

  /**
   * Unregister a command
   */
  unregisterCommand(id: string): void {
    this.commands.delete(id);
  }

  /**
   * Get all registered commands
   */
  getCommands(): VoiceCommand[] {
    return Array.from(this.commands.values());
  }

  /**
   * Get commands by category
   */
  getCommandsByCategory(category: VoiceCommandCategory): VoiceCommand[] {
    return this.getCommands().filter(cmd => cmd.category === category);
  }

  /**
   * Start listening for voice commands
   */
  startListening(continuous: boolean = false): boolean {
    if (!this.recognition) {
      logger.voice?.error('Speech recognition not available');
      return false;
    }

    if (this.isListening) {
      return true;
    }

    this.continuousMode = continuous;
    this.isListening = true;

    try {
      this.recognition.start();
      return true;
    } catch (e) {
      logger.voice?.error('Failed to start recognition:', e);
      this.isListening = false;
      return false;
    }
  }

  /**
   * Stop listening for voice commands
   */
  stopListening(): void {
    this.isListening = false;
    this.continuousMode = false;

    if (this.recognition) {
      try {
        this.recognition.stop();
      } catch (e) {
        // Already stopped, ignore
      }
    }

    this.setState('idle');
  }

  /**
   * Toggle listening state
   */
  toggleListening(continuous: boolean = false): boolean {
    if (this.isListening) {
      this.stopListening();
      return false;
    } else {
      return this.startListening(continuous);
    }
  }

  /**
   * Check if currently listening
   */
  getIsListening(): boolean {
    return this.isListening;
  }

  /**
   * Get current state
   */
  getState(): VoiceState {
    return this.currentState;
  }

  /**
   * Subscribe to state changes
   */
  subscribe(listener: VoiceStateListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * Configure wake word
   */
  setWakeWord(word: string): void {
    this.wakeWord = word.toLowerCase();
  }

  /**
   * Enable/disable wake word requirement
   */
  setWakeWordEnabled(enabled: boolean): void {
    this.wakeWordEnabled = enabled;
  }

  /**
   * Check if wake word is enabled
   */
  getWakeWordEnabled(): boolean {
    return this.wakeWordEnabled;
  }
}

// Singleton instance
let voiceService: VoiceCommandService | null = null;

export function getVoiceCommandService(): VoiceCommandService {
  if (!voiceService) {
    voiceService = new VoiceCommandService();
  }
  return voiceService;
}

// Add voice logger category
if (typeof logger.voice === 'undefined') {
  (logger as any).voice = {
    info: (...args: any[]) => console.log('[Voice]', ...args),
    debug: (...args: any[]) => console.debug('[Voice]', ...args),
    warn: (...args: any[]) => console.warn('[Voice]', ...args),
    error: (...args: any[]) => console.error('[Voice]', ...args),
  };
}
