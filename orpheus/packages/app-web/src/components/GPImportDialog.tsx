/**
 * Guitar Pro Import Dialog
 * Handles importing GP files into the Tab Editor
 */

import {
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Button,
  makeStyles,
  shorthands,
  tokens,
  Spinner,
  MessageBar,
  MessageBarBody,
  MessageBarTitle,
} from '@fluentui/react-components';
import {
  ArrowUploadRegular,
  MusicNote2Regular,
  CheckmarkRegular,
  WarningRegular,
} from '@fluentui/react-icons';
import { useState, useRef, useCallback } from 'react';
import { GuitarProParser } from '@orpheus/guitar-pro-parser';
import type { MaestroProject } from '@orpheus/shared-types';

const useStyles = makeStyles({
  dropZone: {
    ...shorthands.border('2px', 'dashed', tokens.colorNeutralStroke1),
    ...shorthands.borderRadius('8px'),
    ...shorthands.padding('40px'),
    textAlign: 'center',
    cursor: 'pointer',
    transitionProperty: 'all',
    transitionDuration: '200ms',
    backgroundColor: tokens.colorNeutralBackground1,
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
      borderColor: tokens.colorBrandStroke1,
    },
  },
  dropZoneActive: {
    backgroundColor: tokens.colorBrandBackground2,
    borderColor: tokens.colorBrandStroke1,
  },
  dropIcon: {
    fontSize: '48px',
    color: tokens.colorBrandForeground1,
    marginBottom: '16px',
  },
  dropText: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '8px',
  },
  dropSubtext: {
    fontSize: '13px',
    color: tokens.colorNeutralForeground3,
  },
  hiddenInput: {
    display: 'none',
  },
  fileInfo: {
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderRadius('8px'),
    marginTop: '16px',
  },
  fileInfoRow: {
    display: 'flex',
    justifyContent: 'space-between',
    marginBottom: '8px',
    fontSize: '14px',
  },
  fileInfoLabel: {
    color: tokens.colorNeutralForeground3,
  },
  fileInfoValue: {
    fontWeight: tokens.fontWeightSemibold,
  },
  warningList: {
    marginTop: '12px',
    fontSize: '13px',
    color: tokens.colorPaletteYellowForeground2,
  },
  trackList: {
    marginTop: '16px',
    maxHeight: '200px',
    overflowY: 'auto',
  },
  trackItem: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('8px'),
    ...shorthands.padding('8px'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('4px'),
    marginBottom: '4px',
  },
  trackIcon: {
    color: tokens.colorBrandForeground1,
  },
  spinnerContainer: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    ...shorthands.gap('16px'),
    ...shorthands.padding('40px'),
  },
});

interface GPImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImport: (project: MaestroProject) => void;
}

interface ParseResult {
  project: MaestroProject;
  version: string;
  warnings: string[];
}

