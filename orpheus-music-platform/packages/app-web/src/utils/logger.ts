/**
 * Logger Utility
 * Centralized logging that can be disabled in production
 */

const isDevelopment = import.meta.env.DEV;

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LoggerOptions {
  prefix?: string;
  enabled?: boolean;
}

class Logger {
  private prefix: string;
  private enabled: boolean;

  constructor(options: LoggerOptions = {}) {
    this.prefix = options.prefix ? `[${options.prefix}]` : '';
    this.enabled = options.enabled ?? isDevelopment;
  }

  private formatMessage(level: LogLevel, ...args: unknown[]): void {
    if (!this.enabled) return;

    const message = this.prefix ? [this.prefix, ...args] : args;

    switch (level) {
      case 'debug':
        // eslint-disable-next-line no-console
        console.debug(...message);
        break;
      case 'info':
        // eslint-disable-next-line no-console
        console.log(...message);
        break;
      case 'warn':
        // eslint-disable-next-line no-console
        console.warn(...message);
        break;
      case 'error':
        // Always log errors, even in production
        // eslint-disable-next-line no-console
        console.error(...message);
        break;
    }
  }

  debug(...args: unknown[]): void {
    this.formatMessage('debug', ...args);
  }

  info(...args: unknown[]): void {
    this.formatMessage('info', ...args);
  }

  log(...args: unknown[]): void {
    this.formatMessage('info', ...args);
  }

  warn(...args: unknown[]): void {
    this.formatMessage('warn', ...args);
  }

  error(...args: unknown[]): void {
    // Errors always logged
    const message = this.prefix ? [this.prefix, ...args] : args;
    // eslint-disable-next-line no-console
    console.error(...message);
  }
}

// Pre-configured loggers for common modules
export const logger = {
  app: new Logger({ prefix: 'App' }),
  accessibility: new Logger({ prefix: 'Accessibility' }),
  alphaTab: new Logger({ prefix: 'alphaTab' }),
  shortcuts: new Logger({ prefix: 'Shortcuts' }),
  undo: new Logger({ prefix: 'Undo' }),
  export: new Logger({ prefix: 'Export' }),
  session: new Logger({ prefix: 'Session' }),
  playback: new Logger({ prefix: 'Playback' }),
  aiChat: new Logger({ prefix: 'AI Chat' }),
  toast: new Logger({ prefix: 'Toast' }),
  fileImport: new Logger({ prefix: 'FileImport' }),
  record: new Logger({ prefix: 'Record' }),
  noteScheduler: new Logger({ prefix: 'NoteScheduler' }),
  projectSave: new Logger({ prefix: 'ProjectSave' }),
  autoSave: new Logger({ prefix: 'AutoSave' }),
  master: new Logger({ prefix: 'Master' }),
  distribute: new Logger({ prefix: 'Distribute' }),
  audioRecorder: new Logger({ prefix: 'AudioRecorder' }),
  midiPlayback: new Logger({ prefix: 'MIDIPlayback' }),
};

// Create a custom logger for a specific module
export function createLogger(prefix: string, enabled?: boolean): Logger {
  return new Logger({ prefix, enabled });
}

// Default export for simple usage
export default logger;
