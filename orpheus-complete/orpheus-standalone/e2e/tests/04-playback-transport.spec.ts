import { test, expect } from '@playwright/test';

/**
 * E2E Tests: Playback & Transport
 * Tests playback controls, timeline navigation, and transport features
 */

test.describe('Playback & Transport', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');
    await page.waitForSelector('[data-testid="project-workspace"]');
  });

  test('should play and stop audio', async ({ page }) => {
    // Click play
    await page.click('[data-testid="transport-play-button"]');

    // Verify playing state
    await expect(page.locator('[data-testid="transport-play-button"]')).toHaveClass(/playing/);
    await expect(page.locator('[data-testid="playhead"]')).toHaveCSS('animation-play-state', 'running');

    // Click stop
    await page.click('[data-testid="transport-stop-button"]');

    // Verify stopped
    await expect(page.locator('[data-testid="transport-play-button"]')).not.toHaveClass(/playing/);

    // Verify playhead returned to start
    const playheadPosition = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');
    expect(parseFloat(playheadPosition!)).toBe(0);
  });

  test('should pause and resume playback', async ({ page }) => {
    // Start playback
    await page.click('[data-testid="transport-play-button"]');
    await page.waitForTimeout(1000);

    // Pause
    await page.click('[data-testid="transport-pause-button"]');

    // Get current position
    const pausePosition = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');

    // Wait a bit
    await page.waitForTimeout(500);

    // Verify position didn't change
    const stillPausedPosition = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');
    expect(stillPausedPosition).toBe(pausePosition);

    // Resume
    await page.click('[data-testid="transport-play-button"]');
    await page.waitForTimeout(500);

    // Verify playback resumed
    const resumedPosition = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');
    expect(parseFloat(resumedPosition!)).toBeGreaterThan(parseFloat(pausePosition!));
  });

  test('should loop playback', async ({ page }) => {
    // Set loop region
    await page.click('[data-testid="loop-button"]');

    // Set loop start
    await page.locator('[data-testid="timeline"]').click({ position: { x: 100, y: 50 } });
    await page.keyboard.press('I'); // Loop in

    // Set loop end
    await page.locator('[data-testid="timeline"]').click({ position: { x: 300, y: 50 } });
    await page.keyboard.press('O'); // Loop out

    // Verify loop region visible
    await expect(page.locator('[data-testid="loop-region"]')).toBeVisible();

    // Start playback
    await page.click('[data-testid="transport-play-button"]');

    // Wait for loop to cycle
    await page.waitForTimeout(3000);

    // Verify playhead looped back
    const position = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');
    const loopStart = await page.locator('[data-testid="loop-start"]')
      .getAttribute('data-position');

    expect(parseFloat(position!)).toBeGreaterThanOrEqual(parseFloat(loopStart!));
  });

  test('should seek to position', async ({ page }) => {
    const timeline = page.locator('[data-testid="timeline"]');

    // Click on timeline
    await timeline.click({ position: { x: 200, y: 50 } });

    // Verify playhead moved
    const position = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');

    expect(parseFloat(position!)).toBeGreaterThan(0);
  });

  test('should navigate with keyboard shortcuts', async ({ page }) => {
    // Space to play/pause
    await page.keyboard.press('Space');
    await expect(page.locator('[data-testid="transport-play-button"]')).toHaveClass(/playing/);

    await page.keyboard.press('Space');
    await expect(page.locator('[data-testid="transport-play-button"]')).not.toHaveClass(/playing/);

    // Home to go to start
    await page.locator('[data-testid="timeline"]').click({ position: { x: 200, y: 50 } });
    await page.keyboard.press('Home');

    const position = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');
    expect(parseFloat(position!)).toBe(0);

    // End to go to end
    await page.keyboard.press('End');
    const endPosition = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');
    expect(parseFloat(endPosition!)).toBeGreaterThan(0);
  });

  test('should snap to grid', async ({ page }) => {
    // Enable snap
    await page.click('[data-testid="snap-button"]');
    await expect(page.locator('[data-testid="snap-button"]')).toHaveClass(/active/);

    // Set snap to 1/4 note
    await page.click('[data-testid="snap-menu"]');
    await page.click('[data-testid="snap-quarter"]');

    // Click on timeline
    await page.locator('[data-testid="timeline"]').click({ position: { x: 123, y: 50 } });

    // Verify snapped to grid
    const position = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');

    // Position should be on a quarter note boundary
    const tempo = 120; // BPM
    const quarterNoteTime = 60 / tempo;
    const remainder = parseFloat(position!) % quarterNoteTime;
    expect(remainder).toBeLessThan(0.01);
  });

  test('should change tempo', async ({ page }) => {
    // Open tempo settings
    await page.click('[data-testid="tempo-display"]');

    // Change tempo
    await page.fill('[data-testid="tempo-input"]', '140');
    await page.keyboard.press('Enter');

    // Verify tempo changed
    await expect(page.locator('[data-testid="tempo-display"]')).toHaveText('140');

    // Verify playback speed affected
    await page.click('[data-testid="transport-play-button"]');
    await page.waitForTimeout(1000);

    // Position should advance faster
    const position = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');
    expect(parseFloat(position!)).toBeGreaterThan(0.5);
  });

  test('should metronome click during playback', async ({ page }) => {
    // Enable metronome
    await page.click('[data-testid="metronome-button"]');
    await expect(page.locator('[data-testid="metronome-button"]')).toHaveClass(/active/);

    // Start playback
    await page.click('[data-testid="transport-play-button"]');

    // Verify metronome visual feedback
    await expect(page.locator('[data-testid="metronome-indicator"]')).toHaveClass(/flashing/);
  });

  test('should show time in different formats', async ({ page }) => {
    const timeDisplay = page.locator('[data-testid="time-display"]');

    // Default should be bars:beats
    await expect(timeDisplay).toHaveText(/\d+:\d+:\d+/);

    // Switch to seconds
    await timeDisplay.click();
    await page.click('[data-testid="time-format-seconds"]');
    await expect(timeDisplay).toHaveText(/\d+\.\d+s/);

    // Switch to samples
    await timeDisplay.click();
    await page.click('[data-testid="time-format-samples"]');
    await expect(timeDisplay).toHaveText(/\d+ samples/);
  });

  test('should zoom timeline in and out', async ({ page }) => {
    const timeline = page.locator('[data-testid="timeline"]');
    const initialWidth = await timeline.evaluate(el => el.scrollWidth);

    // Zoom in
    await page.click('[data-testid="zoom-in-button"]');
    const zoomedInWidth = await timeline.evaluate(el => el.scrollWidth);
    expect(zoomedInWidth).toBeGreaterThan(initialWidth);

    // Zoom out
    await page.click('[data-testid="zoom-out-button"]');
    await page.click('[data-testid="zoom-out-button"]');
    const zoomedOutWidth = await timeline.evaluate(el => el.scrollWidth);
    expect(zoomedOutWidth).toBeLessThan(zoomedInWidth);

    // Zoom to fit
    await page.click('[data-testid="zoom-fit-button"]');
    // Timeline should show all content
  });

  test('should scroll timeline horizontally', async ({ page }) => {
    const timeline = page.locator('[data-testid="timeline-scroll"]');

    // Scroll right
    await timeline.evaluate(el => el.scrollLeft = 500);
    const scrollPosition = await timeline.evaluate(el => el.scrollLeft);
    expect(scrollPosition).toBeGreaterThan(0);
  });

  test('should follow playhead during playback', async ({ page }) => {
    // Enable follow mode
    await page.click('[data-testid="follow-playhead-button"]');

    // Start playback
    await page.click('[data-testid="transport-play-button"]');

    // Wait for playback to advance
    await page.waitForTimeout(2000);

    // Verify timeline scrolled to keep playhead visible
    const timeline = page.locator('[data-testid="timeline-scroll"]');
    const scrollPosition = await timeline.evaluate(el => el.scrollLeft);
    expect(scrollPosition).toBeGreaterThan(0);
  });

  test('should punch in/out recording', async ({ page }) => {
    // Set punch in point
    await page.locator('[data-testid="timeline"]').click({ position: { x: 100, y: 50 } });
    await page.keyboard.press('Control+I');

    // Set punch out point
    await page.locator('[data-testid="timeline"]').click({ position: { x: 300, y: 50 } });
    await page.keyboard.press('Control+O');

    // Verify punch region visible
    await expect(page.locator('[data-testid="punch-region"]')).toBeVisible();

    // Arm track
    const track = page.locator('[data-testid="track-item"]').first();
    await track.locator('[data-testid="track-record-arm"]').click();

    // Start playback and record
    await page.click('[data-testid="transport-record-button"]');
    await page.click('[data-testid="transport-play-button"]');

    // Recording should only happen in punch region
    await page.waitForTimeout(3000);

    // Stop
    await page.click('[data-testid="transport-stop-button"]');

    // Verify recording only in punch region
    const recording = track.locator('[data-testid="audio-region"]').last();
    const recordingStart = await recording.getAttribute('data-start');
    const punchStart = await page.locator('[data-testid="punch-region"]')
      .getAttribute('data-start');

    expect(parseFloat(recordingStart!)).toBeCloseTo(parseFloat(punchStart!), 1);
  });
});
