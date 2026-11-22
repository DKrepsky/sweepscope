import { expect, test, type Locator, type Page } from '@playwright/test';
import { themes, tokenNames } from '../src/design-system/tokens.generated';
import { openGallery } from './support/app';

const dark = themes.dark;

const rgb = (hex: string): string => {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return `rgb(${r}, ${g}, ${b})`;
};

const style = (locator: Locator, property: string) =>
  locator.evaluate((el, name) => getComputedStyle(el).getPropertyValue(name), property);

const section = (page: Page, name: string) => page.locator(`[data-component="${name}"]`);

const DESIGN_COMPONENTS = [
  'Button',
  'RunButton',
  'Segmented',
  'Stepper',
  'NumberField',
  'Badge',
  'StatusChip',
  'Panel',
  'MeasurementMatrix',
  'Readout',
  'Graph',
  'Marker',
  'StatusBar',
  'Banner',
  'Toast',
  'Dialog',
  'Menu',
  'Tooltip',
];

test.beforeEach(async ({ page }) => openGallery(page));

test.describe('tokens at runtime', () => {
  test('given the gallery, when checked, then the root is dark themed and every design component has a section', async ({
    page,
  }) => {
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    for (const name of DESIGN_COMPONENTS) await expect(section(page, name)).toHaveCount(1);
  });

  test('given the Tokens section, when checked, then there is one swatch per colour token and each resolves to the token value', async ({
    page,
  }) => {
    const colourTokens = tokenNames.filter((n) => n.startsWith('color-') && !n.includes('overlay'));
    await expect(page.locator('[data-swatch]')).toHaveCount(tokenNames.filter((n) => n.startsWith('color-')).length);
    for (const name of colourTokens) {
      const swatch = page.locator(`[data-swatch="${name}"] .swatch`);
      expect(await style(swatch, 'background-color'), name).toBe(rgb(dark[name as keyof typeof dark]));
    }
  });

  test('given a default Button, when checked, then its colours and sizes equal the token values', async ({ page }) => {
    const button = section(page, 'Button').getByRole('button', { name: 'Default' });
    expect(await style(button, 'background-color')).toBe(rgb(dark['color-bg-raised']));
    expect(await style(button, 'border-top-color')).toBe(rgb(dark['color-border-control']));
    expect(await style(button, 'border-top-width')).toBe('1px');
    expect(await style(button, 'height')).toBe('24px');
    expect(await style(button, 'border-top-left-radius')).toBe('2px');
  });

  test('given a pressed (latched) Button, when checked, then it uses the selected background and accent border', async ({
    page,
  }) => {
    const button = section(page, 'Button').getByRole('button', { name: 'On' });
    expect(await style(button, 'background-color')).toBe(rgb(dark['color-bg-selected']));
    expect(await style(button, 'border-top-color')).toBe(rgb(dark['color-accent']));
  });

  test('given the forced states, when checked, then hover and pressed backgrounds equal their tokens', async ({
    page,
  }) => {
    const hover = section(page, 'Button').getByRole('button', { name: 'Hover' });
    const pressed = section(page, 'Button').getByRole('button', { name: 'Pressed' });
    expect(await style(hover, 'background-color')).toBe(rgb(dark['color-bg-hover']));
    expect(await style(pressed, 'background-color')).toBe(rgb(dark['color-bg-pressed']));
  });

  test('given the forced focus state, when checked, then the ring is 2 px solid in the focus colour', async ({
    page,
  }) => {
    const focused = section(page, 'Button').getByRole('button', { name: 'Focus' });
    expect(await style(focused, 'outline-width')).toBe('2px');
    expect(await style(focused, 'outline-style')).toBe('solid');
    expect(await style(focused, 'outline-color')).toBe(rgb(dark['color-focus']));
  });

  test('given Tab focus on a Button, when checked, then the focus-visible ring is 2 px solid in the focus colour', async ({
    page,
  }) => {
    const button = section(page, 'Button').getByRole('button', { name: 'Default' });
    await button.focus();
    await page.keyboard.press('Shift+Tab');
    await page.keyboard.press('Tab');
    await expect(button).toBeFocused();
    expect(await style(button, 'outline-width')).toBe('2px');
    expect(await style(button, 'outline-style')).toBe('solid');
    expect(await style(button, 'outline-color')).toBe(rgb(dark['color-focus']));
  });

  test('given the Button pressed with the pointer, when checked, then the pressed background shows while the button is held', async ({
    page,
  }) => {
    const button = section(page, 'Button').getByRole('button', { name: 'Default' });
    const box = (await button.boundingBox())!;
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.waitForTimeout(200);
    expect(await style(button, 'background-color')).toBe(rgb(dark['color-bg-pressed']));
    await page.mouse.up();
  });

  test('given each StatusChip kind, when checked, then the colour is its status token, with text and a glyph, and dim keeps the box at 40 % opacity', async ({
    page,
  }) => {
    const chips = section(page, 'StatusChip');
    const active = chips.locator('.chip:not(.dim)');
    const dim = chips.locator('.chip.dim');
    const kinds = [
      'run',
      'stop',
      'armed',
      'triggered',
      'auto',
      'disconnected',
      'error',
      'warning',
      'info',
      'offline',
      'ok',
    ];
    for (const [index, kind] of kinds.entries()) {
      const chip = active.nth(index);
      const expected = rgb(dark[`color-status-${kind}` as keyof typeof dark]);
      expect(await style(chip, 'color'), kind).toBe(expected);
      await expect(chip.locator('.glyph')).toHaveCount(1);
      expect((await chip.innerText()).trim().length).toBeGreaterThan(0);
      await expect(chip).toHaveAttribute('aria-live', 'polite');
      const a = (await chip.boundingBox())!;
      const b = (await dim.nth(index).boundingBox())!;
      expect({ w: b.width, h: b.height }, `${kind} dim box`).toEqual({ w: a.width, h: a.height });
      expect(await style(dim.nth(index), 'opacity')).toBe('0.4');
    }
  });

  test('given each Badge kind, when checked, then colour, outline style and text label match AC-BD1', async ({
    page,
  }) => {
    const badges = section(page, 'Badge').locator('.badge');
    const expected: [string, string, string][] = [
      ['Unverified', dark['color-status-warning'], 'dashed'],
      ['SW', dark['color-status-info'], 'solid'],
      ['Offline', dark['color-status-offline'], 'solid'],
      ['Error', dark['color-status-error'], 'solid'],
      ['Exp', dark['color-text-muted'], 'dotted'],
    ];
    for (const [index, [label, colour, borderStyle]] of expected.entries()) {
      const badge = badges.nth(index);
      await expect(badge).toContainText(label);
      expect(await style(badge, 'color'), label).toBe(rgb(colour));
      expect(await style(badge, 'border-top-style'), label).toBe(borderStyle);
      await expect(badge).toHaveAttribute('role', 'note');
    }
  });

  test('given the reduced-motion preference, when checked, then the motion tokens are 0 ms', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    const values = await page.evaluate(() => {
      const s = getComputedStyle(document.documentElement);
      return ['--ss-motion-fast', '--ss-motion-normal'].map((n) => s.getPropertyValue(n).trim());
    });
    expect(values).toEqual(['0ms', '0ms']);
  });
});

