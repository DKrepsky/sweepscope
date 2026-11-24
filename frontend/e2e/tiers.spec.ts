import { expect, test, type Page } from '@playwright/test';
import { box, openMain, SCOPE_SCREEN, shell, tiers, type TextScale } from './support/app';
import { emit } from './support/tauri-stub';

// Rows of the design 3.2 example table: window width in CSS px, text scale, expected width tier.
const table: [number, TextScale, string][] = [
  [3840, 100, 'w1'],
  [3840, 200, 'w1'],
  [2560, 150, 'w1'],
  [2560, 200, 'w2'],
  [2048, 125, 'w1'],
  [2048, 150, 'w2'],
  [2048, 200, 'w3'],
  [1920, 100, 'w1'],
  [1920, 150, 'w2'],
  [1920, 200, 'scroll'],
  [1536, 100, 'w1'],
  [1536, 125, 'w3'],
  [1536, 150, 'w3'],
  [1536, 200, 'scroll'],
  [1280, 100, 'w2'],
  [1280, 125, 'w3'],
  [1280, 150, 'scroll'],
  [960, 100, 'scroll'],
];

test.describe('width tiers (design 3.2)', () => {
  for (const [width, scale, expected] of table) {
    test(`given ${width} px at text scale ${scale} %, when loaded, then the width tier is ${expected}`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height: 1440 });
      await openMain(page, scale);
      expect((await tiers(page)).width).toBe(expected);
    });
  }

  test('given a fixed window, when the text scale is changed from 100 to 200 in the Settings control, then the tier is re-evaluated and persisted', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await openMain(page, 100);
    expect((await tiers(page)).width).toBe('w1');
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: '200 %' }).click();
    await expect(shell(page)).toHaveAttribute('data-width-tier', 'scroll');
    expect(await page.evaluate(() => document.documentElement.style.fontSize)).toBe('200%');
    const log = await page.evaluate(() => (window as unknown as { __ipcLog: { command: string }[] }).__ipcLog);
    expect(log.map((c) => c.command)).toContain('save_ui_state');
  });
});

test.describe('height tiers (design 3.2)', () => {
  // Calibrated thresholds (layout-calibration.md), wide column at 1440 px: h1 from 50 rem (800 px), h2 from 48.75 rem (780 px), h4 from the floor.
  const heights: [number, string][] = [
    [900, 'h1'],
    [800, 'h1'],
    [799, 'h2'],
    [780, 'h2'],
    [779, 'h4'],
    [720, 'h4'],
    [576, 'scroll'],
  ];
  for (const [height, expected] of heights) {
    test(`given a wide window ${height} px high at 100 %, when loaded, then the height tier is ${expected}`, async ({
      page,
    }) => {
      await page.setViewportSize({ width: 1440, height });
      await openMain(page, 100);
      expect((await tiers(page)).height).toBe(expected);
    });
  }
});

