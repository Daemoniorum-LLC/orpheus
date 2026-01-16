import { Button } from '@persona-framework/ui';
import { TypingAnimation } from './TypingAnimation';
import { Send, X, Trash2, GripVertical, MessageCircle } from 'lucide-react';
import { useAppStore, useCurrentMode } from '../store/app-store';
import { useState, useEffect, useRef, useCallback } from 'react';
import { getAIChatClient, type ChatMessage } from '../services/ai-chat';
import { AIQuickActions } from './AIQuickActions';
import { useIsWidescreen } from '../hooks/useMediaQuery';
import { cn } from '../lib/utils';
import { showInfo } from '../services/toast';
import { safeStorage } from '../utils/storage';

const PERSONAS = {
  compose: {
    name: '🎼 Music Theory Tutor',
    description: 'I can help you with chords, scales, progressions, and composition techniques. Ask me about harmony, melody writing, or song structure!',
  },
  record: {
    name: '🎙️ Session Assistant',
    description: 'I\'ll help you set up your recording session and get the best takes. Ask about microphone placement, gain staging, or recording techniques!',
  },
  mix: {
    name: '🎚️ Mixing Engineer',
    description: 'I can suggest EQ, compression, and other mixing techniques for your tracks. Ask me about balance, effects, or troubleshooting mix issues!',
  },
  master: {
    name: '✨ Mastering Engineer',
    description: 'I\'ll help you achieve broadcast-standard loudness and prepare for distribution. Ask about LUFS targets, platform requirements, or mastering chains!',
  },
  practice: {
    name: '🎸 Guitar Coach',
    description: 'I can help you practice techniques, build speed, and improve your playing. Ask me about exercises, practice routines, or overcoming technical challenges!',
  },
  distribute: {
    name: '🌍 Distribution Manager',
    description: 'I can help you prepare your music for release and navigate streaming platforms. Ask about DistroKid, metadata, release strategies, or royalties!',
  },
};

const MIN_WIDTH = 300;
const MAX_WIDTH = 600;
const DEFAULT_WIDTH = 380;
const STORAGE_KEY = 'orpheus-ai-panel-width';

function getStoredWidth(): number {
  const stored = safeStorage.getItem(STORAGE_KEY);
  if (stored) {
    const width = parseInt(stored, 10);
    if (!isNaN(width) && width >= MIN_WIDTH && width <= MAX_WIDTH) {
      return width;
    }
  }
  return DEFAULT_WIDTH;
}

