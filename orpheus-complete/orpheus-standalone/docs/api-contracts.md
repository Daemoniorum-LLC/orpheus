# Orpheus - API Contracts & Service Interfaces

**Version:** 1.0
**Date:** November 16, 2025
**Status:** Specification

## Overview

This document defines the API contracts and service interfaces between Orpheus's frontend (React), backend (Spring Boot/Hydra), and AI layer (Leviathan). These contracts ensure clean separation of concerns and enable independent development of each layer.

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                  Frontend (React/TypeScript)             │
│  - UI Components                                         │
│  - State Management                                      │
│  - Mode Controllers                                      │
└─────────────────────┬───────────────────────────────────┘
                      │ REST/WebSocket
┌─────────────────────▼───────────────────────────────────┐
│              Backend (Spring Boot/Hydra)                 │
│  - Session Management                                    │
│  - File Storage                                          │
│  - Collaboration                                         │
│  - AI Request Routing                                    │
└─────────────────────┬───────────────────────────────────┘
                      │ Internal API
┌─────────────────────▼───────────────────────────────────┐
│            AI Layer (Leviathan + Grimoire)               │
│  - Music Theory Tutor                                    │
│  - Guitar Coach                                          │
│  - Mixing Engineer                                       │
│  - Mastering Engineer                                    │
└─────────────────────────────────────────────────────────┘
```

## Frontend ↔ Backend API

### Base URLs

```
Production:  https://api.maestro.ai/v1
Development: http://localhost:8080/api/v1
```

### Authentication

```typescript
// Header for all authenticated requests
Authorization: Bearer <jwt_token>

// Token structure
interface AuthToken {
  userId: string;
  email: string;
  tier: 'free' | 'creator' | 'pro' | 'ultimate';
  expiresAt: number; // Unix timestamp
}
```

### Projects API

#### GET /projects
**Description:** List all projects for the authenticated user

**Request:**
```typescript
GET /projects?page=0&size=20&sort=modified,desc

QueryParams:
  - page: number (default: 0)
  - size: number (default: 20, max: 100)
  - sort: string (default: "modified,desc")
  - filter?: string (search by title/tags)
```

**Response:**
```typescript
{
  "content": [
    {
      "id": "uuid",
      "title": "My Song",
      "artist": "Artist Name",
      "created": "2025-11-16T10:00:00Z",
      "modified": "2025-11-16T14:30:00Z",
      "duration": 225, // seconds
      "tempo": 120,
      "key": "C",
      "tags": ["rock", "demo"],
      "thumbnail": "https://cdn.maestro.ai/thumbnails/uuid.jpg"
    }
  ],
  "totalElements": 42,
  "totalPages": 3,
  "size": 20,
  "number": 0
}
```

#### POST /projects
**Description:** Create a new project

**Request:**
```typescript
POST /projects

Body:
{
  "title": "My New Song",
  "artist": "Artist Name",
  "tempo": 120,
  "key": "C",
  "timeSignature": { "numerator": 4, "denominator": 4 }
}
```

**Response:**
```typescript
{
  "id": "uuid",
  "title": "My New Song",
  "created": "2025-11-16T15:00:00Z",
  ...
}
```

#### GET /projects/{id}
**Description:** Get a specific project

**Response:**
```typescript
{
  "id": "uuid",
  "project": { /* Full .maestro project structure */ }
}
```

#### PUT /projects/{id}
**Description:** Update a project

**Request:**
```typescript
PUT /projects/{id}

Body: { /* Full or partial .maestro project */ }
```

**Response:**
```typescript
{
  "id": "uuid",
  "modified": "2025-11-16T15:30:00Z",
  "version": 5
}
```

#### DELETE /projects/{id}
**Description:** Delete a project

**Response:**
```typescript
{
  "deleted": true,
  "id": "uuid"
}
```

### Audio Files API

#### POST /projects/{id}/audio
**Description:** Upload audio file for a project

**Request:**
```typescript
POST /projects/{id}/audio

Content-Type: multipart/form-data

Body:
  - file: File (WAV, MP3, FLAC, etc.)
  - trackId: string
  - regionId: string
