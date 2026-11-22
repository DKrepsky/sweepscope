import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { openGallery, openMain } from './support/app';

const TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'];

interface Finding {
  id: string;
  impact: string | null | undefined;
  nodes: string[];
}

/**
 * Dim status chips (40 % opacity by design 9) and disabled controls are inactive UI components, exempt from
 * WCAG 1.4.3, but axe cannot tell because they are not natively disabled: they are excluded here.
 */
const INACTIVE = ['.dim', '.disabled', '[aria-disabled="true"]'];

async function findings(page: Page): Promise<Finding[]> {
  let builder = new AxeBuilder({ page }).withTags(TAGS);
  for (const selector of INACTIVE) builder = builder.exclude(selector);
  const result = await builder.analyze();
  return result.violations.map((v) => ({
    id: v.id,
    impact: v.impact,
    nodes: v.nodes.slice(0, 6).map((n) => `${n.target.join(' ')} :: ${n.failureSummary?.split('\n')[1] ?? ''}`),
  }));
}

/** BUG-10 (label in name) is open: the general checks ignore it, a dedicated check tracks it. */
const isKnownLabel = (f: Finding) => f.id === 'label-content-name-mismatch';

/** The `text-disabled` token (#5f666d) is only used by disabled controls, which are exempt from 1.4.3. */
const withoutDisabledText = (f: Finding): Finding => ({
  ...f,
  nodes: f.id === 'color-contrast' ? f.nodes.filter((n) => !n.includes('foreground color: #5f666d')) : f.nodes,
});

async function violations(page: Page): Promise<Finding[]> {
  return (await findings(page)).map(withoutDisabledText).filter((f) => f.nodes.length > 0 && !isKnownLabel(f));
}

test.describe('axe (WCAG 2.1 A and AA, includes colour contrast)', () => {
  test('given the component gallery, when axe runs, then there are no violations', async ({ page }) => {
    await openGallery(page);
    await page.waitForTimeout(300);
    expect(await violations(page)).toEqual([]);
  });

  test('given the main window at 1440 x 900, when axe runs, then there are no violations', async ({ page }) => {
    await openMain(page);
    await page.waitForTimeout(300);
    expect(await violations(page)).toEqual([]);
  });

  test('given the main window at text scale 200 %, when axe runs, then there are no violations', async ({ page }) => {
    await page.setViewportSize({ width: 2560, height: 1440 });
    await openMain(page, 200);
    await page.waitForTimeout(300);
    expect(await violations(page)).toEqual([]);
  });

  test('given the main window at the compact tier with the drawer open, when axe runs, then there are no violations', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await openMain(page, 125);
    await page.getByRole('button', { name: 'Measurements drawer' }).click();
    expect(await violations(page)).toEqual([]);
  });

  test('given the Settings popover, the persistence menu and the Accuracy popover, when axe runs on each, then there are no violations', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    const settings = await violations(page);
    await page.getByRole('button', { name: 'Settings' }).click();
    await page.getByRole('button', { name: /^Persist/ }).click();
    const menu = await violations(page);
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: /^Accuracy/ }).click();
    const accuracy = await violations(page);
    expect({ settings, menu, accuracy }).toEqual({ settings: [], menu: [], accuracy: [] });
  });

  test('given the gallery with the modal dialog open, when axe runs, then there are no violations', async ({
    page,
  }) => {
    await openGallery(page);
    await page.getByRole('button', { name: 'Open modal dialog' }).click();
    expect(await violations(page)).toEqual([]);
  });

  test('given the Stop button and chip, when axe checks colour contrast, then the Stop text meets 4.5:1', async ({
    page,
  }) => {
    await openMain(page);
    expect(
      (await findings(page)).filter((f) => f.id === 'color-contrast' && f.nodes.some((n) => n.startsWith('.stop'))),
    ).toEqual([]);
  });

  test('given the channel row and scale strip buttons, when axe checks label in name, then the visible text is part of the accessible name', async ({
    page,
  }) => {
    await openMain(page);
    expect((await findings(page)).filter(isKnownLabel)).toEqual([]);
  });
});
