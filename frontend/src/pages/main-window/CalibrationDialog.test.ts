import { render, screen, within, fireEvent } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { selectCalibrationDialog, type CalibrationDialogModel } from '../../app/calibration';
import { zeroCalibrationState } from '../../app/instrument/test-data';
import CalibrationDialog from './CalibrationDialog.svelte';

type Session = NonNullable<NonNullable<Parameters<typeof zeroCalibrationState>[0]>['session']>;

type Offsets = NonNullable<Session['table']>[number]['offsets'];

const idle: Session = {
  step: 'intro',
  channel: null,
  doneChannels: [false, false, false, false],
  progress: null,
  notShorted: null,
  canContinue: false,
  overridden: [false, false, false, false],
  failure: null,
  table: null,
};

const value = (text: string, unit: string) => ({ value: Number(text), text, unit });

const RANGES = ['2 mV', '5 mV', '10 mV', '20 mV', '50 mV', '100 mV', '200 mV', '500 mV', '1 V', '2 V', '5 V', '10 V'];
const table = RANGES.map((range) => {
  const [text = '', unit = ''] = range.split(' ');
  return {
    range: value(text, `${unit}/div`),
    offsets: [value('0.66', 'mV'), value('\u221241.2', '\u00b5V'), value('0', 'V'), null] as Offsets,
  };
});

function dialogModel(
  session: Partial<Session>,
  unit: { calibrated: boolean; storedAtUtc: string | null } = {
    calibrated: false,
    storedAtUtc: null,
  },
): CalibrationDialogModel {
  const state = zeroCalibrationState({
    session: { ...idle, ...session },
    unit: { ...unit, tooltip: { heading: '', channels: [{ cells: [] }, { cells: [] }, { cells: [] }, { cells: [] }] } },
  });
  return selectCalibrationDialog(state) as CalibrationDialogModel;
}

function setup(session: Partial<Session>, unit?: { calibrated: boolean; storedAtUtc: string | null }) {
  const handlers = { onstart: vi.fn(), oncalibrate: vi.fn(), oncontinue: vi.fn(), onclose: vi.fn() };
  const view = render(CalibrationDialog, { props: { dialog: dialogModel(session, unit), ...handlers } });
  return { ...view, ...handlers };
}

beforeEach(() => {
  HTMLDialogElement.prototype.showModal = vi.fn(function (this: HTMLDialogElement) {
    this.setAttribute('open', '');
  });
  HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
    this.removeAttribute('open');
  });
});

const dialogElement = () => document.querySelector('dialog') as HTMLDialogElement;
const channelStep: Partial<Session> = { step: 'channel', channel: 2, doneChannels: [true, false, false, false] };

describe('CalibrationDialog intro', () => {
  it('given the intro step, when shown, then the title, step number, why and the three bullets are shown', () => {
    setup({});

    expect(screen.getByText('Zero calibration')).toBeVisible();
    expect(screen.getByText('Step 1 of 6')).toBeVisible();
    expect(screen.getByRole('heading', { name: 'Why calibrate' })).toBeVisible();
    expect(screen.getAllByRole('listitem').map((item) => item.textContent)).toEqual([
      'You short each of the 4 inputs in turn',
      'Acquisition is paused while this dialog is open',
      'Nothing is stored until you finish, and the factory calibration is never changed',
    ]);
  });

  it('given the intro step, when shown, then the footer has Cancel and Start only', () => {
    setup({});

    expect(screen.getAllByRole('button').map((button) => button.textContent?.trim())).toEqual(['Cancel', 'Start']);
  });

  it('given an uncalibrated device, when the intro is shown, then there is no calibrated note', () => {
    setup({});

    expect(screen.queryByText('This device is calibrated')).toBeNull();
  });

  it('given a calibrated device, when the intro is shown, then the note names the date on its own line', () => {
    setup({}, { calibrated: true, storedAtUtc: '2026-11-24T14:02:00Z' });

    const note = screen.getByText('This device is calibrated').closest('div') as HTMLElement;

    expect(note.querySelector('br')).not.toBeNull();
    expect(note.textContent).toMatch(/Last calibrated at \d{4}-\d{2}-\d{2} \d{2}:\d{2}/);
  });

  it('given the intro step, when Start is pressed, then onstart is called', async () => {
    const { onstart } = setup({});

    await userEvent.click(screen.getByRole('button', { name: 'Start' }));

    expect(onstart).toHaveBeenCalledOnce();
  });
});

