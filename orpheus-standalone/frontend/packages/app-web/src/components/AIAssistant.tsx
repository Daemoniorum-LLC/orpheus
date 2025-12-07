/**
 * AI Assistant Panel - Context-aware AI chat interface
 */

import {
  makeStyles,
  shorthands,
  tokens,
  Button,
  Textarea,
  Spinner,
} from '@fluentui/react-components';
import { Send24Regular, Dismiss24Regular, Delete24Regular } from '@fluentui/react-icons';
import { useAppStore, useCurrentMode } from '../store/app-store';
import { useState, useEffect, useRef } from 'react';
import { getAIChatClient, type ChatMessage } from '../services/ai-chat';
import { AIQuickActions } from './AIQuickActions';

const useStyles = makeStyles({
  panel: {
    width: '380px',
    backgroundColor: tokens.colorNeutralBackground2,
    ...shorthands.borderLeft('1px', 'solid', tokens.colorNeutralStroke1),
    display: 'flex',
    flexDirection: 'column',
  },
  header: {
    ...shorthands.padding('16px'),
    ...shorthands.borderBottom('1px', 'solid', tokens.colorNeutralStroke1),
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: tokens.colorNeutralBackground1,
  },
  title: {
    fontSize: '16px',
    fontWeight: tokens.fontWeightSemibold,
  },
  headerActions: {
    display: 'flex',
    ...shorthands.gap('8px'),
  },
  content: {
    flex: 1,
    overflow: 'auto',
    ...shorthands.padding('16px'),
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
  personaInfo: {
    ...shorthands.padding('16px'),
    backgroundColor: tokens.colorBrandBackground2,
    ...shorthands.borderRadius('8px'),
    ...shorthands.border('1px', 'solid', tokens.colorBrandStroke1),
  },
  personaName: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '8px',
    color: tokens.colorBrandForeground1,
  },
  personaDesc: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
    lineHeight: '1.5',
  },
  messagesContainer: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('12px'),
  },
  message: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('6px'),
    maxWidth: '85%',
  },
  messageUser: {
    alignSelf: 'flex-end',
  },
  messageAssistant: {
    alignSelf: 'flex-start',
  },
  messageBubble: {
    ...shorthands.padding('12px', '16px'),
    ...shorthands.borderRadius('12px'),
    fontSize: '13px',
    lineHeight: '1.5',
    whiteSpace: 'pre-wrap',
    wordWrap: 'break-word',
  },
  bubbleUser: {
    backgroundColor: tokens.colorBrandBackground,
    color: tokens.colorNeutralForegroundOnBrand,
  },
  bubbleAssistant: {
    backgroundColor: tokens.colorNeutralBackground3,
    color: tokens.colorNeutralForeground1,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
  },
  messageTime: {
    fontSize: '10px',
    color: tokens.colorNeutralForeground3,
    paddingLeft: '4px',
    paddingRight: '4px',
  },
  inputArea: {
    ...shorthands.padding('16px'),
    ...shorthands.borderTop('1px', 'solid', tokens.colorNeutralStroke1),
    backgroundColor: tokens.colorNeutralBackground1,
  },
  inputContainer: {
    display: 'flex',
    ...shorthands.gap('8px'),
    alignItems: 'flex-end',
  },
  emptyState: {
    textAlign: 'center',
    ...shorthands.padding('32px', '16px'),
    color: tokens.colorNeutralForeground3,
  },
  emptyStateTitle: {
    fontSize: '14px',
    fontWeight: tokens.fontWeightSemibold,
    marginBottom: '8px',
  },
  emptyStateText: {
    fontSize: '12px',
    lineHeight: '1.5',
  },
  typingIndicator: {
    display: 'flex',
    ...shorthands.gap('4px'),
    alignItems: 'center',
    ...shorthands.padding('12px', '16px'),
    ...shorthands.borderRadius('12px'),
    backgroundColor: tokens.colorNeutralBackground3,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke1),
    maxWidth: '85%',
  },
  typingDot: {
    width: '6px',
    height: '6px',
    ...shorthands.borderRadius('50%'),
    backgroundColor: tokens.colorNeutralForeground3,
  },
});

