/**
 * Main Toolbar - File operations, playback controls, etc.
 */

import {
  ToolbarButton,
  ToolbarDivider,
  Tooltip,
  makeStyles,
  shorthands,
  tokens,
} from '@fluentui/react-components';
import {
  FolderOpen24Regular,
  Save24Regular,
  ArrowDownload24Regular,
  Play24Regular,
  Pause24Regular,
  Stop24Regular,
  BotRegular,
  ArrowUndo24Regular,
  ArrowRedo24Regular,
  QuestionCircle24Regular,
} from '@fluentui/react-icons';
import { useAppStore } from '../store/app-store';
import { getPlaybackCoordinator } from '../services/playback-coordinator';
import { importFile } from '../services/file-import';
import { showSuccess, showError, showInfo } from '../services/toast';
import { saveProject } from '../services/project-save';
import { KeyboardShortcutsDialog } from './KeyboardShortcutsDialog';
import { ExportDialog } from './ExportDialog';
import { useEffect, useState } from 'react';

const useStyles = makeStyles({
  toolbar: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.padding('8px', '16px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
    ...shorthands.gap('8px'),
  },
  logo: {
    fontSize: '18px',
    fontWeight: 700,
    marginRight: '16px',
    fontFamily: 'var(--font-display)',
    letterSpacing: '-0.02em',
    background: 'linear-gradient(135deg, var(--color-phthalo-highlight) 0%, var(--color-phthalo-bright) 100%)',
    WebkitBackgroundClip: 'text',
    WebkitTextFillColor: 'transparent',
    backgroundClip: 'text',
  },
  projectName: {
    fontSize: '14px',
    color: tokens.colorNeutralForeground2,
    marginLeft: 'auto',
  },
});

