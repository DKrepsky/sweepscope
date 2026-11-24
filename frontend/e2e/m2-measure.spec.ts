import { expect, test, type Page } from '@playwright/test';
import { connectionSnapshot, instrumentState, measurementsState } from '../src/app/instrument/test-data';
import type { Measurement } from '../src/api/generated/Measurement';
import type { MeasurementId } from '../src/api/generated/MeasurementId';
import { openMain as open } from './support/app';
import { sine, wireFrame } from './support/frames';
import { emit, ipcLog, sendFrame, type StubOptions } from './support/tauri-stub';

/** M2 (#13 to #20) against the IPC stub: cursors, measurements, File menu, shortcuts and toasts. */

async function openMain(page: Page, options: StubOptions = {}): Promise<void> {
  await open(page, 100, options);
  await expect.poll(async () => (await ipcLog(page)).some((call) => call.command === 'open_frame_stream')).toBe(true);
}

async function liveFrame(page: Page): Promise<void> {
  await sendFrame(page, wireFrame({ mask: 0b0001, count: 2501, value: sine }));
  await expect(page.locator('.graph')).toHaveAttribute('data-state', 'live');
}

const IDS: MeasurementId[] = [
  'frequency',
  'period',
  'vpp',
  'vmax',
  'vmin',
  'vmean',
  'vrms',
  'positiveWidth',
  'negativeWidth',
  'dutyCycle',
  'riseTime',
  'fallTime',
];

const LABELS: Record<MeasurementId, string> = {
  frequency: 'Frequency',
  period: 'Period',
  vpp: 'Vpp',
  vmax: 'Vmax',
  vmin: 'Vmin',
  vmean: 'Vmean',
  vrms: 'Vrms',
  positiveWidth: '+Width',
  negativeWidth: '-Width',
  dutyCycle: 'Duty',
  riseTime: 'Rise time',
  fallTime: 'Fall time',
};

const value = (id: MeasurementId, text: string, unit: string): Measurement => ({
  id,
  label: LABELS[id],
  value: { value: 1, text, unit },
  invalid: null,
});

const invalid = (id: MeasurementId, text: string): Measurement => ({
  id,
  label: LABELS[id],
  value: null,
  invalid: { reason: 'fewerThanOnePeriod', text },
});

async function sendMeasurements(page: Page, patch: { unverified?: string | null; clipped?: boolean } = {}) {
  const measurements = IDS.map((id) => {
    if (id === 'frequency') return value(id, '1.000', 'kHz');
    if (id === 'period') return value(id, '1.000', 'ms');
    if (id === 'vpp') return value(id, '1.91', 'V');
    if (id === 'positiveWidth') return value(id, '500', 'us');
    return invalid(id, 'Fewer than one period');
  });
  const state = measurementsState({
    revision: 5,
    accuracyNote: 'Voltage readings are accurate to about 2 to 3 counts.',
    channels: [{ channel: 1, clipped: patch.clipped ?? false, unverified: patch.unverified ?? null, measurements }],
  });
  await emit(page, 'measurements_changed', state);
}

const toast = (page: Page) =>
  page.locator('[role="status"], [role="alert"]').filter({ hasText: /Saved|Loaded|could not|Permission|denied/ });

async function chooseFile(page: Page, item: RegExp): Promise<void> {
  await page.getByRole('button', { name: /^File/ }).click();
  await page.getByRole('menuitem', { name: item }).click();
}

const callsOf = async (page: Page, command: string) => (await ipcLog(page)).filter((call) => call.command === command);