const PERSONAS = {
  compose: {
    name: 'Music Theory Tutor',
    description: 'I can help you with chords, scales, progressions, and composition techniques. Ask me about harmony, melody writing, or song structure.',
  },
  record: {
    name: 'Session Assistant',
    description: 'I\'ll help you set up your recording session and get the best takes. Ask about microphone placement, gain staging, or recording techniques.',
  },
  mix: {
    name: 'Mixing Engineer',
    description: 'I can suggest EQ, compression, and other mixing techniques for your tracks. Ask me about balance, effects, or troubleshooting mix issues.',
  },
  master: {
    name: 'Mastering Engineer',
    description: 'I\'ll help you achieve broadcast-standard loudness and prepare for distribution. Ask about LUFS targets, platform requirements, or mastering chains.',
  },
  practice: {
    name: 'Guitar Coach',
    description: 'I can help you practice techniques, build speed, and improve your playing. Ask me about exercises, practice routines, or overcoming technical challenges.',
  },
  distribute: {
    name: 'Distribution Manager',
    description: 'I can help you prepare your music for release and navigate streaming platforms. Ask about DistroKid, metadata, release strategies, or royalties.',
  },
};

export function AIAssistant() {
  const styles = useStyles();
  const { aiAssistantOpen, setAIAssistantOpen } = useAppStore();
  const mode = useCurrentMode();
  const [input, setInput] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isTyping, setIsTyping] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const chatClient = getAIChatClient();

  const currentPersona = PERSONAS[mode];

  // Scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  // Start new session when mode changes
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
    // Auto-send after a brief delay so user can see the prompt
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
    <div className={styles.panel}>
      <div className={styles.header}>
        <div className={styles.title}>AI Assistant</div>
        <div className={styles.headerActions}>
          <Button
            icon={<Delete24Regular />}
            appearance="subtle"
            size="small"
            onClick={handleClearChat}
            title="Clear chat"
          />
          <Button
            icon={<Dismiss24Regular />}
            appearance="subtle"
            size="small"
            onClick={() => setAIAssistantOpen(false)}
            title="Close"
          />
        </div>
      </div>

      <div className={styles.content}>
        {/* Persona Info */}
        <div className={styles.personaInfo}>
          <div className={styles.personaName}>{currentPersona.name}</div>
          <div className={styles.personaDesc}>{currentPersona.description}</div>
        </div>

        {/* Quick Actions - only show when no messages */}
        {messages.length === 0 && (
          <AIQuickActions onActionClick={handleQuickAction} />
        )}

        {/* Messages */}
        {messages.length === 0 ? (
          <div className={styles.emptyState}>
            <div className={styles.emptyStateTitle}>Start a conversation!</div>
            <div className={styles.emptyStateText}>
              Ask me anything about {mode === 'compose' ? 'composition' : mode === 'record' ? 'recording' : mode === 'mix' ? 'mixing' : mode === 'master' ? 'mastering' : mode === 'practice' ? 'practice' : 'distribution'} and I'll help you out.
            </div>
          </div>
        ) : (
          <div className={styles.messagesContainer}>
            {messages.map((message) => (
              <div
                key={message.id}
                className={`${styles.message} ${
                  message.role === 'user' ? styles.messageUser : styles.messageAssistant
                }`}
              >
                <div
                  className={`${styles.messageBubble} ${
                    message.role === 'user' ? styles.bubbleUser : styles.bubbleAssistant
                  }`}
                >
                  {message.content}
                </div>
                <div className={styles.messageTime}>{formatTime(message.timestamp)}</div>
              </div>
            ))}

            {/* Typing indicator */}
            {isTyping && (
              <div className={styles.typingIndicator}>
                <Spinner size="tiny" />
                <span style={{ fontSize: '12px', color: tokens.colorNeutralForeground2 }}>
                  AI is thinking...
                </span>
              </div>
            )}

            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      {/* Input Area */}
      <div className={styles.inputArea}>
        <div className={styles.inputContainer}>
          <Textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={`Ask ${currentPersona.name.split(' ')[1]} a question...`}
            resize="vertical"
            style={{ flex: 1 }}
            rows={2}
          />
          <Button
            icon={<Send24Regular />}
            appearance="primary"
            onClick={handleSend}
            disabled={!input.trim() || isTyping}
            title="Send message (Enter)"
          />
        </div>
      </div>
    </div>
  );
}
