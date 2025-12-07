import { Button, Tooltip, TooltipTrigger, TooltipContent, Divider } from '@persona-framework/ui';
import {
  FolderOpen,
  Save,
  Download,
  Play,
  Pause,
  Square,
  Bot,
  Undo,
  Redo,
  HelpCircle,
} from 'lucide-react';
import { useAppStore } from '../store/app-store';
import { getPlaybackCoordinator } from '../services/playback-coordinator';
import { importFile } from '../services/file-import';
import { showSuccess, showError, showInfo } from '../services/toast';
import { saveProject } from '../services/project-save';
import { KeyboardShortcutsDialog } from './KeyboardShortcutsDialog';
import { ExportDialog } from './ExportDialog';
import { useEffect, useState } from 'react';

export function Toolbar() {
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

  useEffect(() => {
    if (project) {
      coordinator.initialize(project).then(() => {
        console.log('[Toolbar] PlaybackCoordinator initialized');
      }).catch((error) => {
        console.error('[Toolbar] Failed to initialize coordinator:', error);
      });
    }
  }, [project]);

  useEffect(() => {
    const unsubscribe = coordinator.onStateChange((state) => {
      setIsPlaying(state.isPlaying);
      setCurrentTime(state.position.absolute.seconds);

      const minutes = Math.floor(state.position.absolute.seconds / 60);
      const seconds = Math.floor(state.position.absolute.seconds % 60);
      setPosition(`${minutes}:${seconds.toString().padStart(2, '0')}`);
    });

    return unsubscribe;
  }, [setIsPlaying, setCurrentTime]);

  const handleOpenFile = async () => {
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

        const result = await importFile(file, {
          onProgress: (progress, message) => {
            setLoadingProgress(progress);
            setLoadingMessage(message);
          },
        });

        if (result.success && result.project) {
          setProject(result.project);

          if (result.rawFileBuffer) {
            setRawFileBuffer(result.rawFileBuffer);
          }

          const fileType = file.name.endsWith('.maestro') || file.name.endsWith('.json')
            ? 'project'
            : 'Guitar Pro file';
          showSuccess(`Successfully imported ${fileType}: ${file.name}`, 4000);

          if (result.warnings && result.warnings.length > 0) {
            result.warnings.forEach((warning) => {
              console.warn('[Import]', warning);
            });
          }

          console.log('[Toolbar] Import successful:', result);
        } else {
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
    <div className="flex items-center px-4 py-2 bg-background border-b border-border gap-2">
      <div className="text-xl font-bold mr-4 bg-gradient-to-br from-[#667eea] to-[#764ba2] bg-clip-text text-transparent">
        🎸 Orpheus
      </div>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="sm" onClick={handleOpenFile}>
            <FolderOpen className="h-4 w-4 mr-2" />
            Open
          </Button>
        </TooltipTrigger>
        <TooltipContent>Open project or Guitar Pro file</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="sm" onClick={handleSave} disabled={!project}>
            <Save className="h-4 w-4 mr-2" />
            Save
          </Button>
        </TooltipTrigger>
        <TooltipContent>Save project</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="sm" onClick={() => setExportDialogOpen(true)} disabled={!project}>
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
        </TooltipTrigger>
        <TooltipContent>Export project (multiple formats)</TooltipContent>
      </Tooltip>

      <Divider orientation="vertical" className="h-6" />

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="sm" onClick={() => undo()} disabled={!canUndo()}>
            <Undo className="h-4 w-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Undo (Ctrl+Z)</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="sm" onClick={() => redo()} disabled={!canRedo()}>
            <Redo className="h-4 w-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Redo (Ctrl+Shift+Z)</TooltipContent>
      </Tooltip>

      <Divider orientation="vertical" className="h-6" />

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="sm" onClick={handlePlayPause} disabled={!project}>
            {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
          </Button>
        </TooltipTrigger>
        <TooltipContent>{isPlaying ? 'Pause' : 'Play'}</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="sm" onClick={handleStop} disabled={!project}>
            <Square className="h-4 w-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Stop</TooltipContent>
      </Tooltip>

      {project && (
        <>
          <Divider orientation="vertical" className="h-6" />
          <div className="text-sm font-mono min-w-[60px] text-center">
            {position}
          </div>
        </>
      )}

      <Divider orientation="vertical" className="h-6" />

      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={aiAssistantOpen ? 'default' : 'ghost'}
            size="sm"
            onClick={() => setAIAssistantOpen(!aiAssistantOpen)}
          >
            <Bot className="h-4 w-4 mr-2" />
            AI Assistant
          </Button>
        </TooltipTrigger>
        <TooltipContent>AI Assistant</TooltipContent>
      </Tooltip>

      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="sm" onClick={() => setKeyboardShortcutsOpen(true)}>
            <HelpCircle className="h-4 w-4 mr-2" />
            Help
          </Button>
        </TooltipTrigger>
        <TooltipContent>Keyboard Shortcuts (Press ?)</TooltipContent>
      </Tooltip>

      {project && (
        <div className="ml-auto text-sm text-muted-foreground">
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