export function AIAssistant() {
  const { aiAssistantOpen, setAIAssistantOpen } = useAppStore();
  const mode = useCurrentMode();
  const [input, setInput] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isTyping, setIsTyping] = useState(false);
  const [panelWidth, setPanelWidth] = useState(getStoredWidth);
  const [isResizing, setIsResizing] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const resizeRef = useRef<HTMLDivElement>(null);
  const chatClient = getAIChatClient();
  const isWidescreen = useIsWidescreen();

  // Track if we've done the initial mobile collapse (only do it once)
  const hasInitializedRef = useRef(false);

  const currentPersona = PERSONAS[mode];

  // Handle resize drag
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
  }, []);

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isResizing) return;

      // Calculate new width (panel is on right, so we subtract from window width)
      const newWidth = window.innerWidth - e.clientX;
      const clampedWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, newWidth));
      setPanelWidth(clampedWidth);
    },
    [isResizing]
  );

  const handleMouseUp = useCallback(() => {
    if (isResizing) {
      setIsResizing(false);
      // Persist to localStorage
      safeStorage.setItem(STORAGE_KEY, String(panelWidth));
    }
  }, [isResizing, panelWidth]);

  // Attach global mouse listeners for resize
  useEffect(() => {
    if (isResizing) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      // Prevent text selection during resize
      document.body.style.userSelect = 'none';
      document.body.style.cursor = 'col-resize';
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.userSelect = '';
      document.body.style.cursor = '';
    };
  }, [isResizing, handleMouseMove, handleMouseUp]);

  // Auto-close AI panel on initial load for smaller screens (once only)
  useEffect(() => {
    if (!hasInitializedRef.current && !isWidescreen && aiAssistantOpen) {
      setAIAssistantOpen(false);
      showInfo('AI Assistant hidden on smaller screens. Use the AI button to reopen.', 4000);
      hasInitializedRef.current = true;
    } else if (isWidescreen) {
      hasInitializedRef.current = true;
    }
  }, [isWidescreen, aiAssistantOpen, setAIAssistantOpen]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  useEffect(() => {
    if (aiAssistantOpen) {
      chatClient.startSession(mode);
      setMessages(chatClient.getMessages());
    }
  }, [mode, aiAssistantOpen, chatClient]);

  const handleSend = async () => {
    if (!input.trim()) return;

    const userInput = input;
    setInput('');
    setIsTyping(true);

    try {
      await chatClient.sendMessage(userInput, mode);
      setMessages(chatClient.getMessages());
    } catch (error) {
      console.error('[AI Assistant] Error:', error);
    } finally {
      setIsTyping(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleClearChat = () => {
    chatClient.clearSession();
    chatClient.startSession(mode);
    setMessages([]);
  };

  const handleQuickAction = (prompt: string) => {
    setInput(prompt);
    setTimeout(() => {
      setIsTyping(true);
      chatClient.sendMessage(prompt, mode)
        .then(() => {
          setMessages(chatClient.getMessages());
        })
        .catch((error) => {
          console.error('[AI Assistant] Error:', error);
        })
        .finally(() => {
          setIsTyping(false);
          setInput('');
        });
    }, 100);
  };

  const formatTime = (timestamp: number): string => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  };

  // Double-click to reset to default width
  const handleDoubleClick = useCallback(() => {
    setPanelWidth(DEFAULT_WIDTH);
    safeStorage.setItem(STORAGE_KEY, String(DEFAULT_WIDTH));
  }, []);

  if (!aiAssistantOpen) {
    return null;
  }

  return (
    <div
      className="bg-secondary border-l border-border flex flex-col relative"
      style={{ width: panelWidth }}
    >
      {/* Resize handle */}
      <div
        ref={resizeRef}
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize AI panel. Drag left or right to adjust width, double-click to reset."
        aria-valuenow={panelWidth}
        aria-valuemin={MIN_WIDTH}
        aria-valuemax={MAX_WIDTH}
        tabIndex={0}
        onMouseDown={handleMouseDown}
        onDoubleClick={handleDoubleClick}
        onKeyDown={(e) => {
          if (e.key === 'ArrowLeft') {
            setPanelWidth((w) => Math.min(MAX_WIDTH, w + 20));
          } else if (e.key === 'ArrowRight') {
            setPanelWidth((w) => Math.max(MIN_WIDTH, w - 20));
          }
        }}
        className={cn(
          'absolute left-0 top-0 bottom-0 w-1 cursor-col-resize group z-10',
          'hover:bg-primary/50 transition-colors focus-visible:bg-primary focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1',
          isResizing && 'bg-primary'
        )}
        title="Drag to resize, double-click to reset"
      >
        {/* Visual grip indicator */}
        <div
          className={cn(
            'absolute left-0 top-1/2 -translate-y-1/2 -translate-x-1/2',
            'flex items-center justify-center w-4 h-8 rounded bg-border',
            'opacity-0 group-hover:opacity-100 transition-opacity',
            isResizing && 'opacity-100 bg-primary'
          )}
          aria-hidden="true"
        >
          <GripVertical className="h-4 w-4 text-muted-foreground" />
        </div>
      </div>

      <div className="px-4 py-4 border-b border-border flex justify-between items-center bg-background">
        <div className="text-base font-semibold">AI Assistant</div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleClearChat}
            title="Clear chat"
          >
            <Trash2 className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setAIAssistantOpen(false)}
            title="Close"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-auto px-4 py-4 flex flex-col gap-4">
        <div className="px-4 py-4 bg-blue-500/10 rounded-lg border border-blue-500/20">
          <div className="text-sm font-semibold mb-2 text-blue-400">
            {currentPersona.name}
          </div>
          <div className="text-xs text-muted-foreground leading-relaxed">
            {currentPersona.description}
          </div>
        </div>

        {messages.length === 0 && <AIQuickActions onActionClick={handleQuickAction} />}

        {messages.length === 0 ? (
          <div className="text-center px-6 py-6 text-muted-foreground">
            <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-primary/10 flex items-center justify-center">
              <MessageCircle className="w-6 h-6 text-primary" />
            </div>
            <div className="text-sm font-semibold mb-1.5">Ready to help!</div>
            <div className="text-xs leading-relaxed max-w-[240px] mx-auto">
              {mode === 'compose' ? 'Get chord suggestions, melody ideas, or help with song structure.' :
               mode === 'record' ? 'Ask about mic placement, gain staging, or recording techniques.' :
               mode === 'mix' ? 'Get advice on EQ, compression, panning, and effects.' :
               mode === 'master' ? 'Learn about loudness, dynamics, and final polish.' :
               mode === 'practice' ? 'Get tips on technique, exercises, and learning strategies.' :
               'Ask about distribution platforms, metadata, and release strategies.'}
            </div>
            <div className="text-[10px] mt-3 text-muted-foreground/60">
              Press <kbd className="px-1 py-0.5 bg-muted rounded text-[9px]">Enter</kbd> to send
            </div>
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            {messages.map((message) => (
              <div
                key={message.id}
                className={`flex flex-col gap-1.5 max-w-[85%] ${
                  message.role === 'user' ? 'self-end' : 'self-start'
                }`}
              >
                <div
                  className={`px-4 py-3 rounded-xl text-sm leading-relaxed whitespace-pre-wrap break-words ${
                    message.role === 'user'
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted border border-border'
                  }`}
                  style={{ overflowWrap: 'anywhere' }}
                >
                  {message.content}
                </div>
                <div className="text-[10px] text-muted-foreground px-1">
                  {formatTime(message.timestamp)}
                </div>
              </div>
            ))}

            {isTyping && (
              <div className="px-4 py-3 rounded-xl bg-muted border border-border max-w-[85%]">
                <TypingAnimation text="AI is thinking" size="sm" />
              </div>
            )}

            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      <div className="px-4 py-4 border-t border-border bg-background">
        <div className="flex gap-2 items-end">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={`Ask ${currentPersona.name.split(' ')[1]} a question...`}
            aria-label={`Message input for ${currentPersona.name}`}
            className="flex-1 min-h-[60px] max-h-[120px] px-3 py-2 bg-background border border-input rounded-md text-sm resize-y focus:outline-none focus:ring-2 focus:ring-ring"
            rows={2}
          />
          <Button
            variant="default"
            size="sm"
            onClick={handleSend}
            disabled={!input.trim() || isTyping}
            aria-label="Send message"
            title="Send message (Enter)"
            className="h-[60px]"
          >
            <Send className="h-4 w-4" aria-hidden="true" />
          </Button>
        </div>
      </div>
    </div>
  );
}
