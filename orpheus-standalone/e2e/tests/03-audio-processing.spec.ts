import { test, expect } from '@playwright/test';

/**
 * E2E Tests: Audio Processing
 * Tests audio effects, mixing, and mastering features
 */

test.describe('Audio Processing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');
    await page.waitForSelector('[data-testid="project-workspace"]');
  });

  test('should apply EQ to track', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();

    // Open effects panel
    await track.locator('[data-testid="track-effects-button"]').click();

    // Add EQ
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-eq"]');

    // Adjust EQ bands
    await page.locator('[data-testid="eq-band-0-freq"]').fill('100');
    await page.locator('[data-testid="eq-band-0-gain"]').fill('3');
    await page.locator('[data-testid="eq-band-0-q"]').fill('1.0');

    // Verify EQ appears in chain
    await expect(page.locator('[data-testid="effect-eq"]')).toBeVisible();

    // Verify audio processing API called
    await page.waitForResponse(response =>
      response.url().includes('/api/v1/tracks') &&
      response.request().method() === 'PUT'
    );
  });

  test('should apply compression to track', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();

    await track.locator('[data-testid="track-effects-button"]').click();
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-compressor"]');

    // Set compressor parameters
    await page.locator('[data-testid="compressor-threshold"]').fill('-20');
    await page.locator('[data-testid="compressor-ratio"]').fill('4');
    await page.locator('[data-testid="compressor-attack"]').fill('10');
    await page.locator('[data-testid="compressor-release"]').fill('100');

    await expect(page.locator('[data-testid="effect-compressor"]')).toBeVisible();
  });

  test('should apply reverb to track', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();

    await track.locator('[data-testid="track-effects-button"]').click();
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-reverb"]');

    // Adjust reverb parameters
    await page.locator('[data-testid="reverb-room-size"]').fill('0.7');
    await page.locator('[data-testid="reverb-decay"]').fill('1.5');
    await page.locator('[data-testid="reverb-wet-dry"]').fill('0.3');

    await expect(page.locator('[data-testid="effect-reverb"]')).toBeVisible();
  });

  test('should apply delay to track', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();

    await track.locator('[data-testid="track-effects-button"]').click();
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-delay"]');

    // Set delay parameters
    await page.locator('[data-testid="delay-time"]').fill('250');
    await page.locator('[data-testid="delay-feedback"]').fill('0.4');
    await page.locator('[data-testid="delay-wet-dry"]').fill('0.25');

    await expect(page.locator('[data-testid="effect-delay"]')).toBeVisible();
  });

  test('should reorder effects in chain', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    await track.locator('[data-testid="track-effects-button"]').click();

    // Add multiple effects
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-eq"]');

    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-compressor"]');

    // Drag compressor before EQ
    const compressor = page.locator('[data-testid="effect-compressor"]');
    const eq = page.locator('[data-testid="effect-eq"]');

    await compressor.dragTo(eq);

    // Verify order changed
    const effects = page.locator('[data-testid^="effect-"]');
    const firstEffect = await effects.first().getAttribute('data-testid');
    expect(firstEffect).toBe('effect-compressor');
  });

  test('should bypass effect', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    await track.locator('[data-testid="track-effects-button"]').click();

    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-eq"]');

    const eq = page.locator('[data-testid="effect-eq"]');

    // Bypass effect
    await eq.locator('[data-testid="effect-bypass-button"]').click();
    await expect(eq).toHaveClass(/bypassed/);

    // Re-enable
    await eq.locator('[data-testid="effect-bypass-button"]').click();
    await expect(eq).not.toHaveClass(/bypassed/);
  });

  test('should remove effect from chain', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    await track.locator('[data-testid="track-effects-button"]').click();

    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-eq"]');

    const initialCount = await page.locator('[data-testid^="effect-"]').count();

    // Remove effect
    await page.locator('[data-testid="effect-eq"]')
      .locator('[data-testid="effect-remove-button"]').click();

    const newCount = await page.locator('[data-testid^="effect-"]').count();
    expect(newCount).toBe(initialCount - 1);
  });

  test('should load VST3 plugin', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    await track.locator('[data-testid="track-effects-button"]').click();

    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-plugin"]');

    // Browse for plugin
    await page.click('[data-testid="browse-plugins-button"]');

    // Select plugin from list
    await page.click('[data-testid="plugin-item"]').first;
    await page.click('[data-testid="load-plugin-button"]');

    // Verify plugin loaded
    await expect(page.locator('[data-testid="plugin-window"]')).toBeVisible();
  });

  test('should apply master effects', async ({ page }) => {
    // Click master bus
    await page.click('[data-testid="master-bus"]');

    // Add mastering effects
    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-limiter"]');

    // Set limiter threshold
    await page.locator('[data-testid="limiter-threshold"]').fill('-0.1');
    await page.locator('[data-testid="limiter-release"]').fill('50');

    await expect(page.locator('[data-testid="effect-limiter"]')).toBeVisible();
  });

  test('should show audio meters', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();

    // Start playback
    await page.click('[data-testid="transport-play-button"]');

    // Verify meters are active
    await expect(track.locator('[data-testid="peak-meter"]')).toBeVisible();
    await expect(track.locator('[data-testid="rms-meter"]')).toBeVisible();

    // Verify meters show activity
    const meterValue = await track.locator('[data-testid="peak-meter"]')
      .getAttribute('data-level');
    expect(parseFloat(meterValue!)).toBeGreaterThan(-80);
  });

  test('should monitor audio latency', async ({ page }) => {
    // Open performance panel
    await page.click('[data-testid="performance-button"]');

    // Check latency display
    const latency = page.locator('[data-testid="audio-latency"]');
    await expect(latency).toBeVisible();

    const latencyMs = await latency.textContent();
    expect(parseFloat(latencyMs!)).toBeGreaterThan(0);
    expect(parseFloat(latencyMs!)).toBeLessThan(100); // Under 100ms
  });

  test('should export bounced audio', async ({ page }) => {
    // Select region to bounce
    await page.click('[data-testid="select-all-button"]');

    // Click bounce
    const downloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="bounce-audio-button"]');

    // Select format
    await page.click('[data-testid="bounce-format-wav"]');
    await page.click('[data-testid="bounce-confirm"]');

    // Wait for render
    await expect(page.locator('[data-testid="render-progress"]')).toBeVisible();
    await expect(page.locator('[data-testid="render-complete"]')).toBeVisible({ timeout: 30000 });

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/\.wav$/);
  });

  test('should apply automation to effect parameter', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    await track.locator('[data-testid="track-effects-button"]').click();

    await page.click('[data-testid="add-effect-button"]');
    await page.click('[data-testid="effect-eq"]');

    // Enable automation
    const eq = page.locator('[data-testid="effect-eq"]');
    await eq.locator('[data-testid="eq-band-0-gain"]').rightClick();
    await page.click('[data-testid="automate-parameter"]');

    // Verify automation lane appears
    await expect(page.locator('[data-testid="automation-lane"]')).toBeVisible();

    // Draw automation
    const automationLane = page.locator('[data-testid="automation-lane"]');
    await automationLane.click({ position: { x: 100, y: 50 } });
    await automationLane.click({ position: { x: 200, y: 100 } });

    // Verify automation points created
    const points = await page.locator('[data-testid="automation-point"]').count();
    expect(points).toBeGreaterThanOrEqual(2);
  });
});
