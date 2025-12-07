# Orpheus - UX Improvement Roadmap

**Created**: 2025-11-18
**Based on**: UX_AUDIT_REPORT.md
**Current UX Score**: 6.5/10
**Target Score**: 9.0/10

---

## Overview

This roadmap addresses the 47 pain points identified in the UX audit, organized into 3 phases based on impact and effort. The goal is to transform Maestro from "technically impressive" to "genuinely delightful to use."

---

## Phase 1: Critical Fixes & Quick Wins (Target: 7.5/10)

**Timeline**: 2-3 days
**Goal**: Fix blocking issues and deliver immediate UX improvements

### Sprint 1A: Navigation & State Management (4 hours)

**Issues Fixed**: 5 critical navigation issues

- [ ] **Mode selector disabled states** - Disable modes that require a project when no project loaded
- [ ] **Improved "No Project" error states** - Add action buttons to error states
- [ ] **Loading indicators** - Add spinners/progress for file operations
- [ ] **Project requirement indicators** - Visual badges on mode buttons
- [ ] **Toolbar button states** - Disable Save/Export when no project

**Files to Modify**:
- `src/components/ModeSelector.tsx` - Add disabled logic
- `src/modes/*Mode.tsx` - Improve empty states
- `src/components/Toolbar.tsx` - Add loading states

**Expected Impact**: Users no longer hit dead-ends, understand which modes need projects

---

### Sprint 1B: Form Validation & Error Handling (3 hours)

**Issues Fixed**: 4 critical form/error issues

- [ ] **Distribute mode form validation** - Required field validation with inline errors
- [ ] **Confirmation dialogs** - Replace alert() with Fluent UI dialogs
- [ ] **Error boundaries** - Wrap modes in error boundaries
- [ ] **LocalStorage quota handling** - Catch and handle storage errors

**Files to Modify**:
- `src/modes/DistributeMode.tsx` - Add validation logic
- `src/components/ErrorBoundary.tsx` - Create new component
- `src/services/project-save.ts` - Add quota error handling

**Expected Impact**: Users get helpful error messages, no data loss from crashes

---

### Sprint 1C: Record Mode Critical Features (4 hours)

**Issues Fixed**: 3 critical recording issues

- [ ] **Microphone input selection** - Enumerate and select audio devices
- [ ] **Input level metering** - Real-time level indicator before recording
- [ ] **Visual recording indicator** - Pulsing red dot, border, duration timer

**Files to Modify**:
- `src/modes/RecordMode.tsx` - Add device selection dropdown
- `src/services/audio-recorder.ts` - Add device enumeration, level monitoring
- `src/components/InputLevelMeter.tsx` - Create new component

**Expected Impact**: Professional recording workflow, users can verify input before recording

---

### Sprint 1D: Onboarding & Help System (5 hours)

**Issues Fixed**: 4 high-priority documentation issues

- [ ] **First-run welcome modal** - 3-step tutorial for new users
- [ ] **Keyboard shortcuts help** - Modal with all shortcuts (press ?)
- [ ] **Mode tooltips enhancement** - Better descriptions in tooltips
- [ ] **AI Assistant hints** - Explain persona switching

**Files to Create**:
- `src/components/OnboardingDialog.tsx`
- `src/components/KeyboardShortcutsDialog.tsx`
- `src/components/HelpButton.tsx`

**Expected Impact**: New users understand the platform, power users discover shortcuts

---

## Phase 2: Core Functionality & Polish (Target: 8.5/10)

**Timeline**: 3-4 days
**Goal**: Make advertised features actually work

### Sprint 2A: Mix Mode Real Integration (8 hours)

**Issues Fixed**: THE critical Mix mode issue

- [ ] **Load tracks from project** - Use actual imported project tracks instead of demo data
- [ ] **Create audio nodes for tracks** - Wire up Tone.js Players for each track
- [ ] **Real-time metering** - Add VU meters to channel strips
- [ ] **Solo/Mute logic** - Implement proper solo isolation
- [ ] **Track color coding** - Add color selection for visual organization

**Files to Modify**:
- `src/modes/MixMode.tsx` - Complete rewrite of track loading
- `src/components/ChannelStripWithProcessors.tsx` - Add metering UI
- `src/services/audio-processing.ts` - Add track loading from project

**Expected Impact**: Mix mode becomes actually functional for mixing imported projects

---

### Sprint 2B: Master Mode Real Metering (6 hours)

**Issues Fixed**: 3 high-priority Master mode issues

- [ ] **Real LUFS analysis** - Use Tone.Meter for actual loudness measurement
- [ ] **Platform target application** - Actually apply -14 LUFS etc. when selected
- [ ] **Functional export** - Wire up export buttons to actually export audio
- [ ] **A/B comparison** - Add bypass toggle to compare before/after

**Files to Modify**:
- `src/modes/MasterMode.tsx` - Add real metering logic
- `src/services/audio-processing.ts` - Add LUFS calculation
- `src/services/export.ts` - Add audio export functionality

**Expected Impact**: Master mode becomes functional for real mastering work

---

### Sprint 2C: Tab Editor Improvements (6 hours)

