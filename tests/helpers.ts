import { type Page } from '@playwright/test';

export const E2E_EMAIL = 'e2e@test.local';
export const BASE = '/SRE-audit';

export async function clearStorage(page: Page) {
  await page.goto(`${BASE}/`);
  await page.waitForLoadState('networkidle');
  await page.evaluate(() => {
    localStorage.clear();
  });
  await page.reload();
  await page.waitForLoadState('networkidle');
}

export async function seedSession(page: Page) {
  await page.goto(`${BASE}/`);
  await page.waitForLoadState('networkidle');
  await page.evaluate((email) => {
    // gloo LocalStorage::get deserializes via serde_json. Matching the app's
    // persist_state format:
    //   - token: stored as a JSON string ("e2e-token")
    //   - profile: stored as a JSON *string* whose content is the profile JSON
    localStorage.setItem('sre_audit_google_token', JSON.stringify('e2e-token'));
    const profile = {
      email,
      email_verified: true,
      sub: 'e2e-sub',
      name: 'E2E User',
    };
    localStorage.setItem('sre_audit_profile', JSON.stringify(JSON.stringify(profile)));
  }, E2E_EMAIL);
  await page.reload();
  await page.waitForLoadState('networkidle');
  await page.waitForFunction(
    () => document.body?.hasAttribute('data-auth'),
    { timeout: 15_000 },
  );
}