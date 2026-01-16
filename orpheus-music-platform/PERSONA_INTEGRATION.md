# Persona-Framework Integration Guide

Orpheus integrates with the persona-framework backend to provide context-aware AI assistance across all 6 production modes.

## Architecture

```
┌─────────────────────┐
│   Orpheus Web    │
│   (React Frontend)  │
└──────────┬──────────┘
           │ HTTP POST /api/public/chat/{persona_code}
           ▼
┌─────────────────────┐
│ Persona-Framework   │
│   API Gateway       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   AI Personas       │
│  (6 Specialists)    │
└─────────────────────┘
```

## Required Personas

The following personas must be configured in the persona-framework backend:

### 1. Music Theory Tutor (`music-theory-tutor`)
**Purpose:** Assists with composition, harmony, and music theory

**System Prompt:**
```
You are a professional music theory tutor and composition assistant. You help musicians with:
- Chord progressions and harmony
- Scale selection and mode theory
- Melody composition and motif development
- Rhythm and time signature guidance
- Song structure and arrangement

Be concise, practical, and encouraging. Use music theory terminology when appropriate but explain concepts clearly.
```

**Example Interactions:**
- "What chords work well in the key of A minor?"
- "How do I write a guitar solo over a ii-V-I progression?"
- "What scale should I use for a dark, metal vibe?"

---

### 2. Session Assistant (`session-assistant`)
**Purpose:** Guides recording sessions and audio capture

**System Prompt:**
```
You are a professional recording engineer and session assistant. You help musicians with:
- Microphone placement and recording techniques
- Audio interface setup and gain staging
- Take management and performance coaching
- Recording workflow optimization
- Technical troubleshooting

Be practical, supportive, and focus on getting the best performance captured.
```

**Example Interactions:**
- "How do I mic an acoustic guitar?"
- "What's the best input gain for recording?"
- "How do I reduce room echo when recording?"

---

### 3. Mixing Engineer (`mixing-engineer`)
**Purpose:** Provides mixing guidance and technical advice

**System Prompt:**
```
You are a professional mixing engineer. You help musicians with:
- EQ and frequency balance
- Compression and dynamics control
- Reverb, delay, and spatial effects
- Panning and stereo imaging
- Mix bus processing and automation

Be technical yet accessible. Explain the "why" behind mixing decisions.
```

**Example Interactions:**
- "How do I EQ a muddy guitar track?"
- "What compression settings for bass guitar?"
- "My mix sounds cluttered - how do I fix it?"

---

### 4. Mastering Engineer (`mastering-engineer`)
**Purpose:** Advises on mastering and loudness standards

**System Prompt:**
```
You are a professional mastering engineer. You help musicians with:
- Loudness optimization (LUFS, RMS, True Peak)
- Platform-specific mastering (Spotify, Apple Music, etc.)
- EQ and multiband compression
- Stereo enhancement and limiting
- Final format preparation and export

Be precise about technical standards and streaming platform requirements.
```

**Example Interactions:**
- "What's the target LUFS for Spotify?"
- "Should I use limiting or compression for mastering?"
- "How do I prepare a master for vinyl pressing?"

---

### 5. Guitar Coach (`guitar-coach`)
**Purpose:** Provides technique guidance and practice strategies

**System Prompt:**
```
You are a professional guitar instructor and practice coach. You help musicians with:
- Technique development and finger exercises
- Speed building and accuracy training
- Practice routines and goal setting
- Performance anxiety and stage presence
- Music theory applied to guitar

Be motivating, patient, and provide structured practice guidance.
```

**Example Interactions:**
- "How do I build alternate picking speed?"
- "What's a good practice routine for beginners?"
- "How do I overcome tension when playing fast?"

---

### 6. Distribution Manager (`distribution-manager`)
**Purpose:** Explains music distribution and release strategies

**System Prompt:**
```
You are a music distribution expert and release manager. You help musicians with:
- Platform submission requirements (Spotify, Apple Music, etc.)
- Metadata optimization (ISRC, UPC, genre selection)
- Release timing and marketing strategies
- Copyright and publishing information
- Distribution partner selection (DistroKid, CD Baby, etc.)

Be knowledgeable about music industry standards and streaming platforms.
```

**Example Interactions:**
- "How do I submit to DistroKid?"
- "What's an ISRC code and do I need one?"
- "When should I schedule my release date?"

---

## API Contract

### Request Format

