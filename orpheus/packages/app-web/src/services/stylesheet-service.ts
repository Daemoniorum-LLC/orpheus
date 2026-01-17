/**
 * Stylesheet Service - Notation styling and layout customization
 * Provides Guitar Pro 8-style stylesheet/layout customization
 */

import * as alphaTab from '@coderline/alphatab';

/**
 * Font configuration for notation elements
 */
export interface NotationFont {
  family: string;
  size: number;
  style: 'normal' | 'italic' | 'bold' | 'bold-italic';
}

/**
 * Color scheme for notation rendering
 */
export interface NotationColors {
  /** Primary text and notation color */
  primary: string;
  /** Secondary/accent color (e.g., lyrics, annotations) */
  secondary: string;
  /** Background color */
  background: string;
  /** Bar lines and staff lines */
  staffLines: string;
  /** Cursor/playhead color */
  cursor: string;
  /** Selection highlight */
  selection: string;
  /** In-tune indicator */
  inTune: string;
  /** Flat indicator */
  flat: string;
  /** Sharp indicator */
  sharp: string;
}

/**
 * Layout and spacing configuration
 */
export interface NotationLayout {
  /** Scale factor (0.5 to 2.0) */
  scale: number;
  /** Space between staves in pixels */
  staveSpacing: number;
  /** Space between systems (groups of staves) */
  systemSpacing: number;
  /** Left margin in pixels */
  marginLeft: number;
  /** Right margin in pixels */
  marginRight: number;
  /** Top margin in pixels */
  marginTop: number;
  /** Bottom margin in pixels */
  marginBottom: number;
  /** Bar padding */
  barPadding: number;
  /** Layout mode */
  layoutMode: 'page' | 'horizontal';
  /** Stave profile */
  staveProfile: 'default' | 'tab' | 'score' | 'tab-mixed' | 'score-tab';
}

/**
 * Display options for various notation elements
 */
export interface NotationDisplay {
  /** Show standard notation */
  showStandardNotation: boolean;
  /** Show tablature */
  showTablature: boolean;
  /** Show chord diagrams */
  showChordDiagrams: boolean;
  /** Show lyrics */
  showLyrics: boolean;
  /** Show fingering */
  showFingering: boolean;
  /** Show dynamics */
  showDynamics: boolean;
  /** Show tempo markings */
  showTempoMarkings: boolean;
  /** Show bar numbers */
  showBarNumbers: boolean;
  /** Show track names */
  showTrackNames: boolean;
  /** Show time signatures */
  showTimeSignatures: boolean;
  /** Show key signatures */
  showKeySignatures: boolean;
  /** Show copyright */
  showCopyright: boolean;
  /** Show rhythm notation on tab */
  showRhythmOnTab: boolean;
  /** Show effects (bends, slides, etc.) */
  showEffects: boolean;
  /** Notation mode */
  notationMode: 'guitar-pro' | 'songbook' | 'classic';
}

/**
 * Complete stylesheet configuration
 */
export interface NotationStylesheet {
  id: string;
  name: string;
  description?: string;
  fonts: {
    title: NotationFont;
    subtitle: NotationFont;
    lyrics: NotationFont;
    chord: NotationFont;
    fret: NotationFont;
    barNumber: NotationFont;
    tempo: NotationFont;
    tablature: NotationFont;
  };
  colors: NotationColors;
  layout: NotationLayout;
  display: NotationDisplay;
  isBuiltIn: boolean;
  createdAt?: string;
  updatedAt?: string;
}

/**
 * Default dark theme colors
 */
const DARK_COLORS: NotationColors = {
  primary: '#E0E0E0',
  secondary: '#9E9E9E',
  background: '#1E1E1E',
  staffLines: '#424242',
  cursor: '#64B5F6',
  selection: 'rgba(100, 181, 246, 0.3)',
  inTune: '#4CAF50',
  flat: '#F44336',
  sharp: '#FFEB3B',
};

