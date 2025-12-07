/**
 * Command Palette - Quick action launcher (Cmd+K / Ctrl+K)
 * Professional command palette for power users
 */

import { useState, useEffect, useRef, useMemo } from 'react';
import { Search } from 'lucide-react';
import { useAppStore } from '../store/app-store';

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
      id: 'ai-toggle',
      title: 'Toggle AI Assistant',
      description: 'Show or hide AI assistant panel',
      icon: '🤖',
      shortcut: 'Ctrl+.',
      category: 'ai',
      action: () => { setAIAssistantOpen(true); onClose(); },
      keywords: ['ai', 'assistant', 'help', 'chat', 'toggle'],
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
    <div
      className="fixed inset-0 bg-black/70 flex items-start justify-center pt-[10vh] sm:pt-[15vh] z-[10001] backdrop-blur-sm p-4"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
    >
      <div
        className="w-full sm:w-[600px] max-w-[90vw] bg-background rounded-xl shadow-2xl border border-border overflow-hidden"
        onClick={(e) => e.stopPropagation()}
        role="listbox"
        aria-label="Available commands"
      >
        {/* Search input */}
        <div className="p-3 sm:p-4 border-b border-border">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" aria-hidden="true" />
            <input
              ref={inputRef}
              type="text"
              className="w-full pl-10 pr-4 py-3 bg-secondary border border-border rounded-lg text-base focus:outline-none focus:ring-2 focus:ring-ring"
              placeholder="Type a command or search..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              aria-label="Search commands"
              aria-controls="command-results"
              aria-activedescendant={filteredCommands[activeIndex]?.id}
            />
          </div>
        </div>

        {/* Results */}
        <div id="command-results" className="max-h-[60vh] sm:max-h-[400px] overflow-auto">
          {filteredCommands.length === 0 ? (
            <div className="p-8 text-center text-muted-foreground" role="status">
              <div className="text-2xl mb-2">🔍</div>
              <div className="font-medium mb-1">No commands found for "{search}"</div>
              <div className="text-xs">
                Try searching for: <span className="text-foreground/70">save</span>, <span className="text-foreground/70">export</span>, <span className="text-foreground/70">mix</span>, or <span className="text-foreground/70">undo</span>
              </div>
            </div>
          ) : (
            Object.entries(groupedCommands).map(([category, cmds]) => (
              <div key={category} role="group" aria-label={categoryLabels[category]}>
                <div className="px-4 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider bg-secondary">
                  {categoryLabels[category]}
                </div>
                {cmds.map((cmd) => {
                  const globalIndex = filteredCommands.indexOf(cmd);
                  const isActive = globalIndex === activeIndex;
                  return (
                    <div
                      key={cmd.id}
                      id={cmd.id}
                      role="option"
                      aria-selected={isActive}
                      className={`px-3 sm:px-4 py-3 flex items-center justify-between cursor-pointer transition-colors ${
                        isActive
                          ? 'bg-primary/20 border-l-2 border-l-primary font-medium'
                          : 'hover:bg-muted border-l-2 border-l-transparent'
                      }`}
                      onClick={() => cmd.action()}
                      onMouseEnter={() => setActiveIndex(globalIndex)}
                    >
                      <div className="flex items-center gap-3 flex-1 min-w-0">
                        <span className="text-xl flex-shrink-0" aria-hidden="true">{cmd.icon}</span>
                        <div className="flex flex-col gap-0.5 min-w-0">
                          <div className={`text-sm truncate ${isActive ? 'font-semibold text-primary' : 'font-semibold'}`}>{cmd.title}</div>
                          <div className="text-xs text-muted-foreground truncate hidden sm:block">{cmd.description}</div>
                        </div>
                      </div>
                      {cmd.shortcut && (
                        <div className="text-xs text-muted-foreground px-2 py-1 bg-secondary rounded font-mono hidden sm:block flex-shrink-0">
                          {cmd.shortcut}
                        </div>
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
