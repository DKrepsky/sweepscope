import { expect, test, type Page } from '@playwright/test';
import { openMain } from './support/app';

const COLUMNS = [
  '.toggle',
  '.stepper',
  '.field',
  '.seg',
  'button[aria-label^="BW"]',
  'button[aria-label$="probe attenuation"]',
  'button[aria-label^="Inv"]',
];

const columnWidths = (page: Page) =>
  page.evaluate(
    (selectors) =>
      [...document.querySelectorAll<HTMLElement>('[data-channel]')].map((row) =>
        selectors.map((selector) =>
          Math.round(row.querySelector<HTMLElement>(selector)?.getBoundingClientRect().width ?? -1),
        ),
      ),
    COLUMNS,
  );

/** What the eye sees: the element's opacity multiplied by every ancestor's up to the row. */
const effectiveOpacity = (page: Page) =>
  page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>('[data-channel]')].map((row) => {
      const of = (selector: string) => {
        let opacity = 1;
        for (let el = row.querySelector(selector); el && el !== row.parentElement; el = el.parentElement) {
          opacity *= Number(getComputedStyle(el).opacity);
        }
        return opacity;
      };
      return { stepper: of('.step'), coupling: of('.seg') };
    }),
  );

test.describe('channel rows (bug #40)', () => {
  test('given four channel rows with different values, when the dock is shown, then every column has the same width in every row', async ({
    page,
  }) => {
    await openMain(page);

    const widths = await columnWidths(page);

    expect(widths).toHaveLength(4);
    for (const row of widths) expect(row).toEqual(widths[0]);
  });

  test('given a disabled channel row, when compared with its stepper and the enabled rows, then the AC/DC control is dimmed like the stepper and enabled rows are not dimmed', async ({
    page,
  }) => {
    await openMain(page);

    const [first, second, third, disabled] = await effectiveOpacity(page);

    for (const enabled of [first, second, third]) expect(enabled).toEqual({ stepper: 1, coupling: 1 });
    expect(disabled!.coupling).toBe(disabled!.stepper);
    expect(disabled!.coupling).toBeLessThan(1);
  });
});