```

**Response:**
```typescript
{
  "fileId": "uuid",
  "url": "https://cdn.maestro.ai/audio/uuid.wav",
  "size": 10485760, // bytes
  "duration": 180,  // seconds
  "sampleRate": 48000,
  "channels": 2
}
```

#### GET /projects/{id}/audio/{fileId}
**Description:** Download audio file

**Response:** Binary audio data

### AI API

#### POST /ai/request
**Description:** Send a request to an AI persona

**Request:**
```typescript
POST /ai/request

Body:
{
  "persona": "music-theory-tutor",
  "prompt": "What chords work in C major?",
  "context": {
    "projectId": "uuid",
    "mode": "compose",
    "selectedMeasures": [8, 12]
  }
}
```

**Response:**
```typescript
{
  "requestId": "uuid",
  "response": "The key of C major contains these chords: ...",
  "suggestions": [
    {
      "type": "chord",
      "data": { "root": "C", "quality": "major" }
    }
  ],
  "timestamp": "2025-11-16T15:45:00Z"
}
```

#### POST /ai/chat
**Description:** Multi-turn chat with AI persona

**Request:**
```typescript
POST /ai/chat

Body:
{
  "persona": "guitar-coach-ai",
  "sessionId": "uuid", // Optional, for continuing conversation
  "messages": [
    { "role": "user", "content": "Help me with bending technique" },
    { "role": "assistant", "content": "I can help! ..." },
    { "role": "user", "content": "Show me an exercise" }
  ],
  "context": { /* Project context */ }
}
```

**Response:**
```typescript
{
  "sessionId": "uuid",
  "response": "Here's a great bending exercise: ...",
  "timestamp": "2025-11-16T15:50:00Z"
}
```

### Collaboration API

#### WebSocket /ws/collaborate/{projectId}
**Description:** Real-time collaboration WebSocket

**Messages:**

**Client → Server:**
```typescript
{
  "type": "cursor-move",
  "data": {
    "userId": "uuid",
    "position": { "measure": 16, "beat": 2 }
  }
}

{
  "type": "edit",
  "data": {
    "path": "session.tracks.0.volume",
    "oldValue": -3,
    "newValue": -6
  }
}
```

**Server → Client:**
```typescript
{
  "type": "user-joined",
  "data": {
    "userId": "uuid",
    "name": "John Doe",
    "color": "#FF5733"
  }
}

{
  "type": "remote-edit",
  "data": {
    "userId": "uuid",
    "change": { /* Change data */ }
  }
}
```

## Backend ↔ AI Layer API

### Internal AI Service Interface

```java
public interface AIService {

  /**
   * Send a request to an AI persona
   */
  CompletableFuture<AIResponse> request(AIRequest request);

  /**
   * Multi-turn chat with AI persona
   */
  CompletableFuture<AIChatResponse> chat(AIChatRequest request);

  /**
   * Analyze audio/MIDI data
   */
  CompletableFuture<AIAnalysis> analyze(AnalysisRequest request);
}
```

### AIRequest Interface

```java
public class AIRequest {
  private String persona;           // e.g., "music-theory-tutor"
  private String prompt;
  private Map<String, Object> context;
  private String userId;
  private String projectId;
}

public class AIResponse {
  private String requestId;
  private String response;
  private List<AISuggestion> suggestions;
  private LocalDateTime timestamp;
}
```

### Analysis Request

```java
public class AnalysisRequest {
  private AnalysisType type;        // LOUDNESS, FREQUENCY, HARMONY, etc.
  private byte[] audioData;
  private MidiData midiData;
  private Map<String, Object> options;
}

