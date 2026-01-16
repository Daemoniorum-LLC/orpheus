import { test, expect } from '@playwright/test';

/**
 * E2E Tests: Track Management
 * Tests track creation, editing, and audio manipulation
 */

test.describe('Track Management', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate and open a project
    await page.goto('/');
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');
    await page.waitForSelector('[data-testid="project-workspace"]');
  });

  test('should create a new track', async ({ page }) => {
    // Click add track button
    await page.click('[data-testid="add-track-button"]');

    // Fill track details
    await page.fill('[data-testid="track-name-input"]', 'Lead Guitar');
    await page.selectOption('[data-testid="track-instrument-select"]', 'electric-guitar');

    // Submit
    await page.click('[data-testid="create-track-submit"]');

    // Verify track appears in list
    await expect(page.locator('[data-testid="track-list"]')).toContainText('Lead Guitar');

    // Verify track has controls
    const track = page.locator('[data-testid="track-item"]').filter({ hasText: 'Lead Guitar' });
    await expect(track.locator('[data-testid="track-volume-slider"]')).toBeVisible();
    await expect(track.locator('[data-testid="track-pan-slider"]')).toBeVisible();
    await expect(track.locator('[data-testid="track-mute-button"]')).toBeVisible();
    await expect(track.locator('[data-testid="track-solo-button"]')).toBeVisible();
  });

  test('should adjust track volume', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();

    // Get initial volume
    const volumeSlider = track.locator('[data-testid="track-volume-slider"]');
    const initialVolume = await volumeSlider.inputValue();

    // Adjust volume to 80%
    await volumeSlider.fill('0.8');

    // Verify value changed
    const newVolume = await volumeSlider.inputValue();
    expect(parseFloat(newVolume)).toBeCloseTo(0.8, 1);

    // Verify API call
    await page.waitForResponse(response =>
      response.url().includes('/api/v1/tracks') &&
      response.request().method() === 'PUT'
    );
  });

  test('should mute and unmute track', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    const muteButton = track.locator('[data-testid="track-mute-button"]');

    // Mute track
    await muteButton.click();
    await expect(muteButton).toHaveAttribute('data-muted', 'true');

    // Verify visual feedback
    await expect(track).toHaveClass(/muted/);

    // Unmute track
    await muteButton.click();
    await expect(muteButton).toHaveAttribute('data-muted', 'false');
    await expect(track).not.toHaveClass(/muted/);
  });

  test('should solo track', async ({ page }) => {
    const tracks = page.locator('[data-testid="track-item"]');
    const firstTrack = tracks.first();
    const soloButton = firstTrack.locator('[data-testid="track-solo-button"]');

    // Solo first track
    await soloButton.click();
    await expect(soloButton).toHaveAttribute('data-soloed', 'true');

    // Verify other tracks are dimmed
    const otherTracks = tracks.nth(1);
    await expect(otherTracks).toHaveClass(/dimmed/);

    // Unsolo
    await soloButton.click();
    await expect(soloButton).toHaveAttribute('data-soloed', 'false');
  });

  test('should pan track left and right', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    const panSlider = track.locator('[data-testid="track-pan-slider"]');

    // Pan left
    await panSlider.fill('0.0');
    expect(parseFloat(await panSlider.inputValue())).toBeCloseTo(0.0, 1);

    // Pan right
    await panSlider.fill('1.0');
    expect(parseFloat(await panSlider.inputValue())).toBeCloseTo(1.0, 1);

    // Center
    await panSlider.fill('0.5');
    expect(parseFloat(await panSlider.inputValue())).toBeCloseTo(0.5, 1);
  });

  test('should record audio to track', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();

    // Arm track for recording
    await track.locator('[data-testid="track-record-arm"]').click();
    await expect(track.locator('[data-testid="track-record-arm"]')).toHaveClass(/armed/);

    // Start recording
    await page.click('[data-testid="transport-record-button"]');
    await expect(page.locator('[data-testid="recording-indicator"]')).toBeVisible();

    // Record for 2 seconds
    await page.waitForTimeout(2000);

    // Stop recording
    await page.click('[data-testid="transport-stop-button"]');

    // Verify audio region appears
    await expect(track.locator('[data-testid="audio-region"]')).toBeVisible();
  });

  test('should upload audio file to track', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();

    // Click upload button
    await track.locator('[data-testid="track-upload-button"]').click();

    // Upload file
    const fileInput = page.locator('[data-testid="audio-file-input"]');
    await fileInput.setInputFiles({
      name: 'test-audio.wav',
      mimeType: 'audio/wav',
      buffer: Buffer.alloc(44100 * 2 * 2) // 1 second of stereo audio
    });

    // Wait for upload
    await expect(page.locator('[data-testid="upload-progress"]')).toBeVisible();
    await expect(page.locator('[data-testid="upload-complete"]')).toBeVisible({ timeout: 10000 });

    // Verify audio region
    await expect(track.locator('[data-testid="audio-region"]')).toBeVisible();
  });

  test('should delete track', async ({ page }) => {
    // Get initial track count
    const initialCount = await page.locator('[data-testid="track-item"]').count();

    // Click delete on first track
    const track = page.locator('[data-testid="track-item"]').first();
    await track.locator('[data-testid="track-menu-button"]').click();
    await page.click('[data-testid="delete-track-option"]');

    // Confirm deletion
    await page.click('[data-testid="confirm-delete-track"]');

    // Verify track removed
    const newCount = await page.locator('[data-testid="track-item"]').count();
    expect(newCount).toBe(initialCount - 1);
  });

  test('should duplicate track', async ({ page }) => {
    const initialCount = await page.locator('[data-testid="track-item"]').count();

    // Get first track name
    const firstTrack = page.locator('[data-testid="track-item"]').first();
    const trackName = await firstTrack.locator('[data-testid="track-name"]').textContent();

    // Duplicate track
    await firstTrack.locator('[data-testid="track-menu-button"]').click();
    await page.click('[data-testid="duplicate-track-option"]');

    // Verify new track created
    const newCount = await page.locator('[data-testid="track-item"]').count();
    expect(newCount).toBe(initialCount + 1);

    // Verify duplicated track has similar name
    await expect(page.locator('[data-testid="track-name"]').last()).toContainText(trackName!);
  });

  test('should reorder tracks via drag and drop', async ({ page }) => {
    // Get first two tracks
    const firstTrack = page.locator('[data-testid="track-item"]').nth(0);
    const secondTrack = page.locator('[data-testid="track-item"]').nth(1);

    const firstName = await firstTrack.locator('[data-testid="track-name"]').textContent();
    const secondName = await secondTrack.locator('[data-testid="track-name"]').textContent();

    // Drag first track below second
    await firstTrack.dragTo(secondTrack);

    // Verify order changed
    const newFirstName = await page.locator('[data-testid="track-item"]').nth(0)
      .locator('[data-testid="track-name"]').textContent();

    expect(newFirstName).toBe(secondName);
  });

  test('should trim audio region', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    const region = track.locator('[data-testid="audio-region"]').first();

    // Click trim tool
    await page.click('[data-testid="trim-tool-button"]');

    // Drag region edge
    const regionEdge = region.locator('[data-testid="region-end-handle"]');
    await regionEdge.hover();
    await page.mouse.down();
    await page.mouse.move(100, 0, { steps: 10 });
    await page.mouse.up();

    // Verify region resized
    const newWidth = await region.evaluate(el => el.clientWidth);
    expect(newWidth).toBeLessThan(await region.evaluate(el => el.clientWidth));
  });

  test('should apply fade in/out to region', async ({ page }) => {
    const track = page.locator('[data-testid="track-item"]').first();
    const region = track.locator('[data-testid="audio-region"]').first();

    // Click on region to select
    await region.click();

    // Apply fade in
    await page.click('[data-testid="fade-in-button"]');
    await expect(region.locator('[data-testid="fade-in-indicator"]')).toBeVisible();

    // Apply fade out
    await page.click('[data-testid="fade-out-button"]');
    await expect(region.locator('[data-testid="fade-out-indicator"]')).toBeVisible();
  });
});
