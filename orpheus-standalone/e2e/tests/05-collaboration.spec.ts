import { test, expect } from '@playwright/test';

/**
 * E2E Tests: Collaboration Features
 * Tests real-time collaboration, sharing, and multi-user workflows
 */

test.describe('Collaboration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');
    await page.waitForSelector('[data-testid="project-workspace"]');
  });

  test('should invite collaborator to project', async ({ page }) => {
    // Open share dialog
    await page.click('[data-testid="share-project-button"]');

    // Enter collaborator email
    await page.fill('[data-testid="collaborator-email-input"]', 'collaborator@example.com');

    // Select role
    await page.selectOption('[data-testid="collaborator-role-select"]', 'editor');

    // Send invite
    await page.click('[data-testid="send-invite-button"]');

    // Verify invite sent
    await expect(page.locator('[data-testid="invite-success"]')).toBeVisible();

    // Verify collaborator appears in list
    await expect(page.locator('[data-testid="collaborator-list"]'))
      .toContainText('collaborator@example.com');
  });

  test('should set collaborator permissions', async ({ page }) => {
    await page.click('[data-testid="share-project-button"]');

    // Add collaborator as viewer
    await page.fill('[data-testid="collaborator-email-input"]', 'viewer@example.com');
    await page.selectOption('[data-testid="collaborator-role-select"]', 'viewer');
    await page.click('[data-testid="send-invite-button"]');

    // Verify viewer permissions
    const viewer = page.locator('[data-testid="collaborator-item"]')
      .filter({ hasText: 'viewer@example.com' });
    await expect(viewer.locator('[data-testid="collaborator-role"]')).toHaveText('Viewer');

    // Change to editor
    await viewer.locator('[data-testid="change-role-button"]').click();
    await page.click('[data-testid="role-editor"]');

    // Verify role changed
    await expect(viewer.locator('[data-testid="collaborator-role"]')).toHaveText('Editor');
  });

  test('should remove collaborator', async ({ page }) => {
    await page.click('[data-testid="share-project-button"]');

    const initialCount = await page.locator('[data-testid="collaborator-item"]').count();

    // Remove first collaborator
    await page.locator('[data-testid="collaborator-item"]').first()
      .locator('[data-testid="remove-collaborator-button"]').click();

    // Confirm removal
    await page.click('[data-testid="confirm-remove"]');

    // Verify removed
    const newCount = await page.locator('[data-testid="collaborator-item"]').count();
    expect(newCount).toBe(initialCount - 1);
  });

  test('should show real-time cursor positions', async ({ page, context }) => {
    // Open second tab as collaborator
    const page2 = await context.newPage();
    await page2.goto('/');
    await page2.goto('/projects');
    await page2.click('[data-testid="project-card"]');

    // Move playhead on page2
    await page2.locator('[data-testid="timeline"]').click({ position: { x: 200, y: 50 } });

    // Verify cursor visible on page1
    await expect(page.locator('[data-testid="remote-cursor"]')).toBeVisible();

    // Verify cursor position matches
    const cursorPosition = await page.locator('[data-testid="remote-cursor"]')
      .getAttribute('data-position');
    const playheadPosition = await page2.locator('[data-testid="playhead"]')
      .getAttribute('data-position');

    expect(cursorPosition).toBe(playheadPosition);

    await page2.close();
  });

  test('should sync track changes across clients', async ({ page, context }) => {
    const page2 = await context.newPage();
    await page2.goto('/');
    await page2.goto('/projects');
    await page2.click('[data-testid="project-card"]');

    // Add track on page1
    await page.click('[data-testid="add-track-button"]');
    await page.fill('[data-testid="track-name-input"]', 'Synced Track');
    await page.click('[data-testid="create-track-submit"]');

    // Wait for sync
    await page2.waitForTimeout(1000);

    // Verify track appears on page2
    await expect(page2.locator('[data-testid="track-list"]')).toContainText('Synced Track');

    await page2.close();
  });

  test('should handle concurrent edits gracefully', async ({ page, context }) => {
    const page2 = await context.newPage();
    await page2.goto('/');
    await page2.goto('/projects');
    await page2.click('[data-testid="project-card"]');

    const track = page.locator('[data-testid="track-item"]').first();
    const track2 = page2.locator('[data-testid="track-item"]').first();

    // Both users edit volume simultaneously
    await Promise.all([
      track.locator('[data-testid="track-volume-slider"]').fill('0.8'),
      track2.locator('[data-testid="track-volume-slider"]').fill('0.6'),
    ]);

    // Wait for conflict resolution
    await page.waitForTimeout(1000);

    // Verify last-write-wins or merge strategy
    const finalVolume = await track.locator('[data-testid="track-volume-slider"]').inputValue();
    const finalVolume2 = await track2.locator('[data-testid="track-volume-slider"]').inputValue();

    // Both should have same value
    expect(finalVolume).toBe(finalVolume2);

    await page2.close();
  });

  test('should show active collaborators', async ({ page, context }) => {
    // Open second session
    const page2 = await context.newPage();
    await page2.goto('/');
    await page2.goto('/projects');
    await page2.click('[data-testid="project-card"]');

    // Verify presence indicator
    await expect(page.locator('[data-testid="active-collaborators"]')).toHaveText(/2 active/);

    // Verify avatar shown
    await expect(page.locator('[data-testid="collaborator-avatar"]')).toHaveCount(2);

    // Close second session
    await page2.close();

    // Verify count updated
    await page.waitForTimeout(500);
    await expect(page.locator('[data-testid="active-collaborators"]')).toHaveText(/1 active/);
  });

  test('should lock tracks to prevent concurrent edits', async ({ page, context }) => {
    const page2 = await context.newPage();
    await page2.goto('/');
    await page2.goto('/projects');
    await page2.click('[data-testid="project-card"]');

    const track = page.locator('[data-testid="track-item"]').first();

    // User 1 starts editing
    await track.click();

    // Verify track locked
    await expect(track).toHaveClass(/editing/);

    // User 2 tries to edit same track
    const track2 = page2.locator('[data-testid="track-item"]').first();
    await expect(track2).toHaveClass(/locked/);

    // User 2 sees locked indicator
    await expect(track2.locator('[data-testid="lock-indicator"]')).toBeVisible();

    await page2.close();
  });

  test('should sync playback across clients', async ({ page, context }) => {
    const page2 = await context.newPage();
    await page2.goto('/');
    await page2.goto('/projects');
    await page2.click('[data-testid="project-card"]');

    // Start playback on page1
    await page.click('[data-testid="transport-play-button"]');

    // Verify page2 starts playing
    await expect(page2.locator('[data-testid="transport-play-button"]')).toHaveClass(/playing/);

    // Verify synchronized position
    await page.waitForTimeout(1000);

    const position1 = await page.locator('[data-testid="playhead"]')
      .getAttribute('data-position');
    const position2 = await page2.locator('[data-testid="playhead"]')
      .getAttribute('data-position');

    expect(Math.abs(parseFloat(position1!) - parseFloat(position2!))).toBeLessThan(0.1);

    await page2.close();
  });

  test('should display chat messages', async ({ page }) => {
    // Open chat panel
    await page.click('[data-testid="chat-button"]');

    // Send message
    await page.fill('[data-testid="chat-input"]', 'Hey team, check out the new guitar track!');
    await page.keyboard.press('Enter');

    // Verify message appears
    await expect(page.locator('[data-testid="chat-messages"]'))
      .toContainText('Hey team, check out the new guitar track!');

    // Verify timestamp
    await expect(page.locator('[data-testid="chat-message"]').last()
      .locator('[data-testid="message-time"]')).toBeVisible();
  });

  test('should notify on collaborator actions', async ({ page, context }) => {
    const page2 = await context.newPage();
    await page2.goto('/');
    await page2.goto('/projects');
    await page2.click('[data-testid="project-card"]');

    // User 2 adds track
    await page2.click('[data-testid="add-track-button"]');
    await page2.fill('[data-testid="track-name-input"]', 'Bass');
    await page2.click('[data-testid="create-track-submit"]');

    // User 1 sees notification
    await expect(page.locator('[data-testid="notification"]'))
      .toContainText('added track "Bass"');

    await page2.close();
  });

  test('should handle offline/online transitions', async ({ page, context }) => {
    // Simulate going offline
    await context.setOffline(true);

    // Verify offline indicator
    await expect(page.locator('[data-testid="offline-indicator"]')).toBeVisible();

    // Make changes offline
    const track = page.locator('[data-testid="track-item"]').first();
    await track.locator('[data-testid="track-volume-slider"]').fill('0.9');

    // Go back online
    await context.setOffline(false);

    // Verify sync indicator
    await expect(page.locator('[data-testid="syncing-indicator"]')).toBeVisible();
    await expect(page.locator('[data-testid="online-indicator"]')).toBeVisible({ timeout: 5000 });

    // Verify changes synced
    await page.reload();
    await page.waitForSelector('[data-testid="project-workspace"]');

    const volume = await track.locator('[data-testid="track-volume-slider"]').inputValue();
    expect(parseFloat(volume)).toBeCloseTo(0.9, 1);
  });

  test('should export project with all collaborator changes', async ({ page }) => {
    // Export project
    const downloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="export-project-button"]');

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/\.maestro$/);

    // Verify export includes collaboration metadata
    const buffer = await download.createReadStream();
    const content = await new Promise<string>((resolve) => {
      let data = '';
      buffer.on('data', chunk => data += chunk);
      buffer.on('end', () => resolve(data));
    });

    const project = JSON.parse(content);
    expect(project.collaboration).toBeDefined();
    expect(project.collaboration.collaborators).toBeDefined();
  });
});