test.describe('layout per width tier', () => {
  test('given tier w1, when checked, then tool labels show, the sidebar is in the flow and there is no drawer button', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1600, height: 900 });
    await openMain(page);
    await expect(page.getByRole('button', { name: 'Cursors' })).toHaveText(/Cursors/);
    await expect(page.locator('[data-region="sidebar"]')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Measurements drawer' })).toHaveCount(0);
  });

  test('given tier w2, when checked, then the secondary tools lose their text but keep an accessible name and the sidebar stays', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await openMain(page);
    expect((await tiers(page)).width).toBe('w2');
    const cursors = page.getByRole('button', { name: 'Cursors' });
    await expect(cursors).toBeVisible();
    await expect(cursors).not.toHaveText(/Cursors/);
    await expect(page.locator('[data-region="sidebar"]')).toBeVisible();
  });

  test('given tier w3, when the drawer button is pressed, then the sidebar shows over the screen and Escape closes it', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await openMain(page, 125);
    expect((await tiers(page)).width).toBe('w3');
    await expect(page.locator('[data-region="sidebar"]')).toHaveCount(0);
    await page.getByRole('button', { name: 'Measurements drawer' }).click();
    await expect(page.locator('[data-region="sidebar"]')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.locator('[data-region="sidebar"]')).toHaveCount(0);
  });

  test('given tier w3, when checked, then the status bar keeps connection and sample rate visible and wraps the rest to a second row', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 1300 });
    await openMain(page, 125);
    expect((await tiers(page)).width).toBe('w3');
    for (const item of ['connection', 'sample-rate']) await expect(page.locator(`[data-item="${item}"]`)).toBeVisible();
    const first = await box(page, '[data-item="connection"]');
    const wrapped = await box(page, '[data-item="renderer"]');
    expect(wrapped.y).toBeGreaterThan(first.y + first.height - 1);
    const tag = await box(page, '[data-item="calibration"]');
    expect(tag.y).toBeLessThan(first.y + first.height);
    const viewport = page.viewportSize()!;
    for (const item of ['connection', 'sample-rate', 'record', 'capture', 'renderer', 'calibration']) {
      const b = await box(page, `[data-item="${item}"]`);
      expect(b.x + b.width).toBeLessThanOrEqual(viewport.width + 1);
    }
  });

  for (const [width, scale] of [
    [3840, 100],
    [1440, 100],
    [1280, 100],
    [1280, 125],
    [960, 100],
    [1536, 200],
  ] as [number, TextScale][]) {
    test(`given ${width} px at ${scale} %, when checked, then connection and sample rate are visible in the status bar`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height: 900 });
      await openMain(page, scale);
      await expect(page.locator('[data-item="connection"]')).toBeVisible();
      await expect(page.locator('[data-item="sample-rate"]')).toBeVisible();
    });
  }

  test('given tier h2, when checked, then the record overview is hidden; given h1, it is shown', async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await openMain(page);
    const overview = page.locator('[data-region="readout"] .overview');
    const before = await overview.count();
    await page.setViewportSize({ width: 1440, height: 790 });
    await expect(shell(page)).toHaveAttribute('data-height-tier', 'h2');
    expect(before).toBeGreaterThan(0);
    await expect(overview).toHaveCount(0);
  });

  test('given tier h3, when checked, then the Vertical group shows channel tabs and one channel row', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1024, height: 900 });
    await openMain(page);
    await expect(shell(page)).toHaveAttribute('data-height-tier', 'h3');
    await expect(page.getByRole('group', { name: 'Channel tabs' }).getByRole('button')).toHaveCount(5);
  });
});

test.describe('scope screen sizing (design 3.3)', () => {
  const sizes: [number, number, TextScale][] = [
    [3840, 2160, 100],
    [2560, 1440, 100],
    [1920, 1080, 100],
    [1440, 900, 100],
    [1280, 800, 100],
    [1280, 720, 100],
    [2560, 1080, 100],
    [1920, 1080, 150],
    [1600, 900, 125],
    [1280, 900, 125],
    [2560, 1440, 200],
  ];
  for (const [width, height, scale] of sizes) {
    test(`given ${width} x ${height} at ${scale} %, when checked, then the scope screen is 10:8 and at least 32 x 25.6 rem and fully inside the stage`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height });
      await openMain(page, scale);
      await page.waitForTimeout(150);
      const screen = await box(page, SCOPE_SCREEN);
      expect(Math.abs(screen.width / screen.height - 1.25)).toBeLessThan(0.01);
      const rem = (16 * scale) / 100;
      expect(screen.width).toBeGreaterThanOrEqual(32 * rem - 1);
      expect(screen.height).toBeGreaterThanOrEqual(25.6 * rem - 1);
      const stage = await box(page, '[data-region="scope"]');
      expect(screen.y + screen.height).toBeLessThanOrEqual(stage.y + stage.height + 1);
    });
  }
});

