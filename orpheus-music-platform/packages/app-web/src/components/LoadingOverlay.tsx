/**
 * Loading Overlay - Global loading indicator with progress tracking
 */

import { Spinner, Button, Progress } from '@persona-framework/ui';
import { X } from 'lucide-react';
import { useAppStore } from '../store/app-store';

export function LoadingOverlay() {
  const {
    isLoading,
    loadingMessage,
    loadingProgress,
    loadingCancellable,
    cancelLoading,
  } = useAppStore();

  if (!isLoading) {
    return null;
  }

  const hasProgress = loadingProgress >= 0;

  const handleCancel = () => {
    if (cancelLoading) {
      cancelLoading();
    }
  };

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-[9999] flex-col gap-4 p-4"
      role="alertdialog"
      aria-modal="true"
      aria-label={loadingMessage || 'Loading'}
      aria-busy="true"
    >
      <div className="bg-background p-6 sm:p-8 rounded-lg border border-border shadow-2xl text-center w-full max-w-[90vw] sm:min-w-[400px] sm:max-w-[500px]">
        <Spinner className="h-10 w-10 mx-auto" aria-hidden="true" />

        {loadingMessage && (
          <div className="mt-4 text-sm text-muted-foreground mb-2" role="status">
            {loadingMessage}
          </div>
        )}

        {hasProgress && (
          <div className="mt-5 w-full">
            <Progress value={loadingProgress} className="h-2" aria-label={`Loading progress: ${Math.round(loadingProgress)}%`} />
            <div className="text-xs text-muted-foreground mt-2 font-mono" aria-hidden="true">
              {Math.round(loadingProgress)}% complete
            </div>
          </div>
        )}

        {loadingCancellable && cancelLoading && (
          <div className="mt-5 flex justify-center">
            <Button variant="secondary" onClick={handleCancel} aria-label="Cancel loading">
              <X className="h-4 w-4 mr-2" aria-hidden="true" />
              Cancel
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
