# Integration Tests

Comprehensive integration tests for the Orpheus standalone application UI.

## Overview

Integration tests verify how multiple components work together across complete user workflows. Unlike unit tests that test components in isolation, these tests validate:

- **Component Interactions**: How components communicate and share state
- **Data Flow**: API calls, state updates, and re-renders across the application
- **User Workflows**: Complete end-to-end scenarios from a user's perspective
- **Error Handling**: Recovery from failures across multiple components
- **Performance**: Behavior under load with many tracks and complex projects

## Test Suites

### 1. Project Workflow Integration (`project-workflow.integration.test.tsx`)
**30+ tests** covering project creation, management, and navigation:

- Create new project and navigate to workspace
- Load existing projects and open workspace
- Delete projects with confirmation
- Update project metadata (BPM, time signature)
- Handle project creation errors and retry
- Navigate between projects list and workspace
- Browser back button behavior

**Key Scenarios:**
```typescript
// Complete project creation flow
Create project → Fill form → Submit → Navigate to workspace → Verify display

// Project deletion flow
Load projects → Delete → Confirm → API call → Refresh list → Verify removed
```

### 2. Track Management Integration (`track-workflow.integration.test.tsx`)
**25+ tests** covering track creation, editing, and playback:

- Create and configure multiple tracks
- Adjust volume, pan, mute, solo across tracks
- Record audio and create regions
- Track and timeline synchronization
- Track and mixer panel synchronization
- Handle mute/solo interactions

**Key Scenarios:**
```typescript
// Multi-track creation
Add drums → Add bass → Add guitar → Verify all visible → Adjust levels

// Recording workflow
Arm track → Start recording → Record → Stop → Verify region created
```

### 3. Mixing Workflow Integration (`mixing-workflow.integration.test.tsx`)
**35+ tests** covering mixing, effects, and audio processing:

- Build complex effects chains (EQ → Compressor → Reverb → Delay)
- Configure effect parameters
- Bypass effects while maintaining chain
- Multi-track mixing with different settings
- Master bus processing
- Volume automation
- Export/bounce with effects applied

**Key Scenarios:**
```typescript
// Effects chain workflow
Add EQ → Configure bands → Add compressor → Set threshold/ratio →
Add reverb → Adjust wet/dry → Play and monitor

// Master bus workflow
Open master bus → Add limiter → Configure → Apply to mix → Export
```

### 4. Collaboration Integration (`collaboration-workflow.integration.test.tsx`)
**30+ tests** covering real-time collaboration features:

- Share project with collaborators
- Manage collaborator roles (owner/editor/viewer)
- Remove collaborators
- Show active collaborator presence
- Online/offline status transitions
- Chat messaging
- Synchronize track changes across users
- Handle concurrent edits
- Permission-based action restrictions

**Key Scenarios:**
```typescript
// Sharing workflow
Open share dialog → Enter email → Select role → Send invite →
Verify in list → Change role → Remove collaborator

// Real-time collaboration
User 1 edits track → Update syncs → User 2 sees change →
Send chat message → Notification appears
```

### 5. Full Application Integration (`full-application.integration.test.tsx`)
**20+ tests** covering complete production workflows:

- **Complete Music Production**: Create project → Add tracks → Mix → Add effects → Export
- **Collaborative Production**: Multi-user session with presence
- **Error Recovery**: Network failures, auto-save retry, graceful degradation
- **Performance Under Load**: 20+ track projects, UI responsiveness
- **Data Persistence**: Page refresh without data loss

**Key Scenario - Full Production Workflow:**
```typescript
1. Create new project ("My Epic Song", 128 BPM, Em)
2. Add 3 tracks (Drums, Bass, Guitar)
3. Set volumes (0.85, 0.80, 0.75)
4. Pan guitar right (0.7)
5. Add EQ to guitar
6. Add compressor to drums
7. Set loop points
8. Play and monitor
9. Export as 24-bit/48kHz WAV
```

## Running Integration Tests

### Run all integration tests
```bash
npm run test:integration
```

