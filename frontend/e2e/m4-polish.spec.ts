import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { connectionSnapshot, instrumentState, recordingState } from '../src/app/instrument/test-data';
import { openGallery, openMain as open, type TextScale } from './support/app';
import { emit, ipcLog, tooltipFor, zeroStub, type StubOptions } from './support/tauri-stub';
import type { InstrumentState } from '../src/api';

/**
 * M4 Polish (#33, #34, #50) against the IPC stub, written by QA: calibration tag and tooltip, the dialog in every step,
 * the connection item, the marker removal. The real wiring is checked in the real window (test plan R cases).
 * Test plan: docs/specs/m4-polish/test-plan.md.
 */

const unitOf = (calibrated: boolean) => ({
  calibrated,
  storedAtUtc: calibrated ? '2026-11-24T14:05:00Z' : null,
  tooltip: tooltipFor(calibrated),
});
const CALIBRATED = { unit: unitOf(true) };
const NOT_CALIBRATED = { unit: unitOf(false) };

/** The instrument slice with the given channels enabled and each on a range index (0 = 2 mV/div) and probe. */
const instrumentWith = (
  revision: number,
  channels: { enabled: boolean; range?: number; probe?: number }[],
): InstrumentState => {
  const base = instrumentState({ revision });
  return {
    ...base,
    channels: base.channels.map((channel, index) => {
      const set = channels[index] ?? { enabled: false };
      const probe = set.probe ?? 1;
      return {
        ...channel,
        enabled: set.enabled,
        probe,
        probeText: `${probe}x`,
        range: { ...channel.range, index: set.range ?? 0 },
      };
    }),
  };
};

const openMain = async (page: Page, options: StubOptions = {}, percent: TextScale = 100) => {
  await open(page, percent, { zero: NOT_CALIBRATED, ...options });
  await expect.poll(async () => (await ipcLog(page)).some((call) => call.command === 'open_frame_stream')).toBe(true);
};
const tag = (page: Page) =>
  page.getByRole('button', { name: /^(No calibration|Calibrated)\. Open calibration dialog$/ });
const dialog = (page: Page) => page.getByRole('dialog', { name: 'Zero calibration' });
const heading = (page: Page) => dialog(page).locator('[data-step-focus]');
const status = (page: Page) => page.locator('[data-region="status"]');
const calls = async (page: Page, name: string) => (await ipcLog(page)).filter((call) => call.command === name);

async function toDone(page: Page): Promise<void> {
  await tag(page).click();
  await dialog(page).getByRole('button', { name: 'Start' }).click();
  for (let channel = 1; channel <= 4; channel++) {
    await expect(heading(page)).toHaveText(`Channel ${channel}: short the input`);
    await dialog(page).getByRole('button', { name: 'Calibrate' }).click();
  }
  await expect(heading(page)).toContainText('Calibration completed');
}

test.describe('calibration status tag', () => {
  test('given a connected unit without offsets, when shown, then the tag reads NO CALIBRATION at the right end of the bar', async ({
    page,
  }) => {
    await openMain(page);
    await expect(tag(page)).toHaveText(/No calibration/i);
    await expect(tag(page)).toHaveAttribute('aria-label', 'No calibration. Open calibration dialog');
    await expect(status(page).locator('.item').last()).toHaveAttribute('data-item', 'calibration');
    const box = await tag(page).boundingBox();
    const bar = await status(page).boundingBox();
    expect(box!.x + box!.width).toBeGreaterThan(bar!.x + bar!.width - 40);
  });

  test('given a unit with stored offsets, when shown, then the tag reads CALIBRATED with an icon and green colour', async ({
    page,
  }) => {
    await openMain(page, { zero: CALIBRATED });
    await expect(tag(page)).toHaveText(/Calibrated/i);
    await expect(tag(page).locator('svg')).toHaveCount(1);
    await page.waitForTimeout(200);
    const colour = await tag(page).evaluate((element) => getComputedStyle(element).color);
    expect(colour).toBe('rgb(79, 209, 139)');
  });

  test('given a scope that is gone, when shown, then the tag is a disabled Calibration - with the reason and cannot open the dialog', async ({
    page,
  }) => {
    await openMain(page, { zero: { availability: { enabled: false, reason: 'No device' } } });
    const disabled = status(page).getByRole('button', { name: 'Calibration -' });
    await expect(disabled).toBeDisabled();
    await expect(disabled).toHaveAttribute('title', 'No device');
    await disabled.click({ force: true });
    expect(await calls(page, 'open_zero_calibration')).toHaveLength(0);
  });

  test('given a recording in progress, when shown, then calibration is unavailable and says to stop recording first', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'recording_changed', recordingState({ revision: 5, active: true, fileName: 'a.ssroll' }));
    const disabled = status(page).getByRole('button', { name: 'Calibration -' });
    await expect(disabled).toHaveAttribute('title', 'Stop recording first');
  });

  test('given the device is lost while idle, when the slice says so, then the tag becomes disabled', async ({
    page,
  }) => {
    await openMain(page, { zero: CALIBRATED });
    await zeroStub.loseDevice(page);
    await expect(status(page).getByRole('button', { name: 'Calibration -' })).toBeDisabled();
  });

  test('given not calibrated, when the tag is hovered for half a second, then the tooltip shows the typical values and the click hint', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).hover();
    const tip = page.getByRole('tooltip');
    await expect(tip).toBeVisible({ timeout: 2000 });
    await expect(tip).toContainText('No calibration');
    await expect(tip).toContainText('Typical zero error:');
    await expect(tip.locator('li')).toHaveText([
      'CH1 +0.66 mV at 2 mV/div',
      'CH2 +0.66 mV at 2 mV/div',
      'CH3 +0.66 mV at 2 mV/div',
    ]);
    await expect(tip).toContainText('Click to calibrate.');
    await expect(tip).not.toContainText('Click to calibrate again.');
  });

  test('given calibrated, when the tag takes keyboard focus, then the tooltip shows at once with the offsets and Escape hides it', async ({
    page,
  }) => {
    await openMain(page, { zero: CALIBRATED });
    await tag(page).focus();
    const tip = page.getByRole('tooltip');
    await expect(tip).toBeVisible();
    await expect(tip.locator('li')).toHaveText([
      'CH1 +0.66 mV at 2 mV/div',
      'CH2 +1.04 mV at 2 mV/div',
      'CH3 \u22120.83 mV at 2 mV/div',
    ]);
    await expect(tip).toContainText('Click to calibrate again.');
    await expect(tip).toContainText(/^Calibrated \d/);
    await page.keyboard.press('Escape');
    await expect(tip).toHaveCount(0);
  });

  test('given the tag has the focus, when Enter or Space is pressed, then the backend is asked to open the dialog', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).focus();
    await page.keyboard.press('Enter');
    await expect(dialog(page)).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(dialog(page)).toHaveCount(0);
    await tag(page).focus();
    await page.keyboard.press('Space');
    await expect(dialog(page)).toBeVisible();
    expect(await calls(page, 'open_zero_calibration')).toHaveLength(2);
  });
});

