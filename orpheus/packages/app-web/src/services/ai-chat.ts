/**
 * AI Chat Service
 * Integrates with persona-framework for context-aware AI assistance
 */

import type { AppMode } from '../store/app-store';
import axios from 'axios';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  mode?: AppMode;
}

export interface ChatSession {
  id: string;
  mode: AppMode;
  messages: ChatMessage[];
  createdAt: number;
}

/**
 * Persona codes for each mode (maps to persona-framework personas)
 */
const PERSONA_CODES: Record<AppMode, string> = {
  compose: 'music-theory-tutor',
  record: 'session-assistant',
  mix: 'mixing-engineer',
  master: 'mastering-engineer',
  practice: 'guitar-coach',
  distribute: 'distribution-manager',
};

/**
 * AI Chat Client
 */
export class AIChatClient {
  private sessions: Map<string, ChatSession> = new Map();
  private currentSessionId: string | null = null;

  /**
   * Start a new chat session for a specific mode
   */
  startSession(mode: AppMode): string {
    const sessionId = `session-${Date.now()}`;
    const session: ChatSession = {
      id: sessionId,
      mode,
      messages: [
        {
          id: `msg-${Date.now()}`,
          role: 'system',
          content: this.getSystemPrompt(mode),
          timestamp: Date.now(),
          mode,
        },
      ],
      createdAt: Date.now(),
    };

    this.sessions.set(sessionId, session);
    this.currentSessionId = sessionId;

    console.log(`[AI Chat] Started session ${sessionId} for mode: ${mode}`);
    return sessionId;
  }

  /**
   * Send a message and get AI response
   */
  async sendMessage(content: string, mode: AppMode): Promise<ChatMessage> {
    if (!this.currentSessionId) {
      this.startSession(mode);
    }

    const session = this.sessions.get(this.currentSessionId!);
    if (!session) {
      throw new Error('No active session');
    }

    // Add user message
    const userMessage: ChatMessage = {
      id: `msg-${Date.now()}-user`,
      role: 'user',
      content,
      timestamp: Date.now(),
      mode,
    };

    session.messages.push(userMessage);

    // Get AI response (simulated for now - replace with actual API call)
    const response = await this.generateResponse(content, mode, session.messages);

    const assistantMessage: ChatMessage = {
      id: `msg-${Date.now()}-assistant`,
      role: 'assistant',
      content: response,
      timestamp: Date.now(),
      mode,
    };

    session.messages.push(assistantMessage);

    console.log(`[AI Chat] User: ${content}`);
    console.log(`[AI Chat] Assistant: ${response}`);

    return assistantMessage;
  }

  /**
   * Get current session messages
   */
  getMessages(): ChatMessage[] {
    if (!this.currentSessionId) return [];
    const session = this.sessions.get(this.currentSessionId);
    return session ? session.messages.filter((m) => m.role !== 'system') : [];
  }

  /**
   * Clear current session
   */
  clearSession(): void {
    if (this.currentSessionId) {
      this.sessions.delete(this.currentSessionId);
      this.currentSessionId = null;
    }
  }