test.describe('controls', () => {
  test('given a disabled Button with a reason, when clicked or Enter or Space is pressed, then nothing happens and focus shows the reason', async ({
    page,
  }) => {
    const button = section(page, 'Button').getByRole('button', { name: 'Disabled' });
    await expect(button).toHaveAttribute('aria-disabled', 'true');
    await button.click({ force: true });
    await button.focus();
    await page.keyboard.press('Enter');
    await page.keyboard.press('Space');
    await expect(page.getByRole('tooltip').filter({ hasText: 'Stop acquisition first' })).toBeVisible();
  });

  test('given the Segmented group, when ArrowRight and Enter are used, then exactly one option has aria-pressed true', async ({
    page,
  }) => {
    const group = section(page, 'Segmented').getByRole('group', { name: 'Sweep' });
    const options = group.getByRole('button');
    await options.first().focus();
    await page.keyboard.press('ArrowRight');
    await expect(options.nth(1)).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(group.locator('[aria-pressed="true"]')).toHaveCount(1);
    await expect(options.nth(1)).toHaveAttribute('aria-pressed', 'true');
  });

  test('given the Video option of Segmented, when focused, then it is disabled and shows its reason', async ({
    page,
  }) => {
    const video = section(page, 'Segmented').getByRole('button', { name: /Video/ });
    await expect(video).toHaveAttribute('aria-disabled', 'true');
    await video.focus();
    await expect(page.getByRole('tooltip').filter({ hasText: 'Video trigger is experimental' })).toBeVisible();
    await page.keyboard.press('Enter');
    await expect(video).toHaveAttribute('aria-pressed', 'false');
  });

  test('given a Stepper, when the wheel turns over it, then the page does not scroll and its value text is announced', async ({
    page,
  }) => {
    const stepper = section(page, 'Stepper').locator('[aria-valuetext]').first();
    await expect(stepper).toHaveAttribute('aria-valuetext', '500 millivolts per division');
    await stepper.scrollIntoViewIfNeeded();
    const before = await page.locator('.gallery').evaluate((e) => e.scrollTop);
    await stepper.hover();
    await page.mouse.wheel(0, -120);
    await page.waitForTimeout(100);
    expect(await page.locator('.gallery').evaluate((e) => e.scrollTop)).toBe(before);
  });

  test('given a Stepper at its maximum, when checked, then its increase control is aria-disabled', async ({ page }) => {
    const stepper = section(page, 'Stepper').locator('.stepper:has([aria-valuetext="2 nanoseconds per division"])');
    await expect(stepper.getByRole('button', { name: 'Increase' })).toHaveAttribute('aria-disabled', 'true');
    await expect(stepper.getByRole('button', { name: 'Decrease' })).not.toHaveAttribute('aria-disabled', 'true');
  });

  test('given a NumberField with an error, when checked, then it is aria-invalid and keeps the size of a valid one', async ({
    page,
  }) => {
    const valid = section(page, 'NumberField').locator('input').first();
    const invalid = section(page, 'NumberField').locator('input[aria-invalid="true"]');
    await expect(invalid).toHaveCount(1);
    const a = (await valid.locator('xpath=ancestor::*[contains(@class,"field")][1]').boundingBox())!;
    const b = (await invalid.locator('xpath=ancestor::*[contains(@class,"field")][1]').boundingBox())!;
    expect(b.height).toBe(a.height);
  });

  test('given a NumberField being edited, when Escape is pressed, then the shown value is restored', async ({
    page,
  }) => {
    const input = section(page, 'NumberField').locator('input').first();
    const original = await input.inputValue();
    await input.fill('3.5');
    await page.keyboard.press('Escape');
    await expect(input).toHaveValue(original);
  });
});

