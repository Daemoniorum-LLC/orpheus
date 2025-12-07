/**
 * New Project Dialog
 * Select a template and create a new project
 */

import {
  Dialog,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Button,
  Input,
  Label,
  makeStyles,
  shorthands,
  tokens,
  Card,
} from '@fluentui/react-components';
import { DocumentAdd24Regular } from '@fluentui/react-icons';
import { useState } from 'react';
import { PROJECT_TEMPLATES, createProjectFromTemplate } from '../services/project-templates';
import { useAppStore } from '../store/app-store';
import { showSuccess } from '../services/toast';

const useStyles = makeStyles({
  content: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
  templateGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))',
    ...shorthands.gap('12px'),
    marginTop: '12px',
  },
  templateCard: {
    cursor: 'pointer',
    ...shorthands.padding('16px'),
    ...shorthands.border('2px', 'solid', 'transparent'),
    transition: 'all 0.2s ease',
    '&:hover': {
      backgroundColor: tokens.colorNeutralBackground2,
    },
  },
  templateCardSelected: {
    ...shorthands.border('2px', 'solid', tokens.colorBrandBackground),
    backgroundColor: tokens.colorNeutralBackground2,
  },
  templateName: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '4px',
  },
  templateGenre: {
    fontSize: '12px',
    color: tokens.colorBrandBackground,
    marginBottom: '8px',
  },
  templateDesc: {
    fontSize: '13px',
    color: tokens.colorNeutralForeground2,
    lineHeight: '1.4',
  },
  templateSettings: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    marginTop: '8px',
  },
  formSection: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('8px'),
  },
});

export interface NewProjectDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function NewProjectDialog({ open, onOpenChange }: NewProjectDialogProps) {
  const styles = useStyles();
  const { setProject } = useAppStore();
  const [selectedTemplate, setSelectedTemplate] = useState(PROJECT_TEMPLATES[0].id);
  const [title, setTitle] = useState('');
  const [artist, setArtist] = useState('');

  const handleCreate = () => {
    const template = PROJECT_TEMPLATES.find((t) => t.id === selectedTemplate);
    if (!template) return;

    const project = createProjectFromTemplate(
      template,
      title || 'Untitled Project',
      artist
    );

    setProject(project);
    showSuccess(`Created new project: ${title || 'Untitled Project'}`, 3000);
    console.log('[NewProject] Created project from template:', template.name);

    // Reset form
    setTitle('');
    setArtist('');
    setSelectedTemplate(PROJECT_TEMPLATES[0].id);
    onOpenChange(false);
  };

  const selectedTemplateData = PROJECT_TEMPLATES.find((t) => t.id === selectedTemplate);

  return (
    <Dialog open={open} onOpenChange={(_, data) => onOpenChange(data.open)}>
      <DialogSurface style={{ maxWidth: '800px' }}>
        <DialogBody>
          <DialogTitle>Create New Project</DialogTitle>
          <DialogContent className={styles.content}>
            <div className={styles.formSection}>
              <Label htmlFor="project-title">Project Title</Label>
              <Input
                id="project-title"
                placeholder="My Awesome Song"
                value={title}
                onChange={(_, data) => setTitle(data.value)}
              />
            </div>

            <div className={styles.formSection}>
              <Label htmlFor="project-artist">Artist (Optional)</Label>
              <Input
                id="project-artist"
                placeholder="Your Name"
                value={artist}
                onChange={(_, data) => setArtist(data.value)}
              />
            </div>

            <div className={styles.formSection}>
              <Label>Select Template</Label>
              <div className={styles.templateGrid}>
                {PROJECT_TEMPLATES.map((template) => (
                  <Card
                    key={template.id}
                    className={`${styles.templateCard} ${
                      selectedTemplate === template.id ? styles.templateCardSelected : ''
                    }`}
                    onClick={() => setSelectedTemplate(template.id)}
                  >
                    <div className={styles.templateName}>{template.name}</div>
                    <div className={styles.templateGenre}>{template.genre}</div>
                    <div className={styles.templateDesc}>{template.description}</div>
                    <div className={styles.templateSettings}>
                      {template.tempo} BPM • {template.key} • {template.timeSignature.numerator}/
                      {template.timeSignature.denominator} • {template.trackCount} track
                      {template.trackCount > 1 ? 's' : ''}
                    </div>
                  </Card>
                ))}
              </div>
            </div>

            {selectedTemplateData && (
              <div style={{ fontSize: '14px', color: tokens.colorNeutralForeground2 }}>
                <strong>Tracks included:</strong>{' '}
                {selectedTemplateData.trackNames.join(', ')}
              </div>
            )}
          </DialogContent>
          <DialogActions>
            <Button appearance="secondary" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button appearance="primary" icon={<DocumentAdd24Regular />} onClick={handleCreate}>
              Create Project
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
