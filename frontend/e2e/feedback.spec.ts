import { expect, test, type Page } from '@playwright/test';
import { openGallery, openMain } from './support/app';
import { stubTauri } from './support/tauri-stub';

/** Makes every save of the UI state fail, so the shell raises its "could not be saved" warning Toast. */
async function openMainWithFailingSaves(page: Page) {
  await stubTauri(page);
  await page.addInitScript(() => {
    const internals = (window as unknown as { __TAURI_INTERNALS__: { invoke: (c: string, a: unknown) => unknown } })
      .__TAURI_INTERNALS__;
    const original = internals.invoke;
    internals.invoke = async (command: string, args: unknown) => {
      if (command === 'save_ui_state') throw new Error('disk full');
      return original(command, args);
    };
  });
  await page.goto('/');
  await expect(page.getByTestId('main-window')).toBeVisible();
}

type ToastTimes = { shownAt: number | null; goneAt: number | null };

/** Times the Toast inside the page: a slow page load would otherwise eat the margin of a wait that starts later. */
async function recordToastTimes(page: Page, text: string) {
  await page.addInitScript((wanted) => {
    const times: { shownAt: number | null; goneAt: number | null } = { shownAt: null, goneAt: null };
    (window as unknown as { toastTimes: typeof times }).toastTimes = times;
    new MutationObserver(() => {
      const shown = [...document.querySelectorAll('[data-component="Toast"]')].some((toast) =>
        toast.textContent?.includes(wanted),
      );
      if (shown) times.shownAt ??= performance.now();
      else if (times.shownAt !== null) times.goneAt ??= performance.now();
    }).observe(document, { childList: true, subtree: true });
  }, text);
}

const toastTimes = (page: Page) => page.evaluate(() => (window as unknown as { toastTimes: ToastTimes }).toastTimes);

test.describe('Toast timing (spec 11.4), measured in real time with a margin of 0.6 s', () => {
  test('given an info Toast, when it is shown, then it is still there at 3.4 s and gone by 4.8 s', async ({ page }) => {
    await recordToastTimes(page, 'Setup saved');
    await openGallery(page);
    await expect.poll(async () => (await toastTimes(page)).goneAt, { timeout: 10_000 }).not.toBeNull();
    const { shownAt, goneAt } = await toastTimes(page);
    const lifetime = goneAt! - shownAt!;
    expect(lifetime).toBeGreaterThan(3400);
    expect(lifetime).toBeLessThan(4800);
  });

  test('given a warning Toast from a failed text-size save, when it is shown, then it is still there at 7.4 s and gone by 8.8 s', async ({
    page,
  }) => {
    await openMainWithFailingSaves(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: '125 %' }).click();
    const toast = page.getByText(/could not be saved/);
    await expect(toast).toBeVisible();
    await page.waitForTimeout(7400);
    await expect(toast).toBeVisible();
    await expect(toast).toBeHidden({ timeout: 1400 });
  });

  test('given the warning Toast, when the pointer rests on it, then it stays past 8.8 s and goes after the pointer leaves', async ({
    page,
  }) => {
    await openMainWithFailingSaves(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: '125 %' }).click();
    const toast = page.getByText(/could not be saved/);
    await expect(toast).toBeVisible();
    await toast.hover();
    await page.waitForTimeout(9000);
    await expect(toast).toBeVisible();
    await page.mouse.move(0, 0);
    await expect(toast).toBeHidden({ timeout: 9000 });
  });
});

test.describe('Menu trigger and Tooltip through the real shell', () => {
  test('given the persistence button in the Toolbar, when the menu opens and closes, then aria-haspopup and aria-expanded follow and focus returns to the button', async ({
    page,
  }) => {
    await openMain(page);
    const trigger = page.locator('[data-region="toolbar"]').getByRole('button', { name: /^Persist/ });
    await expect(trigger).toHaveAttribute('aria-haspopup', 'menu');
    await expect(trigger).toHaveAttribute('aria-expanded', 'false');
    await trigger.click();
    await expect(page.getByRole('menu')).toBeVisible();
    await expect(trigger).toHaveAttribute('aria-expanded', 'true');
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menu')).toHaveCount(0);
    await expect(trigger).toHaveAttribute('aria-expanded', 'false');
    await expect(trigger).toBeFocused();
  });

  test('given the Toolbar menu, when Tab is pressed inside it, then it closes and focus returns to the button', async ({
    page,
  }) => {
    await openMain(page);
    const trigger = page.locator('[data-region="toolbar"]').getByRole('button', { name: /^Persist/ });
    await trigger.focus();
    await page.keyboard.press('Enter');
    await expect(page.getByRole('menu')).toBeVisible();
    await page.keyboard.press('Tab');
    await expect(page.getByRole('menu')).toHaveCount(0);
    await expect(trigger).toHaveAttribute('aria-expanded', 'false');
  });

  test('given the disabled Video option in the real Trigger group, when focused, then the control is described by the Tooltip that shows its reason', async ({
    page,
  }) => {
    await openMain(page);
    const video = page.locator('[data-region="trigger"]').getByRole('button', { name: /Video/ });
    await video.focus();
    const tip = page.getByRole('tooltip');
    await expect(tip).toBeVisible();
    const id = await tip.getAttribute('id');
    expect(id).toBeTruthy();
    expect(await video.getAttribute('aria-describedby')).toContain(id!);
    await page.keyboard.press('Escape');
    await expect(tip).toHaveCount(0);
    expect((await video.getAttribute('aria-describedby')) ?? '').not.toContain(id!);
  });
});