test.describe('M2 cursors (#14)', () => {
  test('given a live frame, when C is pressed, then two vertical and two horizontal cursors appear and C removes them', async ({
    page,
  }) => {
    await openMain(page);
    await liveFrame(page);
    await page.keyboard.press('c');
    await expect(page.getByRole('slider', { name: /Cursor/ })).toHaveCount(4);
    await expect(page.getByRole('slider', { name: 'Cursor X1' })).toHaveAttribute('aria-orientation', 'horizontal');
    await expect(page.getByRole('slider', { name: 'Cursor Y1' })).toHaveAttribute('aria-orientation', 'vertical');
    await page.keyboard.press('c');
    await expect(page.getByRole('slider', { name: /Cursor/ })).toHaveCount(0);
  });

  test('given the readout, when cursors are added, then X1, X2, ΔX, 1/ΔX, Y1, Y2 and ΔY show with units', async ({
    page,
  }) => {
    await openMain(page);
    await liveFrame(page);
    await page.keyboard.press('c');
    const panel = page.locator('aside, [data-region="sidebar"]').first();
    for (const label of ['X1', 'X2', 'ΔX', '1/ΔX', 'Y1', 'Y2', 'ΔY']) await expect(panel).toContainText(label);
    await expect(panel).toContainText('ΔX');
    await expect(panel).toContainText(/\d+(\.\d+)? (ks|s|ms|us|ns)/);
    await expect(panel).toContainText(/\d+(\.\d+)? (V|mV)/);
    await expect(panel).toContainText(/\d+(\.\d+)? (mHz|Hz|kHz)/);
    await expect(panel).toContainText('left edge of the screen, not the trigger');
  });

  test('given a focused cursor, when arrows, Shift and Delete are pressed, then it moves one step, ten steps and is removed', async ({
    page,
  }) => {
    await openMain(page);
    await liveFrame(page);
    await page.keyboard.press('c');
    const x1 = page.getByRole('slider', { name: 'Cursor X1' });
    const start = Number(await x1.getAttribute('aria-valuenow'));
    await x1.focus();
    await page.keyboard.press('ArrowRight');
    const one = Number(await x1.getAttribute('aria-valuenow'));
    expect(one - start).toBeCloseTo(1 / 250, 4);
    await page.keyboard.press('Shift+ArrowRight');
    expect(Number(await x1.getAttribute('aria-valuenow')) - one).toBeCloseTo(10 / 250, 4);
    await page.keyboard.press('Delete');
    await expect(page.getByRole('slider', { name: /Cursor/ })).toHaveCount(3);
  });

  test('given a cursor, when it is dragged with the mouse, then the position and the spoken value follow', async ({
    page,
  }) => {
    await openMain(page);
    await liveFrame(page);
    await page.keyboard.press('c');
    const x2 = page.getByRole('slider', { name: 'Cursor X2' });
    const before = await x2.getAttribute('aria-valuetext');
    const box = (await x2.boundingBox())!;
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2 - 120, box.y + box.height / 2, { steps: 4 });
    await page.mouse.up();
    expect(Number(await x2.getAttribute('aria-valuenow'))).toBeLessThan(0.7);
    expect(await x2.getAttribute('aria-valuetext')).not.toBe(before);
  });

  test('given the empty screen, when C is pressed, then no cursor is added', async ({ page }) => {
    test.fail(true, 'BUG-12: toggleCursors has no empty-graph guard, so C adds cursors on an empty screen');
    await openMain(page);
    await page.keyboard.press('c');
    await expect(page.getByRole('slider', { name: /Cursor/ })).toHaveCount(0);
  });

  test('given the empty screen, when the Cursors tool is read, then it is disabled with the reason No waveform', async ({
    page,
  }) => {
    test.fail(true, 'BUG-11: the toolbar Cursors tool has no handler and no disabled state or reason');
    await openMain(page);
    const tool = page.getByRole('toolbar').getByRole('button', { name: /^Cursors/ });
    await expect(tool).toHaveAttribute('aria-disabled', 'true');
  });

  test('given a live frame, when the toolbar Cursors tool is clicked, then the cursors appear', async ({ page }) => {
    test.fail(true, 'BUG-11: the toolbar Cursors tool has no click handler');
    await openMain(page);
    await liveFrame(page);
    await page
      .getByRole('toolbar')
      .getByRole('button', { name: /^Cursors/ })
      .click();
    await expect(page.getByRole('slider', { name: /Cursor/ })).toHaveCount(4);
  });

  test('given frozen data after a disconnect, when C is pressed, then cursors still work', async ({ page }) => {
    await openMain(page);
    await liveFrame(page);
    await emit(
      page,
      'connection_changed',
      connectionSnapshot({ revision: 9, state: 'disconnected', source: null, identity: null, simulatorActive: false }),
    );
    await emit(page, 'instrument_changed', instrumentState({ revision: 9, running: false }));
    await expect(page.locator('.graph')).toHaveAttribute('data-state', 'frozen');
    await page.keyboard.press('c');
    await expect(page.getByRole('slider', { name: /Cursor/ })).toHaveCount(4);
  });
});