/**
 * Default light theme colors
 */
const LIGHT_COLORS: NotationColors = {
  primary: '#212121',
  secondary: '#757575',
  background: '#FFFFFF',
  staffLines: '#9E9E9E',
  cursor: '#1976D2',
  selection: 'rgba(25, 118, 210, 0.2)',
  inTune: '#2E7D32',
  flat: '#C62828',
  sharp: '#F57F17',
};

/**
 * Sepia theme colors (for print/classic look)
 */
const SEPIA_COLORS: NotationColors = {
  primary: '#3E2723',
  secondary: '#5D4037',
  background: '#F5F0E1',
  staffLines: '#8D6E63',
  cursor: '#795548',
  selection: 'rgba(121, 85, 72, 0.2)',
  inTune: '#558B2F',
  flat: '#BF360C',
  sharp: '#EF6C00',
};

/**
 * Default fonts
 */
const DEFAULT_FONTS: NotationStylesheet['fonts'] = {
  title: { family: 'Georgia, serif', size: 24, style: 'bold' },
  subtitle: { family: 'Georgia, serif', size: 16, style: 'italic' },
  lyrics: { family: 'Times New Roman, serif', size: 11, style: 'normal' },
  chord: { family: 'Arial, sans-serif', size: 12, style: 'bold' },
  fret: { family: 'Arial, sans-serif', size: 13, style: 'normal' },
  barNumber: { family: 'Arial, sans-serif', size: 9, style: 'normal' },
  tempo: { family: 'Arial, sans-serif', size: 10, style: 'bold' },
  tablature: { family: 'Roboto Mono, monospace', size: 12, style: 'normal' },
};

/**
 * Default layout configuration
 */
const DEFAULT_LAYOUT: NotationLayout = {
  scale: 1.0,
  staveSpacing: 10,
  systemSpacing: 20,
  marginLeft: 40,
  marginRight: 40,
  marginTop: 40,
  marginBottom: 40,
  barPadding: 3,
  layoutMode: 'page',
  staveProfile: 'tab-mixed',
};

/**
 * Default display options
 */
const DEFAULT_DISPLAY: NotationDisplay = {
  showStandardNotation: true,
  showTablature: true,
  showChordDiagrams: true,
  showLyrics: true,
  showFingering: true,
  showDynamics: true,
  showTempoMarkings: true,
  showBarNumbers: true,
  showTrackNames: true,
  showTimeSignatures: true,
  showKeySignatures: true,
  showCopyright: true,
  showRhythmOnTab: true,
  showEffects: true,
  notationMode: 'guitar-pro',
};

/**
 * Built-in stylesheet presets
 */
