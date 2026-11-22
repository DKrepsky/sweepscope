import { expect, test } from '@playwright/test';
import { openGallery, openMain, type TextScale } from './support/app';

test.skip(({ browserName }) => browserName !== 'chromium', 'Visual snapshots are stored for Chromium only');

// Regression guard for later steps. The first comparison against the design prototype is a manual UI check (test plan V-1).
const windows: [string, number, number, TextScale][] = [
  ['w1-h1-1440x900', 1440, 900, 100],
  ['w2-h1-1280x900', 1280, 900, 100],
  ['w3-h1-1280x900-125', 1280, 900, 125],
  ['scroll-h1-960x700', 960, 700, 100],
  ['w1-h2-1440x760', 1440, 760, 100],
  ['w1-h3-1440x640', 1440, 640, 100],
  ['w1-h1-2560x1440-200', 2560, 1440, 200],
];

for (const [name, width, height, scale] of windows) {
  test(`given the main window at ${name}, when rendered, then it matches its snapshot`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await openMain(page, scale);
    await page.waitForTimeout(400);
    await page.mouse.move(0, 0);
    await expect(page).toHaveScreenshot(`main-${name}.png`);
  });
}

test('given the component gallery, when rendered, then each section matches its snapshot', async ({ page }) => {
  await page.setViewportSize({ width: 1400, height: 1000 });
  await openGallery(page);
  await page.waitForTimeout(300);
  const names = await page
    .locator('[data-component]')
    .evaluateAll((els) => els.map((e) => e.getAttribute('data-component')!));
  // Toast (self-dismisses after 4 s), Graph and Tooltip are excluded: Graph has a known redraw defect (BUG-6) and is covered by graph.spec.ts.
  for (const name of names.filter((n) => !['Graph', 'Tooltip', 'Toast'].includes(n))) {
    const section = page.locator(`[data-component="${name}"]`);
    await section.scrollIntoViewIfNeeded();
    // Small crops: a few glyphs of antialiasing noise between two CI runs (seen: 518 px in Dialog) exceed the global 0.2 % ratio.
    // 1 % still fails on any moved, resized or recoloured element.
    await expect(section).toHaveScreenshot(`gallery-${name}.png`, { maxDiffPixelRatio: 0.01 });
  }
});