### Run specific test suite
```bash
npm run test:integration -- project-workflow
npm run test:integration -- track-workflow
npm run test:integration -- mixing-workflow
npm run test:integration -- collaboration-workflow
npm run test:integration -- full-application
```

### Run with UI (watch mode)
```bash
npm run test:integration:ui
```

### Run with coverage
```bash
npm run test:integration:coverage
```

## Test Patterns

### 1. Multi-Component Testing
```typescript
render(
  <BrowserRouter>
    <App />
  </BrowserRouter>
);

// Tests navigation between routes
// Tests state sharing across components
// Tests real routing behavior
```

### 2. API Integration (Not Mocked)
```typescript
mockFetch
  .mockResolvedValueOnce({ ok: true, json: async () => mockData1 })
  .mockResolvedValueOnce({ ok: true, json: async () => mockData2 });

// Simulates real API call sequences
// Tests error handling across API calls
// Verifies correct API usage
```

### 3. User Workflow Simulation
```typescript
// Simulate complete user action sequence
fireEvent.click(addButton);
fireEvent.change(input, { target: { value: 'test' } });
fireEvent.click(submitButton);

await waitFor(() => {
  expect(screen.getByText('Success')).toBeInTheDocument();
});
```

### 4. Async State Synchronization
```typescript
// User 1 makes change
fireEvent.change(volumeSlider, { target: { value: '0.5' } });

// Wait for API call
await waitFor(() => {
  expect(mockFetch).toHaveBeenCalledWith('/api/tracks/1', ...);
});

// User 2 should see update
await waitFor(() => {
  expect(otherUserComponent).toHaveValue('0.5');
});
```

## Best Practices

### 1. Test Complete Workflows
✅ Test from user action to final result
✅ Include all intermediate steps
✅ Verify state at each critical point

### 2. Use Realistic Data
✅ Use realistic project structures
✅ Include edge cases (empty states, max values)
✅ Test with multiple entities (tracks, effects)

### 3. Handle Async Operations
✅ Always use `waitFor` for async updates
✅ Test loading states
✅ Verify error states

### 4. Clean Test Isolation
```typescript
beforeEach(() => {
  mockFetch = jest.fn();
  global.fetch = mockFetch;
});

afterEach(() => {
  jest.clearAllMocks();
});
```

### 5. Descriptive Test Names
```typescript
it('should complete full song creation workflow from start to export', async () => {
  // Test body clearly shows the complete workflow
});
```

## Coverage Goals

Integration tests focus on:
- ✅ **User Workflows** - 100% of critical paths
- ✅ **Component Integration** - All major component interactions
- ✅ **API Integration** - All endpoints used correctly
- ✅ **Error Scenarios** - Recovery from all error types
- ✅ **Edge Cases** - Empty states, max values, concurrency

## Debugging Integration Tests

### View test execution
```bash
npm run test:integration:ui
```

### Debug specific test
```bash
npm run test:integration -- --grep "should complete full song creation"
```

### Enable verbose logging
```typescript
// Add to test file
console.log('Current state:', screen.debug());
```

### Check API calls
```typescript
console.log('Fetch calls:', mockFetch.mock.calls);
```

## Performance Considerations

Integration tests are slower than unit tests because they:
- Render complete component trees
- Simulate real user interactions
- Wait for async operations
- Test full workflows

**Optimization strategies:**
- Run unit tests first for quick feedback
- Run integration tests before commits
- Use CI/CD for full integration test suite
- Parallelize test execution when possible

## Continuous Integration

Integration tests are run:
- ✅ On pull requests (all suites)
- ✅ Before merges (required passing)
- ✅ Nightly (full suite with coverage)
- ✅ On release branches (comprehensive validation)

## Contributing

When adding new features:

1. **Add integration tests** for new workflows
2. **Update existing tests** if behavior changes
3. **Document complex scenarios** in test descriptions
4. **Ensure tests are deterministic** and isolated
5. **Run full suite** before creating PR

## License

MIT