export function Toolbar() {
  const styles = useStyles();
  const {
    project,
    isPlaying,
    setIsPlaying,
    setCurrentTime,
    aiAssistantOpen,
    setAIAssistantOpen,
    setProject,
    setRawFileBuffer,
    setIsLoading,
    setLoadingMessage,
    setLoadingProgress,
    undo,
    redo,
    canUndo,
    canRedo,
  } = useAppStore();

  const [position, setPosition] = useState('0:00');
  const [keyboardShortcutsOpen, setKeyboardShortcutsOpen] = useState(false);
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const coordinator = getPlaybackCoordinator();

  // Initialize coordinator when project changes
  useEffect(() => {
    if (project) {
      coordinator.initialize(project).then(() => {
        console.log('[Toolbar] PlaybackCoordinator initialized');
      }).catch((error) => {
        console.error('[Toolbar] Failed to initialize coordinator:', error);
      });
    }
  }, [project]);

  // Subscribe to playback state changes
  useEffect(() => {
    const unsubscribe = coordinator.onStateChange((state) => {
      setIsPlaying(state.isPlaying);
      setCurrentTime(state.position.absolute.seconds);

      // Format position for display
      const minutes = Math.floor(state.position.absolute.seconds / 60);
      const seconds = Math.floor(state.position.absolute.seconds % 60);
      setPosition(`${minutes}:${seconds.toString().padStart(2, '0')}`);
    });

    return unsubscribe;
  }, [setIsPlaying, setCurrentTime]);

  const handleOpenFile = async () => {
    // File import dialog
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.maestro,.maestro.json,.gp,.gpx,.gp5,.gp4,.gp3,.gp6,.gp7';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      try {
        console.log('[Toolbar] Importing file:', file.name);
        setIsLoading(true);
        setLoadingProgress(0);
        setLoadingMessage(`Importing ${file.name}...`);
        showInfo(`Importing ${file.name}...`);

        // Import the file with progress tracking
        const result = await importFile(file, {
          onProgress: (progress, message) => {
            setLoadingProgress(progress);
            setLoadingMessage(message);
          },
        });

        if (result.success && result.project) {
          // Store project in state
          setProject(result.project);

          // Store raw buffer for alphaTab if available
          if (result.rawFileBuffer) {
            setRawFileBuffer(result.rawFileBuffer);
          }

          // Show success toast
          const fileType = file.name.endsWith('.maestro') || file.name.endsWith('.json')
            ? 'project'
            : 'Guitar Pro file';
          showSuccess(`Successfully imported ${fileType}: ${file.name}`, 4000);

          // Show warnings if any
          if (result.warnings && result.warnings.length > 0) {
            result.warnings.forEach((warning) => {
              console.warn('[Import]', warning);
            });
          }

          console.log('[Toolbar] Import successful:', result);
        } else {
          // Show error toast
          showError(result.error || 'Failed to import file', 6000);
          console.error('[Toolbar] Import failed:', result.error);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error';
        showError(`Failed to import file: ${message}`, 6000);
        console.error('[Toolbar] Import error:', error);
      } finally {
        setIsLoading(false);
        setLoadingMessage('');
        setLoadingProgress(-1);
      }
    };
    input.click();
  };

  const handleSave = () => {
    if (project) {
      saveProject(project);
    }
  };

  const handlePlayPause = async () => {
    const state = coordinator.getState();
    if (state.isPlaying) {
      coordinator.pause();
    } else {
      await coordinator.play();
    }
  };

  const handleStop = () => {
    coordinator.stop();
  };

  return (
    <div className={styles.toolbar}>
      <div className={styles.logo}>ORPHEUS</div>

      <Tooltip content="Open project or Guitar Pro file" relationship="label">
        <ToolbarButton
          icon={<FolderOpen24Regular />}
          onClick={handleOpenFile}
        >
          Open
        </ToolbarButton>
      </Tooltip>

      <Tooltip content="Save project" relationship="label">
        <ToolbarButton
          icon={<Save24Regular />}
          onClick={handleSave}
          disabled={!project}
        >
          Save
        </ToolbarButton>
      </Tooltip>

      <Tooltip content="Export project (multiple formats)" relationship="label">
        <ToolbarButton
          icon={<ArrowDownload24Regular />}
          onClick={() => setExportDialogOpen(true)}
          disabled={!project}
        >
          Export
        </ToolbarButton>
      </Tooltip>

      <ToolbarDivider />

      <Tooltip content="Undo (Ctrl+Z)" relationship="label">
        <ToolbarButton
          icon={<ArrowUndo24Regular />}
          onClick={() => undo()}
          disabled={!canUndo()}
        />
      </Tooltip>

      <Tooltip content="Redo (Ctrl+Shift+Z)" relationship="label">
        <ToolbarButton
          icon={<ArrowRedo24Regular />}
          onClick={() => redo()}
          disabled={!canRedo()}
        />
      </Tooltip>

      <ToolbarDivider />

      <Tooltip content={isPlaying ? 'Pause' : 'Play'} relationship="label">
        <ToolbarButton
          icon={isPlaying ? <Pause24Regular /> : <Play24Regular />}
          onClick={handlePlayPause}
          disabled={!project}
        />
      </Tooltip>

      <Tooltip content="Stop" relationship="label">
        <ToolbarButton
          icon={<Stop24Regular />}
          onClick={handleStop}
          disabled={!project}
        />
      </Tooltip>

      {project && (
        <>
          <ToolbarDivider />
          <div style={{ fontSize: '14px', fontFamily: 'monospace', minWidth: '60px', textAlign: 'center' }}>
            {position}
          </div>
        </>
      )}

      <ToolbarDivider />

      <Tooltip content="AI Assistant" relationship="label">
        <ToolbarButton
          icon={<BotRegular />}
          appearance={aiAssistantOpen ? 'primary' : 'subtle'}
          onClick={() => setAIAssistantOpen(!aiAssistantOpen)}
        >
          AI Assistant
        </ToolbarButton>
      </Tooltip>

      <Tooltip content="Keyboard Shortcuts (Press ?)" relationship="label">
        <ToolbarButton
          icon={<QuestionCircle24Regular />}
          appearance="subtle"
          onClick={() => setKeyboardShortcutsOpen(true)}
        >
          Help
        </ToolbarButton>
      </Tooltip>

      {project && (
        <div className={styles.projectName}>
          {project.project.metadata.title || 'Untitled Project'}
        </div>
      )}

      <KeyboardShortcutsDialog
        open={keyboardShortcutsOpen}
        onClose={() => setKeyboardShortcutsOpen(false)}
      />

      <ExportDialog
        open={exportDialogOpen}
        onClose={() => setExportDialogOpen(false)}
      />
    </div>
  );
}