  /**
   * Get system prompt for each mode
   */
  private getSystemPrompt(mode: AppMode): string {
    const prompts: Record<AppMode, string> = {
      compose: `You are a professional music theory tutor and composition assistant. You help musicians with:
- Chord progressions and harmony
- Scale selection and mode theory
- Melody composition and motif development
- Rhythm and time signature guidance
- Song structure and arrangement
Be concise, practical, and encouraging. Use music theory terminology when appropriate.`,

      record: `You are a professional recording engineer and session assistant. You help musicians with:
- Microphone placement and recording techniques
- Audio interface setup and gain staging
- Take management and performance coaching
- Recording workflow optimization
- Technical troubleshooting
Be practical, supportive, and focus on getting the best performance captured.`,

      mix: `You are a professional mixing engineer. You help musicians with:
- EQ and frequency balance
- Compression and dynamics control
- Reverb, delay, and spatial effects
- Panning and stereo imaging
- Mix bus processing and automation
Be technical yet accessible. Explain the "why" behind mixing decisions.`,

      master: `You are a professional mastering engineer. You help musicians with:
- Loudness optimization (LUFS, RMS, True Peak)
- Platform-specific mastering (Spotify, Apple Music, etc.)
- EQ and multiband compression
- Stereo enhancement and limiting
- Final format preparation and export
Be precise about technical standards and streaming platform requirements.`,

      practice: `You are a professional guitar instructor and practice coach. You help musicians with:
- Technique development and finger exercises
- Speed building and accuracy training
- Practice routines and goal setting
- Performance anxiety and stage presence
- Music theory applied to guitar
Be motivating, patient, and provide structured practice guidance.`,

      distribute: `You are a music distribution expert and release manager. You help musicians with:
- Platform submission requirements (Spotify, Apple Music, etc.)
- Metadata optimization (ISRC, UPC, genre selection)
- Release timing and marketing strategies
- Copyright and publishing information
- Distribution partner selection (DistroKid, CD Baby, etc.)
Be knowledgeable about music industry standards and streaming platforms.`,
    };

    return prompts[mode] || 'You are a helpful music production assistant.';
  }

  /**
   * Generate AI response using persona-framework
   */
  private async generateResponse(
    userMessage: string,
    mode: AppMode,
    history: ChatMessage[]
  ): Promise<string> {
    try {
      // Get persona code for current mode
      const personaCode = PERSONA_CODES[mode];

      // Convert history to persona-framework format
      const messages = history
        .filter((m) => m.role !== 'system')
        .map((m) => ({
          role: m.role,
          content: m.content,
        }));

      // Add current user message
      messages.push({
        role: 'user' as const,
        content: userMessage,
      });

      // Call Leviathan persona-framework API
      const response = await axios.post(`/api/public/chat/${personaCode}`, {
        messages,
        temperature: 0.7,
      });

      // Leviathan returns ChatCompletionResponse with 'text' field
      return response.data.text;
    } catch (error) {
      console.warn('[AI Chat] Persona-framework API unavailable, using fallback responses:', error);
      return this.generateFallbackResponse(userMessage, mode);
    }
  }

