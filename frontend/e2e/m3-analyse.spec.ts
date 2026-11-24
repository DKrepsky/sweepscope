import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { acquisitionStatus, instrumentState, measurementsState } from '../src/app/instrument/test-data';
import type { Measurement } from '../src/api/generated/Measurement';
import type { MeasurementId } from '../src/api/generated/MeasurementId';
import { openMain as open } from './support/app';
import { sine, wireFrame } from './support/frames';
import { emit, ipcLog, sendFrame, type StubOptions } from './support/tauri-stub';

/**
 * M3 Analyse (#22 to #31) against the IPC stub, written by QA. Cases marked BUG-n are known defects of the product as
 * wired: they use `test.fail` so the suite stays green and turns red when the defect is fixed (then remove the annotation).
 * Numbering: BUG-21 = BUG-M3-1 (inert analysis wiring), 22 cursors in roll, 23 toolbar overflow, 24 width error, 25 SW badge contrast, 26 No signal tag.
 * Test plan: docs/specs/m3-analyse/test-plan.md.
 */

const ANALYSE_COMMANDS = [
  'arm_single',
  'start_autoset',
  'set_math',
  'set_persistence',
  'set_spectrum',
  'start_recording',
  'stop_recording',
  'open_roll_stream',
  'close_roll_stream',
  'open_spectrum_stream',
  'close_spectrum_stream',
];

const scripted = (extra: StubOptions['commands'] = {}): StubOptions => ({
  commands: { ...Object.fromEntries(ANALYSE_COMMANDS.map((name) => [name, { result: null }])), ...extra },
});

async function openMain(page: Page, options: StubOptions = scripted()): Promise<void> {
  await open(page, 100, options);
  await expect.poll(async () => (await ipcLog(page)).some((call) => call.command === 'open_frame_stream')).toBe(true);
}

const calls = async (page: Page, name: string) => (await ipcLog(page)).filter((call) => call.command === name);
const toolbar = (page: Page) => page.locator('[data-region="toolbar"]');
const button = (page: Page, name: string | RegExp) =>
  page.getByRole('button', typeof name === 'string' ? { name, exact: true } : { name });
const status = (page: Page) => page.locator('[data-region="status"]');

const ROLL_REASON = 'Trigger is not available in roll mode (500 ms/div and slower). Use a faster time base to trigger.';
const FRAME_VIEWS_REASON = 'XY, FFT, persistence, math and cursors need a captured record; not available in roll mode.';

const rollState = () => {
  const base = instrumentState({ revision: 20 });
  const off = (reason: string) => ({ enabled: false, reason });
  return instrumentState({
    revision: 20,
    horizontal: { ...base.horizontal, roll: true },
    controls: {
      ...base.controls,
      force: off(ROLL_REASON),
      trigger: off(ROLL_REASON),
      single: off('Single needs a captured record; not available in roll mode.'),
      autoset: off('Auto-setup needs a captured record; not available in roll mode.'),
      frameViews: off(FRAME_VIEWS_REASON),
    },
  });
};

const IDS: MeasurementId[] = ['frequency', 'period', 'vpp'];
const row = (id: MeasurementId, label: string, text: string, unit: string): Measurement => ({
  id,
  label,
  value: { value: 1, text, unit },
  invalid: null,
});

async function emitAlias(page: Page, revision: number, on: boolean): Promise<void> {
  const measurements = IDS.map((id) => row(id, id, '1.000', 'kHz'));
  await emit(
    page,
    'measurements_changed',
    measurementsState({
      revision,
      channels: [{ channel: 1, clipped: false, unverified: null, measurements }],
      warnings: on
        ? [
            {
              id: 'alias-ch1',
              severity: 'warning',
              channel: 1,
              short: 'Alias CH1',
              message:
                'Possible aliasing on CH1. Its frequency (1.000 kHz) is more than one fifth of the sample rate (2.50 kS/s), so the display may show a wrong shape or frequency.',
            },
          ]
        : [],
    }),
  );
}