export const BUILT_IN_STYLESHEETS: NotationStylesheet[] = [
  {
    id: 'default-dark',
    name: 'Default Dark',
    description: 'Standard dark theme with full notation display',
    fonts: DEFAULT_FONTS,
    colors: DARK_COLORS,
    layout: DEFAULT_LAYOUT,
    display: DEFAULT_DISPLAY,
    isBuiltIn: true,
  },
  {
    id: 'default-light',
    name: 'Default Light',
    description: 'Standard light theme for bright environments',
    fonts: DEFAULT_FONTS,
    colors: LIGHT_COLORS,
    layout: DEFAULT_LAYOUT,
    display: DEFAULT_DISPLAY,
    isBuiltIn: true,
  },
  {
    id: 'print-ready',
    name: 'Print Ready',
    description: 'High contrast theme optimized for printing',
    fonts: {
      ...DEFAULT_FONTS,
      title: { family: 'Georgia, serif', size: 28, style: 'bold' },
      fret: { family: 'Arial, sans-serif', size: 14, style: 'bold' },
    },
    colors: {
      ...LIGHT_COLORS,
      primary: '#000000',
      staffLines: '#000000',
    },
    layout: {
      ...DEFAULT_LAYOUT,
      scale: 1.1,
      marginLeft: 60,
      marginRight: 60,
      marginTop: 60,
      marginBottom: 60,
    },
    display: {
      ...DEFAULT_DISPLAY,
      showCopyright: true,
    },
    isBuiltIn: true,
  },
  {
    id: 'tab-only',
    name: 'Tablature Only',
    description: 'Clean tablature view without standard notation',
    fonts: DEFAULT_FONTS,
    colors: DARK_COLORS,
    layout: {
      ...DEFAULT_LAYOUT,
      staveProfile: 'tab',
    },
    display: {
      ...DEFAULT_DISPLAY,
      showStandardNotation: false,
      showTablature: true,
      showRhythmOnTab: true,
    },
    isBuiltIn: true,
  },
  {
    id: 'score-only',
    name: 'Standard Notation Only',
    description: 'Classical music notation without tablature',
    fonts: DEFAULT_FONTS,
    colors: DARK_COLORS,
    layout: {
      ...DEFAULT_LAYOUT,
      staveProfile: 'score',
    },
    display: {
      ...DEFAULT_DISPLAY,
      showStandardNotation: true,
      showTablature: false,
    },
    isBuiltIn: true,
  },
  {
    id: 'compact',
    name: 'Compact',
    description: 'Dense layout for maximum content visibility',
    fonts: {
      ...DEFAULT_FONTS,
      fret: { family: 'Arial, sans-serif', size: 11, style: 'normal' },
    },
    colors: DARK_COLORS,
    layout: {
      ...DEFAULT_LAYOUT,
      scale: 0.85,
      staveSpacing: 6,
      systemSpacing: 12,
      marginLeft: 20,
      marginRight: 20,
      marginTop: 20,
      marginBottom: 20,
    },
    display: {
      ...DEFAULT_DISPLAY,
      showChordDiagrams: false,
    },
    isBuiltIn: true,
  },
  {
    id: 'songbook',
    name: 'Songbook Style',
    description: 'Clean, simple layout for chord charts and lyrics',
    fonts: {
      ...DEFAULT_FONTS,
      lyrics: { family: 'Georgia, serif', size: 13, style: 'normal' },
      chord: { family: 'Arial, sans-serif', size: 14, style: 'bold' },
    },
    colors: SEPIA_COLORS,
    layout: {
      ...DEFAULT_LAYOUT,
      scale: 1.1,
    },
    display: {
      ...DEFAULT_DISPLAY,
      showStandardNotation: false,
      showTablature: true,
      showChordDiagrams: true,
      showLyrics: true,
      showDynamics: false,
      notationMode: 'songbook',
    },
    isBuiltIn: true,
  },
  {
    id: 'performance',
    name: 'Performance Mode',
    description: 'Large, easy-to-read notation for live performance',
    fonts: {
      ...DEFAULT_FONTS,
      fret: { family: 'Arial, sans-serif', size: 18, style: 'bold' },
      chord: { family: 'Arial, sans-serif', size: 16, style: 'bold' },
    },
    colors: {
      ...DARK_COLORS,
      background: '#000000',
      primary: '#FFFFFF',
    },
    layout: {
      ...DEFAULT_LAYOUT,
      scale: 1.4,
      layoutMode: 'horizontal',
      staveSpacing: 20,
    },
    display: {
      ...DEFAULT_DISPLAY,
      showBarNumbers: true,
      showChordDiagrams: false,
      showLyrics: false,
    },
    isBuiltIn: true,
  },
];

/**
 * Storage key for user stylesheets
 */
const STORAGE_KEY = 'orpheus:stylesheets';
const ACTIVE_STYLESHEET_KEY = 'orpheus:activeStylesheet';

/**
 * Stylesheet Service - Manages notation styling and layout
 */
export class StylesheetService {
  private stylesheets: Map<string, NotationStylesheet> = new Map();
  private activeStylesheetId: string = 'default-dark';
  private listeners: Array<(stylesheet: NotationStylesheet) => void> = [];