  /**
   * Fallback simulated responses when persona-framework API is unavailable
   */
  private async generateFallbackResponse(
    userMessage: string,
    mode: AppMode
  ): Promise<string> {
    // Simulate API delay
    await new Promise((resolve) => setTimeout(resolve, 500));

    const responses: Record<AppMode, Record<string, string>> = {
      compose: {
        default: "I can help you with chord progressions, scales, melody writing, and music theory! What specific aspect of composition would you like to explore?",
        chord: "Great question about chords! In the key you're working in, here are some common progressions to try:\n\n• I-V-vi-IV (very popular in pop/rock)\n• ii-V-I (jazz standard)\n• I-vi-IV-V (classic doo-wop)\n\nWhat key are you composing in?",
        scale: "For guitar, the most common scales are:\n\n• Major (bright, happy)\n• Minor (sad, dark)\n• Pentatonic (versatile, rock/blues)\n• Blues scale (adds that bluesy feel)\n\nWhich style are you going for?",
      },
      record: {
        default: "I'm here to help with your recording session! Are you tracking guitars, vocals, or another instrument?",
        guitar: "For recording guitar, here's what I recommend:\n\n• Place the mic 6-12 inches from the 12th fret\n• Angle it slightly toward the soundhole\n• Use a cardioid pattern microphone\n• Set input gain so peaks hit around -12dB\n\nAre you recording acoustic or electric?",
        microphone: "Good question! For most home recording:\n\n• Condenser mic: Best for acoustic guitar, vocals\n• Dynamic mic: Great for electric guitar amps, loud sources\n• Ribbon mic: Warm tone, good for guitar cabs\n\nWhat's your budget and what are you recording?",
      },
      mix: {
        default: "Let's get your mix sounding great! What track or element are you working on?",
        eq: "Here's my EQ approach for a balanced mix:\n\n• Cut the mud: High-pass filter at 80-100Hz on most tracks\n• Find problem frequencies: Sweep with a narrow boost, then cut\n• Boost with care: Wide boosts, narrow cuts\n• Lead elements: Small boost around 2-5kHz for presence\n\nWhat instrument are you EQing?",
        compression: "Compression basics:\n\n• Ratio: Start with 3:1 for moderate control\n• Attack: Fast (1-10ms) for transient control, slow (20-50ms) to preserve punch\n• Release: Match the tempo (100-300ms typical)\n• Threshold: Aim for 3-6dB of gain reduction\n\nLet me know what you're compressing!",
      },
      master: {
        default: "I'll help you prepare your master for release! What platform are you targeting?",
        spotify: "Spotify mastering targets:\n\n• Integrated LUFS: -14 LUFS (they normalize to this)\n• True Peak: -1dBTP maximum\n• Dynamic Range: 8-12dB is ideal\n• Format: 44.1kHz/24-bit WAV or FLAC\n\nAvoid over-limiting! Spotify will turn down hot masters.",
        loudness: "Loudness standards by platform:\n\n• Spotify: -14 LUFS\n• Apple Music: -16 LUFS\n• YouTube: -14 LUFS\n• Tidal: -14 LUFS\n• SoundCloud: -8 to -13 LUFS (louder preferred)\n\nAim for your target platform, and you'll be within range for others!",
      },
      practice: {
        default: "Let's build your guitar skills! What technique are you working on?",
        speed: "Speed training tips:\n\n• Start SLOW with a metronome (50-60 BPM)\n• Perfect the motion before adding speed\n• Increase by 5 BPM only when you nail it 3 times\n• Practice in short bursts (5-10 min) with breaks\n• Tension is the enemy - stay relaxed!\n\nWhat passage are you working on?",
        technique: "Essential guitar techniques to master:\n\n• Alternate picking (down-up-down-up)\n• Hammer-ons and pull-offs (legato)\n• String skipping\n• Economy picking\n• Sweep picking (advanced)\n\nWhich one would you like to focus on?",
      },
      distribute: {
        default: "I'll help you get your music on streaming platforms! Are you ready to upload your track?",
        distrokid: "DistroKid is a great choice! Here's what you need:\n\n• Audio: WAV file, 44.1kHz/24-bit or higher\n• Artwork: 3000x3000px JPG (exactly square)\n• Metadata: Artist name, track title, genre\n• ISRC: Auto-generated by DistroKid (optional: provide your own)\n• Release date: At least 2 weeks out for playlisting\n\nAnnual fee: $22.99 for unlimited uploads, keep 100% royalties!",
        metadata: "Metadata best practices:\n\n• Track Title: Avoid special characters, emojis\n• Artist Name: Exactly as you want it to appear\n• Genre: Choose the most accurate primary genre\n• Release Date: Schedule 2-4 weeks ahead for editorial review\n• ISRC: Required for tracking (auto-generated is fine)\n\nNeed help with any specific field?",
      },
    };

    const messageLower = userMessage.toLowerCase();

    // Find relevant response based on keywords
    const modeResponses = responses[mode];
    for (const [keyword, response] of Object.entries(modeResponses)) {
      if (keyword !== 'default' && messageLower.includes(keyword)) {
        return response;
      }
    }

    // Return default response for the mode
    return modeResponses.default;
  }
}

// Singleton instance
let chatClientInstance: AIChatClient | null = null;

/**
 * Get the global AI chat client
 */
export function getAIChatClient(): AIChatClient {
  if (!chatClientInstance) {
    chatClientInstance = new AIChatClient();
  }
  return chatClientInstance;
}

/**
 * Reset the chat client (for testing)
 */
export function resetAIChatClient(): void {
  if (chatClientInstance) {
    chatClientInstance.clearSession();
    chatClientInstance = null;
  }
}