public enum AnalysisType {
  LOUDNESS,          // LUFS, dynamic range
  FREQUENCY,         // Spectrum analysis
  HARMONY,           // Chord detection
  KEY_DETECTION,     // Musical key
  TEMPO_DETECTION,   // BPM
  MIX_ANALYSIS,      // Overall mix quality
  MASTER_ANALYSIS    // Mastering readiness
}
```

## Data Transfer Objects (DTOs)

### Project DTO

```typescript
interface ProjectDTO {
  id: string;
  title: string;
  artist?: string;
  created: string;  // ISO 8601
  modified: string;
  metadata: ProjectMetadataDTO;
  compositionSummary?: {
    scoreCount: number;
    measureCount: number;
  };
  sessionSummary?: {
    trackCount: number;
    totalDuration: number;
  };
}
```

### Audio File DTO

```typescript
interface AudioFileDTO {
  id: string;
  projectId: string;
  trackId: string;
  regionId: string;
  filename: string;
  url: string;
  size: number;
  duration: number;
  sampleRate: number;
  bitDepth: number;
  channels: number;
  format: 'wav' | 'mp3' | 'flac' | 'ogg';
  uploaded: string;
}
```

### AI Suggestion DTO

```typescript
interface AISuggestionDTO {
  id: string;
  type: 'eq' | 'compression' | 'chord' | 'scale' | 'progression' | 'exercise';
  description: string;
  parameters?: Record<string, any>;
  applied: boolean;
  feedback?: 'positive' | 'negative';
}
```

## Error Handling

### Standard Error Response

```typescript
{
  "error": {
    "code": "PROJECT_NOT_FOUND",
    "message": "Project with ID 'uuid' not found",
    "status": 404,
    "timestamp": "2025-11-16T16:00:00Z",
    "path": "/api/v1/projects/uuid"
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Invalid or expired auth token |
| `FORBIDDEN` | 403 | User doesn't have permission |
| `PROJECT_NOT_FOUND` | 404 | Project doesn't exist |
| `AUDIO_FILE_TOO_LARGE` | 413 | Audio file exceeds size limit |
| `INVALID_AUDIO_FORMAT` | 400 | Unsupported audio format |
| `AI_REQUEST_FAILED` | 500 | AI service error |
| `AI_QUOTA_EXCEEDED` | 429 | User exceeded AI request quota |
| `COLLABORATION_ERROR` | 500 | WebSocket collaboration error |

## Rate Limiting

### API Rate Limits

| Endpoint | Free Tier | Creator | Pro | Ultimate |
|----------|-----------|---------|-----|----------|
| `/projects/*` | 100/hour | 1000/hour | Unlimited | Unlimited |
| `/ai/request` | 20/month | 200/month | Unlimited | Unlimited |
| `/audio/*` | 10/day | 100/day | 500/day | Unlimited |

**Rate Limit Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1637074800
```

## Versioning

### API Versioning Strategy

- Version in URL: `/api/v1/`, `/api/v2/`
- Breaking changes require new version
- Non-breaking changes can be added to existing version
- Minimum 6 months support for deprecated versions

### Version Changes

**v1 → v2 (planned):**
- Add video sync endpoints
- Enhanced collaboration features
- Dolby Atmos support

## Security

### Authentication Flow

```
1. User logs in → POST /auth/login
2. Server returns JWT token
3. Frontend stores token (secure, httpOnly cookie or localStorage)
4. All requests include: Authorization: Bearer <token>
5. Server validates token on each request
6. Token expires after 24 hours → refresh required
```

### Data Encryption

- All API traffic over HTTPS (TLS 1.3)
- Audio files encrypted at rest (AES-256)
- Project data encrypted in database
- WebSocket connections secured with WSS

## Performance Targets

| Operation | Target | Critical |
|-----------|--------|----------|
| GET /projects | <200ms | <500ms |
| GET /projects/{id} | <300ms | <1s |
| POST /projects | <500ms | <2s |
| PUT /projects/{id} | <500ms | <2s |
| Audio upload | <5s (10MB) | <30s |
| AI request | <3s | <10s |
| WebSocket latency | <100ms | <500ms |

## Testing

### Contract Testing

- Use Spring Cloud Contract for API contracts
- Automated tests verify request/response schemas
- Mock AI service for frontend testing
- Integration tests for full stack

### Example Contract Test

```java
@Test
public void should_return_project_list() {
  given()
    .header("Authorization", "Bearer " + validToken)
  .when()
    .get("/api/v1/projects")
  .then()
    .statusCode(200)
    .body("content", hasSize(greaterThan(0)))
    .body("content[0].id", notNullValue())
    .body("content[0].title", notNullValue());
}
```

## Documentation

### OpenAPI / Swagger

- Full OpenAPI 3.0 specification
- Available at: `/api/v1/swagger-ui.html`
- Auto-generated from Spring Boot controllers
- Includes examples and schemas

### Postman Collection

- Complete Postman collection for all endpoints
- Environment variables for dev/staging/prod
- Example requests with realistic data

---

**Version History:**
- 1.0 (2025-11-16): Initial specification

**Next Steps:**
1. Implement backend API controllers
2. Generate OpenAPI spec
3. Create frontend API client
4. Add integration tests
5. Performance testing
