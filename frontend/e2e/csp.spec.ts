import { readFileSync } from 'node:fs';
import { expect, test, type Page } from '@playwright/test';
import { stubTauri } from './support/tauri-stub';

const PREVIEW = `http://localhost:${process.env.E2E_PREVIEW_PORT ?? 4173}`;
const config = JSON.parse(readFileSync(new URL('../../src-tauri/tauri.conf.json', import.meta.url), 'utf8')) as {
  app: { security: { csp: string } };
};
const CSP = config.app.security.csp;

/** Serves the production bundle with the Content-Security-Policy of tauri.conf.json, as the Tauri window does. */
async function openWithCsp(page: Page): Promise<string[]> {
  const violations: string[] = [];
  await page.route(`${PREVIEW}/**`, async (route) => {
    const response = await route.fetch();
    await route.fulfill({ response, headers: { ...response.headers(), 'content-security-policy': CSP } });
  });
  await page.exposeFunction('reportViolation', (text: string) => violations.push(text));
  await page.addInitScript(() =>
    document.addEventListener('securitypolicyviolation', (event) =>
      (window as unknown as { reportViolation: (t: string) => void }).reportViolation(
        `${event.violatedDirective} ${event.blockedURI}`,
      ),
    ),
  );
  await stubTauri(page);
  await page.goto(`${PREVIEW}/`);
  await expect(page.getByTestId('main-window')).toBeVisible();
  return violations;
}

test.describe('content security policy of the Tauri window', () => {
  test('given the policy of tauri.conf.json, when parsed, then it has no unsafe-eval and no remote origin', () => {
    expect(CSP).not.toContain('unsafe-eval');
    expect(CSP).not.toMatch(/https?:\/\/(?!ipc\.localhost)/);
    expect(CSP).toContain("default-src 'self'");
  });

  test('given the production bundle under that policy, when it loads and runs, then nothing is blocked', async ({
    page,
  }) => {
    const violations = await openWithCsp(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: '125 %' }).click();
    await page.waitForTimeout(300);
    expect(violations).toEqual([]);
  });

  test('given the policy, when the page tries a remote script, a remote image, an inline script and a remote fetch, then each is blocked', async ({
    page,
  }) => {
    const violations = await openWithCsp(page);
    const outcome = await page.evaluate(async () => {
      const blockedFetch = await fetch('https://example.com/data').then(
        () => 'allowed',
        () => 'blocked',
      );
      const script = document.createElement('script');
      script.src = 'https://example.com/remote.js';
      document.head.append(script);
      const image = new Image();
      image.src = 'https://example.com/pixel.png';
      const inline = document.createElement('script');
      inline.textContent = 'window.__inlineRan = true';
      document.head.append(inline);
      await new Promise((resolve) => setTimeout(resolve, 300));
      return { blockedFetch, inlineRan: (window as unknown as { __inlineRan?: boolean }).__inlineRan === true };
    });
    expect(outcome).toEqual({ blockedFetch: 'blocked', inlineRan: false });
    expect(violations.some((v) => v.startsWith('script-src'))).toBe(true);
    expect(violations.some((v) => v.startsWith('img-src'))).toBe(true);
    expect(violations.some((v) => v.startsWith('connect-src'))).toBe(true);
  });
});

test.describe('production bundle', () => {
  test('given the production bundle, when the gallery page is requested, then the gallery is not served', async ({
    page,
  }) => {
    await page.goto(`${PREVIEW}/gallery.html`);
    await expect(page.getByTestId('gallery')).toHaveCount(0);
    await expect(page.getByRole('heading', { name: /component gallery/i })).toHaveCount(0);
  });

  test('given the production bundle, when the main window loads, then it renders without console errors', async ({
    page,
  }) => {
    const errors: string[] = [];
    page.on('console', (m) => m.type() === 'error' && errors.push(m.text()));
    page.on('pageerror', (e) => errors.push(e.message));
    await stubTauri(page);
    await page.goto(`${PREVIEW}/`);
    await expect(page.getByTestId('main-window')).toBeVisible();
    expect(errors).toEqual([]);
  });
});
