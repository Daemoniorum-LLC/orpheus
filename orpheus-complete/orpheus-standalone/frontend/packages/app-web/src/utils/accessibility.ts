/**
 * Accessibility Utilities
 * ARIA labels, keyboard navigation, and screen reader support
 */

/**
 * Standard keyboard event handlers
 */
export const keyboardHandlers = {
  /**
   * Handle Enter or Space key for button-like elements
   */
  activateOnEnterOrSpace: (callback: () => void) => (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      callback();
    }
  },

  /**
   * Handle Escape key to close dialogs/modals
   */
  closeOnEscape: (callback: () => void) => (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      callback();
    }
  },

  /**
   * Handle arrow key navigation in lists
   */
  navigateList: (options: {
    items: any[];
    currentIndex: number;
    onIndexChange: (index: number) => void;
    onSelect?: () => void;
  }) => (e: React.KeyboardEvent) => {
    const { items, currentIndex, onIndexChange, onSelect } = options;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        onIndexChange(Math.min(currentIndex + 1, items.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        onIndexChange(Math.max(currentIndex - 1, 0));
        break;
      case 'Home':
        e.preventDefault();
        onIndexChange(0);
        break;
      case 'End':
        e.preventDefault();
        onIndexChange(items.length - 1);
        break;
      case 'Enter':
        e.preventDefault();
        onSelect?.();
        break;
    }
  },
};

/**
 * ARIA labels for common UI patterns
 */
export const ariaLabels = {
  // Toolbar actions
  openFile: 'Open project or Guitar Pro file',
  saveProject: 'Save current project',
  exportProject: 'Export project in multiple formats',
  undo: 'Undo last action',
  redo: 'Redo last undone action',
  play: 'Play audio',
  pause: 'Pause audio',
  stop: 'Stop playback',
  aiAssistant: 'Open AI assistant panel',
  keyboardShortcuts: 'View keyboard shortcuts',

  // Mode selector
  composeMode: 'Switch to Compose mode - Tab editor with Guitar Pro support',
  recordMode: 'Switch to Record mode - Multi-track audio recording',
  mixMode: 'Switch to Mix mode - Professional mixing console',
  masterMode: 'Switch to Master mode - AI-powered mastering',
  practiceMode: 'Switch to Practice mode - Speed trainer and loops',
  distributeMode: 'Switch to Distribute mode - Upload to streaming platforms',

  // Dialogs
  closeDialog: 'Close dialog',
  confirmAction: 'Confirm action',
  cancelAction: 'Cancel action',

  // Export
  selectFormat: (format: string) => `Select ${format} export format`,
  selectPreset: (preset: string) => `Select ${preset} export preset`,

  // AI Assistant
  sendMessage: 'Send message to AI assistant',
  clearChat: 'Clear chat history',
  quickAction: (action: string) => `Quick action: ${action}`,
};

/**
 * Screen reader announcements (using aria-live)
 */
export class ScreenReaderAnnouncer {
  private static container: HTMLDivElement | null = null;

  static initialize() {
    if (this.container) return;

    // Create hidden aria-live region
    this.container = document.createElement('div');
    this.container.setAttribute('role', 'status');
    this.container.setAttribute('aria-live', 'polite');
    this.container.setAttribute('aria-atomic', 'true');
    this.container.style.position = 'absolute';
    this.container.style.left = '-10000px';
    this.container.style.width = '1px';
    this.container.style.height = '1px';
    this.container.style.overflow = 'hidden';

    document.body.appendChild(this.container);
  }

  static announce(message: string, priority: 'polite' | 'assertive' = 'polite') {
    if (!this.container) {
      this.initialize();
    }

    if (this.container) {
      this.container.setAttribute('aria-live', priority);
      this.container.textContent = message;

      // Clear after announcement
      setTimeout(() => {
        if (this.container) {
          this.container.textContent = '';
        }
      }, 1000);
    }
  }
}

/**
 * Focus management utilities
 */
export const focusManagement = {
  /**
   * Trap focus within an element (for modals/dialogs)
   */
  trapFocus: (element: HTMLElement, onEscape?: () => void) => {
    const focusableSelector =
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
    const focusableElements = element.querySelectorAll<HTMLElement>(focusableSelector);
    const firstFocusable = focusableElements[0];
    const lastFocusable = focusableElements[focusableElements.length - 1];

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && onEscape) {
        e.preventDefault();
        onEscape();
        return;
      }

      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        // Shift + Tab
        if (document.activeElement === firstFocusable) {
          e.preventDefault();
          lastFocusable?.focus();
        }
      } else {
        // Tab
        if (document.activeElement === lastFocusable) {
          e.preventDefault();
          firstFocusable?.focus();
        }
      }
    };

    element.addEventListener('keydown', handleKeyDown);

    // Focus first element
    firstFocusable?.focus();

    // Return cleanup function
    return () => {
      element.removeEventListener('keydown', handleKeyDown);
    };
  },

  /**
   * Return focus to previously focused element
   */
  returnFocus: (previousElement: HTMLElement | null) => {
    if (previousElement && document.contains(previousElement)) {
      previousElement.focus();
    }
  },
};

/**
 * High contrast mode detection
 */
export function isHighContrastMode(): boolean {
  // Check for Windows high contrast mode
  if (window.matchMedia) {
    return (
      window.matchMedia('(prefers-contrast: high)').matches ||
      window.matchMedia('(-ms-high-contrast: active)').matches
    );
  }
  return false;
}

/**
 * Reduced motion preference detection
 */
export function prefersReducedMotion(): boolean {
  if (window.matchMedia) {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  }
  return false;
}

/**
 * Skip to content link (for keyboard navigation)
 */
export function createSkipLink(targetId: string, label: string = 'Skip to main content'): void {
  const skipLink = document.createElement('a');
  skipLink.href = `#${targetId}`;
  skipLink.textContent = label;
  skipLink.className = 'skip-link';
  skipLink.style.position = 'absolute';
  skipLink.style.left = '-10000px';
  skipLink.style.width = '1px';
  skipLink.style.height = '1px';
  skipLink.style.overflow = 'hidden';

  // Show on focus
  skipLink.addEventListener('focus', () => {
    skipLink.style.position = 'fixed';
    skipLink.style.top = '10px';
    skipLink.style.left = '10px';
    skipLink.style.width = 'auto';
    skipLink.style.height = 'auto';
    skipLink.style.padding = '8px 16px';
    skipLink.style.backgroundColor = '#000';
    skipLink.style.color = '#fff';
    skipLink.style.textDecoration = 'none';
    skipLink.style.zIndex = '10000';
  });

  skipLink.addEventListener('blur', () => {
    skipLink.style.position = 'absolute';
    skipLink.style.left = '-10000px';
    skipLink.style.width = '1px';
    skipLink.style.height = '1px';
    skipLink.style.padding = '0';
  });

  document.body.insertBefore(skipLink, document.body.firstChild);
}
