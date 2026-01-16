/**
 * Compose Mode - Tablature and notation editing (Cadenza AI)
 */

import { makeStyles, shorthands, tokens, Button, Spinner, Tab, TabList } from '@fluentui/react-components';
import { ArrowUpload24Regular, DocumentAdd24Regular, Eye24Regular, Edit24Regular } from '@fluentui/react-icons';
import { useProject, useAppStore } from '../store/app-store';
import { useState, useEffect } from 'react';
import { importFile, formatFileSize } from '../services/file-import';
import { useAlphaTab } from '../hooks/useAlphaTab';
import { getPlaybackCoordinator } from '../services/playback-coordinator';
import { NewProjectDialog } from '../components/NewProjectDialog';
import { TabEditor } from '../components/TabEditor';
import { MessageDialog, type MessageType } from '../components/MessageDialog';

const useStyles = makeStyles({
  container: {
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
  },
  header: {
    ...shorthands.padding('16px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  title: {
    fontSize: '20px',
    fontWeight: tokens.fontWeightSemibold,
  },
  content: {
    flex: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexDirection: 'column',
    ...shorthands.gap('24px'),
  },
  emptyState: {
    textAlign: 'center',
    maxWidth: '500px',
  },
  emptyStateTitle: {
    fontSize: '24px',
    fontWeight: 600,
    marginBottom: '12px',
  },
  emptyStateDesc: {
    fontSize: '14px',
    color: tokens.colorNeutralForeground2,
    lineHeight: '1.6',
    marginBottom: '24px',
  },
  tablatureView: {
    flex: 1,
    width: '100%',
    backgroundColor: '#ffffff',
    ...shorthands.borderRadius('8px'),
    ...shorthands.padding('16px'),
    overflow: 'auto',
  },
  alphaTabContainer: {
    width: '100%',
    height: '100%',
  },
  projectInfo: {
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('8px'),
    marginBottom: '16px',
  },
  infoRow: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '14px',
    marginBottom: '8px',
  },
  label: {
    fontWeight: tokens.fontWeightSemibold,
  },
});

export function ComposeMode() {
  const styles = useStyles();
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
        console.log(`[ComposeMode] Starting import of ${file.name}`);

        const result = await importFile(file);

        if (result.success && result.project) {
          console.log(`[ComposeMode] Import successful in ${result.importTime.toFixed(2)}ms`);
          console.log(`[ComposeMode] File size: ${formatFileSize(result.fileSize)}`);

          setProject(result.project);
          setRawFileBuffer(result.rawFileBuffer || null);

          // Show warnings if any
          if (result.warnings && result.warnings.length > 0) {
            console.warn('[ComposeMode] Import warnings:', result.warnings);
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
          console.error('[ComposeMode] Import failed:', result.error);
          setMessageDialog({
            open: true,
            title: 'Import Failed',
            message: result.error || 'Unknown error occurred',
            type: 'error',
          });
        }
      } catch (error) {
        console.error('[ComposeMode] Import error:', error);
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
    <div className={styles.container}>
      <div className={styles.header}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <div className={styles.title}>🎼 Compose - Cadenza AI</div>
          {project && (
            <TabList
              selectedValue={viewMode}
              onTabSelect={(_, data) => setViewMode(data.value as 'view' | 'edit')}
              size="small"
            >
              <Tab icon={<Eye24Regular />} value="view">
                View
              </Tab>
              <Tab icon={<Edit24Regular />} value="edit">
                Edit
              </Tab>
            </TabList>
          )}
        </div>
        <Button
          icon={importing ? <Spinner size="tiny" /> : <ArrowUpload24Regular />}
          appearance="primary"
          onClick={handleImportGP}
          disabled={importing}
        >
          {importing ? 'Importing...' : 'Import Guitar Pro'}
        </Button>
      </div>

      {!project ? (
        <div className={styles.content}>
          <div className={styles.emptyState}>
            <div className={styles.emptyStateTitle}>
              Welcome to Compose Mode
            </div>
            <div className={styles.emptyStateDesc}>
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
            <div style={{ display: 'flex', gap: '12px', justifyContent: 'center' }}>
              <Button
                size="large"
                appearance="primary"
                icon={<DocumentAdd24Regular />}
                onClick={() => setNewProjectDialogOpen(true)}
              >
                New Project
              </Button>
              <Button
                size="large"
                appearance="secondary"
                icon={importing ? <Spinner size="small" /> : <ArrowUpload24Regular />}
                onClick={handleImportGP}
                disabled={importing}
              >
                {importing ? 'Importing...' : 'Import Guitar Pro'}
              </Button>
            </div>
          </div>
        </div>
      ) : (
        <div className={styles.content} style={{ alignItems: 'stretch' }}>
          {viewMode === 'view' ? (
            <>
              <div className={styles.projectInfo}>
                <div className={styles.infoRow}>
                  <span className={styles.label}>Title:</span>
                  <span>{project.project.metadata.title || 'Untitled'}</span>
                </div>
                {project.project.metadata.artist && (
                  <div className={styles.infoRow}>
                    <span className={styles.label}>Artist:</span>
                    <span>{project.project.metadata.artist}</span>
                  </div>
                )}
                <div className={styles.infoRow}>
                  <span className={styles.label}>Tempo:</span>
                  <span>{project.project.metadata.tempo || 120} BPM</span>
                </div>
                <div className={styles.infoRow}>
                  <span className={styles.label}>Key:</span>
                  <span>{project.project.metadata.key || 'C'}</span>
                </div>
                <div className={styles.infoRow}>
                  <span className={styles.label}>Tracks:</span>
                  <span>{(project.project.composition.tracks || []).length}</span>
                </div>
                <div className={styles.infoRow}>
                  <span className={styles.label}>Measures:</span>
                  <span>{(project.project.composition.measures || []).length}</span>
                </div>
              </div>

              <div className={styles.tablatureView}>
                <div ref={containerRef} className={styles.alphaTabContainer} />
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
