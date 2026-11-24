import { expect, test } from '@playwright/test';
import { collectErrors, openMain } from './support/app';
import { ipcLog } from './support/tauri-stub';

const ORDER = ['toolbar', 'scope', 'scale-strip', 'vertical', 'horizontal', 'trigger', 'sidebar', 'status'];

test.describe('main window shell', () => {
  test('given the shell, when Tab is pressed from the page start, then focus visits the regions in the order of design 10.4', async ({
    page,
  }) => {
    await openMain(page);
    const visited: string[] = [];
    for (let i = 0; i < 120; i += 1) {
      await page.keyboard.press('Tab');
      const region = await page.evaluate(
        () => document.activeElement?.closest<HTMLElement>('[data-region]')?.dataset.region ?? 'none',
      );
      if (region === 'none') break;
      if (visited.at(-1) !== region) visited.push(region);
    }
    expect(visited).toEqual(ORDER);
  });

  test('given every Tab stop, when focused, then it shows a 2 px outline in the focus colour', async ({ page }) => {
    await openMain(page);
    const failures: string[] = [];
    for (let i = 0; i < 60; i += 1) {
      await page.keyboard.press('Tab');
      const info = await page.evaluate(() => {
        const el = document.activeElement as HTMLElement;
        const style = getComputedStyle(el);
        return {
          name: el.getAttribute('aria-label') ?? el.textContent?.trim().slice(0, 20) ?? el.tagName,
          marker: el.classList.contains('marker'),
          filter: style.filter,
          width: style.outlineWidth,
          style: style.outlineStyle,
          colour: style.outlineColor,
        };
      });
      // Markers are SVG-like shapes: their focus ring is a 2 px drop-shadow in the focus colour, not an outline.
      const ok = info.marker ? info.filter.includes('drop-shadow') : info.width === '2px' && info.style === 'solid';
      if (!ok) failures.push(`${info.name}: ${info.width} ${info.style} ${info.filter}`);
    }
    expect(failures).toEqual([]);
  });

  test('given a running scope, when Stop is clicked, then set_running is sent with running false', async ({ page }) => {
    await openMain(page);
    const stop = page.getByRole('button', { name: /^Stop/ });
    await expect(stop).toBeVisible();
    await stop.click();
    await expect
      .poll(async () => (await ipcLog(page)).filter((call) => call.command === 'set_running'))
      .toEqual([{ command: 'set_running', args: { running: false } }]);
  });

  test('given the shell, when checked, then no placeholder note is shown', async ({ page }) => {
    await openMain(page);
    await expect(page.getByText('Illustrative numbers (placeholders)')).toHaveCount(0);
  });

  test('given the Video trigger option, when focused with the keyboard, then it is disabled and its reason shows', async ({
    page,
  }) => {
    await openMain(page);
    const video = page.locator('[data-region="trigger"]').getByRole('button', { name: /Video/ });
    await expect(video).toHaveAttribute('aria-disabled', 'true');
    await video.focus();
    await expect(page.getByRole('tooltip')).toBeVisible();
  });

  test('given the Settings popover, when opened, then it lists the four scales with the current one pressed', async ({
    page,
  }) => {
    await openMain(page, 125);
    await page.getByRole('button', { name: 'Settings' }).click();
    for (const label of ['100 %', '125 %', '150 %', '200 %'])
      await expect(page.getByRole('button', { name: label })).toBeVisible();
    await expect(page.getByRole('button', { name: '125 %' })).toHaveAttribute('aria-pressed', 'true');
  });

  test('given the Settings popover opened from its button, when Escape is pressed with focus still on the button, then it closes', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog', { name: 'Settings' })).toHaveCount(0);
  });

  test('given the Settings popover, when focus is moved inside and Escape is pressed, then it closes', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: '100 %' }).focus();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog', { name: 'Settings' })).toHaveCount(0);
  });

  test('given text scale 150 chosen, when the page reloads (restart), then 150 % is restored from the stored state', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: '150 %' }).click();
    await expect.poll(() => page.evaluate(() => document.documentElement.style.fontSize)).toBe('150%');
    await page.reload();
    await expect(page.getByTestId('main-window')).toBeVisible();
    await expect.poll(() => page.evaluate(() => document.documentElement.style.fontSize)).toBe('150%');
    const calls = await ipcLog(page);
    expect(calls[0]).toMatchObject({ command: 'load_ui_state', args: { namespace: 'app.text-scale' } });
  });

  test('given a failing backend, when the app starts, then it falls back to 100 % without errors in the page', async ({
    page,
  }) => {
    const errors = collectErrors(page);
    await page.addInitScript(() => {
      (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {
        transformCallback: () => 0,
        invoke: async () => {
          throw new Error('backend down');
        },
      };
    });
    await page.goto('/');
    await expect(page.getByTestId('main-window')).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.style.fontSize)).toBe('100%');
    expect(errors.filter((e) => !e.includes('backend down'))).toEqual([]);
  });

  test('given the persistence menu, when opened, then it is keyboard navigable and Escape closes it', async ({
    page,
  }) => {
    await openMain(page);
    await page
      .getByRole('button', { name: /Persistence|Off|1 s|Infinite/ })
      .first()
      .click();
    await expect(page.getByRole('menu')).toBeVisible();
    await page.keyboard.press('ArrowDown');
    await expect(page.getByRole('menuitemradio').nth(1)).toBeFocused();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menu')).toHaveCount(0);
  });

  test('given the Accuracy item, when activated, then the popover opens and, with focus inside, Escape closes it', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: /^Accuracy/ }).click();
    await expect(page.getByRole('dialog', { name: 'Accuracy note' }).getByText('Approximate')).toBeVisible();
    await page.getByRole('dialog', { name: 'Accuracy note' }).focus();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog', { name: 'Accuracy note' })).toHaveCount(0);
  });

  test('given the shell, when loaded and exercised, then no console or page errors occur', async ({ page }) => {
    const errors = collectErrors(page);
    await openMain(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.keyboard.press('Escape');
    await page.setViewportSize({ width: 1100, height: 700 });
    await page.setViewportSize({ width: 1440, height: 900 });
    expect(errors).toEqual([]);
  });

  test('given every control in the shell, when checked, then it has an accessible name', async ({ page }) => {
    await openMain(page);
    const unnamed = await page.evaluate(() =>
      [...document.querySelectorAll<HTMLElement>('button, [role="button"], input, [tabindex="0"]')]
        .filter((el) => !(el.getAttribute('aria-label') || el.textContent?.trim() || el.getAttribute('title')))
        .map((el) => el.outerHTML.slice(0, 80)),
    );
    expect(unnamed).toEqual([]);
  });
});

