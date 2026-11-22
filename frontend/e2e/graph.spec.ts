import { expect, test, type Page } from '@playwright/test';
import { openGallery, openMain, SCOPE_SCREEN } from './support/app';

const SCALES = [1, 1.25, 1.5, 2];

/** Lit run lengths along one row and one column of the graticule canvas (alpha above 0). */
async function scanGraticule(page: Page) {
  return page.evaluate(() => {
    const canvas = document.querySelector<HTMLCanvasElement>('[aria-label="Scope screen"] canvas')!;
    const ctx = canvas.getContext('2d')!;
    const runs = (data: Uint8ClampedArray, length: number) => {
      const found: number[] = [];
      const colours = new Set<string>();
      let run = 0;
      const key = (i: number) => `${data[i * 4]},${data[i * 4 + 1]},${data[i * 4 + 2]}`;
      const background = '16,18,20'; // color-bg-scope
      for (let i = 0; i < length; i += 1) {
        const lit = key(i) !== background;
        if (lit) colours.add(key(i));
        if (lit) run += 1;
        else if (run) {
          found.push(run);
          run = 0;
        }
      }
      if (run) found.push(run);
      return { found, colours: [...colours] };
    };
    // A row and a column between major lines and away from the axis ticks.
    const y = Math.floor(canvas.height * 0.0625);
    const x = Math.floor(canvas.width * 0.05);
    const row = ctx.getImageData(0, y, canvas.width, 1).data;
    const column = ctx.getImageData(x, 0, 1, canvas.height).data;
    const cssWidth = canvas.getBoundingClientRect().width;
    return {
      dpr: window.devicePixelRatio,
      cssWidth,
      widthPx: canvas.width,
      heightPx: canvas.height,
      row: runs(row, canvas.width),
      column: runs(column, canvas.height),
    };
  });
}

/**
 * The graticule is only redrawn when the size changes (see BUG-6), so the check forces one fresh draw by resizing
 * the window, then reads the layer back.
 */
async function openWithFreshGraticule(page: Page) {
  await openMain(page);
  await page.setViewportSize({ width: 1500, height: 940 });
  await page.waitForTimeout(300);
}

for (const scale of SCALES) {
  test.describe(`device scale ${scale}`, () => {
    test.use({ deviceScaleFactor: scale });

    test(`given scale ${scale}, when the graticule is drawn, then the canvas has device resolution and every line is one device pixel wide in a token colour`, async ({
      page,
    }) => {
      // BUG-8: at 1.5 and 2 the hairline is 2 device pixels wide but is centred on a pixel centre, so it spreads over 3 pixels.
      test.fail(scale >= 1.5, 'BUG-8: graticule lines are not one crisp device pixel at scale 1.5 and 2');
      await openWithFreshGraticule(page);
      const scan = await scanGraticule(page);
      expect(scan.dpr).toBe(scale);
      expect(scan.widthPx).toBe(Math.round(scan.cssWidth * scale));
      // 9 major lines (one of them the axis) plus the two frame lines.
      expect(scan.row.found, `row runs ${scan.row.found.join(',')}`).toEqual(Array(11).fill(1));
      // 7 major lines plus the two frame lines.
      expect(scan.column.found, `column runs ${scan.column.found.join(',')}`).toEqual(Array(9).fill(1));
      // No anti-aliased blend: only the token colours of the grid appear.
      expect(scan.row.colours.length).toBeLessThanOrEqual(3);
      expect(scan.column.colours.length).toBeLessThanOrEqual(3);
    });

    test(`given scale ${scale}, when checked, then the scope screen matches its stored snapshot`, async ({
      page,
      browserName,
    }) => {
      test.skip(browserName !== 'chromium', 'Snapshots are stored for Chromium only');
      await openWithFreshGraticule(page);
      await expect(page.locator(SCOPE_SCREEN)).toHaveScreenshot(`scope-screen-${scale}.png`);
    });
  });
}

