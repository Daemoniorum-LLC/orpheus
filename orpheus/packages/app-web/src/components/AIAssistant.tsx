import { Button, Spinner } from '@persona-framework/ui';
import { Send, X, Trash2 } from 'lucide-react';
import { useAppStore, useCurrentMode } from '../store/app-store';
import { useState, useEffect, useRef } from 'react';
import { getAIChatClient, type ChatMessage } from '../services/ai-chat';
import { AIQuickActions } from './AIQuickActions';

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

export function AIAssistant() {
  const { aiAssistantOpen, setAIAssistantOpen } = useAppStore();
  const mode = useCurrentMode();
  const [input, setInput] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isTyping, setIsTyping] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const chatClient = getAIChatClient();

  const currentPersona = PERSONAS[mode];

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

  if (!aiAssistantOpen) {
    return null;
  }

  return (
    <div className="w-[380px] bg-secondary border-l border-border flex flex-col">
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
          <div className="text-center px-8 py-8 text-muted-foreground">
            <div className="text-sm font-semibold mb-2">Start a conversation!</div>
            <div className="text-xs leading-relaxed">
              Ask me anything about {mode === 'compose' ? 'composition' : mode === 'record' ? 'recording' : mode === 'mix' ? 'mixing' : mode === 'master' ? 'mastering' : mode === 'practice' ? 'practice' : 'distribution'} and I'll help you out.
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
                >
                  {message.content}
                </div>
                <div className="text-[10px] text-muted-foreground px-1">
                  {formatTime(message.timestamp)}
                </div>
              </div>
            ))}

            {isTyping && (
              <div className="flex gap-1 items-center px-4 py-3 rounded-xl bg-muted border border-border max-w-[85%]">
                <Spinner className="h-4 w-4" />
                <span className="text-xs text-muted-foreground">
                  AI is thinking...
                </span>
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
            className="flex-1 min-h-[60px] max-h-[120px] px-3 py-2 bg-background border border-input rounded-md text-sm resize-y focus:outline-none focus:ring-2 focus:ring-ring"
            rows={2}
          />
          <Button
            variant="default"
            size="sm"
            onClick={handleSend}
            disabled={!input.trim() || isTyping}
            title="Send message (Enter)"
            className="h-[60px]"
          >
            <Send className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
