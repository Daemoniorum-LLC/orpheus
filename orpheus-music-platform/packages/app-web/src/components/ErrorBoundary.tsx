/**
 * Error Boundary - Catches React errors and provides recovery UI
 */

import { Component, type ReactNode, type ErrorInfo, useState } from 'react';
import { Button } from '@persona-framework/ui';
import { RefreshCw, Bug } from 'lucide-react';
import { logger } from '../utils/logger';
import { safeStorage, safeSessionStorage } from '../utils/storage';

interface ErrorBoundaryProps {
  children: ReactNode;
  /** Fallback UI to show on error */
  fallback?: ReactNode;
  /** Callback when error occurs */
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    logger.app.error('ErrorBoundary caught error:', error, errorInfo);

    this.setState({
      errorInfo,
    });

    // Call optional error handler
    this.props.onError?.(error, errorInfo);
  }

  handleReload = (): void => {
    window.location.reload();
  };

  handleReset = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  handleClearData = (): void => {
    // Clear only Orpheus-specific keys, not all localStorage
    const orpheusKeys = safeStorage.getKeys().filter(
      (key) => key.startsWith('orpheus-') || key.startsWith('maestro-')
    );
    orpheusKeys.forEach((key) => safeStorage.removeItem(key));

    const sessionKeys = safeSessionStorage.getKeys().filter(
      (key) => key.startsWith('orpheus-') || key.startsWith('maestro-')
    );
    sessionKeys.forEach((key) => safeSessionStorage.removeItem(key));

    window.location.reload();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      // Use custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error UI
      return <ErrorFallbackUI
        error={this.state.error}
        onReload={this.handleReload}
        onReset={this.handleReset}
        onClearData={this.handleClearData}
      />;
    }

    return this.props.children;
  }
}

interface ErrorFallbackUIProps {
  error: Error | null;
  onReload: () => void;
  onReset: () => void;
  onClearData: () => void;
}

function ErrorFallbackUI({ error, onReload, onReset, onClearData }: ErrorFallbackUIProps) {
  const [clearDataConfirmOpen, setClearDataConfirmOpen] = useState(false);
  const [confirmText, setConfirmText] = useState('');
  const CONFIRM_PHRASE = 'DELETE';

  const handleClearDataCancel = () => {
    setClearDataConfirmOpen(false);
    setConfirmText(''); // Reset confirmation text
  };

  const handleClearDataConfirm = () => {
    if (confirmText === CONFIRM_PHRASE) {
      onClearData();
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-screen p-10 text-center bg-background">
      <Bug className="h-16 w-16 text-destructive mb-6" />

      <h1 className="text-2xl font-semibold mb-4">
        Something went wrong
      </h1>

      <p className="text-base text-muted-foreground mb-8 max-w-[600px] leading-relaxed">
        We're sorry, but Orpheus encountered an unexpected error.
        Your work may have been auto-saved. Try reloading the page to continue.
      </p>

      {error && (
        <div className="p-4 bg-muted rounded-lg mb-8 max-w-[800px] w-full text-left font-mono text-xs text-muted-foreground overflow-auto max-h-[150px]">
          <div className="font-semibold mb-2">Error Details:</div>
          <div>{error.message}</div>
        </div>
      )}

      <div className="flex gap-3">
        <Button onClick={onReload}>
          <RefreshCw className="h-4 w-4 mr-2" />
          Reload Page
        </Button>
        <Button variant="secondary" onClick={onReset}>
          Try Again
        </Button>
        <Button variant="ghost" onClick={() => setClearDataConfirmOpen(true)}>
          Clear All Data & Reload
        </Button>
      </div>

      {/* Double confirmation dialog with type-to-confirm */}
      {clearDataConfirmOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-background border border-border rounded-lg p-6 max-w-md w-full shadow-xl">
            <h2 className="text-lg font-semibold text-destructive flex items-center gap-2 mb-4">
              <Bug className="h-5 w-5" />
              Delete All Data?
            </h2>
            <p className="text-sm text-muted-foreground mb-4">
              This will <strong>permanently delete ALL</strong> your saved data, including:
            </p>
            <ul className="text-sm text-muted-foreground mb-4 list-disc list-inside space-y-1">
              <li>Auto-saved projects</li>
              <li>Preferences and settings</li>
              <li>Session history</li>
              <li>Export history</li>
            </ul>
            <p className="text-sm font-medium mb-2">
              Type <code className="px-1.5 py-0.5 bg-destructive/10 text-destructive rounded font-mono">{CONFIRM_PHRASE}</code> to confirm:
            </p>
            <input
              type="text"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value.toUpperCase())}
              placeholder="Type DELETE to confirm"
              className="w-full px-3 py-2 border border-border rounded-lg mb-4 bg-background focus:outline-none focus:ring-2 focus:ring-destructive"
              autoFocus
            />
            <div className="flex gap-3 justify-end">
              <Button variant="outline" onClick={handleClearDataCancel}>
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={handleClearDataConfirm}
                disabled={confirmText !== CONFIRM_PHRASE}
              >
                Delete Everything
              </Button>
            </div>
          </div>
        </div>
      )}

      <p className="text-xs text-muted-foreground mt-8">
        If this problem persists, please report it on our GitHub issues page.
      </p>
    </div>
  );
}
