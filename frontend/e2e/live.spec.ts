import { expect, test, type Page } from '@playwright/test';
import {
  connectionSnapshot,
  instrumentFacts,
  instrumentState,
  acquisitionStatus,
} from '../src/app/instrument/test-data';
import { openMain as open, SCOPE_SCREEN } from './support/app';
import { sine, wireFrame } from './support/frames';
import { emit, ipcLog, sendFrame } from './support/tauri-stub';

/** Counts bright canvas pixels (trace colours, also when dimmed); the graticule is darker and sits in the empty baseline. */
async function colouredPixels(page: Page): Promise<number> {
  return page.evaluate((selector) => {
    const canvases = [...document.querySelectorAll<HTMLCanvasElement>(`${selector} canvas`)];
    let count = 0;
    for (const canvas of canvases) {
      const data = canvas.getContext('2d')!.getImageData(0, 0, canvas.width, canvas.height).data;
      for (let i = 0; i < data.length; i += 4) {
        const [r, g, b, a] = [data[i] ?? 0, data[i + 1] ?? 0, data[i + 2] ?? 0, data[i + 3] ?? 0];
        if (a > 100 && Math.max(r, g, b) > 80) count += 1;
      }
    }
    return count;
  }, SCOPE_SCREEN);
}

/** Opens the main window and waits until the frame stream is open, so a frame sent next is not lost. */
async function openMain(page: Page): Promise<void> {
  await open(page);
  await expect.poll(async () => (await ipcLog(page)).some((call) => call.command === 'open_frame_stream')).toBe(true);
}

const graph = (page: Page) => page.locator('.graph');

