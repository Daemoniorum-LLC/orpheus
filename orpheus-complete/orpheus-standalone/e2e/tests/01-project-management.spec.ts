import { test, expect } from '@playwright/test';

/**
 * E2E Tests: Project Management
 * Tests the complete project lifecycle from creation to deletion
 */

test.describe('Project Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should create a new project', async ({ page }) => {
    // Click new project button
    await page.click('[data-testid="new-project-button"]');

    // Fill in project details
    await page.fill('[data-testid="project-title-input"]', 'My E2E Test Song');
    await page.fill('[data-testid="project-artist-input"]', 'E2E Test Artist');
    await page.selectOption('[data-testid="project-bpm-input"]', '140');
    await page.selectOption('[data-testid="project-time-signature-input"]', '4/4');
    await page.selectOption('[data-testid="project-key-input"]', 'C');

    // Submit form
    await page.click('[data-testid="create-project-submit"]');

    // Verify project was created
    await expect(page.locator('[data-testid="project-title"]')).toHaveText('My E2E Test Song');
    await expect(page.locator('[data-testid="project-bpm"]')).toHaveText('140');

    // Verify API call was made
    const projectId = await page.getAttribute('[data-testid="project-id"]', 'data-id');
    expect(projectId).toBeTruthy();
  });

  test('should list all projects', async ({ page }) => {
    await page.goto('/projects');
    await page.waitForLoadState('networkidle');

    // Wait for projects to load
    await page.waitForSelector('[data-testid="project-list"]');

    // Verify projects are displayed
    const projectCards = await page.locator('[data-testid="project-card"]').count();
    expect(projectCards).toBeGreaterThan(0);

    // Verify project cards have required info
    const firstProject = page.locator('[data-testid="project-card"]').first();
    await expect(firstProject.locator('[data-testid="project-card-title"]')).toBeVisible();
    await expect(firstProject.locator('[data-testid="project-card-date"]')).toBeVisible();
  });

  test('should open existing project', async ({ page }) => {
    // Go to projects list
    await page.goto('/projects');
    await page.waitForSelector('[data-testid="project-list"]');

    // Click on first project
    await page.click('[data-testid="project-card"]');

    // Verify project loaded
    await expect(page.locator('[data-testid="project-workspace"]')).toBeVisible();
    await expect(page.locator('[data-testid="track-list"]')).toBeVisible();
    await expect(page.locator('[data-testid="timeline"]')).toBeVisible();
  });

  test('should update project metadata', async ({ page }) => {
    // Open first project
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');

    // Click settings
    await page.click('[data-testid="project-settings-button"]');

    // Update title
    const newTitle = 'Updated E2E Project ' + Date.now();
    await page.fill('[data-testid="project-title-input"]', newTitle);

    // Update BPM
    await page.fill('[data-testid="project-bpm-input"]', '150');

    // Save changes
    await page.click('[data-testid="save-project-settings"]');

    // Wait for success message
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();

    // Verify changes persisted
    await page.reload();
    await expect(page.locator('[data-testid="project-title"]')).toContainText(newTitle);
    await expect(page.locator('[data-testid="project-bpm"]')).toHaveText('150');
  });

  test('should delete project', async ({ page }) => {
    // Create a project to delete
    await page.click('[data-testid="new-project-button"]');
    await page.fill('[data-testid="project-title-input"]', 'Project to Delete');
    await page.click('[data-testid="create-project-submit"]');

    const projectId = await page.getAttribute('[data-testid="project-id"]', 'data-id');

    // Click delete button
    await page.click('[data-testid="project-settings-button"]');
    await page.click('[data-testid="delete-project-button"]');

    // Confirm deletion
    await page.click('[data-testid="confirm-delete-button"]');

    // Verify redirected to projects list
    await expect(page).toHaveURL(/\/projects/);

    // Verify project no longer exists
    await page.waitForLoadState('networkidle');
    const deletedProject = page.locator(`[data-id="${projectId}"]`);
    await expect(deletedProject).toHaveCount(0);
  });

  test('should save project automatically', async ({ page }) => {
    // Open project
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');

    // Make a change
    await page.click('[data-testid="add-track-button"]');

    // Wait for auto-save indicator
    await expect(page.locator('[data-testid="auto-save-indicator"]')).toHaveText(/Saving.../);

    // Wait for saved confirmation
    await expect(page.locator('[data-testid="auto-save-indicator"]')).toHaveText(/Saved/, {
      timeout: 5000
    });
  });

  test('should export project', async ({ page }) => {
    // Open project
    await page.goto('/projects');
    await page.click('[data-testid="project-card"]');

    // Click export
    const downloadPromise = page.waitForEvent('download');
    await page.click('[data-testid="export-project-button"]');

    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/\.maestro$/);
  });

  test('should import project', async ({ page }) => {
    await page.goto('/projects');

    // Click import button
    await page.click('[data-testid="import-project-button"]');

    // Upload file (mock)
    const fileInput = page.locator('[data-testid="project-file-input"]');
    await fileInput.setInputFiles({
      name: 'test-project.maestro',
      mimeType: 'application/json',
      buffer: Buffer.from(JSON.stringify({
        project: {
          metadata: {
            title: 'Imported Project',
            tempo: 120,
          }
        }
      }))
    });

    // Wait for import to complete
    await expect(page.locator('[data-testid="import-success"]')).toBeVisible();

    // Verify project appears in list
    await expect(page.locator('text=Imported Project')).toBeVisible();
  });
});