test.describe('M2 measurements (#15)', () => {
  test('given measurement events, when shown, then pinned rows read in engineering notation with units', async ({
    page,
  }) => {
    await openMain(page);
    await liveFrame(page);
    await sendMeasurements(page);
    const panel = page.locator('aside, [data-region="sidebar"]').first();
    await expect(panel).toContainText('Frequency');
    await expect(panel).toContainText('1.000');
    await expect(panel).toContainText('kHz');
    await expect(panel).toContainText('Voltage readings are accurate to about 2 to 3 counts.');
  });

  test('given a cell that does not fit, when shown, then it reads - with a footnote reason', async ({ page }) => {
    await openMain(page);
    await liveFrame(page);
    await sendMeasurements(page);
    const panel = page.locator('aside, [data-region="sidebar"]').first();
    await page.getByRole('button', { name: 'Edit measurements' }).click();
    await page.getByRole('menuitemcheckbox', { name: /Vmax/ }).click();
    await expect(panel).toContainText('Fewer than one period');
  });

  test('given the pin menu, when opened, then the twelve quantities are offered and all menu labels are inside the window', async ({
    page,
  }) => {
    test.fail(true, 'BUG-13: the pin menu opens at the button and runs off the right edge of the window');
    await openMain(page);
    await liveFrame(page);
    await sendMeasurements(page);
    await page.getByRole('button', { name: 'Edit measurements' }).click();
    await expect(page.getByRole('menuitemcheckbox')).toHaveCount(12);
    const viewport = page.viewportSize()!;
    for (const item of await page.getByRole('menuitemcheckbox').all()) {
      const box = (await item.boundingBox())!;
      expect(box.x + box.width, 'menu item right edge').toBeLessThanOrEqual(viewport.width);
    }
  });

  test('given an unverified range, when shown, then the value carries the ? marker', async ({ page }) => {
    await openMain(page);
    await liveFrame(page);
    await sendMeasurements(page, { unverified: 'Range above 1 V/div is unverified' });
    const panel = page.locator('aside, [data-region="sidebar"]').first();
    await expect(panel.getByText('?').first()).toBeVisible();
  });

  test('given a pinned subset, when the window restarts, then the choice is restored', async ({ page }) => {
    await openMain(page);
    await liveFrame(page);
    await sendMeasurements(page);
    await page.getByRole('button', { name: 'Edit measurements' }).click();
    await page.getByRole('menuitemcheckbox', { name: /Vpp/ }).click();
    await page.keyboard.press('Escape');
    const saved = await callsOf(page, 'save_ui_state');
    expect(JSON.stringify(saved)).toContain('measurements');
    await page.reload();
    await expect.poll(async () => (await ipcLog(page)).some((call) => call.command === 'open_frame_stream')).toBe(true);
    await liveFrame(page);
    await sendMeasurements(page);
    await expect(page.getByText('Vpp', { exact: true })).toHaveCount(0);
  });
});

test.describe('M2 interpolation (#13)', () => {
  test('given the Horizontal group, when Dots is chosen, then set_interpolation is sent once with dots', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: 'Dots', exact: true }).click();
    const calls = await callsOf(page, 'set_interpolation');
    expect(calls).toHaveLength(1);
    expect(JSON.stringify(calls[0]?.args)).toContain('dots');
  });

  test('given a display event with sin(x)/x automatic, when it arrives, then sin(x)/x is selected', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'display_changed', { revision: 4, interpolation: 'sinc', automatic: true });
    await expect(page.getByRole('button', { name: /sin\(x\)\/x/ })).toHaveAttribute('aria-pressed', 'true');
  });
});

