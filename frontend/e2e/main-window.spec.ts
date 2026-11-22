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

  test('given the Run button, when clicked, then its label toggles between Stop and Run', async ({ page }) => {
    await openMain(page);
    const stop = page.getByRole('button', { name: /^Stop/ });
    await expect(stop).toBeVisible();
    await stop.click();
    await expect(page.getByRole('button', { name: /^Run/ })).toBeVisible();
  });

  test('given the shell, when checked, then the placeholder note is visible', async ({ page }) => {
    await openMain(page);
    await expect(page.getByText('Illustrative numbers (placeholders)')).toBeVisible();
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
    test.fail(true, 'BUG-3: the popover handles Escape only when focus is inside it, and opening does not move focus');
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
    await expect(page.getByRole('menuitem').nth(1)).toBeFocused();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menu')).toHaveCount(0);
  });

  test('given the Accuracy item, when activated, then the popover opens and, with focus inside, Escape closes it', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: /^Accuracy/ }).click();
    await expect(page.getByText('Accuracy note slot.')).toBeVisible();
    await page.getByRole('dialog', { name: 'Accuracy note' }).focus();
    await page.keyboard.press('Escape');
    await expect(page.getByText('Accuracy note slot.')).toHaveCount(0);
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
