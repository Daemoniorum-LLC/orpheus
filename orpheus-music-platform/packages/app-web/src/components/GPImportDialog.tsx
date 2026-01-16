/**
 * Guitar Pro Import Dialog
 * Handles importing GP files into the Tab Editor
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@persona-framework/ui';
import { Button } from '@persona-framework/ui';
import { Alert, AlertDescription, AlertTitle } from '@persona-framework/ui';
import { Upload, Music, Check, AlertTriangle, Loader2 } from 'lucide-react';
import { useState, useRef, useCallback } from 'react';
import { GuitarProParser } from '@orpheus/guitar-pro-parser';
import type { MaestroProject } from '@orpheus/shared-types';
import { cn } from '../lib/utils';

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
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && handleClose()}>
      <DialogContent className="max-w-[500px] w-[95vw] sm:w-auto max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Import Guitar Pro File</DialogTitle>
        </DialogHeader>

        {isLoading ? (
          <div className="flex flex-col items-center gap-4 py-10">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
            <div>Parsing Guitar Pro file...</div>
          </div>
        ) : parseResult ? (
          <>
            <Alert variant="default" className="bg-green-500/10 border-green-500/30">
              <Check className="h-4 w-4 text-green-500" />
              <AlertTitle>File parsed successfully!</AlertTitle>
              <AlertDescription>
                Ready to import "{parseResult.project.project.metadata.title}"
              </AlertDescription>
            </Alert>

            <div className="p-4 bg-muted rounded-lg mt-4">
              <div className="flex justify-between mb-2 text-sm">
                <span className="text-muted-foreground">Title:</span>
                <span className="font-semibold">
                  {parseResult.project.project.metadata.title}
                </span>
              </div>
              <div className="flex justify-between mb-2 text-sm">
                <span className="text-muted-foreground">Artist:</span>
                <span className="font-semibold">
                  {parseResult.project.project.metadata.artist || 'Unknown'}
                </span>
              </div>
              <div className="flex justify-between mb-2 text-sm">
                <span className="text-muted-foreground">Format:</span>
                <span className="font-semibold">{parseResult.version}</span>
              </div>
              <div className="flex justify-between mb-2 text-sm">
                <span className="text-muted-foreground">Tempo:</span>
                <span className="font-semibold">
                  {parseResult.project.project.metadata.tempo} BPM
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Tracks:</span>
                <span className="font-semibold">{tracks.length}</span>
              </div>
            </div>

            {tracks.length > 0 && (
              <div className="mt-4 max-h-[200px] overflow-y-auto">
                {tracks.map((track: any, i: number) => (
                  <div key={track.id || i} className="flex items-center gap-2 p-2 bg-background rounded mb-1">
                    <Music className="h-4 w-4 text-primary" />
                    <span>{track.name || `Track ${i + 1}`}</span>
                  </div>
                ))}
              </div>
            )}

            {parseResult.warnings.length > 0 && (
              <div className="mt-3 text-[13px] text-yellow-600">
                <AlertTriangle className="inline h-4 w-4 mr-1 align-middle" />
                {parseResult.warnings.map((w, i) => (
                  <div key={i}>• {w}</div>
                ))}
              </div>
            )}
          </>
        ) : (
          <>
            {error && (
              <Alert variant="destructive" className="mb-4">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle>Import Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            <div
              className={cn(
                'border-2 border-dashed rounded-lg p-10 text-center cursor-pointer transition-all',
                'bg-background hover:bg-muted hover:border-primary',
                isDragging && 'bg-primary/10 border-primary'
              )}
              onDrop={handleDrop}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onClick={() => fileInputRef.current?.click()}
            >
              <Upload className="h-12 w-12 text-primary mx-auto mb-4" />
              <div className="text-base font-semibold mb-2">
                Drop a Guitar Pro file here
              </div>
              <div className="text-sm text-muted-foreground">
                or click to browse (.gp, .gp5, .gpx)
              </div>
            </div>

            <input
              ref={fileInputRef}
              type="file"
              accept=".gp,.gp5,.gp4,.gp3,.gpx"
              className="hidden"
              onChange={handleFileSelect}
            />
          </>
        )}

        <DialogFooter>
          <Button variant="secondary" onClick={handleClose}>
            Cancel
          </Button>
          {parseResult && (
            <Button onClick={handleImport}>
              <Check className="h-4 w-4 mr-2" />
              Import
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
