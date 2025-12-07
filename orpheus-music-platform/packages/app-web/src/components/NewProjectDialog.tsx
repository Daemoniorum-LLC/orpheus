/**
 * New Project Dialog
 * Select a template and create a new project
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  Button,
  Input,
  Label,
  Card,
  CardContent,
} from '@persona-framework/ui';
import { FilePlus } from 'lucide-react';
import { useState } from 'react';
import { PROJECT_TEMPLATES, createProjectFromTemplate } from '../services/project-templates';
import { useAppStore } from '../store/app-store';
import { showSuccess } from '../services/toast';

export interface NewProjectDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewProjectDialog({ open, onOpenChange }: NewProjectDialogProps) {
  const { setProject } = useAppStore();
  const [selectedTemplate, setSelectedTemplate] = useState(PROJECT_TEMPLATES[0].id);
  const [title, setTitle] = useState('');
  const [artist, setArtist] = useState('');
  const [touched, setTouched] = useState(false);

  // Validation
  const trimmedTitle = title.trim();
  const isTitleValid = trimmedTitle.length >= 1;
  const isTitleTooLong = trimmedTitle.length > 100;
  const showTitleError = touched && !isTitleValid;
  const canCreate = isTitleValid && !isTitleTooLong;

  const handleCreate = () => {
    if (!canCreate) {
      setTouched(true);
      return;
    }

    const template = PROJECT_TEMPLATES.find((t) => t.id === selectedTemplate);
    if (!template) return;

    const project = createProjectFromTemplate(
      template,
      trimmedTitle,
      artist.trim()
    );

    setProject(project);
    showSuccess(`Created new project: ${trimmedTitle}`, 3000);

    // Reset form
    setTitle('');
    setArtist('');
    setTouched(false);
    setSelectedTemplate(PROJECT_TEMPLATES[0].id);
    onOpenChange(false);
  };

  const selectedTemplateData = PROJECT_TEMPLATES.find((t) => t.id === selectedTemplate);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-[800px] w-[95vw] sm:w-auto max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create New Project</DialogTitle>
        </DialogHeader>

        <div className="flex flex-col gap-4 py-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="project-title">
              Project Title <span className="text-destructive">*</span>
            </Label>
            <Input
              id="project-title"
              placeholder="My Awesome Song"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              onBlur={() => setTouched(true)}
              maxLength={100}
              className={showTitleError ? 'border-destructive focus-visible:ring-destructive' : ''}
              aria-invalid={showTitleError}
              aria-describedby={showTitleError ? 'title-error' : 'title-counter'}
              required
            />
            <div className="flex justify-between items-center">
              <div>
                {showTitleError && (
                  <p id="title-error" className="text-xs text-destructive">
                    Project title is required
                  </p>
                )}
                {isTitleTooLong && (
                  <p className="text-xs text-destructive">
                    Title must be 100 characters or less
                  </p>
                )}
              </div>
              <p id="title-counter" className={`text-xs ${trimmedTitle.length > 80 ? 'text-yellow-500' : 'text-muted-foreground'}`}>
                {trimmedTitle.length}/100
              </p>
            </div>
          </div>

          <div className="flex flex-col gap-2">
            <Label htmlFor="project-artist">Artist (optional)</Label>
            <Input
              id="project-artist"
              placeholder="Your Name"
              value={artist}
              onChange={(e) => setArtist(e.target.value)}
              maxLength={100}
            />
          </div>

          <div className="flex flex-col gap-2">
            <Label>Select Template</Label>
            <div className="grid grid-cols-[repeat(auto-fill,minmax(250px,1fr))] gap-3 mt-3">
              {PROJECT_TEMPLATES.map((template) => (
                <Card
                  key={template.id}
                  className={`cursor-pointer p-4 border-2 transition-colors ${
                    selectedTemplate === template.id
                      ? 'border-primary bg-primary/5'
                      : 'border-transparent hover:bg-secondary'
                  }`}
                  onClick={() => setSelectedTemplate(template.id)}
                >
                  <CardContent className="p-0">
                    <div className="text-base font-semibold mb-1">{template.name}</div>
                    <div className="text-xs text-primary mb-2">{template.genre}</div>
                    <div className="text-[13px] text-muted-foreground leading-snug">
                      {template.description}
                    </div>
                    <div className="text-[11px] text-muted-foreground mt-2">
                      {template.tempo} BPM • {template.key} • {template.timeSignature.numerator}/
                      {template.timeSignature.denominator} • {template.trackCount} track
                      {template.trackCount > 1 ? 's' : ''}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>

          {selectedTemplateData && (
            <div className="text-sm text-muted-foreground">
              <strong>Tracks included:</strong>{' '}
              {selectedTemplateData.trackNames.join(', ')}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={!canCreate}>
            <FilePlus className="h-4 w-4 mr-2" />
            Create Project
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