test.describe('calibration tooltip rows', () => {
  const lines = async (page: Page) => {
    await tag(page).focus();
    const tip = page.getByRole('tooltip');
    await expect(tip).toBeVisible();
    return tip;
  };

  test('given channels 1 and 3 on mixed ranges and CH2 off, when the tooltip shows, then there is one row per enabled channel at its own range', async ({
    page,
  }) => {
    await openMain(page, { zero: CALIBRATED });
    await emit(
      page,
      'instrument_changed',
      instrumentWith(2, [{ enabled: true, range: 2 }, { enabled: false }, { enabled: true, range: 5 }]),
    );
    const tip = await lines(page);
    await expect(tip.locator('li')).toHaveText(['CH1 +0.66 mV at 10 mV/div', 'CH3 0 V at 100 mV/div']);
    await expect(tip.locator('li').first().locator('b')).toHaveText('CH1');
  });

  test('given not calibrated and mixed ranges, when the tooltip shows, then the rows carry the typical zero error of each range', async ({
    page,
  }) => {
    await openMain(page);
    await emit(
      page,
      'instrument_changed',
      instrumentWith(2, [
        { enabled: true, range: 1 },
        { enabled: true, range: 3 },
      ]),
    );
    const tip = await lines(page);
    await expect(tip).toContainText('Typical zero error:');
    await expect(tip.locator('li')).toHaveText(['CH1 +1.04 mV at 5 mV/div', 'CH2 +0.83 mV at 20 mV/div']);
  });

  test('given a 10x probe on CH1 and a 100x probe on CH2, when the tooltip shows, then the offset and the range are probe scaled', async ({
    page,
  }) => {
    await openMain(page, { zero: CALIBRATED });
    await emit(
      page,
      'instrument_changed',
      instrumentWith(2, [
        { enabled: true, range: 0, probe: 10 },
        { enabled: true, range: 2, probe: 100 },
        { enabled: true, range: 0, probe: 1 },
      ]),
    );
    const tip = await lines(page);
    await expect(tip.locator('li')).toHaveText([
      'CH1 +6.6 mV at 20 mV/div',
      'CH2 +104 mV at 1 V/div',
      'CH3 \u22120.83 mV at 2 mV/div',
    ]);
  });

  test('given the tooltip is open, when a range knob turns and a probe changes, then the rows follow without any backend request', async ({
    page,
  }) => {
    await openMain(page, { zero: CALIBRATED });
    const tip = await lines(page);
    await expect(tip.locator('li').first()).toHaveText('CH1 +0.66 mV at 2 mV/div');
    const before = (await ipcLog(page)).length;
    await emit(page, 'instrument_changed', instrumentWith(2, [{ enabled: true, range: 1 }, { enabled: true }]));
    await expect(tip.locator('li').first()).toHaveText('CH1 +0.66 mV at 5 mV/div');
    await emit(
      page,
      'instrument_changed',
      instrumentWith(3, [{ enabled: true, range: 1, probe: 10 }, { enabled: true }]),
    );
    await expect(tip.locator('li').first()).toHaveText('CH1 +6.6 mV at 50 mV/div');
    await expect(tip.locator('li')).toHaveCount(2);
    expect((await ipcLog(page)).length - before).toBeLessThanOrEqual(0);
  });

  test('given every channel is off, when the tooltip shows, then it says No channel enabled and has no rows, in both states', async ({
    page,
  }) => {
    for (const zero of [NOT_CALIBRATED, CALIBRATED]) {
      await openMain(page, { zero });
      await emit(page, 'instrument_changed', instrumentWith(2, []));
      const tip = await lines(page);
      await expect(tip).toContainText('No channel enabled');
      await expect(tip.locator('li')).toHaveCount(0);
      await page.keyboard.press('Escape');
    }
  });

  test('given both states, when the rows are read, then each row has its form (an upper bound from 50 mV/div when not calibrated) and the notes about gain, extreme positions and resolution are gone', async ({
    page,
  }) => {
    const offset = /^CH\d (?:[+\u2212]\d+(?:\.\d+)? mV|0 V) at \d+ (?:mV|V)\/div$/;
    const bound = /^CH\d up to \d+(?:\.\d+)? (?:mV|V) at \d+ (?:mV|V)\/div$/;
    for (const zero of [NOT_CALIBRATED, CALIBRATED]) {
      await openMain(page, { zero });
      await emit(
        page,
        'instrument_changed',
        instrumentWith(2, [
          { enabled: true, range: 0 },
          { enabled: true, range: 5 },
          { enabled: true, range: 4, probe: 10 },
          { enabled: true, range: 11 },
        ]),
      );
      const tip = await lines(page);
      const rows = await tip.locator('li').allInnerTexts();
      expect(rows).toHaveLength(4);
      for (const row of rows)
        expect(row.replace(/\s+/g, ' ')).toMatch(
          zero === NOT_CALIBRATED ? new RegExp(`${offset.source}|${bound.source}`) : offset,
        );
      await expect(tip).not.toContainText(/\bGain\b|extreme|Resolution|counts/i);
      await page.keyboard.press('Escape');
    }
  });

  test('given the calibrated tooltip, when read, then the caption Typical zero error is absent and the stored date heads it', async ({
    page,
  }) => {
    await openMain(page, { zero: CALIBRATED });
    const tip = await lines(page);
    await expect(tip).not.toContainText('Typical zero error');
    await expect(tip).toContainText(/Calibrated \d/);
  });
});