/** Run control, scale strip, connection and sample rate are inside the viewport without scrolling (AC-L1). */
async function expectEssentialsVisible(page: Page) {
  const viewport = page.viewportSize()!;
  for (const selector of [
    '[aria-label="Run control"]',
    '[data-region="scale-strip"]',
    '[data-item="connection"]',
    '[data-item="sample-rate"]',
  ]) {
    const b = await box(page, selector);
    expect(b.y, selector).toBeGreaterThanOrEqual(0);
    expect(b.y + b.height, selector).toBeLessThanOrEqual(viewport.height + 1);
  }
}

const contentScrolls = (page: Page) =>
  page.locator('[data-region="content"]').evaluate((e) => e.scrollHeight > e.clientHeight + 1);

test.describe('content floor (AC-L1 to AC-L3)', () => {
  const atOrAboveFloor: [number, number, TextScale][] = [
    [1024, 720, 100],
    [1024, 800, 100],
    [1280, 720, 100],
    [1366, 768, 100],
    [1440, 900, 100],
    [1280, 900, 125],
    [1536, 1080, 150],
    [2048, 1440, 200],
  ];
  for (const [width, height, scale] of atOrAboveFloor) {
    test(`given ${width} x ${height} at ${scale} %, when loaded, then the screen is 10:8, at least 32 x 25.6 rem, inside the stage and nothing needs scrolling (AC-L1)`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height });
      await openMain(page, scale);
      await page.waitForTimeout(150);
      const rem = (16 * scale) / 100;
      const screen = await box(page, SCOPE_SCREEN);
      const stage = await box(page, '[data-region="scope"]');
      expect(Math.abs(screen.width / screen.height - 1.25)).toBeLessThan(0.01);
      expect(screen.width).toBeGreaterThanOrEqual(32 * rem - 1);
      expect(screen.height).toBeGreaterThanOrEqual(25.6 * rem - 1);
      expect(screen.y).toBeGreaterThanOrEqual(stage.y - 1);
      expect(screen.y + screen.height).toBeLessThanOrEqual(stage.y + stage.height + 1);
      expect(await contentScrolls(page)).toBe(false);
      await expectEssentialsVisible(page);
    });
  }

  for (const [width, height, scale] of [
    [1024, 576, 100],
    [1280, 900, 200],
  ] as [number, number, TextScale][]) {
    test(`given ${width} x ${height} at ${scale} % (below the floor), when the content is scrolled, then toolbar and status bar stay pinned and the screen keeps its minimum (AC-L2)`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height });
      await openMain(page, scale);
      await page.waitForTimeout(150);
      const rem = (16 * scale) / 100;
      const toolbarBefore = await box(page, '[data-region="toolbar"]');
      const statusBefore = await box(page, '[data-region="status"]');
      expect(await contentScrolls(page)).toBe(true);

      await page.locator('[data-region="content"]').evaluate((e) => e.scrollTo(0, e.scrollHeight));

      expect(await box(page, '[data-region="toolbar"]')).toEqual(toolbarBefore);
      expect(await box(page, '[data-region="status"]')).toEqual(statusBefore);
      expect(statusBefore.y + statusBefore.height).toBeLessThanOrEqual(height + 1);
      const screen = await box(page, SCOPE_SCREEN);
      expect(screen.width).toBeGreaterThanOrEqual(32 * rem - 1);
      expect(screen.height).toBeGreaterThanOrEqual(25.6 * rem - 1);
    });
  }

  test('given a window narrower than the floor, when checked, then the whole content scrolls horizontally and keeps 64 rem', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 900, height: 800 });
    await openMain(page);
    const shellBox = await box(page, '[data-testid="main-window"]');
    expect(shellBox.width).toBeGreaterThanOrEqual(1024 - 1);
    const scrolls = await page.locator('.viewport').evaluate((e) => e.scrollWidth > e.clientWidth);
    expect(scrolls).toBe(true);
  });

  test('given the window height is reduced, when it crosses a threshold, then the collapse is re-evaluated without reload and the screen stays whole (AC-L3)', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1024, height: 1050 });
    await openMain(page);
    await expect(shell(page)).toHaveAttribute('data-height-tier', 'h1');

    await page.setViewportSize({ width: 1024, height: 720 });

    await expect(shell(page)).toHaveAttribute('data-height-tier', 'h4');
    await page.waitForTimeout(150);
    const screen = await box(page, SCOPE_SCREEN);
    expect(screen.height).toBeGreaterThanOrEqual(409);
    expect(await contentScrolls(page)).toBe(false);
  });

  test('given the text scale is raised in Settings, when the window falls below the floor, then it scrolls and keeps the minimum screen (AC-L3)', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await openMain(page);
    expect(await contentScrolls(page)).toBe(false);

    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: '200 %' }).click();

    await expect.poll(() => contentScrolls(page)).toBe(true);
    await expect(shell(page)).toHaveAttribute('data-height-tier', 'scroll');
    const screen = await box(page, SCOPE_SCREEN);
    expect(screen.height).toBeGreaterThanOrEqual(25.6 * 32 - 1);
  });

  test('given tier h4, when the Trigger tab is chosen, then the Trigger group shows and the scale strip carries its summary', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1024, height: 720 });
    await openMain(page);
    await expect(page.locator('[data-region="trigger"]')).toHaveCount(0);
    await expect(page.locator('[data-trigger-summary]')).toBeVisible();

    await page.getByRole('button', { name: 'Trigger' }).click();

    await expect(page.locator('[data-region="trigger"]')).toBeVisible();
  });

  test('given 1440 px at text scale 150 % (60 rem, under the width floor), when loaded, then the text-size control is visible and works without scrolling (C-2)', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1440, height: 1000 });
    await openMain(page, 150);
    const viewport = page.viewportSize()!;
    const settings = await box(page, 'button[aria-label="Settings"]');
    expect(settings.x + settings.width).toBeLessThanOrEqual(viewport.width + 1);

    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: '100 %' }).click();

    expect(await page.evaluate(() => document.documentElement.style.fontSize)).toBe('100%');
  });

  test('given the content scrolled horizontally to the far left, when checked, then the text-size control is still inside the viewport (C-2)', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 900, height: 800 });
    await openMain(page);
    await page.locator('.viewport').evaluate((e) => e.scrollTo(0, 0));

    const settings = await box(page, 'button[aria-label="Settings"]');

    expect(settings.x + settings.width).toBeLessThanOrEqual(901);
  });
});