describe('CalibrationDialog modal behaviour', () => {
  it('given the dialog, when rendered, then it is opened as a native modal named by its title and described by its body', () => {
    setup({});

    expect(HTMLDialogElement.prototype.showModal).toHaveBeenCalledOnce();
    expect(dialogElement()).toHaveAccessibleName('Zero calibration');
    expect(dialogElement()).toHaveAccessibleDescription(/Why calibrate/);
  });

  it('given the dialog, when it opens, then the focus is on the step heading and not on a button', () => {
    setup({});

    expect(screen.getByRole('heading', { name: 'Why calibrate' })).toHaveFocus();
  });

  it('given a channel step, when the next step replaces it, then the focus moves to the new heading', async () => {
    const view = setup(channelStep);
    expect(screen.getByRole('heading', { name: 'Channel 2: short the input' })).toHaveFocus();

    await view.rerender({ dialog: dialogModel({ step: 'channel', channel: 3 }) });
    await tick();

    expect(screen.getByRole('heading', { name: 'Channel 3: short the input' })).toHaveFocus();
  });

  it('given the dialog, when Escape cancels it, then onclose is called', async () => {
    const { onclose } = setup({});

    await fireEvent(dialogElement(), new Event('cancel', { cancelable: true }));

    expect(onclose).toHaveBeenCalledOnce();
  });

  it('given the dialog opened from a button, when it is removed, then the focus returns to that button', async () => {
    const opener = document.createElement('button');
    document.body.append(opener);
    opener.focus();
    const view = setup({});

    view.unmount();
    await Promise.resolve();

    expect(opener).toHaveFocus();
    opener.remove();
  });
});

describe('CalibrationDialog channel steps', () => {
  it('given channel 2, when shown, then the heading, instructions, picture and note use channel 2', () => {
    setup(channelStep);

    expect(screen.getByText('Step 3 of 6')).toBeVisible();
    expect(screen.getByRole('heading', { name: 'Channel 2: short the input' })).toBeVisible();
    expect(screen.getByText('Short the CH2 input:')).toBeVisible();
    expect(screen.getByRole('img', { name: 'Probe tip touching the probe ground clip' })).toBeVisible();
    expect(screen.getByText('An open input is not zero: it picks up the neighbouring channels.')).toBeVisible();
  });

  it('given a channel step, when shown, then the footer has Cancel and Calibrate', () => {
    setup(channelStep);

    expect(screen.getAllByRole('button').map((button) => button.textContent?.trim())).toEqual(['Cancel', 'Calibrate']);
  });

  it('given a channel step, when Calibrate is pressed, then oncalibrate is called', async () => {
    const { oncalibrate } = setup(channelStep);

    await userEvent.click(screen.getByRole('button', { name: 'Calibrate' }));

    expect(oncalibrate).toHaveBeenCalledOnce();
  });

  it('given a channel step, when Cancel is pressed, then onclose is called', async () => {
    const { onclose } = setup(channelStep);

    await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(onclose).toHaveBeenCalledOnce();
  });

  it('given channel 2 in turn after channel 1, when the chips are read, then they say done, current, to do', () => {
    setup(channelStep);

    expect(screen.getByLabelText('Channel 1, done')).toBeVisible();
    expect(screen.getByLabelText('Channel 2, current')).toHaveAttribute('aria-current', 'step');
    expect(screen.getByLabelText('Channel 3, to do')).toBeVisible();
    expect(screen.getByLabelText('Channel 4, to do')).toBeVisible();
  });

  it('given the intro step, when shown, then there is no channel progress row', () => {
    setup({});

    expect(screen.queryByText('Channels')).toBeNull();
  });
});