**Issues Fixed**: 4 medium-priority editing issues

- [ ] **alphaTab sync** - Update alphaTab view when fretboard edited
- [ ] **Undo/Redo in editor** - Local undo stack for tab editing
- [ ] **Keyboard note entry** - Arrow keys to move, number keys for frets
- [ ] **Export to Guitar Pro** - Add .gp5 export (if alphaTab supports)

**Files to Modify**:
- `src/components/TabEditor.tsx` - Add sync logic, undo stack
- `src/hooks/useAlphaTab.ts` - Add re-render trigger
- `src/services/export.ts` - Add GP export

**Expected Impact**: Tab editing becomes smooth and professional

---

### Sprint 2D: Practice Mode Integration (5 hours)

**Issues Fixed**: 3 high-priority Practice mode issues

- [ ] **Connect to project playback** - Play imported tab at trainer speed
- [ ] **Functional loop section** - Actually loop measures when enabled
- [ ] **Visual beat indicators** - Pulsing dots in sync with metronome
- [ ] **Practice statistics** - Track sessions, progress over time

**Files to Modify**:
- `src/modes/PracticeMode.tsx` - Add project integration
- `src/components/SpeedTrainer.tsx` - Wire to playback coordinator
- `src/services/playback-coordinator.ts` - Add loop section support

**Expected Impact**: Practice mode becomes useful for actual practice sessions

---

### Sprint 2E: Distribute Mode Functionality (6 hours)

**Issues Fixed**: 5 high-priority Distribute mode issues

- [ ] **Functional artwork upload** - Add file input, preview, validation
- [ ] **Date picker** - Replace text input with proper date picker
- [ ] **Genre expansion** - Add comprehensive genre list (50+ options)
- [ ] **ISRC/UPC help** - Add tooltips explaining where to get codes
- [ ] **Draft auto-save** - Save form to localStorage on change

**Files to Modify**:
- `src/modes/DistributeMode.tsx` - Add upload, date picker, help tooltips
- `src/components/ArtworkUpload.tsx` - Create new component
- `src/services/distribute.ts` - Add draft save logic

**Expected Impact**: Users can prepare complete release packages

---

## Phase 3: Advanced Features & Accessibility (Target: 9.0/10)

**Timeline**: 4-5 days
**Goal**: Professional-grade polish and accessibility

### Sprint 3A: Accessibility Overhaul (8 hours)

**Issues Fixed**: 7 WCAG violations

- [ ] **Add aria-labels to all icon buttons** - Screen reader support
- [ ] **Keyboard navigation complete** - Tab through all controls
- [ ] **Focus indicators** - Visible focus rings on custom components
- [ ] **Loading state announcements** - aria-live regions for spinners
- [ ] **Form label associations** - Proper label/input relationships
- [ ] **Color contrast fixes** - Meet WCAG AA standards (4.5:1)
- [ ] **High contrast mode** - Add high contrast theme variant

**Files to Modify**:
- All component files - Add aria attributes
- `src/styles/theme.ts` - Add high contrast theme
- `src/App.tsx` - Add global keyboard navigation

**Expected Impact**: Platform usable by screen reader users, meets WCAG AA

---

### Sprint 3B: Advanced Mixing Features (10 hours)

**Issues Fixed**: 3 professional mixing requirements

- [ ] **Aux sends/returns** - Add auxiliary bus architecture
- [ ] **Automation lanes** - Volume/pan automation over time
- [ ] **Master bus processing** - Glue compressor, bus EQ

**Files to Create**:
- `src/components/AuxSend.tsx`
- `src/components/AutomationLane.tsx`
- `src/services/automation.ts`

**Expected Impact**: Pro-level mixing capabilities

---

### Sprint 3C: AI Feature Enhancement (6 hours)

**Issues Fixed**: 3 AI assistant issues

- [ ] **Quick actions persistent** - Suggestions tab always visible
- [ ] **Auto-execute AI actions** - "AI Auto-Master" actually processes
- [ ] **AI Mix Suggestions auto-send** - Don't just open panel, send prompt
- [ ] **Context-aware suggestions** - AI sees current project state

**Files to Modify**:
- `src/components/AIAssistant.tsx` - Add persistent suggestions
- `src/components/AIQuickActions.tsx` - Add auto-execute
- `src/services/ai-chat.ts` - Add context injection

**Expected Impact**: AI becomes truly helpful, not just conversational

---

### Sprint 3D: Performance Optimization (4 hours)

**Issues Fixed**: 5 performance issues

- [ ] **Dynamic alphaTab import** - Only load in Compose mode
- [ ] **Smart auto-save** - Only save on actual changes
- [ ] **Waveform animation pause** - Stop when component hidden
- [ ] **Toast auto-dismiss** - Prevent memory leak
- [ ] **Bundle size analysis** - Optimize imports

**Files to Modify**:
- `src/modes/ComposeMode.tsx` - Dynamic import
- `src/services/project-save.ts` - Smart saving
- `src/components/WaveformVisualizer.tsx` - Pause logic

**Expected Impact**: Faster load times, better performance

---

### Sprint 3E: Mobile Responsiveness (8 hours)