test.describe('#23 aliasing warning', () => {
  test('given an alias warning from the backend, when it arrives, then the status bar, scale chip and measurement header show it', async ({
    page,
  }) => {
    await openMain(page);
    await sendFrame(page, wireFrame({ mask: 1, count: 2501, value: sine }));
    await emitAlias(page, 5, true);
    await expect(status(page)).toContainText('Alias CH1');
    await expect(page.locator('[data-region="scale-strip"] .badge', { hasText: 'Alias' })).toHaveCount(1);
    await expect(page.locator('[data-region="sidebar"] .badge', { hasText: 'Alias' })).toHaveCount(1);
    await expect(page.getByText(/Possible aliasing on CH1/).first()).toBeAttached();
  });

  test('given the warning is shown, when the backend clears it, then every place clears together', async ({ page }) => {
    await openMain(page);
    await emitAlias(page, 5, true);
    await expect(status(page)).toContainText('Alias CH1');
    await emitAlias(page, 6, false);
    await expect(status(page)).not.toContainText('Alias');
    await expect(page.locator('.badge', { hasText: 'Alias' })).toHaveCount(0);
  });

  test('given the warning, when the status bar item is read, then its tooltip is the plain-language backend message', async ({
    page,
  }) => {
    await openMain(page);
    await emitAlias(page, 5, true);
    await expect(status(page).locator('[title*="Possible aliasing on CH1"]')).toHaveCount(1);
  });
});

test.describe('#22 auto-setup states', () => {
  test('given Auto-setup running, when the status says so, then the button is busy, Run is disabled with the reason and no dialog opens', async ({
    page,
  }) => {
    await openMain(page);
    const base = instrumentState({ revision: 30 });
    const reason = { enabled: false, reason: 'Auto-setup is running' };
    await emit(page, 'instrument_changed', { ...base, controls: { ...base.controls, run: reason, single: reason } });
    await emit(
      page,
      'status_changed',
      acquisitionStatus({ revision: 30, phase: 'auto', autoset: { running: true, noSignal: false, outcome: null } }),
    );
    await expect(button(page, /Autoset/)).toHaveAttribute('aria-busy', 'true');
    await expect(button(page, 'Single')).toHaveAttribute('aria-disabled', 'true');
    await expect(page.locator('dialog[open], [role="dialog"], [role="alertdialog"]')).toHaveCount(0);
  });

  test('given a done outcome, when it arrives twice with the same id, then exactly one Toast shows the backend text', async ({
    page,
  }) => {
    await openMain(page);
    const outcome = { id: 1, kind: 'done' as const, text: 'Auto-setup done. CH1 500 mV/div, 200 µs/div.' };
    const status2 = acquisitionStatus({
      revision: 31,
      phase: 'triggered',
      autoset: { running: false, noSignal: false, outcome },
    });
    await emit(page, 'status_changed', status2);
    await emit(page, 'status_changed', { ...status2, revision: 32 });
    await expect(page.getByText('Auto-setup done. CH1 500 mV/div, 200 µs/div.')).toHaveCount(1);
  });

  test('given no signal, when the backend reports it, then the No signal tag shows until the backend clears it', async ({
    page,
  }) => {
    await openMain(page);
    const outcome = { id: 2, kind: 'noSignal' as const, text: 'Auto-setup: no signal on any enabled channel.' };
    await emit(
      page,
      'status_changed',
      acquisitionStatus({ revision: 40, phase: 'auto', autoset: { running: false, noSignal: true, outcome } }),
    );
    await expect(page.getByText('Auto-setup: no signal on any enabled channel.')).toBeVisible();
    await expect(page.getByText('No signal', { exact: true })).toBeVisible();
    await emit(
      page,
      'status_changed',
      acquisitionStatus({ revision: 41, phase: 'auto', autoset: { running: false, noSignal: false, outcome } }),
    );
    await expect(page.getByText('No signal', { exact: true })).toHaveCount(0);
  });

  test('given a failed outcome, when the Toast shows, then it keeps the previous-setup wording and offers Retry', async ({
    page,
  }) => {
    await openMain(page);
    const outcome = {
      id: 3,
      kind: 'failed' as const,
      text: 'Auto-setup could not find a stable setting. Previous setup kept.',
    };
    await emit(
      page,
      'status_changed',
      acquisitionStatus({ revision: 50, phase: 'auto', autoset: { running: false, noSignal: false, outcome } }),
    );
    await expect(page.getByText(outcome.text)).toBeVisible();
    await expect(button(page, 'Retry')).toBeVisible();
  });

  test('given the Autoset button, when it is pressed, then start_autoset is sent', async ({ page }) => {
    test.fail(true, 'BUG-21: the live app wires inert analysis commands, nothing is sent');
    await openMain(page);
    await button(page, /Autoset/).click();
    await expect.poll(async () => (await calls(page, 'start_autoset')).length).toBe(1);
  });

  test('given the A key, when pressed on the page, then start_autoset is sent', async ({ page }) => {
    test.fail(true, 'BUG-21: the live app wires inert analysis commands, nothing is sent');
    await openMain(page);
    await page.locator('[data-testid="main-window"]').click({ position: { x: 5, y: 5 } });
    await page.keyboard.press('a');
    await expect.poll(async () => (await calls(page, 'start_autoset')).length).toBe(1);
  });
});

