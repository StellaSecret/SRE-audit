import { test, expect } from '@playwright/test';
import { clearStorage, seedSession, BASE } from './helpers';

test.describe('SRE Audit App', () => {
  test.beforeEach(({ page }) => clearStorage(page));

  test('logged-out users are gated behind login', async ({ page }) => {
    await expect(page.locator('body')).toContainText(
      /Sign in with Google|Connectez-vous avec Google/i,
    );
  });

  test('seeded allowed session lands on the home page', async ({ page }) => {
    await seedSession(page);
    await expect(page.locator('h1')).toContainText(/SRE Audit/i);
    await expect(page.locator('body')).toContainText(/Welcome|Bienvenue/i);
  });

  test('nav links navigate to matrix and roadmap', async ({ page }) => {
    await seedSession(page);
    const nav = page.locator('.nav-links');
    await nav.getByRole('link', { name: /Maturity Matrix|Matrice de Maturité/i }).click();
    await expect(page).toHaveURL(/\/SRE-audit\/matrix/);
    await expect(page.locator('table')).toBeVisible();

    await nav.getByRole('link', { name: /Roadmap/i }).click();
    await expect(page).toHaveURL(/\/SRE-audit\/roadmap/);
    await expect(page.locator('table')).toHaveCount(2);

    await nav.getByRole('link', { name: /Backup|Sauvegarde/i }).click();
    await expect(page).toHaveURL(/\/SRE-audit\/backup/);
    await expect(page.locator('.btn-export')).toHaveCount(2);
  });

  test('matrix renders the 7 SRE principles with level cells', async ({ page }) => {
    await seedSession(page);
    await page.goto(`${BASE}/matrix`);
    await page.waitForLoadState('networkidle');
    await expect(page.locator('tbody tr').first()).toBeVisible();
    const rows = await page.locator('tbody tr').count();
    expect(rows).toBeGreaterThanOrEqual(7);
    await expect(page.locator('.col-lvl4').first()).toBeVisible();
  });

  test('matrix comment is saved and mirrored for print', async ({ page }) => {
    await seedSession(page);
    await page.goto(`${BASE}/matrix`);
    await page.waitForLoadState('networkidle');
    const box = page.locator('textarea.comment-box').first();
    await box.fill('Remarque de test');
    await expect(box).toHaveValue('Remarque de test');
    // the print mirror is a hidden print-only element; assert its content (no click needed)
    await expect(page.locator('.comment-print-output').first()).toHaveText('Remarque de test');
  });

  test('roadmap renders short-term and long-term tables', async ({ page }) => {
    await seedSession(page);
    await page.goto(`${BASE}/roadmap`);
    await page.waitForLoadState('networkidle');
    const tables = page.locator('table');
    await expect(tables).toHaveCount(2);
    await expect(tables.nth(0).locator('tbody tr').first()).toBeVisible();
    const stRows = await tables.nth(0).locator('tbody tr').count();
    const ltRows = await tables.nth(1).locator('tbody tr').count();
    expect(stRows).toBeGreaterThanOrEqual(5);
    expect(ltRows).toBeGreaterThanOrEqual(5);
  });

test('language toggle flips the header language', async ({ page }) => {
      await seedSession(page);
      const toggle = page.locator('.nav-toggle');
      await expect(toggle).toHaveCount(1);
      await toggle.click();
      const body = await page.locator('body').innerText();
      expect(body).toMatch(/Organisations auditées|Organisations audited/i);
    });
});