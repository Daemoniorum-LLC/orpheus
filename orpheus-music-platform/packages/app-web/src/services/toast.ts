/**
 * Toast Notification Service
 * Simple toast notification system for user feedback
 */

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

type ToastListener = (toasts: Toast[]) => void;

class ToastManager {
  private toasts: Toast[] = [];
  private listeners: Set<ToastListener> = new Set();
  private toastCounter = 0;
  private timeouts: Map<string, NodeJS.Timeout> = new Map();

  /**
   * Show a toast notification
   */
  show(type: ToastType, message: string, duration: number = 5000): string {
    const id = `toast-${Date.now()}-${this.toastCounter++}`;
    const toast: Toast = { id, type, message, duration };

    this.toasts.push(toast);
    this.notifyListeners();

    // Auto-dismiss after duration
    if (duration > 0) {
      const timeoutId = setTimeout(() => {
        this.dismiss(id);
        this.timeouts.delete(id);
      }, duration);
      this.timeouts.set(id, timeoutId);
    }

    console.log(`[Toast] ${type.toUpperCase()}: ${message}`);
    return id;
  }

  /**
   * Dismiss a toast by ID
   */
  dismiss(id: string): void {
    // Clear the timeout if it exists
    const timeoutId = this.timeouts.get(id);
    if (timeoutId) {
      clearTimeout(timeoutId);
      this.timeouts.delete(id);
    }

    this.toasts = this.toasts.filter((t) => t.id !== id);
    this.notifyListeners();
  }

  /**
   * Dismiss all toasts
   */
  dismissAll(): void {
    // Clear all timeouts
    this.timeouts.forEach((timeoutId) => clearTimeout(timeoutId));
    this.timeouts.clear();

    this.toasts = [];
    this.notifyListeners();
  }

  /**
   * Get all active toasts
   */
  getToasts(): Toast[] {
    return [...this.toasts];
  }

  /**
   * Subscribe to toast changes
   */
  subscribe(listener: ToastListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notifyListeners(): void {
    this.listeners.forEach((listener) => listener([...this.toasts]));
  }
}

// Singleton instance
const toastManager = new ToastManager();

/**
 * Default durations by toast type (in milliseconds)
 * - success: Quick confirmations don't need to linger (3s)
 * - info: Standard informational messages (5s)
 * - warning: Important messages that need attention (6s)
 * - error: Critical messages users need time to read (8s)
 */
const DEFAULT_DURATIONS: Record<ToastType, number> = {
  success: 3000,
  info: 5000,
  warning: 6000,
  error: 8000,
};

/**
 * Show a success toast
 */
export function showSuccess(message: string, duration?: number): string {
  return toastManager.show('success', message, duration ?? DEFAULT_DURATIONS.success);
}

/**
 * Show an error toast
 */
export function showError(message: string, duration?: number): string {
  return toastManager.show('error', message, duration ?? DEFAULT_DURATIONS.error);
}

/**
 * Show a warning toast
 */
export function showWarning(message: string, duration?: number): string {
  return toastManager.show('warning', message, duration ?? DEFAULT_DURATIONS.warning);
}

/**
 * Show an info toast
 */
export function showInfo(message: string, duration?: number): string {
  return toastManager.show('info', message, duration ?? DEFAULT_DURATIONS.info);
}

/**
 * Dismiss a toast
 */
export function dismissToast(id: string): void {
  toastManager.dismiss(id);
}

/**
 * Get toast manager instance
 */
export function getToastManager(): ToastManager {
  return toastManager;
}
