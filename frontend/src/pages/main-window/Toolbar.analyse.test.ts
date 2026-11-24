import { screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import type { MainWindowModel, ToolbarModel } from '../../app/main-window';
import { renderToolbar } from './toolbar-test-support';

const setup = (
  change?: (toolbar: ToolbarModel) => ToolbarModel,
  changeModel?: (model: MainWindowModel) => MainWindowModel,
) => {
  const { calls, user } = renderToolbar(change, changeModel);
  return { ...calls, user };
};

const disabled = (element: HTMLElement) => element.getAttribute('aria-disabled') === 'true';

describe('single', () => {
  const armed = (toolbar: ToolbarModel): ToolbarModel => ({
    ...toolbar,
    single: { armed: true, availability: { enabled: true } },
  });

  it('given an idle Single, when it is clicked, then one capture is armed', async () => {
    const { onarmsingle, ontogglerun, user } = setup();

    await user.click(screen.getByRole('button', { name: 'Single' }));

    expect([onarmsingle.mock.calls.length, ontogglerun.mock.calls.length]).toEqual([1, 0]);
  });

  it('given an armed Single, when it is clicked again, then the run is cancelled instead of arming again', async () => {
    const { onarmsingle, ontogglerun, user } = setup(armed);
    const button = screen.getByRole('button', { name: 'Single' });

    await user.click(button);

    expect(button).toHaveAttribute('aria-pressed', 'true');
    expect([onarmsingle.mock.calls.length, ontogglerun.mock.calls.length]).toEqual([0, 1]);
  });

  it('given Single is unavailable with a reason, when it is clicked, then nothing is requested', async () => {
    const { onarmsingle, user } = setup((toolbar) => ({
      ...toolbar,
      single: { armed: false, availability: { enabled: false, reason: 'Auto-setup is running' } },
    }));
    const button = screen.getByRole('button', { name: 'Single' });

    await user.click(button);

    expect([disabled(button), onarmsingle.mock.calls.length]).toEqual([true, 0]);
  });
});

describe('autoset', () => {
  it('given an idle scope, when Autoset is clicked, then auto-setup is requested', async () => {
    const { onautoset, user } = setup();

    await user.click(screen.getByRole('button', { name: 'Autoset' }));

    expect(onautoset).toHaveBeenCalledOnce();
  });

  it('given auto-setup is running, when Autoset is clicked, then it is busy and disabled and requests nothing', async () => {
    const { onautoset, user } = setup(undefined, (model) => ({
      ...model,
      autoset: { ...model.autoset, busy: true, available: { enabled: false, reason: 'Auto-setup is running' } },
    }));
    const button = screen.getByRole('button', { name: 'Autoset' });

    await user.click(button);

    expect(button).toHaveAttribute('aria-busy', 'true');
    expect([disabled(button), onautoset.mock.calls.length]).toEqual([true, 0]);
  });

  it('given a busy run while the backend still says available, when Autoset is clicked, then it requests nothing', async () => {
    const { onautoset, user } = setup(undefined, (model) => ({ ...model, autoset: { ...model.autoset, busy: true } }));

    await user.click(screen.getByRole('button', { name: 'Autoset' }));

    expect(onautoset).not.toHaveBeenCalled();
  });
});

describe('view selector', () => {
  it('given the view selector, when XY is chosen, then the view change is requested', async () => {
    const { onsetview, user } = setup();

    await user.click(screen.getByRole('button', { name: 'XY' }));

    expect(onsetview.mock.calls).toEqual([['xy']]);
  });

  it('given roll mode, when the view selector is read, then XY and FFT are disabled', () => {
    const reason = 'XY, FFT, persistence, math and cursors need a captured record; not available in roll mode.';
    setup(undefined, (model) => ({
      ...model,
      views: {
        ...model.views,
        tabAvailability: { yt: { enabled: true }, xy: { enabled: false, reason }, fft: { enabled: false, reason } },
      },
    }));

    expect([
      disabled(screen.getByRole('button', { name: 'XY' })),
      disabled(screen.getByRole('button', { name: /FFT/ })),
    ]).toEqual([true, true]);
  });
});

describe('persistence menu', () => {
  const at = (value: MainWindowModel['persistence']['value']) => (model: MainWindowModel) => ({
    ...model,
    persistence: { ...model.persistence, value },
  });

  it('given persistence 1 s, when the button is read, then it shows the value and looks on', () => {
    setup(undefined, at('1s'));

    expect(screen.getByRole('button', { name: /Persist: 1 s/ })).toHaveAttribute('aria-pressed', 'true');
  });

  it('given persistence off, when the button is read, then it shows Off and does not look on', () => {
    setup();

    expect(screen.getByRole('button', { name: /Persist: Off/ })).toHaveAttribute('aria-pressed', 'false');
  });

  it('given persistence 2 s, when the menu is opened, then the five values are radios with 2 s checked and Clear follows', async () => {
    const { user } = setup(undefined, at('2s'));

    await user.click(screen.getByRole('button', { name: /Persist: 2 s/ }));

    const radios = screen.getAllByRole('menuitemradio');
    const rows = radios.map((radio) => [
      radio.textContent?.replace('✓', '').trim(),
      radio.getAttribute('aria-checked'),
    ]);
    expect(rows).toEqual([
      ['Off', 'false'],
      ['0.5 s', 'false'],
      ['1 s', 'false'],
      ['2 s', 'true'],
      ['Infinite', 'false'],
    ]);
    expect(screen.getByRole('menuitem', { name: 'Clear persistence' })).toBeInTheDocument();
  });

  it('given the menu is open, when 1 s is chosen, then exactly one persistence change is requested', async () => {
    const { onsetpersistence, onclearpersistence, user } = setup();
    await user.click(screen.getByRole('button', { name: /Persist: Off/ }));

    await user.click(screen.getByRole('menuitemradio', { name: /^1 s/ }));

    expect([onsetpersistence.mock.calls, onclearpersistence.mock.calls.length]).toEqual([[['1s']], 0]);
  });

  it('given the menu is open, when Clear persistence is chosen, then only the clear is requested', async () => {
    const { onsetpersistence, onclearpersistence, user } = setup();
    await user.click(screen.getByRole('button', { name: /Persist: Off/ }));

    await user.click(screen.getByRole('menuitem', { name: 'Clear persistence' }));

    expect([onsetpersistence.mock.calls.length, onclearpersistence.mock.calls.length]).toEqual([0, 1]);
  });

  it('given XY, when the persist button is read, then it is disabled', () => {
    setup(undefined, (model) => ({
      ...model,
      persistence: {
        ...model.persistence,
        available: { enabled: false, reason: 'Persistence is available in Yt only' },
      },
    }));

    expect(disabled(screen.getByRole('button', { name: /Persist: Off/ }))).toBe(true);
  });
});

describe('math tool', () => {
  it('given the Math tool, when it is clicked, then the toggle is requested', async () => {
    const { ontogglemath, user } = setup();

    await user.click(screen.getByRole('button', { name: /^Math/ }));

    expect(ontogglemath).toHaveBeenCalledOnce();
  });

  it('given math is on, when the Math tool is read, then it is pressed', () => {
    setup((toolbar) => ({ ...toolbar, tools: { ...toolbar.tools, math: true } }));

    expect(screen.getByRole('button', { name: /^Math/ })).toHaveAttribute('aria-pressed', 'true');
  });

  it('given roll mode, when the Math tool is read, then it is disabled', () => {
    setup((toolbar) => ({ ...toolbar, mathAvailability: { enabled: false, reason: 'Not available in roll mode.' } }));

    expect(disabled(screen.getByRole('button', { name: /^Math/ }))).toBe(true);
  });
});

describe('record toggle', () => {
  const available = (model: MainWindowModel): MainWindowModel => ({
    ...model,
    recording: { ...model.recording, available: { enabled: true } },
  });

  it('given a scope outside roll, when the Record button is read, then it is disabled', () => {
    setup();

    expect(disabled(screen.getByRole('button', { name: 'Record' }))).toBe(true);
  });

  it('given roll mode, when Record is clicked, then recording starts', async () => {
    const { onstartrecording, user } = setup(undefined, available);

    await user.click(screen.getByRole('button', { name: 'Record' }));

    expect(onstartrecording).toHaveBeenCalledOnce();
  });

  it('given a recording, when Stop rec is clicked, then recording stops and does not start again', async () => {
    const { onstartrecording, onstoprecording, user } = setup(undefined, (model) => ({
      ...available(model),
      recording: { ...model.recording, active: true, available: { enabled: true } },
    }));

    await user.click(screen.getByRole('button', { name: 'Stop rec' }));

    expect([onstartrecording.mock.calls.length, onstoprecording.mock.calls.length]).toEqual([0, 1]);
  });

  it('given a recording whose availability fell, when Stop rec is clicked, then it still stops', async () => {
    const { onstoprecording, user } = setup(undefined, (model) => ({
      ...model,
      recording: { ...model.recording, active: true },
    }));

    await user.click(screen.getByRole('button', { name: 'Stop rec' }));

    expect(onstoprecording).toHaveBeenCalledOnce();
  });
});

describe('status chips', () => {
  it('given a roll chip, when rendered, then it shows Roll', () => {
    setup((toolbar) => ({ ...toolbar, chips: [{ id: 'mode', kind: 'roll', text: 'Roll' }] }));

    expect(screen.getByText('Roll')).toBeInTheDocument();
  });

  it('given a record chip, when rendered, then it shows its text', () => {
    setup((toolbar) => ({ ...toolbar, chips: [{ id: 'rec', kind: 'record', text: 'Rec' }] }));

    expect(screen.getByText('Rec')).toBeInTheDocument();
  });
});