test.describe('AC-L1 at every calibrated height threshold and one pixel below it', () => {
  // [width, scale, height in px, expected height tier]; rem thresholds from layout-calibration.md, px = rem x 16 x scale.
  const rows: [number, TextScale, number, string][] = [
    // wide column (1440 px, W1): h1 50 rem, h2 48.75 rem, h4 from the floor 45 rem.
    [1440, 100, 800, 'h1'],
    [1440, 100, 799, 'h2'],
    [1440, 100, 780, 'h2'],
    [1440, 100, 779, 'h4'],
    [1440, 100, 720, 'h4'],
    // narrow column (1024 px, W3): h1 62.5 rem, h2 60 rem, h3 52.75 rem, h4 from the floor.
    [1024, 100, 1000, 'h1'],
    [1024, 100, 999, 'h2'],
    [1024, 100, 960, 'h2'],
    [1024, 100, 959, 'h3'],
    [1024, 100, 844, 'h3'],
    [1024, 100, 843, 'h4'],
    [1024, 100, 720, 'h4'],
    // the same thresholds in rem at text scale 125 % (1280 px = 64 rem, W3): px = rem x 20.
    [1280, 125, 1250, 'h1'],
    [1280, 125, 1249, 'h2'],
    [1280, 125, 1200, 'h2'],
    [1280, 125, 1199, 'h3'],
    [1280, 125, 1055, 'h3'],
    [1280, 125, 1054, 'h4'],
    [1280, 125, 900, 'h4'],
  ];
  for (const [width, scale, height, expected] of rows) {
    test(`given ${width} x ${height} at ${scale} %, when loaded, then the tier is ${expected} and AC-L1 holds (whole screen, no scrolling, essentials visible)`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height });
      await openMain(page, scale);
      await page.waitForTimeout(150);
      expect((await tiers(page)).height).toBe(expected);
      const rem = (16 * scale) / 100;
      const screen = await box(page, SCOPE_SCREEN);
      const stage = await box(page, '[data-region="scope"]');
      expect(Math.abs(screen.width / screen.height - 1.25)).toBeLessThan(0.01);
      expect(screen.width).toBeGreaterThanOrEqual(32 * rem - 1);
      expect(screen.height).toBeGreaterThanOrEqual(25.6 * rem - 1);
      expect(screen.y).toBeGreaterThanOrEqual(stage.y - 1);
      expect(screen.y + screen.height).toBeLessThanOrEqual(stage.y + stage.height + 1);
      expect(await contentScrolls(page)).toBe(false);
      await expectEssentialsVisible(page);
    });
  }

  test('given 1024 x 719 at 100 % (one pixel under the floor), when loaded, then the tier is scroll and the screen keeps its minimum', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1024, height: 719 });
    await openMain(page, 100);
    expect((await tiers(page)).height).toBe('scroll');
    const screen = await box(page, SCOPE_SCREEN);
    expect(screen.width).toBeGreaterThanOrEqual(512 - 1);
    expect(screen.height).toBeGreaterThanOrEqual(409.6 - 1);
  });
});

