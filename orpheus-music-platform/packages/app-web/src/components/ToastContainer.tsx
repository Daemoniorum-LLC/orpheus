/**
 * Toast Container - Displays toast notifications
 * Features:
 * - Queued screen reader announcements to prevent conflicts
 * - Staggered announcements for multiple simultaneous toasts
 */

import { useEffect, useState, useRef } from 'react';
import { CheckCircle, XCircle, AlertTriangle, Info, X } from 'lucide-react';
import { getToastManager, type Toast, dismissToast } from '../services/toast';
import { cn } from '../lib/utils';

// Announcement queue for screen readers
type Announcement = {
  message: string;
  priority: 'polite' | 'assertive';
};

export function ToastContainer() {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [announcement, setAnnouncement] = useState<string>('');
  const announcementQueue = useRef<Announcement[]>([]);
  const isAnnouncing = useRef(false);
  const prevToastIds = useRef<Set<string>>(new Set());

  // Process announcement queue with delays to prevent overlap
  useEffect(() => {
    const processQueue = () => {
      if (isAnnouncing.current || announcementQueue.current.length === 0) return;

      isAnnouncing.current = true;
      const next = announcementQueue.current.shift()!;

      // Clear first to ensure screen readers detect the change
      setAnnouncement('');

      // Small delay then set new announcement
      setTimeout(() => {
        setAnnouncement(next.message);

        // Clear after announcement is read (estimated 3s for average message)
        setTimeout(() => {
          setAnnouncement('');
          isAnnouncing.current = false;
          // Process next in queue
          processQueue();
        }, 3000);
      }, 100);
    };

    processQueue();
  }, [toasts]);

  // Queue new toast announcements
  useEffect(() => {
    const currentIds = new Set(toasts.map(t => t.id));

    // Find newly added toasts
    toasts.forEach(toast => {
      if (!prevToastIds.current.has(toast.id)) {
        const priority = toast.type === 'error' ? 'assertive' : 'polite';
        const typeLabel = toast.type.charAt(0).toUpperCase() + toast.type.slice(1);
        announcementQueue.current.push({
          message: `${typeLabel}: ${toast.message}`,
          priority
        });
      }
    });

    prevToastIds.current = currentIds;
  }, [toasts]);

  useEffect(() => {
    const manager = getToastManager();
    const unsubscribe = manager.subscribe(setToasts);
    return unsubscribe;
  }, []);

  const getIcon = (type: Toast['type']) => {
    const iconClass = 'h-5 w-5 flex-shrink-0';
    switch (type) {
      case 'success':
        return <CheckCircle className={cn(iconClass, 'text-green-500 animate-success-pulse')} />;
      case 'error':
        return <XCircle className={cn(iconClass, 'text-red-500')} />;
      case 'warning':
        return <AlertTriangle className={cn(iconClass, 'text-yellow-500')} />;
      case 'info':
        return <Info className={cn(iconClass, 'text-blue-500')} />;
    }
  };

  const getBorderClass = (type: Toast['type']) => {
    switch (type) {
      case 'success':
        return 'border-l-4 border-l-green-500';
      case 'error':
        return 'border-l-4 border-l-red-500';
      case 'warning':
        return 'border-l-4 border-l-yellow-500';
      case 'info':
        return 'border-l-4 border-l-blue-500';
    }
  };

  return (
    <>
      {/* Screen reader announcement region - single queue to prevent conflicts */}
      <div
        role="status"
        aria-live="polite"
        aria-atomic="true"
        className="sr-only"
      >
        {announcement}
      </div>

      {toasts.length === 0 ? null : (
        <>
          <style>
            {`
              @keyframes slideInRight {
                from {
                  transform: translateX(100%);
                  opacity: 0;
                }
                to {
                  transform: translateX(0);
                  opacity: 1;
                }
              }
            `}
          </style>
          <div
            className="fixed top-20 right-2 sm:right-5 z-[10000] flex flex-col gap-3 max-w-[calc(100vw-1rem)] sm:max-w-[400px] pointer-events-none"
            role="region"
            aria-label="Notifications"
          >
            {toasts.map((toast) => (
              <div
                key={toast.id}
                className={cn(
                  'pointer-events-auto flex items-center gap-3 py-3.5 px-4 rounded-lg',
                  'bg-background shadow-lg border border-border',
                  'animate-[slideInRight_0.3s_ease-out] min-w-[300px]',
                  getBorderClass(toast.type)
                )}
              >
                {getIcon(toast.type)}
                <div className="flex-1 text-sm leading-relaxed text-foreground">
                  {toast.message}
                </div>
                <button
                  type="button"
                  className="h-8 w-8 min-h-[44px] min-w-[44px] -m-1.5 cursor-pointer text-muted-foreground hover:text-foreground hover:bg-muted/50 rounded transition-colors flex-shrink-0 flex items-center justify-center focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                  onClick={() => dismissToast(toast.id)}
                  aria-label="Dismiss notification"
                >
                  <X className="h-4 w-4" aria-hidden="true" />
                </button>
              </div>
            ))}
          </div>
        </>
      )}
    </>
  );
}
