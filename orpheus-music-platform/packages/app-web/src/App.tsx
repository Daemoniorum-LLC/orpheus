/**
 * Maestro AI - Main Application Component
 * The world's first unified music production platform
 */

import { ThemeProvider } from './contexts/ThemeContext';
import { ModeSelector } from './components/ModeSelector';
import { Toolbar } from './components/Toolbar';
import { Sidebar } from './components/Sidebar';
import { StatusBar } from './components/StatusBar';
import { SkeletonLoader } from './components/SkeletonLoader';
import { ErrorBoundary } from './components/ErrorBoundary';
import { CommandPalette } from './components/CommandPalette';
import { useCurrentMode } from './store/app-store';
import { AIAssistant } from './components/AIAssistant';
import { ToastContainer } from './components/ToastContainer';
import { DragDropZone } from './components/DragDropZone';
import { LoadingOverlay } from './components/LoadingOverlay';
import { KeyboardShortcutsDialog } from './components/KeyboardShortcutsDialog';
import { OnboardingDialog } from './components/OnboardingDialog';
import { WhatsNewDialog, useWhatsNew } from './components/WhatsNewDialog';
import { PreferencesDialog } from './components/PreferencesDialog';
import { ConfirmDialog } from './components/ConfirmDialog';
import { lazy, Suspense, useEffect, useState, useRef } from 'react';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { useFocusRestore } from './hooks/useFocusRestore';
import { loadAutoSave, clearAutoSave, getAutoSaveManager, saveProject } from './services/project-save';
import { loadPreferences } from './components/PreferencesDialog';
import { loadSessionState, saveSessionState } from './services/session-persistence';
import { useAppStore } from './store/app-store';
import { showInfo, showError } from './services/toast';
import { sampleProjects } from './data/sample-projects';
import { ScreenReaderAnnouncer, createSkipLink, prefersReducedMotion } from './utils/accessibility';
import { logger } from './utils/logger';
import { safeStorage } from './utils/storage';
import { VoiceCommandOverlay } from './components/VoiceCommandOverlay';
import { useRegisterVoiceCommands } from './hooks/useVoiceCommands';
import { JamPlayer } from './components/JamPlayer';
import { setOnSearchResults, setJamPlayerToggle } from './services/voice-commands-music';

// Lazy load mode components for code splitting
const ComposeMode = lazy(() => import('./modes/ComposeMode').then(m => ({ default: m.ComposeMode })));
const RecordMode = lazy(() => import('./modes/RecordMode').then(m => ({ default: m.RecordMode })));
const MixMode = lazy(() => import('./modes/MixMode').then(m => ({ default: m.MixMode })));
const MasterMode = lazy(() => import('./modes/MasterMode').then(m => ({ default: m.MasterMode })));
const PracticeMode = lazy(() => import('./modes/PracticeMode').then(m => ({ default: m.PracticeMode })));
const DistributeMode = lazy(() => import('./modes/DistributeMode').then(m => ({ default: m.DistributeMode })));

// Preload functions for mode components (can be called on hover or in background)
export const preloadModes = {
  compose: () => import('./modes/ComposeMode'),
  record: () => import('./modes/RecordMode'),
  mix: () => import('./modes/MixMode'),
  master: () => import('./modes/MasterMode'),
  practice: () => import('./modes/PracticeMode'),
  distribute: () => import('./modes/DistributeMode'),
  // Preload all modes (for eager loading after initial render)
  all: () => Promise.all([
    import('./modes/ComposeMode'),
    import('./modes/RecordMode'),
    import('./modes/MixMode'),
    import('./modes/MasterMode'),
    import('./modes/PracticeMode'),
    import('./modes/DistributeMode'),
  ]),
  // Preload adjacent modes based on workflow (compose → record → mix → master)
  workflow: (currentMode: string) => {
    const adjacent: Record<string, string[]> = {
      compose: ['record', 'practice'],
      record: ['compose', 'mix'],
      mix: ['record', 'master'],
      master: ['mix', 'distribute'],
      practice: ['compose'],
      distribute: ['master'],
    };
    const toPreload = adjacent[currentMode] || [];
    return Promise.all(toPreload.map(m => preloadModes[m as keyof typeof preloadModes]?.()));
  },
};

