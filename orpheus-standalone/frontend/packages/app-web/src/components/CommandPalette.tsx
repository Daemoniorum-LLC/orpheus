/**
 * Command Palette - Quick action launcher (Cmd+K / Ctrl+K)
 * Professional command palette for power users
 */

import { useState, useEffect, useRef, useMemo } from 'react';
import { makeStyles, shorthands, tokens, Input } from '@fluentui/react-components';
import { Search24Regular } from '@fluentui/react-icons';
import { useAppStore } from '../store/app-store';

const useStyles = makeStyles({
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.7)',
    display: 'flex',
    alignItems: 'flex-start',
    justifyContent: 'center',
    paddingTop: '15vh',
    zIndex: 10001,
    backdropFilter: 'blur(4px)',
  },
  palette: {
    width: '600px',
    maxWidth: '90vw',
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRadius('12px'),
    boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
    ...shorthands.overflow('hidden'),
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
  },
  searchContainer: {
    ...shorthands.padding('16px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
  },
  searchInput: {
    width: '100%',
    fontSize: '16px',
  },
  resultsContainer: {
    maxHeight: '400px',
    ...shorthands.overflow('auto'),
  },
  resultItem: {
    ...shorthands.padding('12px', '16px'),
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    cursor: 'pointer',
    ...shorthands.transition('background-color', '150ms'),
    '&:hover': {
      backgroundColor: tokens.colorNeutralBackground2,
    },
  },
  resultItemActive: {
    backgroundColor: tokens.colorBrandBackground2,
    '&:hover': {
      backgroundColor: tokens.colorBrandBackground2,
    },
  },
  resultLeft: {
    display: 'flex',
    alignItems: 'center',
    ...shorthands.gap('12px'),
    flex: 1,
  },
  resultIcon: {
    fontSize: '20px',
  },
  resultText: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('2px'),
  },
  resultTitle: {
    fontSize: '14px',
    fontWeight: 600,
    color: tokens.colorNeutralForeground1,
  },
  resultDescription: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
  },
  resultShortcut: {
    fontSize: '11px',
    color: tokens.colorNeutralForeground2,
    ...shorthands.padding('2px', '6px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.borderRadius('4px'),
    fontFamily: 'monospace',
  },
  emptyState: {
    ...shorthands.padding('32px'),
    textAlign: 'center',
    color: tokens.colorNeutralForeground3,
  },
  categoryHeader: {
    ...shorthands.padding('8px', '16px'),
    fontSize: '11px',
    fontWeight: 600,
    color: tokens.colorNeutralForeground3,
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
    backgroundColor: tokens.colorNeutralBackground2,
  },
});

interface Command {
  id: string;
  title: string;
  description: string;
  icon: string;
  shortcut?: string;
  category: 'mode' | 'file' | 'edit' | 'playback' | 'ai' | 'view';
  action: () => void;
  keywords?: string[];
}

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
}

