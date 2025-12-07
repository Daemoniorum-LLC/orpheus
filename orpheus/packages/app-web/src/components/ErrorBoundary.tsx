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
  // This is a function component but we can't use hooks here since it's called from a class component
  // We'll use inline styles instead
  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      height: '100vh',
      padding: '40px',
      textAlign: 'center',
    }}>
      <Bug24Regular style={{ fontSize: '64px', color: '#d13438', marginBottom: '24px' }} />

      <h1 style={{ fontSize: '24px', fontWeight: 600, marginBottom: '16px' }}>
        Something went wrong
      </h1>

      <p style={{ fontSize: '16px', color: '#666', marginBottom: '32px', maxWidth: '600px', lineHeight: 1.6 }}>
        We're sorry, but Orpheus encountered an unexpected error.
        Your work may have been auto-saved. Try reloading the page to continue.
      </p>

      {error && (
        <div style={{
          padding: '16px',
          backgroundColor: '#f5f5f5',
          borderRadius: '8px',
          marginBottom: '32px',
          maxWidth: '800px',
          width: '100%',
          textAlign: 'left',
          fontFamily: 'monospace',
          fontSize: '12px',
          color: '#666',
          overflow: 'auto',
          maxHeight: '200px',
        }}>
          <div style={{ fontWeight: 600, marginBottom: '8px' }}>Error Details:</div>
          <div>{error.toString()}</div>
          {errorInfo && (
            <div style={{ marginTop: '8px', fontSize: '11px', color: '#999' }}>
              {errorInfo.componentStack}
            </div>
          )}
        </div>
      )}

      <div style={{ display: 'flex', gap: '12px' }}>
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

      <p style={{ fontSize: '12px', color: '#999', marginTop: '32px' }}>
        If this problem persists, please report it on our GitHub issues page.
      </p>
    </div>
  );
}
