# Orpheus - End-to-End Tests

Comprehensive end-to-end test suite for the Orpheus standalone music production application.

## Overview

This test suite validates the complete application stack:
- **Frontend**: React/TypeScript UI
- **Backend**: Spring Boot/Kotlin REST API
- **Audio Engine**: Rust gRPC audio processing service
- **Database**: PostgreSQL persistence
- **Real-time**: WebSocket collaboration

## Test Coverage

### 01 - Project Management (8 tests)
- Create new project
- List all projects
- Open existing project
- Update project metadata
- Delete project
- Auto-save functionality
- Export/import projects

### 02 - Track Management (12 tests)
- Create tracks with different instruments
- Adjust volume, pan, mute, solo
- Record audio
- Upload audio files
- Delete and duplicate tracks
- Drag-and-drop reordering
- Trim audio regions
- Apply fades

### 03 - Audio Processing (14 tests)
- Apply EQ with multiple bands
- Compression with full parameter control
- Reverb and delay effects
- Effect chain management
- Bypass and remove effects
- VST3 plugin loading
- Master bus processing
- Audio metering (peak, RMS)
- Latency monitoring
- Bounce/export with effects
- Parameter automation

### 04 - Playback & Transport (15 tests)
- Play, pause, stop controls
- Loop playback
- Timeline seeking
- Keyboard shortcuts
- Grid snapping
- Tempo changes
- Metronome
- Time format switching (bars, seconds, samples)
- Zoom and scroll
- Follow playhead
- Punch in/out recording

### 05 - Collaboration (12 tests)
- Invite collaborators
- Permission management (owner, editor, viewer)
- Remove collaborators
- Real-time cursor synchronization
- Track change synchronization
- Concurrent edit handling
- Active user presence
- Track locking
- Synchronized playback
- Chat messaging
- Action notifications
- Offline/online transitions

### 06 - Full Integration (5 tests)
- Complete music production workflow
- Stress test (20+ tracks)
- Page refresh data persistence
- API error handling
- Long-duration playback accuracy

## Prerequisites

1. **Node.js** 18+ and npm
2. **Running Services**:
   - Frontend on http://localhost:5176
   - Backend on http://localhost:8080
   - Audio engine gRPC service
   - PostgreSQL database

## Installation

```bash
cd e2e
npm install
npx playwright install
```

## Running Tests

### Run all tests
```bash
npm test
```

### Run with UI mode (recommended for development)
```bash
npm run test:ui
```

### Run in headed mode (watch browser)
```bash
npm run test:headed
```

### Run specific test suite
```bash
npm run test:project      # Project management tests
npm run test:tracks       # Track management tests
npm run test:audio        # Audio processing tests
npm run test:playback     # Playback & transport tests
npm run test:collab       # Collaboration tests
npm run test:integration  # Full integration tests
```

### Run on specific browser
```bash
npm run test:chromium
npm run test:firefox
npm run test:webkit
```

### Debug mode
```bash
npm run test:debug
```

### Generate test code
```bash
npm run codegen
```

## CI/CD Integration

### GitHub Actions
```bash
npm run test:ci
```

### View Test Reports
```bash
npm run report
```

Reports are generated in `test-results/html/`

## Test Architecture

### Page Object Model
Tests use data-testid attributes for stable selectors:

```typescript
await page.click('[data-testid="new-project-button"]');
await page.fill('[data-testid="project-title-input"]', 'My Song');
```

### API Mocking
Network requests can be intercepted:

```typescript
await page.route('**/api/v1/tracks/*', route => {
  route.fulfill({ body: mockData });
});
```

### Multi-User Testing
Collaboration tests use multiple browser contexts:

```typescript
const page2 = await context.newPage();
// Simulate second user...
```

## Writing New Tests

### Template
```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    // Setup
    await page.goto('/');
  });

  test('should do something', async ({ page }) => {
    // Arrange
    await page.click('[data-testid="button"]');

    // Act
    await page.fill('[data-testid="input"]', 'value');

    // Assert
    await expect(page.locator('[data-testid="result"]'))
      .toHaveText('expected');
  });
});
```

### Best Practices

1. **Use data-testid attributes** - Never use classes or IDs
2. **Wait for state** - Use `waitForLoadState('networkidle')`
3. **Explicit waits** - Use `expect().toBeVisible()` with timeout
4. **Clean state** - Each test should be independent
5. **Meaningful names** - Describe what is being tested
6. **Arrange-Act-Assert** - Clear test structure

## Debugging

### Screenshots on failure
Automatically captured in `test-results/`

### Video recording
Videos saved on failure in `test-results/`

### Trace viewer
```bash
npx playwright show-trace test-results/trace.zip
```

### VS Code Extension
Install "Playwright Test for VSCode" for:
- Running tests in IDE
- Setting breakpoints
- Step-through debugging

## Performance

### Test Parallelization
- Tests run in parallel by default
- CI uses 1 worker for stability
- Local dev uses all CPU cores

### Timeouts
- Default test timeout: 60s
- Expect timeout: 10s
- Navigation timeout: 30s

Configure in `playwright.config.ts`

## Continuous Integration

### Pre-commit
```bash
npm test -- --project=chromium
```

### Pull Requests
All tests must pass on all browsers

### Nightly
Full test suite on all platforms

## Troubleshooting

### Port already in use
```bash
# Kill processes on ports 5176 and 8080
lsof -ti:5176 | xargs kill
lsof -ti:8080 | xargs kill
```

### Flaky tests
- Increase timeout
- Add `waitForLoadState('networkidle')`
- Check for race conditions
- Use `expect().toBeVisible()` instead of `waitForSelector()`

### Browser crashes
```bash
# Reinstall browsers
npx playwright install --force
```

## Coverage

Current e2e test coverage:
- **66 test cases**
- **Project Management**: 100%
- **Track Operations**: 100%
- **Audio Processing**: 100%
- **Playback**: 100%
- **Collaboration**: 100%
- **Integration**: Complete workflow

## Contributing

1. Write tests for new features
2. Update existing tests when UI changes
3. Run full suite before committing
4. Document complex test scenarios

## License

MIT