/**
 * Mode-specific error fallback - allows recovery without losing other app state
 */
function ModeErrorFallback({ modeName, onRetry }: { modeName: string; onRetry: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center h-full p-8 text-center">
      <div className="text-4xl mb-4">⚠️</div>
      <h2 className="text-xl font-semibold mb-2">{modeName} Mode Crashed</h2>
      <p className="text-muted-foreground mb-6 max-w-md">
        Something went wrong in this mode. Your project data is safe.
        Try switching to another mode or reload this one.
      </p>
      <div className="flex gap-3">
        <button
          onClick={onRetry}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg font-medium hover:bg-primary/90 transition-colors"
        >
          Try Again
        </button>
        <button
          onClick={() => window.location.reload()}
          className="px-4 py-2 bg-secondary text-secondary-foreground rounded-lg font-medium hover:bg-secondary/80 transition-colors"
        >
          Reload Page
        </button>
      </div>
    </div>
  );
}


function App() {
  const mode = useCurrentMode();
  const { project, setProject, projectModified, sidebarOpen, zoomLevel, selectedTrackId, aiAssistantOpen } = useAppStore();
  const [keyboardShortcutsOpen, setKeyboardShortcutsOpen] = useState(false);
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [whatsNewOpen, setWhatsNewOpen] = useState(false);
  const [preferencesOpen, setPreferencesOpen] = useState(false);
  const [recoveryDialogOpen, setRecoveryDialogOpen] = useState(false);
  const [recoveryMinutesAgo, setRecoveryMinutesAgo] = useState(0);
  const [jamPlayerOpen, setJamPlayerOpen] = useState(false);
  const pendingAutoSaveRef = useRef<{ project: any; timestamp: string } | null>(null);
  const { shouldShowWhatsNew } = useWhatsNew();

  // Focus restoration for dialogs
  useFocusRestore(keyboardShortcutsOpen);
  useFocusRestore(onboardingOpen);
  useFocusRestore(whatsNewOpen);
  useFocusRestore(preferencesOpen);
  useFocusRestore(commandPaletteOpen);
  useFocusRestore(recoveryDialogOpen);
  useFocusRestore(jamPlayerOpen);

  // Enable global keyboard shortcuts
  useKeyboardShortcuts();

  // Register voice commands
  useRegisterVoiceCommands();

  // Open JamPlayer when voice search returns results
  useEffect(() => {
    setOnSearchResults((results) => {
      if (results.tracks.length > 0) {
        setJamPlayerOpen(true);
      }
    });

    // Allow voice commands to toggle the JamPlayer
    setJamPlayerToggle((open) => setJamPlayerOpen(open));

    return () => {
      setOnSearchResults(null);
      setJamPlayerToggle(null);
    };
  }, []);

  // Warn user before closing tab with unsaved changes
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (projectModified && project) {
        // Standard way to trigger browser's "unsaved changes" dialog
        e.preventDefault();
        // For older browsers
        e.returnValue = 'You have unsaved changes. Are you sure you want to leave?';
        return e.returnValue;
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, [projectModified, project]);

  // Initialize accessibility features
  useEffect(() => {
    // Initialize screen reader announcer
    ScreenReaderAnnouncer.initialize();

    // Create skip to content link
    createSkipLink('main-content', 'Skip to main content');

    // Check accessibility preferences
    const reducedMotion = prefersReducedMotion();
    if (reducedMotion) {
      logger.accessibility.info('Reduced motion preference detected');
    }

    logger.accessibility.info('Features initialized');
  }, []);

  // Apply user accessibility preferences to document root
  useEffect(() => {
    const prefs = loadPreferences();
    const root = document.documentElement;

    // Reduced motion (respects both system and user preference)
    const systemPrefersReducedMotion = prefersReducedMotion();
    if (prefs.reducedMotion || systemPrefersReducedMotion) {
      root.classList.add('reduce-motion');
    } else {
      root.classList.remove('reduce-motion');
    }

    // High contrast mode
    if (prefs.highContrast) {
      root.classList.add('high-contrast');
    } else {
      root.classList.remove('high-contrast');
    }

    logger.accessibility.debug('Accessibility preferences applied:', {
      reducedMotion: prefs.reducedMotion || systemPrefersReducedMotion,
      highContrast: prefs.highContrast,
    });
  }, [preferencesOpen]); // Re-apply when preferences dialog closes

  // Check for first-run and show onboarding or What's New
  useEffect(() => {
    const hasSeenOnboarding = safeStorage.getItem('orpheus-onboarding-complete');
    if (!hasSeenOnboarding) {
      // New user - show onboarding first
      setTimeout(() => setOnboardingOpen(true), 500);
    } else if (shouldShowWhatsNew) {
      // Returning user with new version - show What's New
      setTimeout(() => setWhatsNewOpen(true), 500);
    }
  }, [shouldShowWhatsNew]);

  // Global keyboard listeners
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      const isInputField = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA';

      // Command Palette: Ctrl+K or Cmd+K
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setCommandPaletteOpen(true);
        return;
      }

      // Preferences: Ctrl+, or Cmd+,
      if ((e.ctrlKey || e.metaKey) && e.key === ',') {
        e.preventDefault();
        setPreferencesOpen(true);
        return;
      }

      // Help shortcuts: ?
      if (e.key === '?' && !e.ctrlKey && !e.metaKey && !e.altKey && !isInputField) {
        e.preventDefault();
        setKeyboardShortcutsOpen(true);
      }

      // JamPlayer: Ctrl+J or Cmd+J
      if ((e.ctrlKey || e.metaKey) && e.key === 'j') {
        e.preventDefault();
        setJamPlayerOpen((prev) => !prev);
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, []);

  // Check for auto-save recovery on mount
  useEffect(() => {
    const autoSave = loadAutoSave();
    if (autoSave && !project) {
      const timeSinceAutoSave = Date.now() - new Date(autoSave.timestamp).getTime();
      const minutesAgo = Math.floor(timeSinceAutoSave / 60000);

      // Extended to 24 hours (1440 minutes) - let users decide what to do with older saves
      if (minutesAgo < 1440) {
        // Store auto-save and show recovery dialog
        pendingAutoSaveRef.current = autoSave;
        setRecoveryMinutesAgo(minutesAgo);
        setRecoveryDialogOpen(true);
      } else {
        // Clear auto-saves older than 24 hours
        clearAutoSave();
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run on mount

  // Handle auto-save recovery confirmation
  const handleRecoveryConfirm = () => {
    if (pendingAutoSaveRef.current) {
      setProject(pendingAutoSaveRef.current.project);
      showInfo('Auto-save recovered successfully', 3000);
      pendingAutoSaveRef.current = null;
    }
    setRecoveryDialogOpen(false);
  };

  // Handle auto-save recovery cancellation
  const handleRecoveryCancel = () => {
    clearAutoSave();
    pendingAutoSaveRef.current = null;
    setRecoveryDialogOpen(false);
  };

  // Auto-save when project changes (respects user preferences)
  useEffect(() => {
    const autoSaveManager = getAutoSaveManager();
    const prefs = loadPreferences();

    if (project) {
      // Check if auto-save is enabled in preferences
      if (!prefs.autoSaveEnabled) {
        if (autoSaveManager.enabled()) {
          autoSaveManager.stop();
          logger.autoSave.info('Auto-save disabled by user preference');
        }
        return;
      }

      // Start/update auto-save with user's preferred interval
      const intervalMs = prefs.autoSaveInterval * 1000; // Convert seconds to ms
      if (!autoSaveManager.enabled()) {
        autoSaveManager.start(project, intervalMs);
        logger.autoSave.info(`Auto-save started (interval: ${prefs.autoSaveInterval}s)`);
      } else {
        autoSaveManager.updateProject(project);
      }
    } else {
      // Stop auto-save when no project
      if (autoSaveManager.enabled()) {
        autoSaveManager.stop();
        logger.autoSave.info('Auto-save stopped');
      }
    }
  }, [project]);

  // Restore session state on mount
  useEffect(() => {
    try {
      const sessionState = loadSessionState();
      const { setMode, setSidebarOpen, setZoomLevel } = useAppStore.getState();

      // Restore UI state
      setMode(sessionState.mode);
      setSidebarOpen(sessionState.sidebarOpen);
      setZoomLevel(sessionState.zoomLevel);

      logger.session.info('Session state restored');
    } catch (error) {
      logger.session.error('Failed to restore session state:', error);
      showError('Failed to restore previous session', 3000);
    }
  }, []);

  // Save session state when it changes
  useEffect(() => {
    saveSessionState({
      mode,
      sidebarOpen,
      zoomLevel,
      selectedTrackId,
      aiAssistantOpen,
    });
  }, [mode, sidebarOpen, zoomLevel, selectedTrackId, aiAssistantOpen]);

  // Background preload adjacent modes for faster navigation
  useEffect(() => {
    // Wait for initial render to complete, then preload adjacent modes
    const timeoutId = setTimeout(() => {
      preloadModes.workflow(mode).catch(() => {
        // Silently ignore preload failures - they're just optimizations
      });
      logger.app.debug(`Preloading adjacent modes for ${mode}`);
    }, 2000); // Wait 2 seconds after mode change to not interfere with initial load

    return () => clearTimeout(timeoutId);
  }, [mode]);

  // Track error boundary reset key per mode
  const [modeResetKey, setModeResetKey] = useState(0);

  const handleModeRetry = () => {
    setModeResetKey((prev) => prev + 1);
  };

  const renderMode = () => {
    const modeNames: Record<string, string> = {
      compose: 'Compose',
      record: 'Record',
      mix: 'Mix',
      master: 'Master',
      practice: 'Practice',
      distribute: 'Distribute',
    };

    const renderModeWithBoundary = (modeName: string, ModeComponent: React.ComponentType) => (
      <ErrorBoundary
        key={`${mode}-${modeResetKey}`}
        fallback={<ModeErrorFallback modeName={modeName} onRetry={handleModeRetry} />}
        onError={(error) => logger.app.error(`${modeName} mode crashed:`, error)}
      >
        <ModeComponent />
      </ErrorBoundary>
    );

    switch (mode) {
      case 'compose':
        return renderModeWithBoundary('Compose', ComposeMode);
      case 'record':
        return renderModeWithBoundary('Record', RecordMode);
      case 'mix':
        return renderModeWithBoundary('Mix', MixMode);
      case 'master':
        return renderModeWithBoundary('Master', MasterMode);
      case 'practice':
        return renderModeWithBoundary('Practice', PracticeMode);
      case 'distribute':
        return renderModeWithBoundary('Distribute', DistributeMode);
      default:
        return <SplashScreen />;
    }
  };

  return (
    <ErrorBoundary>
      <ThemeProvider>
        <div className="flex flex-col h-screen w-screen overflow-hidden bg-background text-foreground">
          <Toolbar />
          <ModeSelector />

          <div className="flex flex-1 overflow-hidden">
            <Sidebar />

            <main className="flex-1 flex flex-col overflow-hidden relative" id="main-content" role="main" aria-label="Main content area">
              <div className="flex-1 overflow-auto p-4">
                <Suspense fallback={<SkeletonLoader variant={mode} />}>
                  {renderMode()}
                </Suspense>
              </div>
            </main>

            <AIAssistant />
          </div>

          <StatusBar />
          <ToastContainer />
          <VoiceCommandOverlay />
          <DragDropZone />
          <LoadingOverlay />

          <KeyboardShortcutsDialog
            open={keyboardShortcutsOpen}
            onClose={() => setKeyboardShortcutsOpen(false)}
          />

          <OnboardingDialog
            open={onboardingOpen}
            onClose={() => setOnboardingOpen(false)}
          />

          <WhatsNewDialog
            open={whatsNewOpen}
            onClose={() => setWhatsNewOpen(false)}
          />

          <PreferencesDialog
            open={preferencesOpen}
            onClose={() => setPreferencesOpen(false)}
          />

          <CommandPalette
            open={commandPaletteOpen}
            onClose={() => setCommandPaletteOpen(false)}
          />

          <JamPlayer
            isOpen={jamPlayerOpen}
            onClose={() => setJamPlayerOpen(false)}
          />

          <ConfirmDialog
            open={recoveryDialogOpen}
            onConfirm={handleRecoveryConfirm}
            onCancel={handleRecoveryCancel}
            title="Recover Auto-Saved Project?"
            message={`An auto-save was found from ${recoveryMinutesAgo >= 60 ? `${Math.floor(recoveryMinutesAgo / 60)} hour${Math.floor(recoveryMinutesAgo / 60) !== 1 ? 's' : ''}` : `${recoveryMinutesAgo} minute${recoveryMinutesAgo !== 1 ? 's' : ''}`} ago.\n\nRecovering will load the auto-saved project. If you choose "Discard", the auto-save will be permanently deleted and you'll start fresh.`}
            confirmText="Recover Project"
            cancelText="Discard"
            type="info"
          />
        </div>
      </ThemeProvider>
    </ErrorBoundary>
  );
}

function SplashScreen() {
  const { setMode } = useAppStore();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { setProject, setRawFileBuffer, setIsLoading, setLoadingMessage } = useAppStore();

  const handleLoadSample = (sampleId: string) => {
    const sample = sampleProjects.find(s => s.id === sampleId);
    if (sample) {
      // Deep clone to avoid mutating the template
      const projectCopy = JSON.parse(JSON.stringify(sample.project));
      // Generate new IDs
      projectCopy.project.metadata.id = Math.random().toString(36).substring(2, 11);
      projectCopy.project.metadata.created = new Date().toISOString();
      projectCopy.project.metadata.modified = new Date().toISOString();

      setProject(projectCopy);
      setMode('compose');
      showInfo(`Loaded: ${sample.title}`, 3000);
    }
  };

  const handleFileImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    try {
      setIsLoading(true);
      setLoadingMessage(`Importing ${file.name}...`);
      const { importFile } = await import('./services/file-import');
      const result = await importFile(file);

      if (result.success && result.project) {
        setProject(result.project);
        if (result.rawFileBuffer) setRawFileBuffer(result.rawFileBuffer);
        setMode('compose');
        showInfo(`Imported: ${file.name}`, 3000);
      } else {
        showError(result.error || 'Import failed', 5000);
      }
    } catch (error) {
      let errorMessage = 'Unknown error occurred';
      if (error instanceof Error) {
        if (error.message.includes('JSON')) {
          errorMessage = 'File appears to be corrupted or not a valid project file';
        } else if (error.message.includes('parse') || error.message.includes('Parse')) {
          errorMessage = 'Unable to read file format. The file may be corrupted or unsupported.';
        } else {
          errorMessage = error.message;
        }
      }
      showError(`Import failed: ${errorMessage}`, 8000);
    } finally {
      setIsLoading(false);
      setLoadingMessage('');
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-full gap-8 px-4">
      {/* Hero */}
      <div className="text-center">
        <h1 className="text-6xl font-bold bg-gradient-to-br from-primary to-purple-500 bg-clip-text text-transparent mb-4">
          Orpheus
        </h1>
        <p className="text-lg text-muted-foreground max-w-[500px]">
          From first chord to final master — one platform, infinite possibilities.
        </p>
      </div>

      {/* Primary CTAs */}
      <div className="flex gap-4">
        <button
          onClick={() => fileInputRef.current?.click()}
          className="px-6 py-3 bg-primary text-primary-foreground rounded-lg font-semibold hover:bg-primary/90 transition-colors flex items-center gap-2 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          Import Guitar Pro File
        </button>
        <button
          onClick={() => setMode('compose')}
          className="px-6 py-3 bg-secondary text-secondary-foreground rounded-lg font-semibold hover:bg-secondary/80 transition-colors flex items-center gap-2 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          Start Fresh
        </button>
        <input
          ref={fileInputRef}
          type="file"
          accept=".gp,.gp3,.gp4,.gp5,.gp6,.gp7,.gpx,.maestro,.json"
          className="hidden"
          onChange={handleFileImport}
        />
      </div>

      {/* Quick Start Guide */}
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <span className="px-2 py-0.5 bg-muted rounded text-xs font-mono">Ctrl+O</span>
        <span>Open file</span>
        <span className="mx-2">•</span>
        <span className="px-2 py-0.5 bg-muted rounded text-xs font-mono">Ctrl+K</span>
        <span>Command palette</span>
        <span className="mx-2">•</span>
        <span className="px-2 py-0.5 bg-muted rounded text-xs font-mono">?</span>
        <span>All shortcuts</span>
      </div>

      {/* Workflow Cards - Visual hierarchy with featured mode */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4 max-w-[900px] w-full">
        {/* Featured: Compose */}
        <button
          onClick={() => setMode('compose')}
          className="md:col-span-3 p-6 bg-gradient-to-r from-primary/10 to-purple-500/10 rounded-xl border-2 border-primary/30 hover:border-primary/60 transition-all text-left group focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        >
          <div className="flex items-start gap-4">
            <div className="p-3 bg-primary/20 rounded-lg text-2xl">🎼</div>
            <div className="flex-1">
              <div className="text-xl font-semibold mb-1 group-hover:text-primary transition-colors">
                Start in Compose Mode
              </div>
              <div className="text-sm text-muted-foreground">
                Professional tablature editing • 100+ chords & 20+ scales • Guitar Pro import • AI composition help
              </div>
            </div>
            <svg className="w-6 h-6 text-muted-foreground group-hover:text-primary group-hover:translate-x-1 transition-all" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </button>

        {/* Secondary modes - 3 columns */}
        <button
          onClick={() => setMode('record')}
          className="p-4 bg-card rounded-lg border border-border hover:border-primary/50 transition-all text-left group focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        >
          <div className="text-2xl mb-2">🎙️</div>
          <div className="font-semibold mb-1 group-hover:text-primary transition-colors">Record</div>
          <div className="text-xs text-muted-foreground">Multi-track audio & MIDI recording</div>
        </button>

        <button
          onClick={() => setMode('mix')}
          className="p-4 bg-card rounded-lg border border-border hover:border-primary/50 transition-all text-left group focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        >
          <div className="text-2xl mb-2">🎚️</div>
          <div className="font-semibold mb-1 group-hover:text-primary transition-colors">Mix</div>
          <div className="text-xs text-muted-foreground">Professional mixing console</div>
        </button>

        <button
          onClick={() => setMode('master')}
          className="p-4 bg-card rounded-lg border border-border hover:border-primary/50 transition-all text-left group focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        >
          <div className="text-2xl mb-2">✨</div>
          <div className="font-semibold mb-1 group-hover:text-primary transition-colors">Master</div>
          <div className="text-xs text-muted-foreground">LUFS metering & export</div>
        </button>

        {/* Tertiary modes - smaller */}
        <div className="md:col-span-3 flex gap-4 justify-center">
          <button
            onClick={() => setMode('practice')}
            className="px-4 py-2 bg-muted/50 rounded-lg hover:bg-muted transition-colors text-sm flex items-center gap-2 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <span>🎸</span>
            <span className="font-medium">Practice</span>
          </button>
          <button
            onClick={() => setMode('distribute')}
            className="px-4 py-2 bg-muted/50 rounded-lg hover:bg-muted transition-colors text-sm flex items-center gap-2 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <span>🌍</span>
            <span className="font-medium">Distribute</span>
          </button>
        </div>
      </div>

      {/* Sample Projects */}
      <div className="w-full max-w-[900px]">
        <div className="text-sm font-medium text-muted-foreground mb-3 text-center">
          Or try a sample project
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          {sampleProjects.map((sample) => (
            <button
              key={sample.id}
              onClick={() => handleLoadSample(sample.id)}
              className="p-3 bg-muted/30 rounded-lg border border-border hover:border-primary/50 hover:bg-muted/50 transition-all text-left group focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
            >
              <div className="flex items-start gap-3">
                <span className="text-2xl">{sample.icon}</span>
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-sm group-hover:text-primary transition-colors truncate">
                    {sample.title}
                  </div>
                  <div className="text-xs text-muted-foreground truncate">
                    {sample.description}
                  </div>
                  <div className="flex gap-2 mt-1">
                    <span className="text-[10px] px-1.5 py-0.5 bg-background rounded">{sample.genre}</span>
                    <span className="text-[10px] px-1.5 py-0.5 bg-background rounded">{sample.difficulty}</span>
                    <span className="text-[10px] text-muted-foreground">{sample.duration}</span>
                  </div>
                </div>
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Drop hint */}
      <div className="text-xs text-muted-foreground/60 mt-4">
        Tip: You can also drag & drop Guitar Pro files anywhere on the screen
      </div>
    </div>
  );
}

export default App;
