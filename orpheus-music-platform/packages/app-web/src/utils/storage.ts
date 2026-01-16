/**
 * Safe localStorage wrapper with fallback to in-memory storage
 * Handles cases where localStorage is disabled (privacy mode, storage quota exceeded, etc.)
 */

// In-memory fallback storage
const memoryStorage: Map<string, string> = new Map();

// Check if localStorage is available
function isLocalStorageAvailable(): boolean {
  try {
    const testKey = '__storage_test__';
    localStorage.setItem(testKey, testKey);
    localStorage.removeItem(testKey);
    return true;
  } catch {
    return false;
  }
}

// Cache the result to avoid repeated checks
let storageAvailable: boolean | null = null;

function checkStorage(): boolean {
  if (storageAvailable === null) {
    storageAvailable = isLocalStorageAvailable();
    if (!storageAvailable) {
      console.warn(
        '[Storage] localStorage is not available. Using in-memory fallback. Data will not persist across sessions.'
      );
    }
  }
  return storageAvailable;
}

/**
 * Safe wrapper for localStorage that falls back to in-memory storage
 */
export const safeStorage = {
  /**
   * Get an item from storage
   */
  getItem(key: string): string | null {
    if (checkStorage()) {
      try {
        return localStorage.getItem(key);
      } catch {
        return memoryStorage.get(key) ?? null;
      }
    }
    return memoryStorage.get(key) ?? null;
  },

  /**
   * Set an item in storage
   */
  setItem(key: string, value: string): void {
    if (checkStorage()) {
      try {
        localStorage.setItem(key, value);
        return;
      } catch (e) {
        // Quota exceeded or other error - fall back to memory
        console.warn('[Storage] localStorage write failed, using memory:', e);
      }
    }
    memoryStorage.set(key, value);
  },

  /**
   * Remove an item from storage
   */
  removeItem(key: string): void {
    if (checkStorage()) {
      try {
        localStorage.removeItem(key);
      } catch {
        // Ignore errors
      }
    }
    memoryStorage.delete(key);
  },

  /**
   * Clear all items from storage (respects Orpheus namespace)
   */
  clear(): void {
    if (checkStorage()) {
      try {
        localStorage.clear();
      } catch {
        // Ignore errors
      }
    }
    memoryStorage.clear();
  },

  /**
   * Get all keys matching a prefix
   */
  getKeys(prefix?: string): string[] {
    const keys: string[] = [];

    if (checkStorage()) {
      try {
        for (let i = 0; i < localStorage.length; i++) {
          const key = localStorage.key(i);
          if (key && (!prefix || key.startsWith(prefix))) {
            keys.push(key);
          }
        }
      } catch {
        // Fall through to memory storage
      }
    }

    // Also include memory storage keys
    for (const key of memoryStorage.keys()) {
      if (!prefix || key.startsWith(prefix)) {
        if (!keys.includes(key)) {
          keys.push(key);
        }
      }
    }

    return keys;
  },

  /**
   * Check if localStorage is available
   */
  isAvailable(): boolean {
    return checkStorage();
  },
};

/**
 * Safe wrapper for sessionStorage that falls back to in-memory storage
 */
const sessionMemoryStorage: Map<string, string> = new Map();

function isSessionStorageAvailable(): boolean {
  try {
    const testKey = '__session_test__';
    sessionStorage.setItem(testKey, testKey);
    sessionStorage.removeItem(testKey);
    return true;
  } catch {
    return false;
  }
}

let sessionStorageAvailable: boolean | null = null;

function checkSessionStorage(): boolean {
  if (sessionStorageAvailable === null) {
    sessionStorageAvailable = isSessionStorageAvailable();
  }
  return sessionStorageAvailable;
}

export const safeSessionStorage = {
  getItem(key: string): string | null {
    if (checkSessionStorage()) {
      try {
        return sessionStorage.getItem(key);
      } catch {
        return sessionMemoryStorage.get(key) ?? null;
      }
    }
    return sessionMemoryStorage.get(key) ?? null;
  },

  setItem(key: string, value: string): void {
    if (checkSessionStorage()) {
      try {
        sessionStorage.setItem(key, value);
        return;
      } catch {
        // Fall back to memory
      }
    }
    sessionMemoryStorage.set(key, value);
  },

  removeItem(key: string): void {
    if (checkSessionStorage()) {
      try {
        sessionStorage.removeItem(key);
      } catch {
        // Ignore
      }
    }
    sessionMemoryStorage.delete(key);
  },

  getKeys(prefix?: string): string[] {
    const keys: string[] = [];

    if (checkSessionStorage()) {
      try {
        for (let i = 0; i < sessionStorage.length; i++) {
          const key = sessionStorage.key(i);
          if (key && (!prefix || key.startsWith(prefix))) {
            keys.push(key);
          }
        }
      } catch {
        // Fall through
      }
    }

    for (const key of sessionMemoryStorage.keys()) {
      if (!prefix || key.startsWith(prefix)) {
        if (!keys.includes(key)) {
          keys.push(key);
        }
      }
    }

    return keys;
  },
};