test.describe('#24 single shot states', () => {
  test('given the backend phase waiting for trigger, when shown, then the chip reads Waiting for trigger and Single is pressed', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', instrumentState({ revision: 60, running: true }));
    await emit(page, 'status_changed', acquisitionStatus({ revision: 60, phase: 'waitingForTrigger' }));
    await expect(toolbar(page)).toContainText(/Waiting for trigger/i);
    await expect(button(page, 'Single')).toHaveAttribute('aria-pressed', 'true');
  });

  test('given the backend phase done, when shown, then the chips read Stop and Done and Single is not pressed', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', instrumentState({ revision: 61, running: false }));
    await emit(page, 'status_changed', acquisitionStatus({ revision: 61, phase: 'done' }));
    await expect(toolbar(page)).toContainText(/Done/i);
    await expect(button(page, 'Single')).toHaveAttribute('aria-pressed', 'false');
  });

  test('given the Single button, when pressed, then arm_single is sent', async ({ page }) => {
    test.fail(true, 'BUG-21: the live app wires inert analysis commands, nothing is sent');
    await openMain(page);
    await button(page, 'Single').click();
    await expect.poll(async () => (await calls(page, 'arm_single')).length).toBe(1);
  });
});

test.describe('#25 pulse trigger', () => {
  const pulseState = () => {
    const base = instrumentState({ revision: 70 });
    return {
      ...base,
      trigger: {
        ...base.trigger,
        kind: 'pulse' as const,
        pulse: { condition: 'greater' as const, width: { value: 5e-4, text: '500', unit: 'us' }, verified: true },
      },
    };
  };

  test('given the Pulse type, when selected, then set_trigger carries kind pulse', async ({ page }) => {
    await openMain(page);
    await button(page, 'Pulse').click();
    const sent = await calls(page, 'set_trigger');
    expect(sent.at(-1)?.args).toEqual({ patch: { kind: 'pulse' } });
  });

  test('given pulse state, when the Trigger group is read, then polarity, the four conditions and the width are offered', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', pulseState());
    for (const name of [
      'Positive pulse',
      'Negative pulse',
      /Longer than/,
      /Shorter than/,
      /Equal to/,
      /Not equal to/,
    ]) {
      await expect(button(page, name)).toBeVisible();
    }
    await expect(page.getByRole('textbox', { name: 'Pulse width' })).toHaveValue('500');
  });

  test('given the width field, when 500 us is typed, then one set_trigger carries widthText', async ({ page }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', pulseState());
    const field = page.getByRole('textbox', { name: 'Pulse width' });
    await field.fill('250 us');
    await field.press('Enter');
    const sent = await calls(page, 'set_trigger');
    expect(sent.at(-1)?.args).toEqual({ patch: { widthText: '250 us' } });
  });

  test('given a rejected width, when abc is entered, then the field is invalid, shows the error and Escape restores the applied value', async ({
    page,
  }) => {
    const text = 'Invalid width. Enter a time such as 500 µs. Applied: 500 µs.';
    await openMain(page, scripted({ set_trigger: { error: text } }));
    await emit(page, 'instrument_changed', pulseState());
    const field = page.getByRole('textbox', { name: 'Pulse width' });
    await field.fill('abc');
    await field.press('Enter');
    await expect(field).toHaveAttribute('aria-invalid', 'true');
    await field.press('Escape');
    await expect(field).toHaveValue('500');
    await expect(field).not.toHaveAttribute('aria-invalid', 'true');
  });

  test('given the Trigger group, when the Video type is read, then it is disabled and labelled experimental', async ({
    page,
  }) => {
    await openMain(page);
    await expect(button(page, 'Video')).toHaveAttribute('aria-disabled', 'true');
    await expect(page.locator('.badge', { hasText: /^\s*exp\b/i }).first()).toBeVisible();
  });

  test('given a rejected width, when the error shows, then the text appears once and stays inside the dock', async ({
    page,
  }) => {
    test.fail(true, 'BUG-24: the width error is drawn twice and one copy overflows the dock into the sidebar');
    const text = 'Invalid width. Enter a time such as 500 µs. Applied: 500 µs.';
    await openMain(page, scripted({ set_trigger: { error: text } }));
    await emit(page, 'instrument_changed', pulseState());
    const field = page.getByRole('textbox', { name: 'Pulse width' });
    await field.fill('abc');
    await field.press('Enter');
    const copies = page.getByText(text);
    await expect(copies.first()).toBeVisible();
    expect(await copies.count()).toBe(1);
  });
});