```http
POST /api/public/chat/{persona_code}
Content-Type: application/json

{
  "messages": [
    { "role": "user", "content": "What chords work in D major?" }
  ],
  "temperature": 0.7
}
```

### Response Format

```json
{
  "response": "For D major, the diatonic chords are:\n• D major (I)\n• E minor (ii)\n• F# minor (iii)\n• G major (IV)\n• A major (V)\n• B minor (vi)\n• C# diminished (vii°)\n\nA common progression would be D - G - A - D (I-IV-V-I)."
}
```

**Alternative response fields supported:**
- `response.data.response`
- `response.data.content`  
- `response.data` (raw string)

---

## Integration Code

The Orpheus frontend integrates via `ai-chat.ts`:

```typescript
// Persona code mapping
const PERSONA_CODES: Record<AppMode, string> = {
  compose: 'music-theory-tutor',
  record: 'session-assistant',
  mix: 'mixing-engineer',
  master: 'mastering-engineer',
  practice: 'guitar-coach',
  distribute: 'distribution-manager',
};

// API call
const response = await axios.post(`/api/public/chat/${personaCode}`, {
  messages,
  temperature: 0.7,
});
```

---

## Fallback Behavior

If the persona-framework API is unavailable, Orpheus falls back to simulated keyword-based responses. This ensures the UI remains functional even without backend connectivity.

**Fallback triggers:**
- Network error
- 404 Not Found (persona not configured)
- 500 Server Error
- Timeout

**User experience:**
- AI responses still appear in chat
- Console warning logged
- Simulated responses provide basic guidance

---

## Setup Instructions

### 1. Configure Personas in Backend

Create 6 personas in the persona-framework admin interface or via API:

```bash
# Example: Create music-theory-tutor persona
curl -X POST https://your-domain.com/api/personas \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "music-theory-tutor",
    "name": "Music Theory Tutor",
    "systemPrompt": "You are a professional music theory tutor...",
    "temperature": 0.7,
    "maxTokens": 1024
  }'
```

Repeat for all 6 personas listed above.

### 2. Verify API Endpoint

Test each persona endpoint:

```bash
curl -X POST https://your-domain.com/api/public/chat/music-theory-tutor \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "user", "content": "Hello"}],
    "temperature": 0.7
  }'
```

Expected response:
```json
{
  "response": "Hello! I'm your music theory tutor. How can I help you today?"
}
```

### 3. Configure Frontend Environment

Set the API base URL in `.env` (optional, defaults to `/api`):

```env
VITE_API_BASE_URL=/api
```

### 4. Test Integration

1. Start Orpheus web app
2. Open any mode (e.g., Compose Mode)
3. Click AI Assistant button
4. Send a test message
5. Verify response comes from persona-framework (check Network tab)

---

## Monitoring

### Success Metrics
- Check browser Network tab for `POST /api/public/chat/*` requests
- Status 200 indicates successful persona-framework connection
- Console should show: `[AI Chat] User: <message>` and `[AI Chat] Assistant: <response>`

### Fallback Detection
- Console warning: `[AI Chat] Persona-framework API unavailable, using fallback responses`
- Indicates network issue or missing persona configuration

---

## Troubleshooting

### "Persona not found" (404)

**Cause:** Persona code not configured in backend

**Solution:**
1. Verify persona codes match exactly (case-sensitive)
2. Check persona-framework admin for missing personas
3. Create missing personas with correct codes

### "CORS error"

**Cause:** Frontend domain not whitelisted in backend

**Solution:**
1. Add Orpheus domain to CORS whitelist
2. Configure backend to allow `http://localhost:5173` (dev) and production domain

### "Fallback responses always used"

**Cause:** API endpoint unreachable

**Solution:**
1. Verify backend is running
2. Check network connectivity
3. Test API endpoint directly with curl
4. Review browser console for specific error

---

## Future Enhancements

**Planned:**
- Streaming responses for real-time chat
- Conversation history persistence
- Custom persona configuration UI
- Multi-modal support (audio input/output)
- Context injection from active project

---

## Security Considerations

**Public Endpoints:**
- `/api/public/chat/*` endpoints are publicly accessible
- No authentication required for chat functionality
- Rate limiting recommended (e.g., 60 requests/minute per IP)

**Private Data:**
- User messages are NOT stored by Orpheus frontend
- Persona-framework may log conversations (check backend config)
- No project files are sent to AI - only user questions

**Recommended Policies:**
- Implement rate limiting on backend
- Monitor for abuse/spam
- Add optional authentication for advanced features
- GDPR compliance for EU users (data retention, right to deletion)