test.describe('live view against scripted backend events', () => {
  test('given a ready simulator, when the status bar is read, then the identity items and the simulated marker show', async ({
    page,
  }) => {
    await openMain(page);
    const status = page.locator('[data-region="status"]');
    await expect(status).toContainText('Simulator');
    await expect(status).toContainText('Hantek 6254BC');
  });

  test('given a running scope and no frame yet, when the graph is read, then it is empty', async ({ page }) => {
    await openMain(page);
    await expect(graph(page)).toHaveAttribute('data-state', 'empty');
  });

  test('given a frame on channels 1 and 3, when it arrives, then the trace is drawn and the state is live', async ({
    page,
  }) => {
    await openMain(page);
    const empty = await colouredPixels(page);
    await sendFrame(page, wireFrame({ mask: 0b0101, count: 2501, value: sine }));
    await expect(graph(page)).toHaveAttribute('data-state', 'live');
    await expect.poll(() => colouredPixels(page)).toBeGreaterThan(empty + 1500);
    await expect(page.getByRole('img', { name: /Yt display, 10 by 8 divisions\. CH1 at 1 V\/div, CH3/ })).toBeVisible();
  });

  test('given a frame, when a later frame with a one-sample glitch arrives, then the glitch pixels are drawn', async ({
    page,
  }) => {
    await openMain(page);
    const empty = await colouredPixels(page);
    const flat = wireFrame({ mask: 1, count: 2501, value: () => 4 });
    await sendFrame(page, flat);
    await expect.poll(() => colouredPixels(page)).toBeGreaterThan(empty + 500);
    const before = await colouredPixels(page);
    await sendFrame(page, wireFrame({ mask: 1, count: 2501, value: (_c, i) => (i === 1250 ? 0.5 : 4) }));
    // A 3.5 division spike one sample wide adds a visible vertical run at least 3 divisions tall.
    await expect.poll(() => colouredPixels(page)).toBeGreaterThan(before + 60);
  });

  test('given a live graph, when the connection drops, then the graph freezes on the last frame and Disconnected shows', async ({
    page,
  }) => {
    await openMain(page);
    const empty = await colouredPixels(page);
    await sendFrame(page, wireFrame({ mask: 1, count: 2501, value: sine }));
    await expect(graph(page)).toHaveAttribute('data-state', 'live');
    await expect.poll(() => colouredPixels(page)).toBeGreaterThan(empty + 500);
    await emit(
      page,
      'connection_changed',
      connectionSnapshot({ revision: 2, state: 'disconnected', source: null, identity: null, simulatorActive: false }),
    );
    await emit(page, 'instrument_changed', instrumentState({ revision: 2, running: false }));
    await expect(graph(page)).toHaveAttribute('data-state', 'frozen');
    // The last trace is still on the canvas (dimmed, not cleared).
    expect(await colouredPixels(page)).toBeGreaterThan(empty + 500);
    await expect(page.locator('[data-region="status"]')).toContainText('Disconnected');
    await expect(page.getByText(/No scope found\. Plug in/)).toBeVisible();
    await expect(page.getByRole('button', { name: 'Use simulator' })).toBeVisible();
  });

  test('given the frozen disconnected state, when the scope returns, then the graph is live again and the banner is gone', async ({
    page,
  }) => {
    await openMain(page);
    await sendFrame(page, wireFrame({ mask: 1, count: 2501, value: sine }));
    await emit(
      page,
      'connection_changed',
      connectionSnapshot({ revision: 2, state: 'disconnected', source: null, identity: null }),
    );
    await expect(page.getByText(/No scope found\. Plug in/)).toBeVisible();
    await emit(page, 'connection_changed', connectionSnapshot({ revision: 3 }));
    await emit(page, 'instrument_changed', instrumentState({ revision: 3, running: true }));
    await expect(page.getByText(/No scope found\. Plug in/)).toHaveCount(0);
    await expect(graph(page)).toHaveAttribute('data-state', 'live');
  });

  test('given the disconnected banner, when Use simulator and Retry are pressed, then both commands are sent', async ({
    page,
  }) => {
    await openMain(page);
    await emit(
      page,
      'connection_changed',
      connectionSnapshot({ revision: 2, state: 'disconnected', source: null, identity: null }),
    );
    await page.getByRole('button', { name: 'Use simulator' }).click();
    await page.getByRole('button', { name: 'Retry' }).click();
    const commands = (await ipcLog(page)).filter(
      (c) => c.command === 'use_simulator' || c.command === 'retry_connection',
    );
    expect(commands).toEqual([
      { command: 'use_simulator', args: { enabled: true } },
      { command: 'retry_connection', args: {} },
    ]);
  });

  test('given a permission denied error, when the Banner shows, then it names the rule and the command and Copy puts it on the clipboard', async ({
    page,
    context,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'clipboard permission can only be granted in Chromium');
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    await openMain(page);
    const command =
      'sudo cp packaging/udev/99-hantek6xx4.rules /etc/udev/rules.d/ && sudo udevadm control --reload-rules && sudo udevadm trigger';
    await emit(
      page,
      'connection_changed',
      connectionSnapshot({
        revision: 2,
        state: 'error',
        source: 'hardware',
        identity: null,
        simulatorActive: false,
        issue: {
          kind: 'permissionDenied',
          message: 'Permission denied opening the scope. Install the udev rule 99-hantek6xx4.rules.',
          help: { ruleFile: '99-hantek6xx4.rules', installCommand: command, doc: 'docs/usb-permissions.md' },
        },
      }),
    );
    await expect(page.getByText(/99-hantek6xx4\.rules/).first()).toBeVisible();
    const copy = page.getByRole('button', { name: 'Copy command' });
    await expect(copy).toBeVisible();
    await expect(page.getByRole('button', { name: 'Retry' })).toBeVisible();
    await copy.click();
    expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(command);
    // #6 AC2: the control confirms the copy (BUG-4: it does not yet).
    test.fail(true, 'BUG-4: Copy command gives no confirmation');
    await expect(page.getByText(/copied/i)).toBeVisible({ timeout: 1000 });
  });

  test('given the trigger phase events, when they arrive, then the chip reads Armed, Waiting for trigger, Trig’d and Auto', async ({
    page,
  }) => {
    await openMain(page);
    const chips = page.getByRole('status', { name: 'Acquisition state' });
    const cases: [Parameters<typeof acquisitionStatus>[0], string][] = [
      [{ revision: 2, phase: 'armed' }, 'Armed'],
      [{ revision: 3, phase: 'waitingForTrigger' }, 'Waiting for trigger'],
      [{ revision: 4, phase: 'triggered' }, 'Trig’d'],
      [{ revision: 5, phase: 'auto' }, 'Auto'],
    ];
    for (const [patch, text] of cases) {
      await emit(page, 'status_changed', acquisitionStatus(patch));
      await expect(chips).toContainText(text);
    }
  });

  test('given a stopped instrument, when the state arrives, then the toggle reads Run and the chip Stop', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', instrumentState({ revision: 2, running: false }));
    await expect(page.getByRole('button', { name: /^Run/ })).toBeVisible();
    await expect(page.getByRole('status', { name: 'Acquisition state' })).toContainText('Stop');
  });

  test('given the facts event, when it arrives, then sample rate, record length, capture rate and dead time show live numbers', async ({
    page,
  }) => {
    await openMain(page);
    const fact = (value: number, text: string, unit: string) => ({ value, text, unit });
    await emit(
      page,
      'facts_changed',
      instrumentFacts({
        revision: 2,
        sampleRate: fact(1e9, '1.00', 'GS/s'),
        recordPoints: fact(4096, '4.10', 'kpts'),
        captureRate: fact(31, '31.0', 'fps'),
        deadTime: fact(0.62, '62.0', '%'),
      }),
    );
    const status = page.locator('[data-region="status"]');
    await expect(status).toContainText('1.00 GS/s');
    await expect(status).toContainText('4.10 kpts');
    await expect(status).toContainText('31.0 fps');
    await expect(status).toContainText('dead time 62.0 %');
    await emit(page, 'facts_changed', instrumentFacts({ revision: 3, captureRate: fact(12, '12.0', 'fps') }));
    await expect(status).toContainText('12.0 fps');
  });

  test('given an event with a lower revision, when it arrives after a newer one, then the newer state stays', async ({
    page,
  }) => {
    await openMain(page);
    await emit(page, 'instrument_changed', instrumentState({ revision: 9, running: false }));
    await expect(page.getByRole('status', { name: 'Acquisition state' })).toContainText('Stop');
    await emit(page, 'instrument_changed', instrumentState({ revision: 4, running: true }));
    await expect(page.getByRole('status', { name: 'Acquisition state' })).toContainText('Stop');
  });

  test('given a communication error, when the state arrives, then an error banner and the Error state show and the window stays usable', async ({
    page,
  }) => {
    await openMain(page);
    await emit(
      page,
      'connection_changed',
      connectionSnapshot({
        revision: 2,
        state: 'error',
        identity: null,
        issue: { kind: 'communication', message: 'USB transfer failed', help: null },
      }),
    );
    await expect(page.getByText('USB transfer failed').first()).toBeVisible();
    await expect(page.locator('[data-region="status"]')).toContainText('Error');
    await expect(page.getByTestId('main-window')).toBeVisible();
  });
});