test.describe('#27 roll mode availability', () => {
  test('given a roll setup, when the controls are read, then trigger, Single, Autoset, XY, FFT, Persist and Math are disabled and the trigger reason is visible text', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', rollState());
    for (const name of ['Single', 'Force', /Autoset/, 'XY', /FFT/, /Persist/, 'Math', 'Edge', 'Pulse']) {
      await expect(button(page, name)).toHaveAttribute('aria-disabled', 'true');
    }
    await expect(page.getByText(ROLL_REASON).first()).toBeVisible();
  });

  test('given a roll setup, when the Record button is read, then it is enabled', async ({ page }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', rollState());
    await expect(button(page, 'Record')).not.toHaveAttribute('aria-disabled', 'true');
  });

  test('given a roll setup, when the page is open, then the roll stream is opened', async ({ page }) => {
    test.fail(true, 'BUG-21: createRollFeed is never wired in main.ts');
    await openMain(page);
    await emit(page, 'instrument_changed', rollState());
    await expect.poll(async () => (await calls(page, 'open_roll_stream')).length).toBeGreaterThan(0);
  });

  test('given a roll setup with a frame on screen, when the toolbar is read, then Cursors is disabled', async ({
    page,
  }) => {
    await openMain(page);
    await sendFrame(page, wireFrame({ mask: 1, count: 2501, value: sine }));
    await emit(page, 'instrument_changed', rollState());
    await expect(button(page, 'Cursors')).toHaveAttribute('aria-disabled', 'true');
  });

  for (const width of [1024, 1280, 1440]) {
    for (const mode of ['roll', 'yt'] as const) {
      test(`given ${mode} mode at ${width} px, when the toolbar is measured, then it does not overflow and keeps File and Screenshot visible`, async ({
        page,
      }) => {
        await page.setViewportSize({ width, height: 900 });
        await openMain(page);
        if (mode === 'roll') await emit(page, 'instrument_changed', rollState());
        await emit(page, 'status_changed', acquisitionStatus({ revision: 80, phase: 'auto' }));
        const overflow = await toolbar(page).evaluate((el) => el.scrollWidth - el.clientWidth);
        expect(overflow).toBeLessThanOrEqual(0);
        for (const name of [/^File/, 'Screenshot', 'Settings']) {
          const box = await button(page, name).boundingBox();
          expect(box && box.x + box.width).toBeLessThanOrEqual(width);
        }
      });
    }
  }
});