describe('CalibrationDialog measuring', () => {
  const measuring: Partial<Session> = { step: 'measuring', channel: 1, progress: 0.4 };

  it('given a measurement at 40 percent, when shown, then the status line, bar and percent say so', () => {
    setup(measuring);

    expect(screen.getByRole('status')).toHaveTextContent('Measuring CH1, keep the input shorted and still');
    expect(screen.getByRole('progressbar', { name: 'Measuring channel 1' })).toHaveAttribute('aria-valuenow', '40');
    expect(screen.getByText('40 %')).toBeVisible();
  });

  it('given a measurement without a known progress, when shown, then the bar is indeterminate and shows no percent', () => {
    setup({ ...measuring, progress: null });

    expect(screen.getByRole('progressbar')).not.toHaveAttribute('aria-valuenow');
    expect(screen.queryByText(/ %$/)).toBeNull();
  });

  it('given a measurement, when shown, then Cancel is enabled and the primary reads Measuring... and is disabled', () => {
    setup(measuring);

    expect(screen.getByRole('button', { name: 'Cancel' })).not.toHaveAttribute('aria-disabled');
    expect(screen.getByRole('button', { name: 'Measuring...' })).toHaveAttribute('aria-disabled', 'true');
    expect(screen.getByText('Cancel stores nothing.')).toBeVisible();
  });

  it('given a measurement, when the disabled primary is pressed, then nothing is called', async () => {
    const { oncalibrate, onstart } = setup(measuring);

    await userEvent.click(screen.getByRole('button', { name: 'Measuring...' }));

    expect(oncalibrate).not.toHaveBeenCalled();
    expect(onstart).not.toHaveBeenCalled();
  });

  it('given a measurement, when shown, then the dialog names no voltage range', () => {
    setup(measuring);

    expect(dialogElement().textContent).not.toMatch(/\/div/);
  });
});

describe('CalibrationDialog not shorted', () => {
  const notShorted: Partial<Session> = {
    step: 'notShorted',
    channel: 3,
    doneChannels: [true, true, false, false],
    notShorted: ['Measured level 412 mV from zero.'],
    canContinue: true,
  };

  it('given a channel that is not shorted, when shown, then an alert names it, the numbers and the next action', () => {
    setup(notShorted);

    const alert = screen.getByRole('alert');

    expect(alert).toHaveTextContent('The CH3 input does not look shorted. Nothing was stored for this channel.');
    expect(alert).toHaveTextContent('Measured level 412 mV from zero.');
    expect(alert).toHaveTextContent('Check the connection between probe tip and ground clip, then try again.');
  });

  it('given a channel that is not shorted, when shown, then the step number and chips stay on that channel and there is no Skip', () => {
    setup(notShorted);

    expect(screen.getByText('Step 4 of 6')).toBeVisible();
    expect(screen.getByLabelText('Channel 1, done')).toBeVisible();
    expect(screen.getByLabelText('Channel 3, current')).toBeVisible();
    expect(screen.queryByRole('button', { name: /skip/i })).toBeNull();
  });

  it('given a channel that is not shorted, when Try again is pressed, then oncalibrate is called', async () => {
    const { oncalibrate } = setup(notShorted);

    await userEvent.click(screen.getByRole('button', { name: 'Try again' }));

    expect(oncalibrate).toHaveBeenCalledOnce();
  });
});

describe('CalibrationDialog continue', () => {
  const notShorted: Partial<Session> = {
    step: 'notShorted',
    channel: 3,
    doneChannels: [true, true, false, false],
    notShorted: ['Measured level 412 mV from zero.'],
    canContinue: true,
  };

  it('given a rejection that may continue, when the footer is read, then it is Cancel, Continue, Try again', () => {
    setup(notShorted);

    expect(screen.getAllByRole('button').map((button) => button.textContent?.trim())).toEqual([
      'Cancel',
      'Continue',
      'Try again',
    ]);
  });

  it('given a rejection that may continue, when shown, then a note says Continue keeps the offsets of that channel', () => {
    setup(notShorted);

    expect(
      screen.getByText('Continue keeps the measured offsets for CH3, use it only if the input is shorted'),
    ).toBeVisible();
  });

  it('given a rejection that may continue, when Continue is pressed, then oncontinue is called and oncalibrate is not', async () => {
    const { oncontinue, oncalibrate } = setup(notShorted);

    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));

    expect(oncontinue).toHaveBeenCalledOnce();
    expect(oncalibrate).not.toHaveBeenCalled();
  });

  it('given a clipped reading that cannot continue, when shown, then there is no Continue and no note about it', () => {
    setup({ ...notShorted, canContinue: false });

    expect(screen.queryByRole('button', { name: 'Continue' })).toBeNull();
    expect(screen.queryByText(/Continue keeps/)).toBeNull();
    expect(screen.getByRole('button', { name: 'Try again' })).toBeVisible();
  });
});