test.describe('M2 files (#18 #19 #20)', () => {
  test('given a live frame, when Export CSV is chosen, then export_frame csv is sent and a Saved toast names the file only', async ({
    page,
  }) => {
    await openMain(page, { commands: { export_frame: { result: { fileName: 'cap.csv' } } } });
    await liveFrame(page);
    await chooseFile(page, /Export CSV/);
    const calls = await callsOf(page, 'export_frame');
    expect(JSON.stringify(calls[0]?.args)).toContain('csv');
    await expect(toast(page)).toContainText('Saved cap.csv');
  });

  test('given a live frame, when Export raw capture is chosen, then export_frame raw is sent', async ({ page }) => {
    await openMain(page, { commands: { export_frame: { result: { fileName: 'cap.bin' } } } });
    await liveFrame(page);
    await chooseFile(page, /Export raw/);
    expect(JSON.stringify((await callsOf(page, 'export_frame'))[0]?.args)).toContain('raw');
    await expect(toast(page)).toContainText('Saved cap.bin');
  });

  test('given a cancelled dialog, when the export returns nothing, then no toast shows', async ({ page }) => {
    await openMain(page, { commands: { export_frame: { result: null } } });
    await liveFrame(page);
    await chooseFile(page, /Export CSV/);
    await expect.poll(async () => (await callsOf(page, 'export_frame')).length).toBe(1);
    await page.waitForTimeout(300);
    await expect(toast(page)).toHaveCount(0);
  });

  test('given a failing export, when it rejects, then an error toast shows the cause', async ({ page }) => {
    await openMain(page, { commands: { export_frame: { error: 'Permission denied: the folder is read-only' } } });
    await liveFrame(page);
    await chooseFile(page, /Export CSV/);
    await expect(page.getByRole('alert')).toContainText('Permission denied');
  });

  test('given the empty screen, when the File menu opens, then export and screenshot are disabled with No waveform and setups stay enabled', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: /^File/ }).click();
    for (const name of [/Export CSV/, /Export raw/, /Save screenshot/]) {
      await expect(page.getByRole('menuitem', { name })).toHaveAttribute('aria-disabled', 'true');
    }
    await expect(page.getByRole('menuitem', { name: /Save setup/ })).not.toHaveAttribute('aria-disabled', 'true');
    await expect(page.getByRole('menuitem', { name: /Load setup/ })).not.toHaveAttribute('aria-disabled', 'true');
    await page.getByRole('menuitem', { name: /Export CSV/ }).hover();
    await expect(page.getByText('No waveform').first()).toBeVisible();
  });

  test('given a live frame, when the screenshot button is used, then save_screenshot gets PNG bytes', async ({
    page,
  }) => {
    await openMain(page, { commands: { save_screenshot: { result: { fileName: 'shot.png' } } } });
    await liveFrame(page);
    await page.getByRole('button', { name: 'Screenshot' }).click();
    await expect(toast(page)).toContainText('Saved shot.png');
    const calls = await callsOf(page, 'save_screenshot');
    expect(calls).toHaveLength(1);
    const signature = await page.evaluate(
      () =>
        (window as unknown as { __ipcLog: { command: string; args: ArrayLike<number> }[] }).__ipcLog.find(
          (c) => c.command === 'save_screenshot',
        )!.args,
    );
    expect(Array.from(signature as ArrayLike<number>).slice(1, 4)).toEqual([0x50, 0x4e, 0x47].slice(0, 3));
  });

  test('given a failing screenshot save, when it rejects, then an error toast names the cause', async ({ page }) => {
    await openMain(page, { commands: { save_screenshot: { error: 'No space left on the device' } } });
    await liveFrame(page);
    await page.keyboard.press('Control+p');
    await expect(page.getByRole('alert')).toContainText('No space left');
  });

  test('given the shortcuts, when Ctrl+S, Ctrl+E and Ctrl+P are pressed, then the setup, CSV and screenshot commands run', async ({
    page,
  }) => {
    await openMain(page, {
      commands: {
        save_setup: { result: { fileName: 's.json' } },
        export_frame: { result: { fileName: 'c.csv' } },
        save_screenshot: { result: { fileName: 'p.png' } },
      },
    });
    await liveFrame(page);
    await page.keyboard.press('Control+s');
    await page.keyboard.press('Control+e');
    await page.keyboard.press('Control+p');
    await expect.poll(async () => (await callsOf(page, 'save_setup')).length).toBe(1);
    await expect.poll(async () => (await callsOf(page, 'export_frame')).length).toBe(1);
    await expect.poll(async () => (await callsOf(page, 'save_screenshot')).length).toBe(1);
  });

  test('given a setup with warnings, when loaded, then one warning toast lists them', async ({ page }) => {
    await openMain(page, {
      commands: {
        load_setup: {
          result: { fileName: 'a.json', warnings: ['CH1 volts/div 3.00 V is not available; kept 1.00 V'] },
        },
      },
    });
    await chooseFile(page, /Load setup/);
    await expect(toast(page)).toContainText('3.00 V is not available');
    await expect(toast(page)).toHaveCount(1);
  });

  test('given a corrupt setup file, when loaded, then an error toast gives the reason', async ({ page }) => {
    await openMain(page, {
      commands: { load_setup: { error: 'The file is not valid JSON, so it is not a setup file' } },
    });
    await chooseFile(page, /Load setup/);
    await expect(page.getByRole('alert')).toContainText('not valid JSON');
  });

  test('given a text field with focus, when shortcut letters are typed, then nothing runs', async ({ page }) => {
    await openMain(page);
    await liveFrame(page);
    const field = page.locator('input').first();
    await field.focus();
    await page.keyboard.press('c');
    await page.keyboard.press('Control+s');
    await expect(page.getByRole('slider', { name: /Cursor/ })).toHaveCount(0);
    expect(await callsOf(page, 'save_setup')).toHaveLength(0);
  });

  test('given the File menu, when it opens with the keyboard, then it is navigable and Escape closes it', async ({
    page,
  }) => {
    await openMain(page);
    await page.getByRole('button', { name: /^File/ }).focus();
    await page.keyboard.press('Enter');
    await expect(page.getByRole('menu')).toBeVisible();
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('Escape');
    await expect(page.getByRole('menu')).toHaveCount(0);
  });

  test('given the stub snapshot, when the window opens, then it shows no console errors from the new commands', async ({
    page,
  }) => {
    const errors: string[] = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await openMain(page);
    await liveFrame(page);
    await sendMeasurements(page);
    await page.keyboard.press('c');
    expect(errors).toEqual([]);
    expect(instrumentState().channels.length).toBeGreaterThan(0);
  });
});
