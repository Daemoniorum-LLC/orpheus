/**
 * AI History types for tracking AI interactions
 */

import type { MaestroMode } from './common';

export interface AIHistoryData {
  compositionSuggestions: AIInteraction[];
  mixingSuggestions: AIInteraction[];
  masteringSuggestions: AIInteraction[];
  transcriptionResults: AIInteraction[];
  practiceAnalysis: AIInteraction[];
  chatHistory: AIChatMessage[];
}

export interface AIInteraction {
  id: string;
  timestamp: string;
  persona: string;
  type: string;
  input: any;
  output: any;
  applied: boolean;
  feedback?: 'positive' | 'negative' | null;
}

export interface AIChatMessage {
  id: string;
  timestamp: string;
  role: 'user' | 'assistant';
  content: string;
  context?: ChatContext;
}

export interface ChatContext {
  currentMode: MaestroMode;
  selectedTrackId?: string;
  selectedMeasures?: [number, number];
}
