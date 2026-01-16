import { test, expect } from '@playwright/test';

/**
 * E2E Tests: Full Integration
 * Comprehensive end-to-end workflow testing the entire application
 */

test.describe('Full Integration Workflow', () => {
  test('should complete full music production workflow', async ({ page }) => {
    // 1. Create new project
    await page.goto('/');
    await page.click('[data-testid="new-project-button"]');

    await page.fill('[data-testid="project-title-input"]', 'My Epic Song');
    await page.fill('[data-testid="project-artist-input"]', 'The Maestros');
    await page.selectOption('[data-testid="project-bpm-input"]', '128');
    await page.selectOption('[data-testid="project-key-input"]', 'Em');
    await page.click('[data-testid="create-project-submit"]');

    await expect(page.locator('[data-testid="project-title"]')).toHaveText('My Epic Song');

    // 2. Add multiple tracks
    const instruments = ['drums', 'bass', 'electric-guitar', 'synth'];

    for (const instrument of instruments) {
      await page.click('[data-testid="add-track-button"]');
      await page.fill('[data-testid="track-name-input"]', instrument.toUpperCase());
      await page.selectOption('[data-testid="track-instrument-select"]', instrument);
      await page.click('[data-testid="create-track-submit"]');

      await expect(page.locator('[data-testid="track-list"]'))
        .toContainText(instrument.toUpperCase());
    }

    // Verify all tracks created
    const trackCount = await page.locator('[data-testid="track-item"]').count();
    expect(trackCount).toBe(4);

    // 3. Record audio on drums track
    const drumsTrack = page.locator('[data-testid="track-item"]')
      .filter({ hasText: 'DRUMS' });

    await drumsTrack.locator('[data-testid="track-record-arm"]').click();
    await page.click('[data-testid="transport-record-button"]');
    await page.click('[data-testid="transport-play-button"]');

    await page.waitForTimeout(3000);

    await page.click('[data-testid="transport-stop-button"]');

    // Verify recording created
    await expect(drumsTrack.locator('[data-testid="audio-region"]')).toBeVisible();

    // 4. Add effects to guitar track
    const guitarTrack = page.locator('[data-testid="track-item"]')
      .filter({ hasText: 'ELECTRIC-GUITAR' });

    await guitarTrack.locator('[data-testid="track-effects-button"]').click();

    // Add distortion
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-distortion"]');

    // Add EQ
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-eq"]');

    // Adjust EQ
    await page.locator('[data-testid="eq-band-0-freq"]').fill('2000');
    await page.locator('[data-testid="eq-band-0-gain"]').fill('6');

    // Add delay
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-delay"]');

    // 5. Mix tracks
    await drumsTrack.locator('[data-testid="track-volume-slider"]').fill('0.85');
    await page.locator('[data-testid="track-item"]')
      .filter({ hasText: 'BASS' })
      .locator('[data-testid="track-volume-slider"]').fill('0.80');

    await guitarTrack.locator('[data-testid="track-volume-slider"]').fill('0.75');
    await guitarTrack.locator('[data-testid="track-pan-slider"]').fill('0.7'); // Pan right

    // 6. Add master effects
    await page.click('[data-testid="master-bus"]');
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-compressor"]');

    await page.locator('[data-testid="compressor-threshold"]').fill('-12');
    await page.locator('[data-testid="compressor-ratio"]').fill('3');

    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-limiter"]');

    // 7. Set up automation
    await guitarTrack.click();
    await guitarTrack.locator('[data-testid="track-volume-slider"]').rightClick();
    await page.click('[data-testid="automate-parameter"]');

    const automationLane = page.locator('[data-testid="automation-lane"]');
    await automationLane.click({ position: { x: 50, y: 50 } });
    await automationLane.click({ position: { x: 200, y: 80 } });
    await automationLane.click({ position: { x: 350, y: 30 } });

    // 8. Set up loop region for playback
    await page.click('[data-testid="loop-button"]');
    await page.locator('[data-testid="timeline"]').click({ position: { x: 100, y: 50 } });
    await page.keyboard.press('I');
    await page.locator('[data-testid="timeline"]').click({ position: { x: 400, y: 50 } });
    await page.keyboard.press('O');

    // 9. Test playback with all features
    await page.click('[data-testid="metronome-button"]');
    await page.click('[data-testid="transport-play-button"]');

    await page.waitForTimeout(5000);

    // Verify meters showing activity
    for (const track of await page.locator('[data-testid="track-item"]').all()) {
      await expect(track.locator('[data-testid="peak-meter"]')).toBeVisible();
    }

    await page.click('[data-testid="transport-stop-button"]');

    // 10. Save project
    await page.waitForSelector('[data-testid="auto-save-indicator"]');
    await expect(page.locator('[data-testid="auto-save-indicator"]'))
      .toHaveText(/Saved/, { timeout: 10000 });

    // 11. Export final mix
    await page.click('[data-testid="select-all-button"]');

    const downloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="bounce-audio-button"]');
    await page.click('[data-testid="bounce-format-wav"]');
    await page.selectOption('[data-testid="bounce-sample-rate"]', '48000');
    await page.selectOption('[data-testid="bounce-bit-depth"]', '24');
    await page.click('[data-testid="bounce-confirm"]');

    await expect(page.locator('[data-testid="render-progress"]')).toBeVisible();
    await expect(page.locator('[data-testid="render-complete"]'))
      .toBeVisible({ timeout: 60000 });

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/\.wav$/);

    // 12. Verify project can be reopened
    await page.goto('/projects');
    await expect(page.locator('[data-testid="project-card"]')
      .filter({ hasText: 'My Epic Song' })).toBeVisible();

    await page.click('[data-testid="project-card"]');

    // Verify all tracks still there
    await expect(page.locator('[data-testid="track-list"]')).toContainText('DRUMS');
    await expect(page.locator('[data-testid="track-list"]')).toContainText('BASS');
    await expect(page.locator('[data-testid="track-list"]')).toContainText('ELECTRIC-GUITAR');
    await expect(page.locator('[data-testid="track-list"]')).toContainText('SYNTH');

    // Verify settings preserved
    await expect(page.locator('[data-testid="project-bpm"]')).toHaveText('128');
    await expect(page.locator('[data-testid="project-key"]')).toHaveText('Em');
  });

  test('should handle stress test with many tracks', async ({ page }) => {
    await page.goto('/');
    await page.click('[data-testid="new-project-button"]');
    await page.fill('[data-testid="project-title-input"]', 'Stress Test');
    await page.click('[data-testid="create-project-submit"]');

    // Create 20 tracks
    for (let i = 0; i < 20; i++) {
      await page.click('[data-testid="add-track-button"]');
      await page.fill('[data-testid="track-name-input"]', `Track ${i + 1}`);
      await page.click('[data-testid="create-track-submit"]');
    }

    // Verify all tracks created
    const trackCount = await page.locator('[data-testid="track-item"]').count();
    expect(trackCount).toBe(20);

    // Playback should still work
    await page.click('[data-testid="transport-play-button"]');
    await expect(page.locator('[data-testid="transport-play-button"]')).toHaveClass(/playing/);

    await page.waitForTimeout(2000);
    await page.click('[data-testid="transport-stop-button"]');

    // UI should remain responsive
    const responseTime = await page.evaluate(() => performance.now());
    await page.click('[data-testid="zoom-in-button"]');
    const afterClick = await page.evaluate(() => performance.now());

    expect(afterClick - responseTime).toBeLessThan(1000); // Under 1 second
  });

  test('should survive page refresh without data loss', async ({ page }) => {
    // Create project with data
    await page.goto('/');
    await page.click('[data-testid="new-project-button"]');
    await page.fill('[data-testid="project-title-input"]', 'Refresh Test');
    await page.click('[data-testid="create-project-submit"]');

    const projectId = await page.getAttribute('[data-testid="project-id"]', 'data-id');

    // Add track
    await page.click('[data-testid="add-track-button"]');
    await page.fill('[data-testid="track-name-input"]', 'Important Track');
    await page.click('[data-testid="create-track-submit"]');

    // Wait for auto-save
    await expect(page.locator('[data-testid="auto-save-indicator"]'))
      .toHaveText(/Saved/, { timeout: 5000 });

    // Refresh page
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Should still be on same project
    expect(await page.getAttribute('[data-testid="project-id"]', 'data-id')).toBe(projectId);

    // Track should still exist
    await expect(page.locator('[data-testid="track-list"]')).toContainText('Important Track');
  });

  test('should handle API errors gracefully', async ({ page }) => {
    await page.goto('/');
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');

    // Simulate network failure
    await page.route('**/api/v1/tracks/*', route => route.abort());

    // Try to add track
    await page.click('[data-testid="add-track-button"]');
    await page.fill('[data-testid="track-name-input"]', 'Failed Track');
    await page.click('[data-testid="create-track-submit"]');

    // Should show error message
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-message"]'))
      .toContainText(/Failed to create track/i);

    // Application should still be responsive
    await page.unroute('**/api/v1/tracks/*');
    await page.click('[data-testid="transport-play-button"]');
    await expect(page.locator('[data-testid="transport-play-button"]')).toHaveClass(/playing/);
  });

  test('should maintain audio sync across long playback', async ({ page }) => {
    await page.goto('/');
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');

    // Start playback
    await page.click('[data-testid="transport-play-button"]');

    // Let it run for 30 seconds
    await page.waitForTimeout(30000);

    // Verify playhead position is accurate
    const position = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');

    // Should be around 30 seconds (±0.5s tolerance for processing)
    expect(parseFloat(position!)).toBeGreaterThan(29);
    expect(parseFloat(position!)).toBeLessThan(31);

    // Verify no audio glitches reported
    await expect(page.locator('[data-testid="audio-error"]')).not.toBeVisible();
  });
});
