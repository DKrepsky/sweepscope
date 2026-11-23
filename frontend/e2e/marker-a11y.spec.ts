import { expect, test, type Locator } from '@playwright/test';
import { openMain } from './support/app';

const value = async (marker: Locator) => Number(await marker.getAttribute('aria-valuenow'));

test.describe('Marker value for assistive technology (BUG-41)', () => {
  test('given a trigger level marker, when it is dragged, then aria-valuenow follows the pointer before release', async ({
    page,
  }) => {
    await openMain(page);
    const marker = page.getByRole('slider', { name: 'Trigger level' });
    const before = await value(marker);
    const box = (await marker.boundingBox())!;
    const x = box.x + box.width / 2;
    const y = box.y + box.height / 2;
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x, y - 60, { steps: 4 });
    expect(await value(marker)).toBeGreaterThan(before);
    await page.mouse.up();
  });

  test('given a focused trigger level marker, when Alt+ArrowUp is pressed, then aria-valuenow increases at once', async ({
    page,
  }) => {
    await openMain(page);
    const marker = page.getByRole('slider', { name: 'Trigger level' });
    const before = await value(marker);
    await marker.focus();
    await page.keyboard.press('Alt+ArrowUp');
    expect(await value(marker)).toBeGreaterThan(before);
  });

  test('given a moved marker, when it is read, then aria-valuetext is a percentage', async ({ page }) => {
    await openMain(page);
    const marker = page.getByRole('slider', { name: 'Trigger level' });
    await marker.focus();
    await page.keyboard.press('Alt+ArrowDown');
    await expect(marker).toHaveAttribute('aria-valuetext', /^\d+ %$/);
  });
});

test.describe('Segmented options stay on one line (BUG-42)', () => {
  for (const percent of [100, 125, 150, 200] as const) {
    test(`given the Settings popover at ${percent} % text scale, when opened, then every scale option is one line high`, async ({
      page,
    }) => {
      await openMain(page, percent);
      await page.getByRole('button', { name: 'Settings' }).click();
      const heights = new Set<number>();
      for (const label of ['100 %', '125 %', '150 %', '200 %']) {
        const option = page.getByRole('button', { name: label });
        heights.add(Math.round((await option.boundingBox())!.height));
        await expect(option).toHaveCSS('white-space', 'nowrap');
      }
      expect(heights.size).toBe(1);
    });
  }
});
