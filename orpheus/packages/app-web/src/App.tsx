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
import { lazy, Suspense, useEffect, useState } from 'react';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { loadAutoSave, clearAutoSave, getAutoSaveManager } from './services/project-save';
import { loadSessionState, saveSessionState } from './services/session-persistence';
import { useAppStore } from './store/app-store';
import { showInfo, showError } from './services/toast';
import { ScreenReaderAnnouncer, createSkipLink, prefersReducedMotion } from './utils/accessibility';

// Lazy load mode components for code splitting
const ComposeMode = lazy(() => import('./modes/ComposeMode').then(m => ({ default: m.ComposeMode })));
const RecordMode = lazy(() => import('./modes/RecordMode').then(m => ({ default: m.RecordMode })));
const MixMode = lazy(() => import('./modes/MixMode').then(m => ({ default: m.MixMode })));
const MasterMode = lazy(() => import('./modes/MasterMode').then(m => ({ default: m.MasterMode })));
const PracticeMode = lazy(() => import('./modes/PracticeMode').then(m => ({ default: m.PracticeMode })));
const DistributeMode = lazy(() => import('./modes/DistributeMode').then(m => ({ default: m.DistributeMode })));


function App() {
  const mode = useCurrentMode();
  const { project, setProject } = useAppStore();
  const [keyboardShortcutsOpen, setKeyboardShortcutsOpen] = useState(false);
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);

  // Enable global keyboard shortcuts
  useKeyboardShortcuts();

  // Initialize accessibility features
  useEffect(() => {
    // Initialize screen reader announcer
    ScreenReaderAnnouncer.initialize();

    // Create skip to content link
    createSkipLink('main-content', 'Skip to main content');

    // Log accessibility preferences
    const reducedMotion = prefersReducedMotion();
    if (reducedMotion) {
      console.log('[Accessibility] Reduced motion preference detected');
      // Future: Disable animations based on this preference
    }

    console.log('[Accessibility] Features initialized');
  }, []);

  // Check for first-run and show onboarding
  useEffect(() => {
    const hasSeenOnboarding = localStorage.getItem('orpheus-onboarding-complete');
    if (!hasSeenOnboarding) {
      // Delay slightly so the app renders first
      setTimeout(() => setOnboardingOpen(true), 500);
    }
  }, []);

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

      // Help shortcuts: ?
      if (e.key === '?' && !e.ctrlKey && !e.metaKey && !e.altKey && !isInputField) {
        e.preventDefault();
        setKeyboardShortcutsOpen(true);
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

      if (minutesAgo < 60) {
        // Offer to recover if saved within last hour
        // eslint-disable-next-line no-restricted-globals
        const recover = confirm(
          `Auto-save found from ${minutesAgo} minute${minutesAgo !== 1 ? 's' : ''} ago.\n\nRecover the auto-saved project?`
        );

        if (recover) {
          setProject(autoSave.project);
          showInfo('Auto-save recovered successfully', 3000);
          console.log('[App] Auto-save recovered');
        } else {
          clearAutoSave();
        }
      } else {
        // Clear old auto-save
        clearAutoSave();
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run on mount

  // Auto-save when project changes
  useEffect(() => {
    const autoSaveManager = getAutoSaveManager();

    if (project) {
      // Start/update auto-save
      if (!autoSaveManager.enabled()) {
        autoSaveManager.start(project, 30000); // 30 seconds
        console.log('[App] Auto-save started');
      } else {
        autoSaveManager.updateProject(project);
      }
    } else {
      // Stop auto-save when no project
      if (autoSaveManager.enabled()) {
        autoSaveManager.stop();
        console.log('[App] Auto-save stopped');
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

      console.log('[App] Session state restored');
    } catch (error) {
      console.error('[App] Failed to restore session state:', error);
      showError('Failed to restore previous session', 3000);
    }
  }, []);

  // Save session state when it changes
  useEffect(() => {
    const { mode, sidebarOpen, zoomLevel, selectedTrackId, aiAssistantOpen } = useAppStore.getState();
    saveSessionState({
      mode,
      sidebarOpen,
      zoomLevel,
      selectedTrackId,
      aiAssistantOpen,
    });
  }, [mode, useAppStore.getState().sidebarOpen, useAppStore.getState().zoomLevel]);

  const renderMode = () => {
    switch (mode) {
      case 'compose':
        return <ComposeMode />;
      case 'record':
        return <RecordMode />;
      case 'mix':
        return <MixMode />;
      case 'master':
        return <MasterMode />;
      case 'practice':
        return <PracticeMode />;
      case 'distribute':
        return <DistributeMode />;
      default:
        return <SplashScreen />;
    }
  };

  return (
    <ErrorBoundary>
      <ThemeProvider>
        <div className="flex flex-col h-screen w-screen overflow-hidden bg-[#1e1e1e] text-white">
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

          <CommandPalette
            open={commandPaletteOpen}
            onClose={() => setCommandPaletteOpen(false)}
          />
        </div>
      </ThemeProvider>
    </ErrorBoundary>
  );
}

function SplashScreen() {
  return (
    <div className="flex flex-col items-center justify-center h-full gap-6">
      <h1 className="text-5xl font-bold bg-gradient-to-br from-[#667eea] to-[#764ba2] bg-clip-text text-transparent">
        🎸 Orpheus
      </h1>
      <p className="text-xl text-[#999] text-center max-w-[600px]">
        The world's first unified music production platform.
        <br />
        From first chord to final master - one application, infinite possibilities.
      </p>

      <div className="grid grid-cols-[repeat(auto-fit,minmax(250px,1fr))] gap-4 mt-8 max-w-[1000px]">
        <div className="p-5 bg-[#2a2a2a] rounded-lg border border-[#3a3a3a]">
          <div className="text-lg font-semibold mb-2">🎼 Compose Mode</div>
          <div className="text-sm text-[#999] leading-relaxed">
            Professional tablature editing with Guitar Pro import.
            <br />
            100+ chords, 20+ scales, AI composition assistance.
          </div>
        </div>

        <div className="p-5 bg-[#2a2a2a] rounded-lg border border-[#3a3a3a]">
          <div className="text-lg font-semibold mb-2">🎙️ Record Mode</div>
          <div className="text-sm text-[#999] leading-relaxed">
            Multi-track audio recording with real-time monitoring.
            <br />
            VST/AU/AAX plugin hosting, MIDI recording.
          </div>
        </div>

        <div className="p-5 bg-[#2a2a2a] rounded-lg border border-[#3a3a3a]">
          <div className="text-lg font-semibold mb-2">🎚️ Mix Mode</div>
          <div className="text-sm text-[#999] leading-relaxed">
            Professional mixing console with AI suggestions.
            <br />
            EQ, compression, reverb, automation.
          </div>
        </div>

        <div className="p-5 bg-[#2a2a2a] rounded-lg border border-[#3a3a3a]">
          <div className="text-lg font-semibold mb-2">✨ Master Mode</div>
          <div className="text-sm text-[#999] leading-relaxed">
            AI-powered mastering with platform-specific targets.
            <br />
            LUFS metering, multi-format export.
          </div>
        </div>

        <div className="p-5 bg-[#2a2a2a] rounded-lg border border-[#3a3a3a]">
          <div className="text-lg font-semibold mb-2">🎸 Practice Mode</div>
          <div className="text-sm text-[#999] leading-relaxed">
            Speed trainer with AI performance analysis.
            <br />
            Loop sections, track progress, improve technique.
          </div>
        </div>

        <div className="p-5 bg-[#2a2a2a] rounded-lg border border-[#3a3a3a]">
          <div className="text-lg font-semibold mb-2">🌍 Distribute Mode</div>
          <div className="text-sm text-[#999] leading-relaxed">
            Upload to Spotify, Apple Music, and all streaming platforms.
            <br />
            DistroKid integration, release management, analytics.
          </div>
        </div>

        <div className="p-5 bg-[#2a2a2a] rounded-lg border border-[#3a3a3a]">
          <div className="text-lg font-semibold mb-2">🤖 AI Assistance</div>
          <div className="text-sm text-[#999] leading-relaxed">
            7 specialized AI personas guide you through production.
            <br />
            Music theory, composition, mixing, mastering, learning.
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