test.describe('calibration dialog', () => {
  test('given NO CALIBRATION, when clicked, then a modal dialog explains why with Cancel and Start at step 1 of 6', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).click();
    await expect(dialog(page)).toBeVisible();
    await expect(dialog(page)).toContainText('Step 1 of 6');
    await expect(heading(page)).toHaveText('Why calibrate');
    await expect(dialog(page).locator('li')).toHaveCount(3);
    await expect(dialog(page)).toContainText('Acquisition is paused while this dialog is open');
    await expect(dialog(page)).toContainText(
      'Nothing is stored until you finish, and the factory calibration is never changed',
    );
    await expect(dialog(page).getByRole('button')).toHaveText(['Cancel', 'Start']);
    expect(await dialog(page).evaluate((element: HTMLDialogElement) => element.matches(':modal'))).toBe(true);
  });

  test('given step 1, when opened, then the focus is on the heading and Enter does not start anything', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).click();
    await expect(heading(page)).toBeFocused();
    await page.keyboard.press('Enter');
    expect(await calls(page, 'start_zero_calibration')).toHaveLength(0);
    await expect(dialog(page)).toContainText('Step 1 of 6');
  });

  test('given step 1, when opened, then the focused heading shows no outline', async ({ page }) => {
    await openMain(page);
    await tag(page).click();
    await expect(heading(page)).toBeFocused();
    await expect(heading(page)).toHaveCSS('outline-style', 'none');
  });

  test('given step 1, when Start, then channel 1 shows three instructions, the open-input note, chips, and Calibrate not focused', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).click();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await expect(dialog(page)).toContainText('Step 2 of 6');
    await expect(heading(page)).toHaveText('Channel 1: short the input');
    await expect(heading(page)).toBeFocused();
    await expect(dialog(page).locator('ol li')).toHaveCount(3);
    await expect(dialog(page)).toContainText('Never short a live circuit.');
    await expect(dialog(page)).toContainText('An open input is not zero: it picks up the neighbouring channels.');
    await expect(dialog(page).getByRole('img', { name: 'Probe tip touching the probe ground clip' })).toBeVisible();
    await expect(dialog(page).getByLabel('Channel 1, current')).toBeVisible();
    await expect(dialog(page).getByLabel('Channel 2, to do')).toBeVisible();
    await expect(dialog(page).getByRole('button', { name: 'Calibrate' })).not.toBeFocused();
  });

  test('given all four channels shorted, when each is calibrated, then the chips fill in and step 6 shows the table and the tag turns CALIBRATED', async ({
    page,
  }) => {
    await openMain(page);
    await toDone(page);
    await expect(dialog(page)).toContainText('Step 6 of 6');
    await expect(dialog(page)).toContainText('Offsets applied per channel and range and kept after restart');
    await expect(dialog(page).getByLabel('Channel 4, done')).toBeVisible();
    const rows = dialog(page).locator('tbody tr');
    await expect(rows).toHaveCount(12);
    await expect(rows.first().locator('th')).toHaveText('2 mV/div');
    await expect(rows.last().locator('th')).toHaveText('10 V/div');
    await expect(dialog(page).locator('thead th')).toHaveCount(5);
    await expect(dialog(page).getByRole('button')).toHaveText(['Done']);
    expect((await calls(page, 'measure_zero_calibration')).length).toBe(4);
    await expect(tag(page)).toHaveAttribute('aria-label', 'Calibrated. Open calibration dialog');
  });

  test('given the Done step, when Done or Escape, then the dialog closes, the tag stays CALIBRATED and the focus returns to it', async ({
    page,
  }) => {
    await openMain(page);
    await toDone(page);
    await page.keyboard.press('Escape');
    await expect(dialog(page)).toHaveCount(0);
    await expect(tag(page)).toBeFocused();
    await expect(tag(page)).toHaveAttribute('aria-label', 'Calibrated. Open calibration dialog');
    await tag(page).click();
    await expect(dialog(page)).toContainText('This device is calibrated');
    await expect(dialog(page)).toContainText('Last calibrated at');
    await page.keyboard.press('Escape');
    await expect(dialog(page)).toHaveCount(0);
  });

  test('given a calibrated unit, when the tag is clicked, then step 1 says this device is calibrated and when', async ({
    page,
  }) => {
    await openMain(page, { zero: CALIBRATED });
    await tag(page).click();
    await expect(dialog(page)).toContainText('This device is calibrated');
    await expect(dialog(page)).toContainText(/Last calibrated at \S/);
    await expect(dialog(page)).toContainText('Why calibrate');
  });

  test('given measuring with a known progress, when shown, then a progressbar has the value, Cancel is enabled and the primary is disabled', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).click();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await zeroStub.showMeasuring(page, 0.25);
    await expect(heading(page)).toHaveText('Channel 1: measuring');
    const bar = dialog(page).getByRole('progressbar');
    await expect(bar).toHaveAttribute('aria-valuenow', /^(25|0\.25)$/);
    await expect(dialog(page)).toContainText('Measuring CH1, keep the input shorted and still');
    await expect(dialog(page)).toContainText('Cancel stores nothing.');
    await expect(dialog(page).getByRole('button', { name: 'Cancel' })).toBeEnabled();
    await expect(dialog(page).getByRole('button', { name: 'Measuring...' })).toBeDisabled();
  });

  test('given measuring with no progress, when shown, then the bar is indeterminate (no value) and no percent text', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).click();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await zeroStub.showMeasuring(page, null);
    const bar = dialog(page).getByRole('progressbar');
    await expect(bar).toBeVisible();
    await expect(bar).not.toHaveAttribute('aria-valuenow', /.+/);
    await expect(dialog(page)).not.toContainText('%');
  });

  test('given channel 2 is not shorted, when Calibrate, then an alert with the numbers and Try again, no Skip, and the dialog stays on channel 2', async ({
    page,
  }) => {
    await openMain(page);
    await zeroStub.leaveUnshorted(page, [2]);
    await tag(page).click();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await dialog(page).getByRole('button', { name: 'Calibrate' }).click();
    await dialog(page).getByRole('button', { name: 'Calibrate' }).click();
    const alert = dialog(page).getByRole('alert');
    await expect(alert).toContainText('The CH2 input does not look shorted. Nothing was stored for this channel.');
    await expect(alert).toContainText('Peak to peak 49.7 mV at 100 mV/div, limit 24.8 mV.');
    await expect(dialog(page)).toContainText('Check the connection between probe tip and ground clip, then try again.');
    await expect(dialog(page).getByRole('button')).toHaveText(['Cancel', 'Continue', 'Try again']);
    await expect(dialog(page).getByRole('button', { name: /skip/i })).toHaveCount(0);
    await expect(dialog(page).getByLabel('Channel 1, done')).toBeVisible();
    await expect(dialog(page).getByLabel('Channel 2, current')).toBeVisible();
    await expect(tag(page)).toHaveAttribute('aria-label', 'No calibration. Open calibration dialog');
    await zeroStub.leaveUnshorted(page, []);
    await dialog(page).getByRole('button', { name: 'Try again' }).click();
    await expect(heading(page)).toHaveText('Channel 3: short the input');
  });

  test('given channels 1 to 3 measured, when Cancel, then the dialog closes, the tag still says NO CALIBRATION and close was requested', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).click();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    for (let n = 0; n < 3; n++) await dialog(page).getByRole('button', { name: 'Calibrate' }).click();
    await expect(heading(page)).toHaveText('Channel 4: short the input');
    await dialog(page).getByRole('button', { name: 'Cancel' }).click();
    await expect(dialog(page)).toHaveCount(0);
    await expect(tag(page)).toHaveAttribute('aria-label', 'No calibration. Open calibration dialog');
    expect((await calls(page, 'close_zero_calibration')).length).toBe(1);
  });

  test('given any step before Done, when Escape, then it cancels like the Cancel button', async ({ page }) => {
    await openMain(page);
    await tag(page).click();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await page.keyboard.press('Escape');
    await expect(dialog(page)).toHaveCount(0);
    expect((await calls(page, 'close_zero_calibration')).length).toBe(1);
  });

  test('given the device is lost mid-session, when it happens, then the body is an alert with one Close button', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).click();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await zeroStub.loseDevice(page);
    await expect(dialog(page).getByRole('alert')).toContainText(
      'Device disconnected. Calibration cancelled. Nothing was stored.',
    );
    await expect(dialog(page).getByRole('button')).toHaveText(['Close']);
    await dialog(page).getByRole('button', { name: 'Close' }).click();
    await expect(dialog(page)).toHaveCount(0);
  });

  test('given the dialog is open, when Tab is pressed many times, then the focus never reaches the page behind', async ({
    page,
  }) => {
    await openMain(page);
    await tag(page).click();
    for (let n = 0; n < 8; n++) {
      await page.keyboard.press('Tab');
      const inside = await page.evaluate(() => {
        const active = document.activeElement;
        return active === document.body || !!active?.closest('dialog');
      });
      expect(inside).toBe(true);
    }
    await page.keyboard.press('Shift+Tab');
  });

  test('given the dialog is open while running, when shown, then the toolbar chip reads Paused (calibration) and returns when closed', async ({
    page,
  }) => {
    await openMain(page);
    const state = page.getByRole('status', { name: 'Acquisition state' });
    const before = await state.innerText();
    expect(before).not.toContain('Paused (calibration)');
    await tag(page).click();
    await expect(state).toContainText('Paused (calibration)');
    await dialog(page).getByRole('button', { name: 'Cancel' }).click();
    await expect(state).not.toContainText('Paused (calibration)');
    expect(await state.innerText()).toBe(before);
  });

  test('given the Done step, when the table is read with the keyboard, then the region is focusable, named, scrollable and the header stays sticky', async ({
    page,
  }) => {
    await openMain(page, {}, 200);
    await toDone(page);
    const region = dialog(page).getByRole('region', { name: /Stored offsets per voltage range and channel/ });
    await expect(region).toHaveAttribute('tabindex', '0');
    await region.focus();
    await expect(region).toBeFocused();
    const scrolls = await region.evaluate((element) => element.scrollHeight > element.clientHeight);
    expect(scrolls).toBe(true);
    await page.keyboard.press('End');
    await expect.poll(() => region.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
    const header = dialog(page).locator('thead th').first();
    const regionBox = await region.boundingBox();
    const headerBox = await header.boundingBox();
    expect(headerBox!.y).toBeGreaterThanOrEqual(regionBox!.y - 1);
    expect(headerBox!.y).toBeLessThan(regionBox!.y + 40);
    await page.keyboard.press('Tab');
    await expect(dialog(page).getByRole('button', { name: 'Done' })).toBeFocused();
  });

  for (const scale of [100, 200] as const) {
    test(`given text scale ${scale} percent at the smallest window, when the dialog shows each step, then it stays inside the window and the footer is reachable`, async ({
      page,
    }) => {
      await page.setViewportSize({ width: 1024, height: 720 });
      await openMain(page, {}, scale);
      await tag(page).click();
      const inside = async () => {
        const box = await dialog(page).boundingBox();
        expect(box!.x).toBeGreaterThanOrEqual(0);
        expect(box!.y).toBeGreaterThanOrEqual(0);
        expect(box!.x + box!.width).toBeLessThanOrEqual(1024);
        expect(box!.y + box!.height).toBeLessThanOrEqual(720);
        const footer = await dialog(page).locator('footer').boundingBox();
        expect(footer!.y + footer!.height).toBeLessThanOrEqual(720);
        expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      };
      await inside();
      await dialog(page).getByRole('button', { name: 'Start' }).click();
      await inside();
      await zeroStub.showMeasuring(page, 0.4);
      await inside();
      await dialog(page).getByRole('button', { name: 'Cancel' }).click();
      await toDone(page);
      await inside();
    });
  }

  test('given every step, when scanned by axe, then there are no serious or critical violations', async ({ page }) => {
    await openMain(page);
    const scan = async () => {
      const result = await new AxeBuilder({ page }).include('dialog').analyze();
      const bad = result.violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
      expect(bad.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
    };
    await tag(page).click();
    await scan();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await scan();
    await zeroStub.showMeasuring(page, 0.5);
    await scan();
    await dialog(page).getByRole('button', { name: 'Cancel' }).click();
    await toDone(page);
    await scan();
  });
});