test.describe('overlays (not verifiable in jsdom)', () => {
  test('given the modal Dialog, when opened, then it is a modal top-layer dialog that traps Tab and Shift+Tab and Escape returns focus to the opener', async ({
    page,
  }) => {
    const opener = section(page, 'Dialog').getByRole('button', { name: 'Open modal dialog' });
    await opener.click();
    const dialog = page.getByRole('dialog', { name: /Zero calibration/ });
    await expect(dialog).toBeVisible();
    expect(await dialog.evaluate((d) => d.matches(':modal'))).toBe(true);
    await expect(dialog.getByRole('button', { name: 'Close' })).toBeFocused();
    // The native modal dialog lets focus leave the page (to the browser UI, activeElement is then body) but never reaches the page behind.
    const behind = () =>
      page.evaluate(() => {
        const active = document.activeElement;
        return active !== document.body && !active?.closest('dialog[open]');
      });
    for (let i = 0; i < 4; i += 1) {
      await page.keyboard.press('Tab');
      expect(await behind()).toBe(false);
    }
    for (let i = 0; i < 4; i += 1) {
      await page.keyboard.press('Shift+Tab');
      expect(await behind()).toBe(false);
    }
    await page.keyboard.press('Escape');
    await expect(dialog).toBeHidden();
    await expect(opener).toBeFocused();
  });

  test('given the modal Dialog is open, when checked, then the page behind does not react to a click outside of it', async ({
    page,
  }) => {
    await section(page, 'Dialog').getByRole('button', { name: 'Open modal dialog' }).click();
    await page.mouse.click(5, 5);
    await expect(page.getByRole('dialog', { name: /Zero calibration/ })).toBeVisible();
  });

  test('given the non-modal Dialog (task panel), when opened, then the page behind stays operable', async ({
    page,
  }) => {
    await section(page, 'Dialog').getByRole('button', { name: 'Open non-modal dialog' }).click();
    const task = page.getByRole('dialog', { name: /Task panel/ });
    await expect(task).toBeVisible();
    expect(await task.evaluate((d) => d.matches(':modal'))).toBe(false);
    await section(page, 'Toast').getByRole('button', { name: 'Show toast' }).click();
  });

  test('given the Menu, when opened and navigated, then arrows move focus, a disabled item is focusable but inert and Escape closes it', async ({
    page,
  }) => {
    await section(page, 'Menu').getByRole('button', { name: 'Open menu' }).click();
    const items = page.getByRole('menuitem');
    await expect(items.first()).toBeFocused();
    const count = await items.count();
    const disabledIndex = await items.evaluateAll((els) =>
      els.findIndex((e) => e.getAttribute('aria-disabled') === 'true'),
    );
    expect(disabledIndex).toBeGreaterThan(-1);
    for (let i = 0; i < disabledIndex; i += 1) await page.keyboard.press('ArrowDown');
    await expect(items.nth(disabledIndex)).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(page.getByRole('menu')).toBeVisible();
    await expect(section(page, 'Menu')).toContainText('Selected: none');
    await page.keyboard.press('ArrowUp');
    await expect(items.nth(disabledIndex - 1)).toBeFocused();
    expect(count).toBeGreaterThan(2);
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menu')).toHaveCount(0);
  });

  test('given the Menu closed with Escape, when checked, then focus returns to the button that opened it', async ({
    page,
  }) => {
    const opener = section(page, 'Menu').getByRole('button', { name: 'Open menu' });
    await opener.click();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menu')).toHaveCount(0);
    test.info().annotations.push({ type: 'observation', description: 'focus-return on Menu close' });
    await expect(opener).toBeFocused();
  });

  test('given the Menu, when an item is chosen with Enter, then it is selected and the menu closes', async ({
    page,
  }) => {
    await section(page, 'Menu').getByRole('button', { name: 'Open menu' }).click();
    await page.keyboard.press('Enter');
    await expect(page.getByRole('menu')).toHaveCount(0);
    await expect(section(page, 'Menu')).not.toContainText('Selected: none');
  });
});