**Issues Fixed**: 6 mobile layout issues

- [ ] **Responsive mode selector** - Carousel or dropdown on mobile
- [ ] **Mobile sidebar** - Full-screen overlay pattern
- [ ] **Touch-optimized fretboard** - Larger tap targets
- [ ] **Vertical channel strips** - Stack on mobile
- [ ] **Responsive forms** - Stack form fields on mobile
- [ ] **Mobile detection** - Show "Desktop recommended" banner

**Files to Modify**:
- All major components - Add responsive styles
- `src/App.tsx` - Add mobile detection

**Expected Impact**: Usable on tablets, informative on phones

---

## Quick Wins Checklist (Do First!)

These high-impact, low-effort fixes should be completed immediately:

### 30-Minute Fixes
- [x] ~~Add disabled states to mode selector~~ (will do)
- [x] ~~Replace alert() with Fluent UI Dialog~~ (will do)
- [x] ~~Add visual recording indicator CSS~~ (will do)
- [x] ~~Add action buttons to "No Project" states~~ (will do)
- [x] ~~Fix toast auto-dismiss~~ (will do)

### 1-Hour Fixes
- [ ] Add form validation to Distribute mode
- [ ] Add confirmation dialogs for delete actions
- [ ] Add keyboard shortcuts modal
- [ ] Add loading spinners to toolbar buttons
- [ ] Add project requirement badges to modes

### 2-Hour Fixes
- [ ] Implement onboarding tutorial
- [ ] Add error boundaries
- [ ] Add microphone device selection
- [ ] Add input level metering
- [ ] Fix auto-save to only save on changes

---

## Success Metrics

### Phase 1 Success Criteria
- ✅ Zero "No Project Loaded" dead-ends without guidance
- ✅ All forms validate before submission
- ✅ Recording workflow has input verification
- ✅ New users complete onboarding tutorial
- ✅ Error recovery rate > 90%

### Phase 2 Success Criteria
- ✅ Mix mode successfully mixes imported projects
- ✅ Master mode produces valid mastered audio
- ✅ Tab editor changes reflect in real-time
- ✅ Practice mode connects to project playback
- ✅ Distribute mode completes full release flow

### Phase 3 Success Criteria
- ✅ WCAG AA compliance (automated testing passes)
- ✅ Lighthouse Accessibility score > 90
- ✅ 100% keyboard navigable
- ✅ Mobile lighthouse score > 80
- ✅ Bundle size < 2MB gzipped

---

## Timeline Summary

| Phase | Duration | UX Score Target | Focus |
|-------|----------|-----------------|-------|
| **Phase 1** | 2-3 days | 7.5/10 | Critical fixes & quick wins |
| **Phase 2** | 3-4 days | 8.5/10 | Core functionality |
| **Phase 3** | 4-5 days | 9.0/10 | Polish & accessibility |
| **Total** | 9-12 days | 9.0/10 | Production excellence |

---

## Implementation Order

### Today (Day 1)
1. ✅ Create roadmap
2. 🏃 Sprint 1A: Navigation & State Management (4h)
3. 🏃 Sprint 1B: Form Validation (3h)

### Day 2
4. Sprint 1C: Record Mode Features (4h)
5. Sprint 1D: Onboarding System (5h)

### Day 3
6. Sprint 2A: Mix Mode Integration (8h)

### Day 4
7. Sprint 2B: Master Mode Metering (6h)

### Days 5-6
8. Sprint 2C: Tab Editor (6h)
9. Sprint 2D: Practice Mode (5h)
10. Sprint 2E: Distribute Mode (6h)

### Days 7-9
11. Sprint 3A: Accessibility (8h)
12. Sprint 3B: Advanced Mixing (10h)

### Days 10-12
13. Sprint 3C: AI Enhancement (6h)
14. Sprint 3D: Performance (4h)
15. Sprint 3E: Mobile (8h)

---

## Dependencies & Risks

### Technical Dependencies
- **alphaTab API**: Tab editor sync depends on library capabilities
- **Tone.js LUFS**: Need to implement or find library for loudness metering
- **Web Audio API**: Input device selection requires permissions

### Risk Mitigation
- **Scope creep**: Stick to roadmap, defer new features
- **alphaTab limitations**: If export not supported, document limitation
- **Browser compatibility**: Test in Chrome, Firefox, Safari before shipping
- **Performance regression**: Benchmark before/after each sprint

---

## Post-Roadmap Maintenance

After reaching 9.0/10, maintain quality with:

1. **User testing sessions** - Monthly feedback from real musicians
2. **Analytics integration** - Track feature usage, error rates
3. **A/B testing** - Test UX variations
4. **Continuous accessibility audits** - Quarterly WCAG checks
5. **Performance budgets** - Bundle size limits, load time targets

---

## Notes

- This roadmap is aggressive but achievable
- Prioritizes user-facing impact over internal refactoring
- Each sprint delivers tangible UX improvements
- Can adjust timeline based on team capacity
- Quick wins build momentum for larger efforts

**Let's ship excellence!** 🚀

---

**End of Roadmap**
*Ready to execute: Start with Sprint 1A*