test.describe('graticule redraw', () => {
  test('given the scope screen with its graticule, when the Run button is toggled, then the graticule is still drawn', async ({
    page,
  }) => {
    await openMain(page);
    await page.waitForTimeout(300);
    const gridPixels = () =>
      page.evaluate(() => {
        const canvas = document.querySelector<HTMLCanvasElement>('[aria-label="Scope screen"] canvas')!;
        return canvas
          .getContext('2d')!
          .getImageData(0, 0, canvas.width, canvas.height)
          .data.some((v, i) => i % 4 === 3 && v > 0);
      });
    expect(await gridPixels()).toBe(true);
    await page.getByRole('button', { name: /^Stop/ }).click();
    await page.waitForTimeout(300);
    expect(await gridPixels()).toBe(true);
  });

  test('given the gallery graphs, when loaded, then every graph shows its graticule', async ({ page }) => {
    await openGallery(page);
    await page.waitForTimeout(300);
    const blank = await page.evaluate(
      () =>
        [...document.querySelectorAll<HTMLCanvasElement>('[data-component="Graph"] .layers canvas:first-child')].filter(
          (c) =>
            !c
              .getContext('2d')!
              .getImageData(0, 0, c.width, c.height)
              .data.some((v, i) => i % 4 === 3 && v > 0),
        ).length,
    );
    expect(blank).toBe(0);
  });
});

test.describe('graph semantics', () => {
  test('given the gallery graphs, when checked, then each has role img with an accessible summary and the empty one has only the graticule', async ({
    page,
  }) => {
    await openGallery(page);
    const images = page.locator('[data-component="Graph"] [role="img"]');
    await expect(images).toHaveCount(5);
    for (let i = 0; i < 5; i += 1) await expect(images.nth(i)).toHaveAttribute('aria-label', /Illustrative/);
    const frozen = page.locator('[data-component="Graph"] .graph[data-state]');
    expect(await frozen.evaluateAll((els) => els.map((e) => e.getAttribute('data-state')))).toContain('frozen');
  });

  test('given a draggable Marker, when dragged with the mouse, then it follows the pointer along its axis only and is clamped to the track', async ({
    page,
  }) => {
    await openGallery(page);
    const stage = page.locator('[data-component="Marker"] .marker-stage');
    await stage.scrollIntoViewIfNeeded();
    const marker = page
      .locator('[data-component="Marker"]')
      .getByRole('slider', { name: 'Trigger level', exact: true });
    const before = (await marker.boundingBox())!;
    const track = (await stage.boundingBox())!;
    const centre = { x: before.x + before.width / 2, y: before.y + before.height / 2 };
    await page.mouse.move(centre.x, centre.y);
    await page.mouse.down();
    await page.mouse.move(centre.x + 100, centre.y - 40, { steps: 4 });
    const midway = (await marker.boundingBox())!;
    expect(midway.x).toBeCloseTo(before.x, 0);
    // The marker centres on the pointer, so the offset from the press point is within the 8 px hit margin.
    expect(Math.abs(midway.y - (before.y - 40))).toBeLessThanOrEqual(10);
    await page.mouse.move(centre.x + 100, Math.max(1, track.y - 150), { steps: 4 });
    const clamped = (await marker.boundingBox())!;
    expect(Math.abs(clamped.y + clamped.height / 2 - track.y)).toBeLessThanOrEqual(10);
    await page.mouse.up();
  });

  test('given a disabled Marker, when the pointer is pressed and moved, then it does not move and its reason is its title', async ({
    page,
  }) => {
    await openGallery(page);
    const marker = page.locator('[data-component="Marker"]').getByRole('slider', { name: /disabled/ });
    await marker.scrollIntoViewIfNeeded();
    const before = (await marker.boundingBox())!;
    await page.mouse.move(before.x + before.width / 2, before.y + before.height / 2);
    await page.mouse.down();
    await page.mouse.move(before.x + before.width / 2, before.y + 60, { steps: 4 });
    expect((await marker.boundingBox())!.y).toBe(before.y);
    await page.mouse.up();
    await expect(marker).toHaveAttribute('title', 'Trigger not available: offline');
  });

  test('given a disabled Marker, when it receives keyboard focus, then the reason shows in a Tooltip', async ({
    page,
  }) => {
    await openGallery(page);
    const marker = page.locator('[data-component="Marker"]').getByRole('slider', { name: /disabled/ });
    await marker.focus();
    await expect(page.getByRole('tooltip').filter({ hasText: 'Trigger not available: offline' })).toBeVisible();
  });
});