type Snapshot = { instrument: { revision: number; trigger: object } };

/** The stub snapshot with the Pulse trigger layout, the tallest Trigger group the thresholds are calibrated for. */
async function showPulseTrigger(page: Page) {
  const snapshot = await page.evaluate(() =>
    (
      window as unknown as { __TAURI_INTERNALS__: { invoke: (c: string, a: object) => Promise<Snapshot> } }
    ).__TAURI_INTERNALS__.invoke('get_snapshot', {}),
  );
  const pulse = { condition: 'greater', width: { value: 500, text: '500', unit: 'µs' }, verified: true };
  const trigger = { ...snapshot.instrument.trigger, kind: 'pulse', pulse };
  await emit(page, 'instrument_changed', { ...snapshot.instrument, revision: 99, trigger });
  await page.getByRole('textbox', { name: 'Pulse width' }).waitFor();
}

test.describe('AC-L1 with the Pulse trigger layout', () => {
  const rows: [number, number, string][] = [
    [1440, 800, 'h1'],
    [1440, 780, 'h2'],
    [1024, 1000, 'h1'],
    [1024, 960, 'h2'],
    [1024, 844, 'h3'],
  ];
  for (const [width, height, expected] of rows) {
    test(`given the Pulse layout at ${width} x ${height}, when loaded, then the tier is ${expected} and nothing scrolls`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height });
      await openMain(page, 100);
      await showPulseTrigger(page);
      await page.waitForTimeout(150);
      expect((await tiers(page)).height).toBe(expected);
      expect(await contentScrolls(page)).toBe(false);
    });
  }

  test('given a rejected width, when the message shows, then the Trigger group does not change height', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    await openMain(page, 100, { commands: { set_trigger: { error: 'Invalid width' } } });
    await showPulseTrigger(page);
    const before = (await box(page, '[data-region="trigger"]')).height;

    const field = page.getByRole('textbox', { name: 'Pulse width' });
    await field.fill('abc');
    await field.press('Enter');
    await expect(page.getByRole('status').filter({ hasText: 'Invalid width.' })).toBeVisible();

    expect((await box(page, '[data-region="trigger"]')).height).toBe(before);
  });
});

test.describe('row M stays one line', () => {
  const rows: [number, number, number, TextScale][] = [
    [1440, 900, 1440, 100],
    [1440, 780, 1440, 100],
    [1024, 1000, 1024, 100],
    [3840, 2160, 3840, 200],
  ];
  for (const [width, height, , scale] of rows) {
    test(`given ${width} x ${height} at ${scale} %, when loaded, then row M is as high as a channel row`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height });
      await openMain(page, scale);
      const math = await box(page, '[data-math]');
      const channel = await box(page, '[data-channel="1"]');
      expect(math.height).toBeLessThanOrEqual(channel.height + 1);
    });
  }
});