  constructor() {
    this.loadBuiltInStylesheets();
    this.loadUserStylesheets();
    this.loadActiveStylesheet();
  }

  /**
   * Load built-in stylesheets
   */
  private loadBuiltInStylesheets(): void {
    for (const stylesheet of BUILT_IN_STYLESHEETS) {
      this.stylesheets.set(stylesheet.id, stylesheet);
    }
  }

  /**
   * Load user-created stylesheets from localStorage
   */
  private loadUserStylesheets(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const userStylesheets: NotationStylesheet[] = JSON.parse(stored);
        for (const stylesheet of userStylesheets) {
          if (!stylesheet.isBuiltIn) {
            this.stylesheets.set(stylesheet.id, stylesheet);
          }
        }
      }
    } catch (error) {
      console.warn('[StylesheetService] Failed to load user stylesheets:', error);
    }
  }

  /**
   * Load active stylesheet from localStorage
   */
  private loadActiveStylesheet(): void {
    try {
      const stored = localStorage.getItem(ACTIVE_STYLESHEET_KEY);
      if (stored && this.stylesheets.has(stored)) {
        this.activeStylesheetId = stored;
      }
    } catch (error) {
      console.warn('[StylesheetService] Failed to load active stylesheet:', error);
    }
  }

  /**
   * Save user stylesheets to localStorage
   */
  private saveUserStylesheets(): void {
    const userStylesheets = Array.from(this.stylesheets.values()).filter(
      (s) => !s.isBuiltIn
    );
    localStorage.setItem(STORAGE_KEY, JSON.stringify(userStylesheets));
  }

  /**
   * Save active stylesheet to localStorage
   */
  private saveActiveStylesheet(): void {
    localStorage.setItem(ACTIVE_STYLESHEET_KEY, this.activeStylesheetId);
  }

  /**
   * Get all available stylesheets
   */
  getAllStylesheets(): NotationStylesheet[] {
    return Array.from(this.stylesheets.values());
  }

  /**
   * Get built-in stylesheets only
   */
  getBuiltInStylesheets(): NotationStylesheet[] {
    return BUILT_IN_STYLESHEETS;
  }

  /**
   * Get user-created stylesheets
   */
  getUserStylesheets(): NotationStylesheet[] {
    return Array.from(this.stylesheets.values()).filter((s) => !s.isBuiltIn);
  }

  /**
   * Get a stylesheet by ID
   */
  getStylesheet(id: string): NotationStylesheet | undefined {
    return this.stylesheets.get(id);
  }

  /**
   * Get the currently active stylesheet
   */
  getActiveStylesheet(): NotationStylesheet {
    return this.stylesheets.get(this.activeStylesheetId) || BUILT_IN_STYLESHEETS[0];
  }

  /**
   * Set the active stylesheet
   */
  setActiveStylesheet(id: string): void {
    if (this.stylesheets.has(id)) {
      this.activeStylesheetId = id;
      this.saveActiveStylesheet();
      this.notifyListeners();
    }
  }

  /**
   * Create a new user stylesheet
   */
  createStylesheet(stylesheet: Omit<NotationStylesheet, 'id' | 'isBuiltIn'>): NotationStylesheet {
    const id = `user-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
    const newStylesheet: NotationStylesheet = {
      ...stylesheet,
      id,
      isBuiltIn: false,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    this.stylesheets.set(id, newStylesheet);
    this.saveUserStylesheets();

    return newStylesheet;
  }

  /**
   * Update an existing user stylesheet
   */
  updateStylesheet(id: string, updates: Partial<NotationStylesheet>): NotationStylesheet | null {
    const existing = this.stylesheets.get(id);
    if (!existing || existing.isBuiltIn) {
      return null;
    }

    const updated: NotationStylesheet = {
      ...existing,
      ...updates,
      id: existing.id,
      isBuiltIn: false,
      updatedAt: new Date().toISOString(),
    };

    this.stylesheets.set(id, updated);
    this.saveUserStylesheets();

    if (id === this.activeStylesheetId) {
      this.notifyListeners();
    }

    return updated;
  }

  /**
   * Delete a user stylesheet
   */
  deleteStylesheet(id: string): boolean {
    const stylesheet = this.stylesheets.get(id);
    if (!stylesheet || stylesheet.isBuiltIn) {
      return false;
    }

    this.stylesheets.delete(id);
    this.saveUserStylesheets();

    // Reset to default if deleting active stylesheet
    if (id === this.activeStylesheetId) {
      this.activeStylesheetId = 'default-dark';
      this.saveActiveStylesheet();
      this.notifyListeners();
    }

    return true;
  }

  /**
   * Duplicate a stylesheet
   */
  duplicateStylesheet(id: string, newName?: string): NotationStylesheet | null {
    const source = this.stylesheets.get(id);
    if (!source) {
      return null;
    }

    return this.createStylesheet({
      ...source,
      name: newName || `${source.name} (Copy)`,
      description: source.description,
    });
  }

  /**
   * Convert stylesheet to alphaTab Settings
   */
  toAlphaTabSettings(stylesheet: NotationStylesheet): alphaTab.Settings {
    const settings = new alphaTab.Settings();

    // Core settings
    settings.core.engine = 'svg';
    settings.core.logLevel = alphaTab.LogLevel.Warning;

    // Display settings
    settings.display.scale = stylesheet.layout.scale;
    settings.display.layoutMode =
      stylesheet.layout.layoutMode === 'horizontal'
        ? alphaTab.LayoutMode.Horizontal
        : alphaTab.LayoutMode.Page;

    // Stave profile
    switch (stylesheet.layout.staveProfile) {
      case 'tab':
        settings.display.staveProfile = alphaTab.StaveProfile.Tab;
        break;
      case 'score':
        settings.display.staveProfile = alphaTab.StaveProfile.Score;
        break;
      case 'tab-mixed':
        settings.display.staveProfile = alphaTab.StaveProfile.TabMixed;
        break;
      case 'score-tab':
        settings.display.staveProfile = alphaTab.StaveProfile.ScoreTab;
        break;
      default:
        settings.display.staveProfile = alphaTab.StaveProfile.Default;
    }

    // Notation mode
    switch (stylesheet.display.notationMode) {
      case 'songbook':
        settings.notation.notationMode = alphaTab.NotationMode.SongBook;
        break;
      case 'classic':
        settings.notation.notationMode = alphaTab.NotationMode.GuitarPro;
        break;
      default:
        settings.notation.notationMode = alphaTab.NotationMode.GuitarPro;
    }

    // Display elements
    settings.notation.elements.set(
      alphaTab.NotationElement.ScoreTitle,
      stylesheet.display.showTrackNames
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.ScoreSubTitle,
      stylesheet.display.showTrackNames
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.ScoreArtist,
      stylesheet.display.showTrackNames
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.ScoreAlbum,
      stylesheet.display.showTrackNames
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.ScoreWords,
      stylesheet.display.showLyrics
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.ScoreMusic,
      stylesheet.display.showTrackNames
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.ScoreCopyright,
      stylesheet.display.showCopyright
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.EffectTempo,
      stylesheet.display.showTempoMarkings
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.EffectDynamics,
      stylesheet.display.showDynamics
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.GuitarTuning,
      stylesheet.display.showTrackNames
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.TrackNames,
      stylesheet.display.showTrackNames
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.ChordDiagrams,
      stylesheet.display.showChordDiagrams
    );
    settings.notation.elements.set(
      alphaTab.NotationElement.EffectFingering,
      stylesheet.display.showFingering
    );

    // Rhythm slashes on tab staff
    settings.notation.rhythmMode = stylesheet.display.showRhythmOnTab
      ? alphaTab.TabRhythmMode.ShowWithBeams
      : alphaTab.TabRhythmMode.Hidden;

    // Player settings (always enabled)
    settings.player.enablePlayer = true;
    settings.player.enableCursor = true;
    settings.player.enableUserInteraction = true;

    return settings;
  }

  /**
   * Generate CSS variables for a stylesheet
   */
  toCssVariables(stylesheet: NotationStylesheet): Record<string, string> {
    return {
      '--notation-primary': stylesheet.colors.primary,
      '--notation-secondary': stylesheet.colors.secondary,
      '--notation-background': stylesheet.colors.background,
      '--notation-staff-lines': stylesheet.colors.staffLines,
      '--notation-cursor': stylesheet.colors.cursor,
      '--notation-selection': stylesheet.colors.selection,
      '--notation-in-tune': stylesheet.colors.inTune,
      '--notation-flat': stylesheet.colors.flat,
      '--notation-sharp': stylesheet.colors.sharp,
      '--notation-scale': String(stylesheet.layout.scale),
      '--notation-font-title': stylesheet.fonts.title.family,
      '--notation-font-title-size': `${stylesheet.fonts.title.size}px`,
      '--notation-font-lyrics': stylesheet.fonts.lyrics.family,
      '--notation-font-lyrics-size': `${stylesheet.fonts.lyrics.size}px`,
      '--notation-font-chord': stylesheet.fonts.chord.family,
      '--notation-font-chord-size': `${stylesheet.fonts.chord.size}px`,
      '--notation-font-fret': stylesheet.fonts.fret.family,
      '--notation-font-fret-size': `${stylesheet.fonts.fret.size}px`,
      '--notation-font-tab': stylesheet.fonts.tablature.family,
      '--notation-font-tab-size': `${stylesheet.fonts.tablature.size}px`,
      '--notation-margin-left': `${stylesheet.layout.marginLeft}px`,
      '--notation-margin-right': `${stylesheet.layout.marginRight}px`,
      '--notation-margin-top': `${stylesheet.layout.marginTop}px`,
      '--notation-margin-bottom': `${stylesheet.layout.marginBottom}px`,
      '--notation-stave-spacing': `${stylesheet.layout.staveSpacing}px`,
      '--notation-system-spacing': `${stylesheet.layout.systemSpacing}px`,
    };
  }

  /**
   * Subscribe to stylesheet changes
   */
  subscribe(callback: (stylesheet: NotationStylesheet) => void): () => void {
    this.listeners.push(callback);
    return () => {
      this.listeners = this.listeners.filter((cb) => cb !== callback);
    };
  }

  /**
   * Notify listeners of stylesheet change
   */
  private notifyListeners(): void {
    const stylesheet = this.getActiveStylesheet();
    this.listeners.forEach((cb) => cb(stylesheet));
  }

  /**
   * Export stylesheet as JSON
   */
  exportStylesheet(id: string): string | null {
    const stylesheet = this.stylesheets.get(id);
    if (!stylesheet) return null;
    return JSON.stringify(stylesheet, null, 2);
  }

  /**
   * Import stylesheet from JSON
   */
  importStylesheet(json: string): NotationStylesheet | null {
    try {
      const parsed = JSON.parse(json) as NotationStylesheet;
      // Validate required fields
      if (!parsed.name || !parsed.fonts || !parsed.colors || !parsed.layout || !parsed.display) {
        throw new Error('Invalid stylesheet format');
      }
      return this.createStylesheet(parsed);
    } catch (error) {
      console.error('[StylesheetService] Failed to import stylesheet:', error);
      return null;
    }
  }
}

// Singleton instance
let stylesheetInstance: StylesheetService | null = null;

export function getStylesheetService(): StylesheetService {
  if (!stylesheetInstance) {
    stylesheetInstance = new StylesheetService();
  }
  return stylesheetInstance;
}