describe('CalibrationDialog done', () => {
  const done: Partial<Session> = { step: 'done', channel: null, doneChannels: [true, true, true, true], table };

  it('given the done step, when shown, then the success note and the only button Done are shown', () => {
    setup(done);

    expect(screen.getByRole('status')).toHaveTextContent('Calibration completed');
    expect(screen.getByRole('status')).toHaveTextContent(
      'Offsets applied per channel and range and kept after restart, the device is now CALIBRATED',
    );
    expect(screen.getAllByRole('button').map((button) => button.textContent?.trim())).toEqual(['Done']);
    expect(screen.getByText('Step 6 of 6')).toBeVisible();
  });

  it('given channels accepted with Continue, when Done is shown, then one quiet line names each of them', () => {
    setup({ ...done, overridden: [false, true, false, true] });

    expect(screen.getAllByRole('listitem').map((item) => item.textContent)).toEqual([
      'CH2 accepted although the input did not look shorted',
      'CH4 accepted although the input did not look shorted',
    ]);
  });

  it('given no channel accepted with Continue, when Done is shown, then no such line is shown', () => {
    setup(done);

    expect(screen.queryByText(/accepted although/)).toBeNull();
  });

  it('given the done step, when Done is pressed, then onclose is called', async () => {
    const { onclose } = setup(done);

    await userEvent.click(screen.getByRole('button', { name: 'Done' }));

    expect(onclose).toHaveBeenCalledOnce();
  });

  it('given the done step, when shown, then all four chips are done', () => {
    setup(done);

    [1, 2, 3, 4].forEach((n) => expect(screen.getAllByLabelText(`Channel ${n}, done`)).toHaveLength(1));
  });

  it('given the done step, when the table is read, then it has twelve ranges as row headers and a column per channel', () => {
    setup(done);

    const grid = screen.getByRole('table');

    expect(
      within(grid)
        .getAllByRole('rowheader')
        .map((cell) => cell.textContent),
    ).toEqual(RANGES.map((range) => `${range}/div`));
    expect(
      within(grid)
        .getAllByRole('columnheader')
        .map((cell) => cell.textContent?.trim()),
    ).toEqual(['Range', '1', '2', '3', '4']);
  });

  it('given a range with offsets and one without, when the row is read, then values keep the backend text and a missing one is a dash', () => {
    setup(done);

    const row = within(screen.getByRole('table')).getAllByRole('row')[1] as HTMLElement;

    expect(
      within(row)
        .getAllByRole('cell')
        .map((cell) => cell.textContent),
    ).toEqual(['0.66 mV', '−41.2 µV', '0 V', '-']);
  });

  it('given the done step, when the scroll region is read, then it is a focusable named region', () => {
    setup(done);

    const region = screen.getByRole('region', {
      name: 'Stored offsets per voltage range and channel, scrollable',
    });

    expect(region).toHaveAttribute('tabindex', '0');
    expect(region).toContainElement(screen.getByRole('table'));
  });

  it('given the done step, when the focus is read, then it is on the success note and not on Done', () => {
    setup(done);

    expect(screen.getByRole('status')).toHaveFocus();
  });

  it('given the done step, when Escape cancels, then onclose is called as well', async () => {
    const { onclose } = setup(done);

    await fireEvent(dialogElement(), new Event('cancel', { cancelable: true }));

    expect(onclose).toHaveBeenCalledOnce();
  });
});

describe('CalibrationDialog failed', () => {
  const failed: Partial<Session> = {
    step: 'failed',
    channel: 2,
    doneChannels: [true, false, false, false],
    failure: 'Device disconnected. Calibration cancelled. Nothing was stored.',
  };

  it('given a lost device, when shown, then the body is an alert with the message and the only button is Close', () => {
    setup(failed);

    expect(screen.getByRole('alert')).toHaveTextContent(failed.failure ?? '');
    expect(screen.getAllByRole('button').map((button) => button.textContent?.trim())).toEqual(['Close']);
    expect(screen.queryByRole('heading', { name: /short the input/ })).toBeNull();
  });

  it('given a lost device, when Close is pressed, then onclose is called', async () => {
    const { onclose } = setup(failed);

    await userEvent.click(screen.getByRole('button', { name: 'Close' }));

    expect(onclose).toHaveBeenCalledOnce();
  });
});
