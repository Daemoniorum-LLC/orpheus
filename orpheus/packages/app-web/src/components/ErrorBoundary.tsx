/**
 * Error Boundary - Catches React errors and provides recovery UI
 */

import { Component, type ReactNode, type ErrorInfo } from 'react';
import { Button } from '@fluentui/react-components';
import { ArrowClockwise24Regular, Bug24Regular } from '@fluentui/react-icons';

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
    console.error('[ErrorBoundary] Caught error:', error, errorInfo);

    this.setState({
      errorInfo,
    });

    // Call optional error handler
    this.props.onError?.(error, errorInfo);

    // Log to error reporting service (future enhancement)
    // reportError(error, errorInfo);
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
    // Clear all local storage and session storage
    localStorage.clear();
    sessionStorage.clear();
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
        errorInfo={this.state.errorInfo}
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
  errorInfo: ErrorInfo | null;
  onReload: () => void;
  onReset: () => void;
  onClearData: () => void;
}

function ErrorFallbackUI({ error, errorInfo, onReload, onReset, onClearData }: ErrorFallbackUIProps) {
  return (
    <div
      className="flex flex-col items-center justify-center h-screen p-10 text-center bg-background"
      role="alert"
      aria-live="assertive"
    >
      <Bug24Regular className="text-6xl text-error mb-6" aria-hidden="true" />

      <h1 className="text-2xl font-semibold mb-4 text-foreground">
        Something went wrong
      </h1>

      <p className="text-base text-foreground-secondary mb-8 max-w-[600px] leading-relaxed">
        We're sorry, but Orpheus encountered an unexpected error.
        Your work may have been auto-saved. Try reloading the page to continue.
      </p>

      {error && (
        <div className="p-4 bg-background-secondary rounded-lg mb-8 max-w-[800px] w-full text-left font-mono text-xs text-foreground-muted overflow-auto max-h-[200px] border border-border">
          <div className="font-semibold mb-2">Error Details:</div>
          <div className="text-error">{error.toString()}</div>
          {errorInfo && (
            <details className="mt-2">
              <summary className="cursor-pointer text-foreground-muted hover:text-foreground">
                Stack trace
              </summary>
              <pre className="mt-2 text-[11px] whitespace-pre-wrap">
                {errorInfo.componentStack}
              </pre>
            </details>
          )}
        </div>
      )}

      <div className="flex gap-3">
        <Button
          appearance="primary"
          icon={<ArrowClockwise24Regular />}
          onClick={onReload}
        >
          Reload Page
        </Button>
        <Button
          appearance="secondary"
          onClick={onReset}
        >
          Try Again
        </Button>
        <Button
          appearance="subtle"
          onClick={onClearData}
        >
          Clear All Data & Reload
        </Button>
      </div>

      <p className="text-xs text-foreground-muted mt-8">
        If this problem persists, please{' '}
        <a
          href="https://github.com/Daemoniorum-LLC/orpheus/issues"
          className="text-accent hover:underline"
          target="_blank"
          rel="noopener noreferrer"
        >
          report it on our GitHub issues page
        </a>.
      </p>
    </div>
  );
}
