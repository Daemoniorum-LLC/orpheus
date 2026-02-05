/**
 * Maestro AI - Main Application Component
 * The world's first unified music production platform
 */

import { FluentProvider, makeStyles, shorthands } from '@fluentui/react-components';
import daemoniorumDarkTheme from './theme/daemoniorum-theme';
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
import {
  MusicNote224Regular,
  Record24Regular,
  SpeakerSettings24Regular,
  WandRegular,
  LearningApp24Regular,
  CloudArrowUp24Regular,
  BotSparkleRegular,
} from '@fluentui/react-icons';
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

const useStyles = makeStyles({
  root: {
    display: 'flex',
    flexDirection: 'column',
    height: '100vh',
    width: '100vw',
    overflow: 'hidden',
    backgroundColor: 'var(--color-charcoal-850)',
    color: '#f0f0f0',
  },
  mainContent: {
    display: 'flex',
    flex: 1,
    overflow: 'hidden',
  },
  contentArea: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
    position: 'relative',
  },
  modeContainer: {
    flex: 1,
    overflow: 'auto',
    ...shorthands.padding('16px'),
  },
  splashContainer: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    ...shorthands.gap('24px'),
  },
  splashTitle: {
    fontSize: '48px',
    fontWeight: 700,
    fontFamily: 'var(--font-display)',
    letterSpacing: '-0.03em',
    background: 'linear-gradient(135deg, var(--color-phthalo-highlight) 0%, var(--color-phthalo-bright) 100%)',
    WebkitBackgroundClip: 'text',
    WebkitTextFillColor: 'transparent',
    backgroundClip: 'text',
  },
  splashSubtitle: {
    fontSize: '20px',
    color: 'var(--color-charcoal-300)',
    textAlign: 'center',
    maxWidth: '600px',
    lineHeight: '1.6',
  },
  featureGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
    ...shorthands.gap('16px'),
    marginTop: '32px',
    maxWidth: '1000px',
  },
  featureCard: {
    ...shorthands.padding('20px'),
    backgroundColor: 'var(--color-charcoal-650)',
    ...shorthands.borderRadius('6px'),
    ...shorthands.border('1px', 'solid', 'var(--color-charcoal-500)'),
    ...shorthands.transition('all', '200ms', 'ease'),
    ':hover': {
      borderColor: 'var(--color-phthalo-base)',
      boxShadow: '0 4px 12px rgba(0,0,0,0.4)',
    },
  },
  featureTitle: {
    fontSize: '16px',
    fontWeight: 600,
    marginBottom: '8px',
    fontFamily: 'var(--font-display)',
    color: 'var(--color-phthalo-highlight)',
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
  },
  featureIcon: {
    width: '20px',
    height: '20px',
    color: 'var(--color-phthalo-light)',
  },
  featureDesc: {
    fontSize: '13px',
    color: 'var(--color-charcoal-300)',
    lineHeight: '1.6',
  },
  modeLoading: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
});

function App() {
  const styles = useStyles();
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
    const hasSeenOnboarding = localStorage.getItem('maestro-onboarding-complete');
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
    // Wrap each mode in its own ErrorBoundary for isolated error handling
    switch (mode) {
      case 'compose':
        return (
          <ErrorBoundary key="compose">
            <ComposeMode />
          </ErrorBoundary>
        );
      case 'record':
        return (
          <ErrorBoundary key="record">
            <RecordMode />
          </ErrorBoundary>
        );
      case 'mix':
        return (
          <ErrorBoundary key="mix">
            <MixMode />
          </ErrorBoundary>
        );
      case 'master':
        return (
          <ErrorBoundary key="master">
            <MasterMode />
          </ErrorBoundary>
        );
      case 'practice':
        return (
          <ErrorBoundary key="practice">
            <PracticeMode />
          </ErrorBoundary>
        );
      case 'distribute':
        return (
          <ErrorBoundary key="distribute">
            <DistributeMode />
          </ErrorBoundary>
        );
      default:
        return <SplashScreen />;
    }
  };

  return (
    <ErrorBoundary>
      <FluentProvider theme={daemoniorumDarkTheme}>
        <div className={styles.root}>
        <Toolbar />
        <ModeSelector />

        <div className={styles.mainContent}>
          <Sidebar />

          <main className={styles.contentArea} id="main-content" role="main" aria-label="Main content area">
            <div className={styles.modeContainer}>
              <Suspense fallback={<SkeletonLoader variant={mode} />}>
                {renderMode()}
              </Suspense>
            </div>
          </main>

          <AIAssistant />
        </div>

        {/* Status Bar */}
        <StatusBar />

        {/* Toast notifications */}
        <ToastContainer />

        {/* Drag & drop file import */}
        <DragDropZone />

        {/* Loading overlay */}
        <LoadingOverlay />

        {/* Keyboard shortcuts dialog */}
        <KeyboardShortcutsDialog
          open={keyboardShortcutsOpen}
          onClose={() => setKeyboardShortcutsOpen(false)}
        />

        {/* Onboarding tutorial */}
        <OnboardingDialog
          open={onboardingOpen}
          onClose={() => setOnboardingOpen(false)}
        />

        {/* Command Palette */}
        <CommandPalette
          open={commandPaletteOpen}
          onClose={() => setCommandPaletteOpen(false)}
        />
        </div>
      </FluentProvider>
    </ErrorBoundary>
  );
}

function SplashScreen() {
  const styles = useStyles();

  return (
    <div className={styles.splashContainer}>
      <h1 className={styles.splashTitle}>Orpheus</h1>
      <p className={styles.splashSubtitle}>
        The unified music production platform.
        <br />
        From first chord to final master — one application, infinite possibilities.
      </p>

      <div className={styles.featureGrid}>
        <div className={styles.featureCard}>
          <div className={styles.featureTitle}>
            <MusicNote224Regular className={styles.featureIcon} />
            Compose
          </div>
          <div className={styles.featureDesc}>
            Professional tablature editing with Guitar Pro import.
            100+ chords, 20+ scales, AI composition assistance.
          </div>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureTitle}>
            <Record24Regular className={styles.featureIcon} />
            Record
          </div>
          <div className={styles.featureDesc}>
            Multi-track audio recording with real-time monitoring.
            VST/AU/AAX plugin hosting, MIDI recording.
          </div>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureTitle}>
            <SpeakerSettings24Regular className={styles.featureIcon} />
            Mix
          </div>
          <div className={styles.featureDesc}>
            Professional mixing console with AI suggestions.
            EQ, compression, reverb, automation.
          </div>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureTitle}>
            <WandRegular className={styles.featureIcon} />
            Master
          </div>
          <div className={styles.featureDesc}>
            AI-powered mastering with platform-specific targets.
            LUFS metering, multi-format export.
          </div>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureTitle}>
            <LearningApp24Regular className={styles.featureIcon} />
            Practice
          </div>
          <div className={styles.featureDesc}>
            Speed trainer with AI performance analysis.
            Loop sections, track progress, improve technique.
          </div>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureTitle}>
            <CloudArrowUp24Regular className={styles.featureIcon} />
            Distribute
          </div>
          <div className={styles.featureDesc}>
            Upload to Spotify, Apple Music, and streaming platforms.
            Release management, analytics, royalty tracking.
          </div>
        </div>

        <div className={styles.featureCard}>
          <div className={styles.featureTitle}>
            <BotSparkleRegular className={styles.featureIcon} />
            AI Assistant
          </div>
          <div className={styles.featureDesc}>
            7 specialized AI personas guide you through production.
            Music theory, composition, mixing, mastering, learning.
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