export function GPImportDialog({ open, onOpenChange, onImport }: GPImportDialogProps) {
  const styles = useStyles();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [parseResult, setParseResult] = useState<ParseResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleFile = useCallback(async (file: File) => {
    setIsLoading(true);
    setError(null);
    setParseResult(null);

    try {
      const arrayBuffer = await file.arrayBuffer();
      const parser = new GuitarProParser();
      const result = await parser.parse(arrayBuffer);

      setParseResult({
        project: result.project,
        version: result.version,
        warnings: result.warnings || [],
      });
    } catch (err: any) {
      setError(err.message || 'Failed to parse Guitar Pro file');
    } finally {
      setIsLoading(false);
    }
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragging(false);

      const file = e.dataTransfer.files[0];
      if (file && (file.name.endsWith('.gp') || file.name.endsWith('.gp5') || file.name.endsWith('.gpx'))) {
        handleFile(file);
      } else {
        setError('Please drop a Guitar Pro file (.gp, .gp5, or .gpx)');
      }
    },
    [handleFile]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) {
        handleFile(file);
      }
    },
    [handleFile]
  );

  const handleImport = () => {
    if (parseResult) {
      onImport(parseResult.project);
      onOpenChange(false);
      setParseResult(null);
    }
  };

  const handleClose = () => {
    onOpenChange(false);
    setParseResult(null);
    setError(null);
  };

  const tracks = parseResult?.project.project.composition.tracks || [];

  return (
    <Dialog open={open} onOpenChange={(_, data) => data.open ? null : handleClose()}>
      <DialogSurface style={{ maxWidth: '500px' }}>
        <DialogBody>
          <DialogTitle>Import Guitar Pro File</DialogTitle>
          <DialogContent>
            {isLoading ? (
              <div className={styles.spinnerContainer}>
                <Spinner size="large" />
                <div>Parsing Guitar Pro file...</div>
              </div>
            ) : parseResult ? (
              <>
                <MessageBar intent="success">
                  <MessageBarBody>
                    <MessageBarTitle>File parsed successfully!</MessageBarTitle>
                    Ready to import "{parseResult.project.project.metadata.title}"
                  </MessageBarBody>
                </MessageBar>

                <div className={styles.fileInfo}>
                  <div className={styles.fileInfoRow}>
                    <span className={styles.fileInfoLabel}>Title:</span>
                    <span className={styles.fileInfoValue}>
                      {parseResult.project.project.metadata.title}
                    </span>
                  </div>
                  <div className={styles.fileInfoRow}>
                    <span className={styles.fileInfoLabel}>Artist:</span>
                    <span className={styles.fileInfoValue}>
                      {parseResult.project.project.metadata.artist || 'Unknown'}
                    </span>
                  </div>
                  <div className={styles.fileInfoRow}>
                    <span className={styles.fileInfoLabel}>Format:</span>
                    <span className={styles.fileInfoValue}>{parseResult.version}</span>
                  </div>
                  <div className={styles.fileInfoRow}>
                    <span className={styles.fileInfoLabel}>Tempo:</span>
                    <span className={styles.fileInfoValue}>
                      {parseResult.project.project.metadata.tempo} BPM
                    </span>
                  </div>
                  <div className={styles.fileInfoRow}>
                    <span className={styles.fileInfoLabel}>Tracks:</span>
                    <span className={styles.fileInfoValue}>{tracks.length}</span>
                  </div>
                </div>

                {tracks.length > 0 && (
                  <div className={styles.trackList}>
                    {tracks.map((track: any, i: number) => (
                      <div key={track.id || i} className={styles.trackItem}>
                        <MusicNote2Regular className={styles.trackIcon} />
                        <span>{track.name || `Track ${i + 1}`}</span>
                      </div>
                    ))}
                  </div>
                )}

                {parseResult.warnings.length > 0 && (
                  <div className={styles.warningList}>
                    <WarningRegular style={{ verticalAlign: 'middle', marginRight: '4px' }} />
                    {parseResult.warnings.map((w, i) => (
                      <div key={i}>• {w}</div>
                    ))}
                  </div>
                )}
              </>
            ) : (
              <>
                {error && (
                  <MessageBar intent="error" style={{ marginBottom: '16px' }}>
                    <MessageBarBody>
                      <MessageBarTitle>Import Error</MessageBarTitle>
                      {error}
                    </MessageBarBody>
                  </MessageBar>
                )}

                <div
                  className={`${styles.dropZone} ${isDragging ? styles.dropZoneActive : ''}`}
                  onDrop={handleDrop}
                  onDragOver={handleDragOver}
                  onDragLeave={handleDragLeave}
                  onClick={() => fileInputRef.current?.click()}
                >
                  <ArrowUploadRegular className={styles.dropIcon} />
                  <div className={styles.dropText}>
                    Drop a Guitar Pro file here
                  </div>
                  <div className={styles.dropSubtext}>
                    or click to browse (.gp, .gp5, .gpx)
                  </div>
                </div>

                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".gp,.gp5,.gp4,.gp3,.gpx"
                  className={styles.hiddenInput}
                  onChange={handleFileSelect}
                />
              </>
            )}
          </DialogContent>
          <DialogActions>
            <Button appearance="secondary" onClick={handleClose}>
              Cancel
            </Button>
            {parseResult && (
              <Button
                appearance="primary"
                icon={<CheckmarkRegular />}
                onClick={handleImport}
              >
                Import
              </Button>
            )}
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