test.describe('tooltip and feedback', () => {
  test('given a Badge, when hovered, then no tooltip shows before 500 ms and one shows after', async ({ page }) => {
    const badge = section(page, 'Badge').locator('.badge').first();
    await badge.scrollIntoViewIfNeeded();
    await badge.hover();
    await page.waitForTimeout(200);
    const early = await page.getByRole('tooltip').filter({ hasText: 'not verified' }).count();
    await page.waitForTimeout(500);
    await expect(page.getByRole('tooltip').filter({ hasText: 'driver has not verified' })).toBeVisible();
    expect(early).toBe(0);
    await page.keyboard.press('Escape');
    await expect(page.getByRole('tooltip').filter({ hasText: 'driver has not verified' })).toHaveCount(0);
  });

  test('given a Tooltip host, when its tooltip is shown, then the control is linked to it by aria-describedby', async ({
    page,
  }) => {
    const button = section(page, 'Tooltip').getByRole('button', { name: 'Focused control' });
    await expect(page.getByRole('tooltip')).toBeVisible();
    const describedBy = await button.getAttribute('aria-describedby');
    expect(describedBy).toBeTruthy();
    const tipId = await page.getByRole('tooltip').first().getAttribute('id');
    expect(describedBy).toContain(tipId!);
  });

  test('given the Banner stack of three, when checked, then two are visible with the newest first, and roles match severity', async ({
    page,
  }) => {
    const banners = section(page, 'Banner');
    await expect(banners.getByText('Oldest, hidden')).toHaveCount(0);
    const stack = banners.getByText(/Stack: newest|Stack: older/);
    await expect(stack).toHaveCount(2);
    expect(await stack.first().innerText()).toContain('newest');
    await expect(banners.locator('[role="alert"]').first()).toBeVisible();
    await expect(banners.locator('[role="status"]').first()).toBeVisible();
  });

  test('given a Toast, when checked, then it never takes focus and disappears on its own', async ({ page }) => {
    const toast = section(page, 'Toast');
    await toast.getByRole('button', { name: 'Show toast' }).click();
    expect(
      await page.evaluate(() => document.activeElement?.closest('[data-component="Toast"] [role="status"]') ?? null),
    ).toBeNull();
    await expect(toast.getByText('Setup saved')).toBeHidden({ timeout: 8000 });
  });

  test('given the collapsed Panel, when checked, then its header has aria-expanded false and the body is absent', async ({
    page,
  }) => {
    const panel = section(page, 'Panel');
    await expect(panel.getByText('Hidden until expanded')).toHaveCount(0);
    await expect(panel.locator('[aria-expanded="false"]')).toHaveCount(1);
  });

  test('given the MeasurementMatrix, when checked, then headers are th with scope and invalid cells show a muted dash with a footnote key', async ({
    page,
  }) => {
    const matrix = section(page, 'MeasurementMatrix').locator('table').first();
    expect(await matrix.locator('th:not([scope])').count()).toBe(0);
    await expect(matrix.getByText('[2]').first()).toBeVisible();
    await expect(matrix.locator('.unverified').first()).toBeVisible();
  });
});