const buttons = (page: Page) => dialog(page).getByRole('button');
const calibrate = (page: Page) => dialog(page).getByRole('button', { name: 'Calibrate' }).click();
async function toNotShorted(page: Page, channel: number, clipped: number[] = []): Promise<void> {
  await zeroStub.leaveUnshorted(page, [channel], clipped);
  await tag(page).click();
  await dialog(page).getByRole('button', { name: 'Start' }).click();
  for (let at = 1; at <= channel; at++) {
    await expect(heading(page)).toHaveText(`Channel ${at}: short the input`);
    await calibrate(page);
  }
  await expect(dialog(page).getByRole('alert')).toContainText(`The CH${channel} input does not look shorted`);
}

test.describe('Continue after a failed shorted check', () => {
  test('given CH2 does not look shorted, when shown, then the buttons are Cancel, Continue, Try again with the warning note and Try again is primary', async ({
    page,
  }) => {
    await openMain(page);
    await toNotShorted(page, 2);
    await expect(buttons(page)).toHaveText(['Cancel', 'Continue', 'Try again']);
    await expect(dialog(page)).toContainText(
      'Continue keeps the measured offsets for CH2, use it only if the input is shorted',
    );
    await expect(dialog(page).getByRole('button', { name: /skip/i })).toHaveCount(0);
    await expect(heading(page)).toBeFocused();
    await expect(dialog(page).getByLabel('Channel 2, current')).toBeVisible();
    const primary = (name: string) =>
      dialog(page)
        .getByRole('button', { name })
        .evaluate((element) => getComputedStyle(element).backgroundColor);
    expect(await primary('Try again')).not.toBe(await primary('Continue'));
    expect(await primary('Continue')).toBe(await primary('Cancel'));
  });

  test('given CH2 failed, when Continue, then the rest are calibrated, Done lists only CH2 as accepted and the tag turns CALIBRATED', async ({
    page,
  }) => {
    await openMain(page);
    await toNotShorted(page, 2);
    await zeroStub.leaveUnshorted(page, []);
    await dialog(page).getByRole('button', { name: 'Continue' }).click();
    expect(await calls(page, 'continue_zero_calibration')).toHaveLength(1);
    await expect(heading(page)).toHaveText('Channel 3: short the input');
    await expect(dialog(page).getByLabel('Channel 2, done')).toBeVisible();
    await expect(dialog(page)).not.toContainText('Continue keeps');
    await calibrate(page);
    await expect(heading(page)).toHaveText('Channel 4: short the input');
    await calibrate(page);
    await expect(heading(page)).toContainText('Calibration completed');
    const accepted = dialog(page).locator('ul', { hasText: 'accepted although' });
    await expect(accepted.locator('li')).toHaveText(['CH2 accepted although the input did not look shorted']);
    await expect(dialog(page).getByRole('table')).toBeVisible();
    for (const channel of [1, 2, 3, 4]) await expect(dialog(page).getByLabel(`Channel ${channel}, done`)).toBeVisible();
    await expect(buttons(page)).toHaveText(['Done']);
    await dialog(page).getByRole('button', { name: 'Done' }).click();
    await expect(tag(page)).toHaveAttribute('aria-label', 'Calibrated. Open calibration dialog');
  });

  test('given CH4 failed, when Continue, then Done follows at once, and with CH1 and CH4 accepted both are listed in order', async ({
    page,
  }) => {
    await openMain(page);
    await zeroStub.leaveUnshorted(page, [1, 4]);
    await tag(page).click();
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await calibrate(page);
    await dialog(page).getByRole('button', { name: 'Continue' }).click();
    await expect(heading(page)).toHaveText('Channel 2: short the input');
    await calibrate(page);
    await calibrate(page);
    await calibrate(page);
    await expect(dialog(page).getByRole('alert')).toContainText('The CH4 input does not look shorted');
    await dialog(page).getByRole('button', { name: 'Continue' }).click();
    await expect(heading(page)).toContainText('Calibration completed');
    await expect(dialog(page).locator('ul li')).toHaveText([
      'CH1 accepted although the input did not look shorted',
      'CH4 accepted although the input did not look shorted',
    ]);
  });

  test('given all four channels pass, when Done shows, then no accepted line is shown', async ({ page }) => {
    await openMain(page);
    await toDone(page);
    await expect(dialog(page)).not.toContainText('accepted although');
  });

  test('given CH2 failed, when Try again twice and then the input is shorted, then Continue was never sent and nothing is listed as accepted', async ({
    page,
  }) => {
    await openMain(page);
    await toNotShorted(page, 2);
    await dialog(page).getByRole('button', { name: 'Try again' }).click();
    await expect(dialog(page).getByRole('alert')).toContainText('The CH2 input does not look shorted');
    await expect(buttons(page)).toHaveText(['Cancel', 'Continue', 'Try again']);
    await zeroStub.leaveUnshorted(page, []);
    await dialog(page).getByRole('button', { name: 'Try again' }).click();
    await expect(heading(page)).toHaveText('Channel 3: short the input');
    await calibrate(page);
    await calibrate(page);
    await expect(heading(page)).toContainText('Calibration completed');
    expect(await calls(page, 'continue_zero_calibration')).toHaveLength(0);
    await expect(dialog(page)).not.toContainText('accepted although');
  });

  test('given a clipped gate reading (canContinue false), when the not shorted step shows, then there is no Continue and no warning, only Cancel and Try again', async ({
    page,
  }) => {
    await openMain(page);
    await toNotShorted(page, 3, [3]);
    await expect(buttons(page)).toHaveText(['Cancel', 'Try again']);
    await expect(dialog(page)).not.toContainText('Continue keeps');
    await expect(dialog(page).getByRole('button', { name: 'Continue' })).toHaveCount(0);
  });

  test('given the Continue step, when Escape, then it cancels: no continue, close requested, tag unchanged', async ({
    page,
  }) => {
    await openMain(page);
    await toNotShorted(page, 2);
    await page.keyboard.press('Escape');
    await expect(dialog(page)).toHaveCount(0);
    expect(await calls(page, 'continue_zero_calibration')).toHaveLength(0);
    expect(await calls(page, 'close_zero_calibration')).toHaveLength(1);
    await expect(tag(page)).toHaveAttribute('aria-label', 'No calibration. Open calibration dialog');
    await expect(tag(page)).toBeFocused();
  });

  test('given the Continue step, when Cancel after a Continue on an earlier channel, then nothing is stored', async ({
    page,
  }) => {
    await openMain(page);
    await toNotShorted(page, 1);
    await dialog(page).getByRole('button', { name: 'Continue' }).click();
    await expect(heading(page)).toHaveText('Channel 2: short the input');
    await dialog(page).getByRole('button', { name: 'Cancel' }).click();
    await expect(dialog(page)).toHaveCount(0);
    await expect(tag(page)).toHaveAttribute('aria-label', 'No calibration. Open calibration dialog');
  });

  test('given the not shorted step with Continue by keyboard, when Tab runs through the footer, then the order is Cancel, Continue, Try again', async ({
    page,
  }) => {
    await openMain(page);
    await toNotShorted(page, 2);
    await page.keyboard.press('Tab');
    await expect(dialog(page).getByRole('button', { name: 'Cancel' })).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(dialog(page).getByRole('button', { name: 'Continue' })).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(dialog(page).getByRole('button', { name: 'Try again' })).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(dialog(page).getByRole('alert')).toContainText('The CH2 input does not look shorted');
  });

  test('given every step, when the focus lands on the step heading, then no outline is drawn, while the buttons keep their ring', async ({
    page,
  }) => {
    await openMain(page);
    const outline = () => page.evaluate(() => getComputedStyle(document.activeElement as Element).outlineStyle);
    await tag(page).click();
    await expect(heading(page)).toBeFocused();
    expect(await outline()).toBe('none');
    await dialog(page).getByRole('button', { name: 'Start' }).click();
    await expect(heading(page)).toBeFocused();
    expect(await outline()).toBe('none');
    await zeroStub.leaveUnshorted(page, [1]);
    await calibrate(page);
    await expect(heading(page)).toBeFocused();
    expect(await outline()).toBe('none');
    await zeroStub.leaveUnshorted(page, []);
    await dialog(page).getByRole('button', { name: 'Continue' }).click();
    for (let channel = 2; channel <= 4; channel++) await calibrate(page);
    await expect(heading(page)).toContainText('Calibration completed');
    await expect(heading(page)).toBeFocused();
    expect(await outline()).toBe('none');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    const ring = await page.evaluate(() => {
      const style = getComputedStyle(document.activeElement as Element);
      return { tag: document.activeElement?.tagName, style: style.outlineStyle, width: style.outlineWidth };
    });
    expect(ring.style).not.toBe('none');
  });

  test('given the Continue and the accepted Done step, when scanned by axe, then there are no serious or critical violations', async ({
    page,
  }) => {
    await openMain(page);
    const scan = async () => {
      const result = await new AxeBuilder({ page }).include('dialog').analyze();
      const bad = result.violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
      expect(bad.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
    };
    await toNotShorted(page, 2);
    await scan();
    await zeroStub.leaveUnshorted(page, []);
    await dialog(page).getByRole('button', { name: 'Continue' }).click();
    await calibrate(page);
    await calibrate(page);
    await expect(heading(page)).toContainText('Calibration completed');
    await scan();
  });

  test('given the tooltip after Done with an accepted channel, when read, then it shows the stored offsets and no accepted note', async ({
    page,
  }) => {
    await openMain(page);
    await toNotShorted(page, 2);
    await zeroStub.leaveUnshorted(page, []);
    await dialog(page).getByRole('button', { name: 'Continue' }).click();
    await calibrate(page);
    await calibrate(page);
    await dialog(page).getByRole('button', { name: 'Done' }).click();
    await tag(page).focus();
    const tip = page.getByRole('tooltip');
    await expect(tip.locator('li').first()).toHaveText('CH1 +0.66 mV at 2 mV/div');
    await expect(tip).not.toContainText(/accepted/i);
  });
});

const connection = (patch: Parameters<typeof connectionSnapshot>[0], revision: number) =>
  connectionSnapshot({ revision, ...patch });

test.describe('connection item', () => {
  const item = (page: Page) => status(page).locator('[data-item="connection"]');

  test('given the simulator is ready, when shown, then it is a dot, the bold word Ready and the muted detail, with no chips', async ({
    page,
  }) => {
    await openMain(page);
    await expect(item(page).locator('b')).toHaveText('Ready');
    await expect(item(page).locator('.dot')).toHaveCount(1);
    await expect(item(page)).toContainText('Simulator');
    await expect(status(page).locator('.chip')).toHaveCount(0);
    const weight = await item(page)
      .locator('b')
      .evaluate((element) => getComputedStyle(element).fontWeight);
    expect(Number(weight)).toBeGreaterThanOrEqual(600);
  });

  test('given a real scope is ready, when shown, then the detail has the model and no Simulator word', async ({
    page,
  }) => {
    await openMain(page);
    await emit(
      page,
      'connection_changed',
      connection(
        {
          source: 'hardware',
          simulatorActive: false,
          identity: {
            model: 'Hantek 6254BC',
            firmware: '1.2',
            fpgaVersion: '3',
            calibration: 'valid',
            simulated: false,
          },
        },
        2,
      ),
    );
    await expect(item(page)).toContainText('Hantek 6254BC');
    await expect(item(page)).not.toContainText('Simulator');
  });

  const states: [string, Parameters<typeof connectionSnapshot>[0], string][] = [
    ['Connecting', { state: 'connecting', identity: null }, 'Connecting'],
    ['Disconnected', { state: 'disconnected', identity: null, source: null, simulatorActive: false }, 'Disconnected'],
    [
      'Error',
      {
        state: 'error',
        identity: null,
        issue: { kind: 'permissionDenied', message: 'No permission to open the scope' } as never,
      },
      'Error',
    ],
  ];
  for (const [name, patch, word] of states) {
    test(`given the connection is ${name}, when shown, then the bold word is ${word}${name === 'Error' ? ' with a warning icon instead of a dot' : ' with a dot'}`, async ({
      page,
    }) => {
      await openMain(page);
      await emit(page, 'connection_changed', connection(patch, 3));
      await expect(item(page).locator('b')).toHaveText(word);
      await expect(item(page).locator('.dot')).toHaveCount(name === 'Error' ? 0 : 1);
      await expect(item(page).locator('svg')).toHaveCount(name === 'Error' ? 1 : 0);
    });
  }

  test('given no device, when the slice arrives, then the status bar has no Accuracy item', async ({ page }) => {
    await openMain(page);
    await expect(status(page).getByText(/accuracy/i)).toHaveCount(0);
  });

  test('given a 1024 px window at 200 percent text, when shown, then the connection item and the tag are both visible', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1024, height: 720 });
    await openMain(page, {}, 200);
    await expect(item(page)).toBeVisible();
    await expect(tag(page)).toBeVisible();
    await expect(status(page).locator('[data-item="renderer"]')).toHaveCount(0);
  });
});

