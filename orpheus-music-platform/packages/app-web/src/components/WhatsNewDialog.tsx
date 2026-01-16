/**
 * What's New Dialog - Shows changelog and new features
 * Displays on first launch after an update
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
import { Sparkles, Music, Palette, Zap, Bug, Wrench } from 'lucide-react';
import { cn } from '../lib/utils';
import { safeStorage } from '../utils/storage';

// Current version
export const APP_VERSION = '1.2.0';
const VERSION_STORAGE_KEY = 'orpheus-last-seen-version';

export interface ChangelogEntry {
  version: string;
  date: string;
  title: string;
  changes: {
    type: 'feature' | 'improvement' | 'fix' | 'breaking';
    text: string;
  }[];
}

// Changelog data
export const changelog: ChangelogEntry[] = [
  {
    version: '1.2.0',
    date: 'December 2025',
    title: 'Interactive Onboarding & Sample Projects',
    changes: [
      { type: 'feature', text: 'Sample projects - Try Blues, Folk, and Rock guitar patterns instantly' },
      { type: 'feature', text: 'Interactive onboarding with mode exploration' },
      { type: 'feature', text: 'Resizable AI Assistant panel (drag to adjust width)' },
      { type: 'improvement', text: 'Enhanced splash screen with clear CTAs and visual hierarchy' },
      { type: 'improvement', text: 'Typing animation for AI responses' },
      { type: 'fix', text: 'Mix mode now shows actual project tracks instead of fake data' },
    ],
  },
  {
    version: '1.1.0',
    date: 'November 2025',
    title: 'UI Modernization',
    changes: [
      { type: 'feature', text: 'New unified design system with persona-framework/ui' },
      { type: 'feature', text: 'Keyboard navigation for mode selector' },
      { type: 'improvement', text: 'Responsive layout with automatic sidebar collapse' },
      { type: 'improvement', text: 'Better loading skeletons for each mode' },
      { type: 'fix', text: 'Replaced browser confirm dialogs with proper modals' },
    ],
  },
  {
    version: '1.0.0',
    date: 'October 2025',
    title: 'Initial Release',
    changes: [
      { type: 'feature', text: 'Guitar Pro import (.gp3, .gp4, .gp5, .gpx, .gp)' },
      { type: 'feature', text: '6 production modes: Compose, Record, Mix, Master, Practice, Distribute' },
      { type: 'feature', text: 'AI Assistant with context-aware personas' },
      { type: 'feature', text: 'Auto-save with recovery' },
      { type: 'feature', text: 'Command palette (Ctrl+K)' },
    ],
  },
];

// Icon mapping for change types
const changeTypeConfig = {
  feature: { icon: Sparkles, color: 'text-green-500', bg: 'bg-green-500/10', label: 'New' },
  improvement: { icon: Zap, color: 'text-blue-500', bg: 'bg-blue-500/10', label: 'Improved' },
  fix: { icon: Bug, color: 'text-orange-500', bg: 'bg-orange-500/10', label: 'Fixed' },
  breaking: { icon: Wrench, color: 'text-red-500', bg: 'bg-red-500/10', label: 'Breaking' },
};

interface WhatsNewDialogProps {
  open: boolean;
  onClose: () => void;
  showAllVersions?: boolean;
}

export function WhatsNewDialog({ open, onClose, showAllVersions = false }: WhatsNewDialogProps) {
  const handleClose = () => {
    // Mark current version as seen
    safeStorage.setItem(VERSION_STORAGE_KEY, APP_VERSION);
    onClose();
  };

  // Show only latest or all versions
  const versionsToShow = showAllVersions ? changelog : [changelog[0]];

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && handleClose()}>
      <DialogContent className="max-w-[550px] max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-primary" />
            What's New in Orpheus
          </DialogTitle>
          <DialogDescription>
            {showAllVersions
              ? 'Full changelog for all versions'
              : `Version ${APP_VERSION} is here with exciting new features!`}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto py-4 space-y-6">
          {versionsToShow.map((entry) => (
            <div key={entry.version} className="space-y-3">
              {/* Version header */}
              <div className="flex items-center justify-between">
                <div>
                  <span className="text-lg font-semibold">v{entry.version}</span>
                  <span className="text-sm text-muted-foreground ml-2">— {entry.title}</span>
                </div>
                <span className="text-xs text-muted-foreground">{entry.date}</span>
              </div>

              {/* Changes list */}
              <div className="space-y-2">
                {entry.changes.map((change, i) => {
                  const config = changeTypeConfig[change.type];
                  const Icon = config.icon;

                  return (
                    <div
                      key={i}
                      className="flex items-start gap-3 text-sm"
                    >
                      <div className={cn('p-1 rounded', config.bg)}>
                        <Icon className={cn('h-3.5 w-3.5', config.color)} />
                      </div>
                      <div className="flex-1">
                        <span className={cn('text-xs font-medium mr-2', config.color)}>
                          {config.label}
                        </span>
                        <span className="text-foreground">{change.text}</span>
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* Divider between versions */}
              {versionsToShow.indexOf(entry) < versionsToShow.length - 1 && (
                <div className="border-t border-border pt-4" />
              )}
            </div>
          ))}
        </div>

        <DialogFooter className="flex-row justify-between sm:justify-between">
          {!showAllVersions && changelog.length > 1 && (
            <Button variant="ghost" size="sm" onClick={() => window.open('#changelog', '_self')}>
              View full changelog
            </Button>
          )}
          <Button onClick={handleClose}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/**
 * Hook to check if user should see What's New dialog
 */
export function useWhatsNew() {
  const lastSeenVersion = safeStorage.getItem(VERSION_STORAGE_KEY);
  const shouldShow = lastSeenVersion !== APP_VERSION;

  const markAsSeen = () => {
    safeStorage.setItem(VERSION_STORAGE_KEY, APP_VERSION);
  };

  return {
    shouldShowWhatsNew: shouldShow,
    currentVersion: APP_VERSION,
    lastSeenVersion,
    markAsSeen,
  };
}
