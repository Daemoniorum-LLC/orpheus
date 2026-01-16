/**
 * Drag & Drop Zone - Global file drop handler
 */

import { useEffect, useState } from 'react';
import { makeStyles, shorthands, tokens } from '@fluentui/react-components';
import { ArrowUpload24Regular } from '@fluentui/react-icons';
import { useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { showSuccess, showError, showInfo } from '../services/toast';

const useStyles = makeStyles({
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    backdropFilter: 'blur(4px)',
    zIndex: 9999,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexDirection: 'column',
    ...shorthands.gap('24px'),
    pointerEvents: 'none',
    opacity: 0,
    transition: 'opacity 0.2s ease',
  },
  overlayActive: {
    opacity: 1,
    pointerEvents: 'auto',
  },
  dropArea: {
    ...shorthands.border('4px', 'dashed', tokens.colorBrandBackground),
    ...shorthands.borderRadius('16px'),
    ...shorthands.padding('60px', '80px'),
    backgroundColor: 'rgba(255, 255, 255, 0.05)',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    ...shorthands.gap('16px'),
  },
  icon: {
    fontSize: '64px',
    color: tokens.colorBrandBackground,
  },
  title: {
    fontSize: '24px',
    fontWeight: tokens.fontWeightSemibold,
    color: '#ffffff',
  },
  subtitle: {
    fontSize: '14px',
    color: '#cccccc',
  },
});

export function DragDropZone() {
  const styles = useStyles();
  const [isDragging, setIsDragging] = useState(false);
  const { setProject, setRawFileBuffer, setIsLoading, setLoadingMessage } = useAppStore();

  useEffect(() => {
    let dragCounter = 0;

    const handleDragEnter = (e: DragEvent) => {
      e.preventDefault();
      dragCounter++;
      if (e.dataTransfer?.types.includes('Files')) {
        setIsDragging(true);
      }
    };

    const handleDragLeave = (e: DragEvent) => {
      e.preventDefault();
      dragCounter--;
      if (dragCounter === 0) {
        setIsDragging(false);
      }
    };

    const handleDragOver = (e: DragEvent) => {
      e.preventDefault();
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'copy';
      }
    };

    const handleDrop = async (e: DragEvent) => {
      e.preventDefault();
      dragCounter = 0;
      setIsDragging(false);

      const files = e.dataTransfer?.files;
      if (!files || files.length === 0) return;

      const file = files[0];

      try {
        console.log('[DragDrop] Importing file:', file.name);
        setIsLoading(true);
        setLoadingMessage(`Importing ${file.name}...`);
        showInfo(`Importing ${file.name}...`);

        const result = await importFile(file);

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

          console.log('[DragDrop] Import successful:', result);
        } else {
          showError(result.error || 'Failed to import file', 6000);
          console.error('[DragDrop] Import failed:', result.error);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error';
        showError(`Failed to import file: ${message}`, 6000);
        console.error('[DragDrop] Import error:', error);
      } finally {
        setIsLoading(false);
        setLoadingMessage('');
      }
    };

    window.addEventListener('dragenter', handleDragEnter);
    window.addEventListener('dragleave', handleDragLeave);
    window.addEventListener('dragover', handleDragOver);
    window.addEventListener('drop', handleDrop);

    return () => {
      window.removeEventListener('dragenter', handleDragEnter);
      window.removeEventListener('dragleave', handleDragLeave);
      window.removeEventListener('dragover', handleDragOver);
      window.removeEventListener('drop', handleDrop);
    };
  }, [setProject, setRawFileBuffer, setIsLoading, setLoadingMessage]);

  if (!isDragging) return null;

  return (
    <div className={`${styles.overlay} ${isDragging ? styles.overlayActive : ''}`}>
      <div className={styles.dropArea}>
        <ArrowUpload24Regular className={styles.icon} />
        <div className={styles.title}>Drop your file here</div>
        <div className={styles.subtitle}>
          Supports Guitar Pro (.gp3, .gp4, .gp5, .gp6, .gp7) and Maestro projects
        </div>
      </div>
    </div>
  );
}
