/**
 * Compose Mode - Tablature and notation editing (Cadenza AI)
 */

import { Button, Tabs, TabsList, TabsTrigger } from '@persona-framework/ui';
import { Upload, FilePlus, Eye, Pencil, Loader2 } from 'lucide-react';
import { useProject, useAppStore } from '../store/app-store';
import { useState, useEffect } from 'react';
import { importFile, formatFileSize } from '../services/file-import';
import { useAlphaTab } from '../hooks/useAlphaTab';
import { getPlaybackCoordinator } from '../services/playback-coordinator';
import { NewProjectDialog } from '../components/NewProjectDialog';
import { TabEditor } from '../components/TabEditor';
import { MessageDialog, type MessageType } from '../components/MessageDialog';

export function ComposeMode() {
  const project = useProject();
  const { setProject, updateProject, setRawFileBuffer, setIsLoading, setLoadingMessage, rawFileBuffer } = useAppStore();
  const [importing, setImporting] = useState(false);
  const [newProjectDialogOpen, setNewProjectDialogOpen] = useState(false);
  const [viewMode, setViewMode] = useState<'view' | 'edit'>('view');

  // Message dialog state
  const [messageDialog, setMessageDialog] = useState<{
    open: boolean;
    title: string;
    message: string;
    type: MessageType;
  }>({
    open: false,
    title: '',
    message: '',
    type: 'info',
  });

  // alphaTab integration
  const { containerRef, api } = useAlphaTab(
    project,
    {
      enablePlayer: true,
      zoom: 1.0,
      layoutMode: 'page',
    },
    rawFileBuffer
  );

  // Register alphaTab API with playback coordinator
  const coordinator = getPlaybackCoordinator();
  useEffect(() => {
    coordinator.setAlphaTabApi(api);
    return () => {
      coordinator.setAlphaTabApi(null);
    };
  }, [api, coordinator]);

  const handleImportGP = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.gp,.gpx,.gp5,.gp4,.gp3,.maestro';

    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      setImporting(true);
      setIsLoading(true);
      setLoadingMessage(`Importing ${file.name}...`);

      try {
        const result = await importFile(file);

        if (result.success && result.project) {
          setProject(result.project);
          setRawFileBuffer(result.rawFileBuffer || null);

          // Show warnings if any
          if (result.warnings && result.warnings.length > 0) {
            setMessageDialog({
              open: true,
              title: 'File Imported with Warnings',
              message: result.warnings.join('\n\n'),
              type: 'warning',
            });
          } else {
            setMessageDialog({
              open: true,
              title: 'Import Successful',
              message: `Successfully imported ${result.fileName}!\n\nImport time: ${result.importTime.toFixed(2)}ms\nFile size: ${formatFileSize(result.fileSize)}`,
              type: 'success',
            });
          }
        } else {
          setMessageDialog({
            open: true,
            title: 'Import Failed',
            message: result.error || 'Unknown error occurred',
            type: 'error',
          });
        }
      } catch (error) {
        setMessageDialog({
          open: true,
          title: 'Import Error',
          message: error instanceof Error ? error.message : 'Unknown error',
          type: 'error',
        });
      } finally {
        setImporting(false);
        setIsLoading(false);
        setLoadingMessage('');
      }
    };

    input.click();
  };

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 border-b border-border flex justify-between items-center">
        <div className="flex items-center gap-4">
          <div className="text-xl font-semibold">🎼 Compose - Cadenza AI</div>
          {project && (
            <Tabs value={viewMode} onValueChange={(v) => setViewMode(v as 'view' | 'edit')}>
              <TabsList>
                <TabsTrigger value="view" className="flex items-center gap-1.5">
                  <Eye className="h-4 w-4" />
                  View
                </TabsTrigger>
                <TabsTrigger value="edit" className="flex items-center gap-1.5">
                  <Pencil className="h-4 w-4" />
                  Edit
                </TabsTrigger>
              </TabsList>
            </Tabs>
          )}
        </div>
        <Button onClick={handleImportGP} disabled={importing}>
          {importing ? (
            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
          ) : (
            <Upload className="h-4 w-4 mr-2" />
          )}
          {importing ? 'Importing...' : 'Import Guitar Pro'}
        </Button>
      </div>

      {!project ? (
        <div className="flex-1 flex items-center justify-center flex-col gap-6">
          <div className="text-center max-w-[500px]">
            <div className="text-2xl font-semibold mb-3">
              Welcome to Compose Mode
            </div>
            <div className="text-sm text-muted-foreground leading-relaxed mb-6">
              Import a Guitar Pro file (.gp, .gpx, .gp5) to start editing tablature
              and notation. You can also create a new project from scratch.
              <br /><br />
              <strong>Features:</strong>
              <br />
              • Professional tablature editor
              <br />
              • 100+ chord library with voicings
              <br />
              • 20+ scale reference
              <br />
              • AI composition assistance
              <br />
              • Real-time MIDI playback
            </div>
            <div className="flex gap-3 justify-center">
              <Button
                size="lg"
                onClick={() => setNewProjectDialogOpen(true)}
              >
                <FilePlus className="h-4 w-4 mr-2" />
                New Project
              </Button>
              <Button
                size="lg"
                variant="outline"
                onClick={handleImportGP}
                disabled={importing}
              >
                {importing ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : (
                  <Upload className="h-4 w-4 mr-2" />
                )}
                {importing ? 'Importing...' : 'Import Guitar Pro'}
              </Button>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex flex-col items-stretch p-6 gap-6">
          {viewMode === 'view' ? (
            <>
              <div className="p-4 bg-secondary rounded-lg">
                <div className="flex justify-between text-sm mb-2">
                  <span className="font-semibold">Title:</span>
                  <span>{project.project.metadata.title || 'Untitled'}</span>
                </div>
                {project.project.metadata.artist && (
                  <div className="flex justify-between text-sm mb-2">
                    <span className="font-semibold">Artist:</span>
                    <span>{project.project.metadata.artist}</span>
                  </div>
                )}
                <div className="flex justify-between text-sm mb-2">
                  <span className="font-semibold">Tempo:</span>
                  <span>{project.project.metadata.tempo || 120} BPM</span>
                </div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="font-semibold">Key:</span>
                  <span>{project.project.metadata.key || 'C'}</span>
                </div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="font-semibold">Tracks:</span>
                  <span>{(project.project.composition.tracks || []).length}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="font-semibold">Measures:</span>
                  <span>{(project.project.composition.measures || []).length}</span>
                </div>
              </div>

              <div className="flex-1 bg-white rounded-lg p-4 overflow-auto">
                <div ref={containerRef} className="w-full h-full" />
              </div>
            </>
          ) : (
            <TabEditor
              project={project}
              onProjectChange={(updatedProject) => {
                updateProject(updatedProject);
              }}
            />
          )}
        </div>
      )}

      <NewProjectDialog
        open={newProjectDialogOpen}
        onOpenChange={setNewProjectDialogOpen}
      />

      <MessageDialog
        open={messageDialog.open}
        onClose={() => setMessageDialog({ ...messageDialog, open: false })}
        title={messageDialog.title}
        message={messageDialog.message}
        type={messageDialog.type}
      />
    </div>
  );
}
