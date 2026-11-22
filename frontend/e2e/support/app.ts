import { expect, type Page } from '@playwright/test';
import { stubTauri, type StubOptions } from './tauri-stub';

export type TextScale = 100 | 125 | 150 | 200;

export const tauriSettings = (percent: TextScale) => ({
  'app.text-scale': { namespace: 'app.text-scale', version: 1, json: JSON.stringify({ percent }) },
});

/** Opens the main window with the IPC stub; `percent` is the stored text scale (a restart with that setting). */
export async function openMain(page: Page, percent: TextScale = 100, options: StubOptions = {}): Promise<void> {
  await stubTauri(page, percent === 100 ? undefined : tauriSettings(percent), options);
  await page.goto('/');
  await expect(page.getByTestId('main-window')).toBeVisible();
  await expect
    .poll(() => page.evaluate(() => document.documentElement.style.fontSize))
    .toBe(percent === 100 ? '100%' : `${percent}%`);
}

export async function openGallery(page: Page): Promise<void> {
  await page.goto('/gallery.html');
  await expect(page.getByTestId('gallery')).toBeVisible();
}

export const shell = (page: Page) => page.getByTestId('main-window');

export async function tiers(page: Page): Promise<{ width: string | null; height: string | null }> {
  const element = shell(page);
  return {
    width: await element.getAttribute('data-width-tier'),
    height: await element.getAttribute('data-height-tier'),
  };
}

export async function box(page: Page, selector: string) {
  const found = await page.locator(selector).first().boundingBox();
  if (!found) throw new Error(`No box for ${selector}`);
  return found;
}

export const SCOPE_SCREEN = '[aria-label="Scope screen"]';

/** Collects console errors and uncaught page errors. */
export function collectErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(`pageerror: ${error.message}`));
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(`console: ${message.text()}`);
  });
  return errors;
}
