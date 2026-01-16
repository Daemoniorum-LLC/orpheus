/**
 * Keyboard Shortcuts Dialog - Help modal showing all shortcuts
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
  Button,
} from '@persona-framework/ui';
import { Keyboard } from 'lucide-react';

interface Shortcut {
  description: string;
  keys: string[];
}

interface ShortcutSection {
  title: string;
  shortcuts: Shortcut[];
}

const shortcuts: ShortcutSection[] = [
  {
    title: 'Mode Navigation',
    shortcuts: [
      { description: 'Switch to Compose Mode', keys: ['Ctrl', '1'] },
      { description: 'Switch to Record Mode', keys: ['Ctrl', '2'] },
      { description: 'Switch to Mix Mode', keys: ['Ctrl', '3'] },
      { description: 'Switch to Master Mode', keys: ['Ctrl', '4'] },
      { description: 'Switch to Practice Mode', keys: ['Ctrl', '5'] },
      { description: 'Switch to Distribute Mode', keys: ['Ctrl', '6'] },
    ],
  },
  {
    title: 'General',
    shortcuts: [
      { description: 'Show Keyboard Shortcuts', keys: ['?'] },
      { description: 'Command Palette', keys: ['Ctrl', 'K'] },
      { description: 'Toggle AI Assistant', keys: ['Ctrl', '.'] },
      { description: 'Save Project', keys: ['Ctrl', 'S'] },
      { description: 'Export Project', keys: ['Ctrl', 'E'] },
    ],
  },
  {
    title: 'Playback Controls',
    shortcuts: [
      { description: 'Play/Pause', keys: ['Space'] },
      { description: 'Stop', keys: ['Esc'] },
      { description: 'Toggle Metronome', keys: ['M'] },
      { description: 'Toggle Loop', keys: ['L'] },
    ],
  },
  {
    title: 'Recording',
    shortcuts: [
      { description: 'Start/Stop Recording', keys: ['R'] },
      { description: 'Pause Recording', keys: ['P'] },
    ],
  },
];

interface KeyboardShortcutsDialogProps {
  open: boolean;
  onClose: () => void;
}

export function KeyboardShortcutsDialog({ open, onClose }: KeyboardShortcutsDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <DialogContent className="max-w-[600px] w-[95vw] sm:w-auto max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Keyboard className="h-5 w-5" />
            <span>Keyboard Shortcuts</span>
          </DialogTitle>
          <DialogDescription className="sr-only">
            List of all available keyboard shortcuts
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-6 py-4">
          {shortcuts.map((section) => (
            <div key={section.title} className="flex flex-col gap-3">
              <h3 className="text-sm font-semibold text-primary">
                {section.title}
              </h3>
              {section.shortcuts.map((shortcut) => (
                <div
                  key={shortcut.description}
                  className="flex justify-between items-center px-3 py-2 bg-secondary rounded"
                >
                  <span className="text-sm">{shortcut.description}</span>
                  <div className="flex gap-1">
                    {shortcut.keys.map((key, index) => (
                      <span key={index} className="flex items-center">
                        <kbd className="px-2 py-1 bg-background border border-border rounded text-xs font-mono font-semibold text-muted-foreground shadow-sm">
                          {key}
                        </kbd>
                        {index < shortcut.keys.length - 1 && (
                          <span className="mx-1 text-muted-foreground">+</span>
                        )}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          ))}
        </div>

        <DialogFooter>
          <Button onClick={onClose}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