export function CommandPalette({ open, onClose }: CommandPaletteProps) {
  const styles = useStyles();
  const [search, setSearch] = useState('');
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const {
    setMode,
    setAIAssistantOpen,
    undo,
    redo,
    canUndo,
    canRedo,
  } = useAppStore();

  // Define all commands
  const commands: Command[] = useMemo(() => [
    // Mode switching
    {
      id: 'mode-compose',
      title: 'Switch to Compose Mode',
      description: 'Tab editor with Guitar Pro support',
      icon: '🎼',
      shortcut: 'Ctrl+1',
      category: 'mode',
      action: () => { setMode('compose'); onClose(); },
      keywords: ['compose', 'tab', 'guitar', 'editor'],
    },
    {
      id: 'mode-record',
      title: 'Switch to Record Mode',
      description: 'Multi-track audio recording',
      icon: '🎙️',
      shortcut: 'Ctrl+2',
      category: 'mode',
      action: () => { setMode('record'); onClose(); },
      keywords: ['record', 'audio', 'track', 'mic'],
    },
    {
      id: 'mode-mix',
      title: 'Switch to Mix Mode',
      description: 'Professional mixing console',
      icon: '🎚️',
      shortcut: 'Ctrl+3',
      category: 'mode',
      action: () => { setMode('mix'); onClose(); },
      keywords: ['mix', 'eq', 'compression', 'effects'],
    },
    {
      id: 'mode-master',
      title: 'Switch to Master Mode',
      description: 'AI-powered mastering',
      icon: '✨',
      shortcut: 'Ctrl+4',
      category: 'mode',
      action: () => { setMode('master'); onClose(); },
      keywords: ['master', 'lufs', 'loudness', 'final'],
    },
    {
      id: 'mode-practice',
      title: 'Switch to Practice Mode',
      description: 'Speed trainer and loops',
      icon: '🎸',
      shortcut: 'Ctrl+5',
      category: 'mode',
      action: () => { setMode('practice'); onClose(); },
      keywords: ['practice', 'speed', 'trainer', 'loop'],
    },
    {
      id: 'mode-distribute',
      title: 'Switch to Distribute Mode',
      description: 'Upload to streaming platforms',
      icon: '🌍',
      shortcut: 'Ctrl+6',
      category: 'mode',
      action: () => { setMode('distribute'); onClose(); },
      keywords: ['distribute', 'spotify', 'upload', 'release'],
    },
    // Edit actions
    {
      id: 'edit-undo',
      title: 'Undo',
      description: 'Undo last action',
      icon: '↩️',
      shortcut: 'Ctrl+Z',
      category: 'edit',
      action: () => { if (canUndo()) undo(); onClose(); },
      keywords: ['undo', 'revert'],
    },
    {
      id: 'edit-redo',
      title: 'Redo',
      description: 'Redo last undone action',
      icon: '↪️',
      shortcut: 'Ctrl+Shift+Z',
      category: 'edit',
      action: () => { if (canRedo()) redo(); onClose(); },
      keywords: ['redo', 'restore'],
    },
    // AI
    {
      id: 'ai-open',
      title: 'Open AI Assistant',
      description: 'Get AI help with your project',
      icon: '🤖',
      shortcut: 'Ctrl+K',
      category: 'ai',
      action: () => { setAIAssistantOpen(true); onClose(); },
      keywords: ['ai', 'assistant', 'help', 'chat'],
    },
  ], [setMode, setAIAssistantOpen, undo, redo, canUndo, canRedo, onClose]);

  // Filter commands based on search
  const filteredCommands = useMemo(() => {
    if (!search.trim()) return commands;

    const searchLower = search.toLowerCase();
    return commands.filter(cmd => {
      const matchTitle = cmd.title.toLowerCase().includes(searchLower);
      const matchDescription = cmd.description.toLowerCase().includes(searchLower);
      const matchKeywords = cmd.keywords?.some(k => k.includes(searchLower));
      return matchTitle || matchDescription || matchKeywords;
    });
  }, [commands, search]);

  // Group commands by category
  const groupedCommands = useMemo(() => {
    const groups: Record<string, Command[]> = {};
    filteredCommands.forEach(cmd => {
      if (!groups[cmd.category]) {
        groups[cmd.category] = [];
      }
      groups[cmd.category].push(cmd);
    });
    return groups;
  }, [filteredCommands]);

  const categoryLabels: Record<string, string> = {
    mode: 'Modes',
    file: 'File',
    edit: 'Edit',
    playback: 'Playback',
    ai: 'AI Assistant',
    view: 'View',
  };

  // Focus input when opened
  useEffect(() => {
    if (open) {
      setSearch('');
      setActiveIndex(0);
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  }, [open]);

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!open) return;

      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        setActiveIndex(prev => Math.min(prev + 1, filteredCommands.length - 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setActiveIndex(prev => Math.max(prev - 1, 0));
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (filteredCommands[activeIndex]) {
          filteredCommands[activeIndex].action();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [open, activeIndex, filteredCommands, onClose]);

  if (!open) return null;

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.palette} onClick={(e) => e.stopPropagation()}>
        <div className={styles.searchContainer}>
          <Input
            ref={inputRef}
            className={styles.searchInput}
            placeholder="Type a command or search..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            contentBefore={<Search24Regular />}
            appearance="outline"
          />
        </div>

        <div className={styles.resultsContainer}>
          {filteredCommands.length === 0 ? (
            <div className={styles.emptyState}>
              No commands found for "{search}"
            </div>
          ) : (
            Object.entries(groupedCommands).map(([category, cmds]) => (
              <div key={category}>
                <div className={styles.categoryHeader}>{categoryLabels[category]}</div>
                {cmds.map((cmd) => {
                  const globalIndex = filteredCommands.indexOf(cmd);
                  return (
                    <div
                      key={cmd.id}
                      className={`${styles.resultItem} ${globalIndex === activeIndex ? styles.resultItemActive : ''}`}
                      onClick={() => cmd.action()}
                      onMouseEnter={() => setActiveIndex(globalIndex)}
                    >
                      <div className={styles.resultLeft}>
                        <span className={styles.resultIcon}>{cmd.icon}</span>
                        <div className={styles.resultText}>
                          <div className={styles.resultTitle}>{cmd.title}</div>
                          <div className={styles.resultDescription}>{cmd.description}</div>
                        </div>
                      </div>
                      {cmd.shortcut && (
                        <div className={styles.resultShortcut}>{cmd.shortcut}</div>
                      )}
                    </div>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