test.describe('#31 XY view', () => {
  const XY_ONE_CHANNEL = () => {
    const base = instrumentState({ revision: 90 });
    return { ...base, channels: base.channels.map((c) => ({ ...c, enabled: c.number === 1 })) };
  };

  test('given two channels and a frame, when XY is selected, then the screen is described as XY with both axes named', async ({
    page,
  }) => {
    await openMain(page);
    await sendFrame(page, wireFrame({ mask: 0b0011, count: 2501, value: sine }));
    await button(page, 'XY').click();
    await expect(page.getByRole('img', { name: /XY display/ })).toBeVisible();
    await expect(page.getByText(/^X CH1/).first()).toBeVisible();
    await expect(page.getByText(/^Y CH2/).first()).toBeVisible();
  });

  test('given XY, when one channel is on, then the screen note says XY needs two enabled channels', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', XY_ONE_CHANNEL());
    await button(page, 'XY').click();
    await expect(page.getByText('XY needs two enabled channels')).toBeVisible();
    await expect(page.getByText('Only CH1 is on. Enable another channel.')).toBeVisible();
  });

  test('given XY with Y on CH2, when CH2 is switched off, then the note names the channel and axis and Enable CH2 sends set_channel', async ({
    page,
  }) => {
    await openMain(page);
    await button(page, 'XY').click();
    const base = instrumentState({ revision: 91 });
    await emit(page, 'instrument_changed', {
      ...base,
      channels: base.channels.map((c) => ({ ...c, enabled: c.number === 1 || c.number === 3 })),
    });
    await expect(page.getByText('XY cannot draw')).toBeVisible();
    await expect(page.getByText('CH2 (Y) is off.')).toBeVisible();
    await button(page, 'Enable CH2').click();
    const sent = await calls(page, 'set_channel');
    expect(sent.at(-1)?.args).toMatchObject({ channel: 2 });
  });

  test('given XY, when the axis selectors are read, then the other axis channel is disabled in each', async ({
    page,
  }) => {
    await openMain(page);
    await button(page, 'XY').click();
    const x = page.getByRole('group', { name: /X axis/i });
    await expect(x.getByRole('button', { name: '2' })).toHaveAttribute('aria-disabled', 'true');
    const y = page.getByRole('group', { name: /Y axis/i });
    await expect(y.getByRole('button', { name: '1' })).toHaveAttribute('aria-disabled', 'true');
  });

  test('given XY then Yt, when the Horizontal and Trigger readouts are compared, then they are unchanged', async ({
    page,
  }) => {
    await openMain(page);
    const dock = page.locator('.dock');
    const before = await dock.innerText();
    await button(page, 'XY').click();
    await button(page, 'Yt').click();
    expect(await dock.innerText()).toBe(before);
    expect(await calls(page, 'set_trigger')).toHaveLength(0);
  });

  test('given XY, when the Persist and Cursors buttons are read, then both are disabled', async ({ page }) => {
    await openMain(page);
    await button(page, 'XY').click();
    await expect(button(page, /Persist/)).toHaveAttribute('aria-disabled', 'true');
    await expect(button(page, 'Cursors')).toHaveAttribute('aria-disabled', 'true');
  });
});

test.describe('#26 persistence menu', () => {
  test('given the Persist button, when opened, then the radio items are Off, 0.5 s, 1 s, 2 s, Infinite and Clear', async ({
    page,
  }) => {
    await openMain(page);
    await button(page, /Persist/).click();
    for (const name of ['Off', '0.5 s', '1 s', '2 s', 'Infinite']) {
      await expect(page.getByRole('menuitemradio', { name })).toBeVisible();
    }
    await expect(page.getByRole('menuitem', { name: 'Clear persistence' })).toBeVisible();
  });

  test('given the menu, when 1 s is chosen, then set_persistence is sent with 1s', async ({ page }) => {
    test.fail(true, 'BUG-21: the live app wires inert analysis commands, nothing is sent');
    await openMain(page);
    await button(page, /Persist/).click();
    await page.getByRole('menuitemradio', { name: '1 s' }).click();
    const sent = await calls(page, 'set_persistence');
    expect(sent.at(-1)?.args).toEqual({ value: '1s' });
  });
});