test.describe('no verification markers', () => {
  const MARKER = /unverified|partially verified|not verified|verification/i;

  test('given the main window when read, then no text, label, title or class names a verification state', async ({
    page,
  }) => {
    await openMain(page);
    const hits = await page.evaluate((source) => {
      const pattern = new RegExp(source, 'i');
      const found: string[] = [];
      for (const element of document.querySelectorAll('*')) {
        const own = [...element.childNodes]
          .filter((n) => n.nodeType === 3)
          .map((n) => n.textContent)
          .join(' ');
        const attrs = ['aria-label', 'title', 'class', 'data-kind', 'alt'].map((a) => element.getAttribute(a) ?? '');
        if (pattern.test([own, ...attrs].join(' '))) found.push(element.tagName + ' ' + [own, ...attrs].join('|'));
      }
      return found;
    }, MARKER.source);
    expect(hits).toEqual([]);
    expect(await page.locator('.badge:has-text("?")').count()).toBe(0);
  });

  test('given every dock select and menu, when opened, then no item carries a marker', async ({ page }) => {
    await openMain(page);
    const triggers = page.locator('[aria-haspopup="menu"], [aria-haspopup="listbox"]');
    const total = await triggers.count();
    expect(total).toBeGreaterThan(0);
    for (let index = 0; index < total; index++) {
      const trigger = triggers.nth(index);
      if (!(await trigger.isVisible()) || !(await trigger.isEnabled())) continue;
      await trigger.click();
      const popup = page.locator('[role="menu"], [role="listbox"]').first();
      if (await popup.count()) {
        expect(await popup.innerText()).not.toMatch(MARKER);
        expect(await popup.locator('.badge').count()).toBe(0);
      }
      await page.keyboard.press('Escape');
    }
  });

  test('given the Exp badge on the Video trigger, when read, then the tooltip does not say verified', async ({
    page,
  }) => {
    await openMain(page);
    const text = await page.locator('body').innerHTML();
    expect(text).not.toMatch(/not verified on this hardware/i);
  });

  test('given the gallery, when read, then no marker demo is on any page', async ({ page }) => {
    await openGallery(page);
    expect(await page.locator('body').innerText()).not.toMatch(MARKER);
  });
});
