/**
 * Hook to restore focus to the trigger element when a modal closes
 * This ensures keyboard users don't lose their place after dismissing a dialog
 */

import { useEffect, useRef, useCallback } from 'react';

/**
 * Returns a ref to track the element that triggered a modal,
 * and restores focus when the modal closes
 */
export function useFocusRestore(isOpen: boolean) {
  const triggerRef = useRef<HTMLElement | null>(null);

  // Capture the currently focused element when opening
  useEffect(() => {
    if (isOpen) {
      triggerRef.current = document.activeElement as HTMLElement;
    }
  }, [isOpen]);

  // Restore focus when closing
  useEffect(() => {
    if (!isOpen && triggerRef.current) {
      // Small delay to ensure modal is fully unmounted
      const timeoutId = setTimeout(() => {
        if (triggerRef.current && document.body.contains(triggerRef.current)) {
          triggerRef.current.focus();
        }
        triggerRef.current = null;
      }, 0);

      return () => clearTimeout(timeoutId);
    }
  }, [isOpen]);

  // Manual focus restore callback for imperative use
  const restoreFocus = useCallback(() => {
    if (triggerRef.current && document.body.contains(triggerRef.current)) {
      triggerRef.current.focus();
    }
    triggerRef.current = null;
  }, []);

  return { restoreFocus };
}