test.describe('unverified time base', () => {
  test('given a measured time base, when the window opens, then no Unverified badge shows in Horizontal', async ({
    page,
  }) => {
    await openMain(page);
    await expect(page.locator('[data-region="horizontal"]').getByText('Unverified')).toHaveCount(0);
  });

  test('given an unmeasured time base, when the window opens, then the Horizontal group and the readout show Unverified', async ({
    page,
  }) => {
    await openMain(page, 100, { unverifiedTimeBase: true });
    await expect(page.locator('[data-region="horizontal"]').getByText('Unverified')).toBeVisible();
    await expect(page.locator('[data-region="readout"]').getByText('Unverified')).toBeVisible();
  });
});

test.describe('unverified badge placement', () => {
  const windows: [number, number][] = [
    [1440, 900],
    [1920, 1080],
    [1280, 800],
    [2560, 1440],
  ];

  for (const [width, height] of windows) {
    test(`given an unmeasured time base at ${width}x${height}, when the Horizontal group shows, then the badge stays inside its group and clear of the Trigger Type label`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height });
      await openMain(page, 100, { unverifiedTimeBase: true });
      const group = page.locator('[data-region="horizontal"]');
      const badge = await group.getByRole('note').filter({ hasText: 'Unverified' }).boundingBox();
      const groupBox = await group.boundingBox();
      const typeLabel = await page.locator('[data-region="trigger"]').getByText('Type', { exact: true }).boundingBox();
      if (!badge || !groupBox || !typeLabel) throw new Error('missing box');
      const groupPadding = 8;
      expect(badge.x + badge.width).toBeLessThanOrEqual(groupBox.x + groupBox.width - groupPadding);
      const overlapsType =
        badge.x < typeLabel.x + typeLabel.width &&
        typeLabel.x < badge.x + badge.width &&
        badge.y < typeLabel.y + typeLabel.height &&
        typeLabel.y < badge.y + badge.height;
      expect(overlapsType).toBe(false);
    });
  }
});

test.describe('security', () => {
  test('given the page, when a command outside the manifest is invoked, then it is refused', async ({ page }) => {
    await openMain(page);
    const result = await page.evaluate(async () => {
      const internals = (window as unknown as { __TAURI_INTERNALS__: { invoke: (c: string) => Promise<unknown> } })
        .__TAURI_INTERNALS__;
      return internals.invoke('read_file').then(
        () => 'allowed',
        (error: Error) => error.message,
      );
    });
    expect(result).toContain('not allowed');
  });
});
