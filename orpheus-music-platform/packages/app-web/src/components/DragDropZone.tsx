/**
 * Drag & Drop Zone - Global file drop handler
 */

import { useEffect, useState, useRef, useCallback } from 'react';
import { Upload } from 'lucide-react';
import { useAppStore } from '../store/app-store';
import { importFile } from '../services/file-import';
import { showSuccess, showError, showInfo, showWarning } from '../services/toast';
import { cn } from '../lib/utils';
import { ConfirmDialog } from './ConfirmDialog';

// Maximum file size: 50MB
const MAX_FILE_SIZE = 50 * 1024 * 1024;

export function DragDropZone() {
  const [isDragging, setIsDragging] = useState(false);
  const [confirmDialogOpen, setConfirmDialogOpen] = useState(false);
  const pendingFileRef = useRef<File | null>(null);
  const { project, setProject, setRawFileBuffer, setIsLoading, setLoadingMessage } = useAppStore();

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

      // Warn if multiple files were dropped
      if (files.length > 1) {
        showWarning(`Only one file can be imported at a time. Using: ${files[0].name}`, 6000);
      }

      const file = files[0];

      // Check file size
      if (file.size > MAX_FILE_SIZE) {
        showError(`File too large. Maximum size is 50MB. Your file: ${Math.round(file.size / 1024 / 1024)}MB`, 6000);
        return;
      }

      // If there's an existing project, confirm before replacing
      if (project) {
        pendingFileRef.current = file;
        setConfirmDialogOpen(true);
        return;
      }

      // No existing project, proceed directly
      processFile(file);
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
  }, [project, setProject, setRawFileBuffer, setIsLoading, setLoadingMessage]);

  // Process file import
  const processFile = useCallback(async (file: File) => {
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
      console.error('[DragDrop] Import error:', error);
    } finally {
      setIsLoading(false);
      setLoadingMessage('');
    }
  }, [setProject, setRawFileBuffer, setIsLoading, setLoadingMessage]);

  // Handle confirmation to replace project
  const handleConfirmReplace = () => {
    setConfirmDialogOpen(false);
    if (pendingFileRef.current) {
      processFile(pendingFileRef.current);
      pendingFileRef.current = null;
    }
  };

  const handleCancelReplace = () => {
    setConfirmDialogOpen(false);
    pendingFileRef.current = null;
  };

  return (
    <>
      {/* Screen reader announcement for drag state */}
      <div
        role="status"
        aria-live="polite"
        aria-atomic="true"
        className="sr-only"
      >
        {isDragging && 'Drop zone active. Release to import file, or press Escape to cancel. Alternatively, use Ctrl+O to open files.'}
      </div>

      {/* Drag overlay */}
      {isDragging && (
        <div
          className={cn(
            'fixed inset-0 bg-black/80 backdrop-blur-sm z-[9999]',
            'flex items-center justify-center flex-col gap-6',
            'pointer-events-auto'
          )}
          role="dialog"
          aria-modal="true"
          aria-label="File drop zone"
        >
          <div className="border-4 border-dashed border-primary rounded-2xl py-16 px-20 bg-white/5 flex flex-col items-center gap-4">
            <Upload className="h-16 w-16 text-primary" aria-hidden="true" />
            <div className="text-2xl font-semibold text-white">Drop your file here</div>
            <div className="text-sm text-gray-400">
              Supports Guitar Pro (.gp3, .gp4, .gp5, .gp6, .gp7) and Orpheus projects (max 50MB)
            </div>
            <div className="text-xs text-gray-500 mt-2">
              Or press <kbd className="px-1.5 py-0.5 bg-white/10 rounded text-white">Ctrl+O</kbd> to open files
            </div>
          </div>
        </div>
      )}

      {/* Confirmation dialog for replacing existing project */}
      <ConfirmDialog
        open={confirmDialogOpen}
        onConfirm={handleConfirmReplace}
        onCancel={handleCancelReplace}
        title="Replace Current Project?"
        message={`Loading "${pendingFileRef.current?.name || 'this file'}" will replace your current project. Any unsaved changes will be lost.\n\nAre you sure you want to continue?`}
        confirmText="Replace Project"
        cancelText="Cancel"
        type="warning"
      />
    </>
  );
}
