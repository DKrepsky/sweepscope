import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createFakeClient } from './api';
import { createLiveApp } from './app/main-window';
import type { LayoutTier } from './app/layout';
import { stateSnapshot } from './app/instrument/test-data';
import type { TextScalePercent } from './app/text-scale';
import MainWindow from './pages/main-window/MainWindow.svelte';
import { createRecordingContext } from './ui/graph/recording-context';

// The real stack as main.ts composes it, with the fake backend client playing the calibration session.
const tier: LayoutTier = { width: 'w1', height: 'h1', rootFontPx: 16 };

beforeEach(() => {
  window.matchMedia = (() => ({ addEventListener: vi.fn(), removeEventListener: vi.fn() })) as never;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => createRecordingContext() as never);
  HTMLDialogElement.prototype.showModal = vi.fn(function (this: HTMLDialogElement) {
    this.setAttribute('open', '');
  });
  HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
    this.removeAttribute('open');
  });
});

async function openWindow() {
  const client = createFakeClient();
  client.respond('get_snapshot', () => stateSnapshot({ zeroCalibration: client.zeroCalibration.state() }));
  const app = createLiveApp({
    client,
    notify: vi.fn(),
    copyText: vi.fn(),
    composeScreenshot: () => Promise.resolve(new Uint8Array()),
    now: () => 0,
    requestAnimationFrame: () => {},
    afterRender: () => Promise.resolve(),
    probe: { beginFrame: () => {}, endFrame: () => {} },
  });
  await app.start();
  render(MainWindow, {
    props: {
      model: app.model,
      actions: app.model,
      tier: writable(tier),
      textScale: writable<TextScalePercent>(100),
      notifications: writable([]),
      onnotificationdismiss: () => {},
      ontextscale: () => {},
    },
  });
  return { client, app };
}

const dialog = () => screen.findByRole('dialog', { name: 'Zero calibration' });
const press = (name: string) => userEvent.click(screen.getByRole('button', { name }));

describe('calibration dialog in the real composition', () => {
  it('given a ready device, when calibration is opened, then the dialog shows over the window and the toolbar says paused', async () => {
    const { app } = await openWindow();

    app.model.openCalibration();

    expect(await dialog()).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Why calibrate' })).toHaveFocus();
    expect(screen.getByText('Paused (calibration)')).toBeVisible();
  });

  it('given the dialog at the intro, when the user walks all four channels, then the done table lists twelve ranges and Done closes it', async () => {
    const { app } = await openWindow();
    app.model.openCalibration();
    await dialog();

    await press('Start');
    for (const channel of [1, 2, 3, 4]) {
      await screen.findByRole('heading', { name: `Channel ${channel}: short the input` });
      await press('Calibrate');
    }

    const table = within(await screen.findByRole('region', { name: /^Stored offsets/ })).getByRole('table');
    expect(within(table).getAllByRole('rowheader')).toHaveLength(12);
    expect(screen.getByText('Step 6 of 6')).toBeVisible();
    await press('Done');
    await vi.waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(screen.queryByText('Paused (calibration)')).toBeNull();
  });

  it('given a channel that is not shorted, when Try again is pressed after it is shorted, then the walk continues on that channel', async () => {
    const { app, client } = await openWindow();
    client.zeroCalibration.leaveUnshorted(2);
    app.model.openCalibration();
    await dialog();
    await press('Start');
    await screen.findByRole('heading', { name: 'Channel 1: short the input' });
    await press('Calibrate');
    await screen.findByRole('heading', { name: 'Channel 2: short the input' });

    await press('Calibrate');

    expect(await screen.findByRole('alert')).toHaveTextContent('The CH2 input does not look shorted.');
    client.zeroCalibration.leaveUnshorted();
    await press('Try again');
    expect(await screen.findByRole('heading', { name: 'Channel 3: short the input' })).toBeVisible();
  });

  it('given the dialog is open, when the Autoset shortcut key is pressed, then no command reaches the backend', async () => {
    const { app, client } = await openWindow();
    app.model.openCalibration();
    await dialog();
    const before = client.calls.length;

    await userEvent.keyboard('a');

    expect(client.calls.slice(before).map((call) => call.name)).not.toContain('start_autoset');
  });

  it('given a session, when Cancel is pressed, then the dialog closes and the backend is asked to close it', async () => {
    const { app, client } = await openWindow();
    app.model.openCalibration();
    await dialog();

    await press('Cancel');

    await vi.waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(client.calls.map((call) => call.name)).toContain('close_zero_calibration');
  });

  it('given the device is lost during a session, when the dialog updates, then it shows the failure and Close', async () => {
    const { app, client } = await openWindow();
    app.model.openCalibration();
    await dialog();

    client.zeroCalibration.loseDevice();

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Device disconnected. Calibration cancelled. Nothing was stored.',
    );
    expect(screen.getByRole('button', { name: 'Close' })).toBeVisible();
  });
});
