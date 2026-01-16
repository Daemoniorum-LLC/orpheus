# Maestro Backend

Spring Boot REST API for Maestro - Professional music production platform.

## Technology Stack

- **Framework**: Spring Boot 3.2+ with Kotlin
- **Database**: PostgreSQL 15+
- **Cache**: Redis 7+
- **Storage**: AWS S3 / MinIO (local dev)
- **Security**: Spring Security + JWT (TODO)
- **API Documentation**: OpenAPI 3.0 (Springdoc)
- **Build**: Gradle with Kotlin DSL

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   REST API Layer                     │
│   - ProjectController                                │
│   - TrackController                                  │
│   - CollaborationController (TODO)                   │
│   - DistributionController (TODO)                    │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│                 Service Layer                        │
│   - ProjectService                                   │
│   - TrackService                                     │
│   - AudioStorageService (TODO)                       │
│   - LeviathanService (TODO)                          │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│              Repository Layer (JPA)                  │
│   - UserRepository                                   │
│   - ProjectRepository                                │
│   - TrackRepository                                  │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│                  PostgreSQL                          │
└─────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- JDK 17+
- Docker & Docker Compose

### 1. Start Infrastructure

```bash
cd backend
docker-compose up -d
```

This starts:
- **PostgreSQL** on port 5432
- **Redis** on port 6379
- **MinIO** (S3-compatible storage) on ports 9000 (API) and 9001 (Console)

### 2. Build & Run

```bash
./gradlew bootRun
```

The API will be available at: `http://localhost:8080`

### 3. Access Services

- **API**: http://localhost:8080/api/v1
- **Swagger UI**: http://localhost:8080/swagger-ui
- **OpenAPI Docs**: http://localhost:8080/api-docs
- **MinIO Console**: http://localhost:9001 (minioadmin / minioadmin)
- **Actuator**: http://localhost:8080/actuator

## API Endpoints

### Projects

```
POST   /api/v1/projects           Create project
GET    /api/v1/projects           List all projects
GET    /api/v1/projects/{id}      Get project details
PUT    /api/v1/projects/{id}      Update project
DELETE /api/v1/projects/{id}      Delete project
```

### Tracks

```
POST   /api/v1/projects/{id}/tracks   Add track to project
GET    /api/v1/projects/{id}/tracks   List project tracks
GET    /api/v1/tracks/{id}            Get track details
PUT    /api/v1/tracks/{id}            Update track
DELETE /api/v1/tracks/{id}            Delete track
PUT    /api/v1/tracks/{id}/audio      Update audio file URL
```

## Database Schema

```sql
users
  - id (UUID, PK)
  - email (VARCHAR, UNIQUE)
  - username (VARCHAR, UNIQUE)
  - password_hash (VARCHAR)
  - created_at, updated_at

projects
  - id (UUID, PK)
  - user_id (UUID, FK)
  - title, artist, bpm, time_signature, key
  - metadata (JSONB)
  - created_at, updated_at

tracks
  - id (UUID, PK)
  - project_id (UUID, FK)
  - name, instrument, track_number
  - tablature (JSONB) - alphaTab format
  - audio_file_url (VARCHAR) - S3 URL
  - processors (JSONB) - EQ, compressor, effects
  - volume, pan, muted, soloed
  - created_at, updated_at

collaborators
  - id (UUID, PK)
  - project_id (UUID, FK)
  - user_id (UUID, FK)
  - role (ENUM: OWNER, EDITOR, VIEWER)

distributions
  - id (UUID, PK)
  - project_id (UUID, FK)
  - title, artist, album, genre, isrc, upc
  - artwork_url, platforms (JSONB)
  - status (ENUM: DRAFT, SUBMITTED, PUBLISHED)
```

## Development

### Build

```bash
./gradlew build
```

### Run Tests

```bash
./gradlew test
```

### Database Migrations

Uses **Flyway** for schema management.

Migrations are in: `src/main/resources/db/migration/`

- `V1__initial_schema.sql` - Core schema
- `V2__seed_test_data.sql` - Test data

### Test Data

The development environment includes:

**Test User**:
- Email: `test@maestro.ai`
- Username: `testuser`
- Password: `password123`
- User ID: `00000000-0000-0000-0000-000000000001`

**Sample Project**:
- Title: "Demo Song"
- Artist: "Test Artist"
- 4 tracks: Lead Guitar, Rhythm Guitar, Bass, Drums

### Environment Variables

```bash
# Database
DB_HOST=localhost
DB_PORT=5432
DB_NAME=maestro
DB_USER=postgres
DB_PASSWORD=postgres

# Redis
REDIS_HOST=localhost
REDIS_PORT=6379

# S3 / MinIO
S3_BUCKET=maestro-audio
S3_ENDPOINT=http://localhost:9000
AWS_REGION=us-east-1

# Audio Service (Rust gRPC)
AUDIO_SERVICE_HOST=localhost
AUDIO_SERVICE_PORT=50051

# Leviathan AI
LEVIATHAN_URL=http://localhost:8081
LEVIATHAN_API_KEY=

# JWT (TODO)
JWT_SECRET=changeme-in-production
```

## TODO

- [ ] JWT authentication implementation
- [ ] S3 file upload service
- [ ] Collaboration WebSocket handlers
- [ ] Distribution service
- [ ] Leviathan AI integration (gRPC client)
- [ ] Audio processing service integration (Rust gRPC client)
- [ ] User profile endpoints
- [ ] Project export/import (Guitar Pro files)
- [ ] Integration tests
- [ ] Kubernetes deployment configs

## Production Deployment

### Docker Build

```bash
./gradlew bootBuildImage
```

### Kubernetes

See `k8s/` directory (TODO) for deployment manifests.

## License

Copyright © 2024 Orpheus