test.describe('#29 FFT view', () => {
  test('given Yt, when FFT is selected, then set_spectrum active true is sent once and false on return', async ({
    page,
  }) => {
    test.fail(true, 'BUG-21: the live app wires inert analysis commands, nothing is sent');
    await openMain(page);
    await button(page, /FFT/).click();
    await button(page, 'Yt').click();
    const sent = (await calls(page, 'set_spectrum')).map((c) => c.args);
    expect(sent).toEqual([{ patch: { active: true } }, { patch: { active: false } }]);
  });

  test('given the FFT view, when it is open, then the spectrum stream is opened and the peak marker panel shows', async ({
    page,
  }) => {
    test.fail(true, 'BUG-21: createSpectrumFeed is never wired in main.ts');
    await openMain(page);
    await button(page, /FFT/).click();
    await expect.poll(async () => (await calls(page, 'open_spectrum_stream')).length).toBeGreaterThan(0);
  });

  test('given the FFT view, when the layout is read, then the peak marker panel replaces Cursors and the scale offers dBV and Linear', async ({
    page,
  }) => {
    await openMain(page);
    await button(page, /FFT/).click();
    await expect(page.getByText('PEAK MARKER')).toBeVisible();
    await expect(button(page, 'dBV')).toBeVisible();
    await expect(button(page, 'Linear').first()).toBeVisible();
  });
});

test.describe('#30 math row', () => {
  test('given math unavailable because CH2 is off, when shown, then the chip names the reason in words', async ({
    page,
  }) => {
    await openMain(page);
    const base = instrumentState({ revision: 100 });
    await emit(page, 'instrument_changed', {
      ...base,
      math: { ...base.math, enabled: true, status: { kind: 'unavailable', reason: 'CH2 is off' } },
    });
    await expect(page.getByText(/M\s+A−B\s+unavailable: CH2 is off/)).toBeVisible();
  });

  test('given sources on different scales, when math is on, then the Mixed badge shows with the explicit scale', async ({
    page,
  }) => {
    await openMain(page);
    const base = instrumentState({ revision: 101 });
    await emit(page, 'instrument_changed', {
      ...base,
      math: {
        ...base.math,
        enabled: true,
        status: {
          kind: 'mixed',
          detail: 'CH1 500 mV/div, CH2 200 mV/div. The math trace has its own scale, 500 mV/div.',
        },
      },
    });
    await expect(page.getByText('Mixed', { exact: false }).first()).toBeVisible();
  });

  test('given the math toggle, when pressed, then set_math enabled true is sent', async ({ page }) => {
    test.fail(true, 'BUG-21: the live app wires inert analysis commands, nothing is sent');
    await openMain(page);
    await button(page, 'M, math channel enabled').click();
    const sent = await calls(page, 'set_math');
    expect(sent.at(-1)?.args).toEqual({ patch: { enabled: true } });
  });
});

test.describe('#28 recording', () => {
  test('given a roll setup, when Record is pressed, then start_recording is sent', async ({ page }) => {
    test.fail(true, 'BUG-21: the live app wires inert analysis commands, nothing is sent');
    await openMain(page);
    await emit(page, 'instrument_changed', rollState());
    await button(page, 'Record').click();
    await expect.poll(async () => (await calls(page, 'start_recording')).length).toBe(1);
  });

  test('given a recording failure, when recording_changed arrives, then one error Banner offers Choose new file and Dismiss', async ({
    page,
  }) => {
    test.fail(true, 'BUG-21: the recording slice is not handed to the model');
    await openMain(page);
    await emit(page, 'recording_changed', {
      revision: 5,
      active: false,
      fileName: null,
      duration: { value: 0, text: '0:00:00', unit: '' },
      bytesWritten: { value: 0, text: '0', unit: 'B' },
      failure: { id: 1, text: 'the file was removed', fileName: 'a.ssroll', bytes: 15246 },
    });
    await expect(page.getByText(/Recording stopped: the file was removed/)).toBeVisible();
    await expect(button(page, 'Choose new file')).toBeVisible();
    await expect(button(page, 'Dismiss')).toBeVisible();
  });
});

test.describe('accessibility of the new screens', () => {
  const axe = (page: Page) =>
    new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .exclude('[aria-disabled="true"]')
      .exclude('.dim')
      .analyze();

  for (const view of ['XY', 'FFT'] as const) {
    test(`given the ${view} view, when axe runs, then there are no violations`, async ({ page }) => {
      await openMain(page);
      await sendFrame(page, wireFrame({ mask: 0b0011, count: 2501, value: sine }));
      await button(page, view === 'FFT' ? /FFT/ : 'XY').click();
      expect((await axe(page)).violations).toEqual([]);
    });
  }

  test('given a roll setup with the trigger reason, when axe runs, then there are no violations', async ({ page }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', rollState());
    expect((await axe(page)).violations).toEqual([]);
  });
});
