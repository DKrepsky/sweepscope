import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get, writable, type Writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { LayoutTier } from '../../app/layout';
import { createMockMainWindow, type MainWindowModel } from '../../app/main-window';
import type { Notification } from '../../app/notifications';
import type { TextScalePercent } from '../../app/text-scale';
import { createRecordingContext } from '../../ui/graph/recording-context';
import MainWindow from './MainWindow.svelte';
import { calledNames, spyActions, type SpyActions } from './test-support';

const tier: LayoutTier = { width: 'w1', height: 'h1', rootFontPx: 16 };

beforeEach(() => {
  window.matchMedia = (() => ({ addEventListener: vi.fn(), removeEventListener: vi.fn() })) as never;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => createRecordingContext() as never);
});

function setup(change: (model: MainWindowModel) => MainWindowModel = (model) => model) {
  const actions = spyActions();
  const model: Writable<MainWindowModel> = writable(change(get(createMockMainWindow())));
  render(MainWindow, {
    props: {
      model,
      actions,
      tier: writable(tier),
      textScale: writable<TextScalePercent>(100),
      notifications: writable<Notification[]>([]),
      onnotificationdismiss: () => {},
      ontextscale: () => {},
    },
  });
  return { actions, model, user: userEvent.setup() };
}

const onlyCall = (actions: SpyActions) => calledNames(actions);

const dock = (region: string) => document.querySelector<HTMLElement>(`[data-region="${region}"]`)!;
const row = (channel: number) => document.querySelector<HTMLElement>(`[data-channel="${channel}"]`)!;

describe('toolbar actions', () => {
  it('given the toolbar, when Run or Stop is clicked, then toggleRun is the only action called', async () => {
    const { actions, user } = setup();
    await user.click(screen.getByRole('button', { name: /Stop/ }));
    expect(onlyCall(actions)).toEqual(['toggleRun']);
  });

  it('given the toolbar, when Force is clicked, then forceTrigger is the only action called', async () => {
    const { actions, user } = setup();
    await user.click(screen.getByRole('button', { name: 'Force' }));
    expect(onlyCall(actions)).toEqual(['forceTrigger']);
  });

  it('given run is unavailable with a reason, when Run is focused, then the reason is the tooltip and a click does nothing', async () => {
    const { actions, user } = setup((model) => ({
      ...model,
      toolbar: { ...model.toolbar, availability: { enabled: false, reason: 'No scope connected' } },
    }));
    const run = screen.getByRole('button', { name: /Stop/ });

    await user.click(run);
    run.focus();

    expect(run).toHaveAttribute('aria-disabled', 'true');
    expect(await screen.findByRole('tooltip')).toHaveTextContent('No scope connected');
    expect(onlyCall(actions)).toEqual([]);
  });

  it('given force is unavailable with a reason, when Force is clicked, then nothing is called and it carries the reason', async () => {
    const { actions, user } = setup((model) => ({
      ...model,
      toolbar: { ...model.toolbar, force: { enabled: false, reason: 'Force needs a running scope' } },
    }));
    const force = screen.getByRole('button', { name: 'Force' });

    await user.click(force);
    force.focus();

    expect(await screen.findByRole('tooltip')).toHaveTextContent('Force needs a running scope');
    expect(onlyCall(actions)).toEqual([]);
  });
});

describe('vertical actions', () => {
  it('given a channel row, when its number toggle is clicked, then setChannelEnabled gets the opposite state', async () => {
    const { actions, user } = setup();
    await user.click(within(row(2)).getByRole('button', { name: 'Channel 2 enabled' }));
    expect(actions.setChannelEnabled).toHaveBeenCalledExactlyOnceWith(2, false);
  });

  it('given a channel that cannot be toggled, when its number toggle is clicked, then nothing is called and the reason is the title', async () => {
    const { actions, user } = setup((model) => ({
      ...model,
      channels: model.channels.map((c) =>
        c.number === 3 ? { ...c, enableReason: 'Only two channels at this rate' } : c,
      ),
    }));
    const toggle = within(row(3)).getByRole('button', { name: 'Channel 3 enabled' });

    await user.click(toggle);

    expect(toggle).toHaveAttribute('title', 'Only two channels at this rate');
    expect(onlyCall(actions)).toEqual([]);
  });

  it('given a channel row, when its volts per division is stepped, then stepVoltsPerDiv gets the channel and direction', async () => {
    const { actions, user } = setup();
    const stepper = within(row(1));
    await user.click(stepper.getByRole('button', { name: 'Increase' }));
    await user.click(stepper.getByRole('button', { name: 'Decrease' }));
    expect(actions.stepVoltsPerDiv.mock.calls).toEqual([
      [1, 1],
      [1, -1],
    ]);
  });

  it('given a channel row, when its volts per division is double-clicked, then resetVoltsPerDiv is called for it', async () => {
    const { actions, user } = setup();
    await user.dblClick(within(row(1)).getByRole('spinbutton', { name: /volts per division/ }));
    expect(actions.resetVoltsPerDiv).toHaveBeenCalledExactlyOnceWith(1);
  });

  it('given the slower direction is unavailable, when increase is clicked, then no step is sent and the reason is shown', async () => {
    const { actions, user } = setup((model) => ({
      ...model,
      channels: model.channels.map((c) =>
        c.number === 1
          ? {
              ...c,
              voltsPerDiv: {
                ...c.voltsPerDiv,
                step: { faster: { enabled: true }, slower: { enabled: false, reason: 'Largest range' } },
              },
            }
          : c,
      ),
    }));
    const increase = within(row(1)).getByRole('button', { name: 'Increase' });

    await user.click(increase);
    within(row(1))
      .getByRole('spinbutton', { name: /volts per division/ })
      .focus();

    expect(onlyCall(actions)).toEqual([]);
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Largest range');
  });

  it('given a position typed in the field, when it is committed, then commitChannelPosition gets the text', async () => {
    const { actions, user } = setup();
    const field = within(row(1)).getByRole('textbox', { name: /vertical position/ });
    await user.clear(field);
    await user.type(field, '1.5{Enter}');
    expect(actions.commitChannelPosition).toHaveBeenCalledExactlyOnceWith(1, '1.5');
  });

  it('given a channel row, when coupling is chosen, then setCoupling gets the channel and coupling', async () => {
    const { actions, user } = setup();
    const current = get(createMockMainWindow()).channels[0]!.coupling;
    const other = current === 'ac' ? 'DC' : 'AC';
    await user.click(within(row(1)).getByRole('button', { name: other }));
    expect(actions.setCoupling).toHaveBeenCalledExactlyOnceWith(1, other.toLowerCase());
  });

  it('given a channel row, when BW, probe and invert are clicked, then each calls its own action', async () => {
    const { actions, user } = setup();
    const scope = within(row(2));
    await user.click(scope.getByRole('button', { name: /bandwidth limit/ }));
    await user.click(scope.getByRole('button', { name: /probe attenuation/ }));
    await user.click(scope.getByRole('button', { name: /invert/ }));
    expect(calledNames(actions)).toEqual(['toggleBandwidth', 'cycleProbe', 'toggleInvert']);
    expect(actions.toggleBandwidth).toHaveBeenCalledWith(2);
    expect(actions.cycleProbe).toHaveBeenCalledWith(2);
    expect(actions.toggleInvert).toHaveBeenCalledWith(2);
  });

  it('given a channel row, when a spot on the row that is not a control is clicked, then selectChannel is called for it', async () => {
    const { actions, user } = setup();
    await user.click(row(3));
    expect(actions.selectChannel).toHaveBeenCalledExactlyOnceWith(3);
  });

  it('given the vertical group is unavailable, when its controls are clicked, then nothing is called and the reason is the tooltip', async () => {
    const reason = 'No scope connected';
    const { actions, user } = setup((model) => ({
      ...model,
      channels: model.channels.map((c) => ({ ...c, availability: { enabled: false, reason }, enableReason: reason })),
    }));
    const invert = within(row(1)).getByRole('button', { name: /invert/ });

    await user.click(invert);
    await user.click(within(row(1)).getByRole('button', { name: 'Channel 1 enabled' }));
    invert.focus();

    expect(onlyCall(actions)).toEqual([]);
    expect(await screen.findByRole('tooltip')).toHaveTextContent(reason);
  });
});

describe('horizontal actions', () => {
  it('given the horizontal group, when time per division is stepped and reset, then the time actions are called', async () => {
    const { actions, user } = setup();
    const group = within(dock('horizontal'));
    await user.click(group.getAllByRole('button', { name: 'Decrease' })[0]!);
    await user.dblClick(group.getByRole('spinbutton', { name: 'Time per division' }));
    expect(actions.stepTimePerDiv).toHaveBeenCalledWith(-1);
    expect(actions.resetTimePerDiv).toHaveBeenCalledOnce();
  });

  it('given the horizontal group, when trigger position is stepped, then stepTriggerPosition gets the direction', async () => {
    const { actions, user } = setup();
    const stepper = screen.getByRole('spinbutton', { name: /Trigger position/ }).parentElement!;
    await user.click(within(stepper).getByRole('button', { name: 'Increase' }));
    expect(actions.stepTriggerPosition).toHaveBeenCalledExactlyOnceWith(1);
  });

  it('given the horizontal group, when an interpolation is chosen, then setInterpolation is called with the mode', async () => {
    const { actions, user } = setup();
    await user.click(screen.getByRole('button', { name: 'Dots' }));
    expect(actions.setInterpolation).toHaveBeenCalledExactlyOnceWith('dots');
  });

  it('given the horizontal group is unavailable, when time per division is stepped, then nothing is called and the reason shows', async () => {
    const { actions, user } = setup((model) => ({
      ...model,
      horizontal: { ...model.horizontal, availability: { enabled: false, reason: 'Roll mode' } },
    }));
    const group = within(dock('horizontal'));

    await user.click(group.getAllByRole('button', { name: 'Increase' })[0]!);
    group.getByRole('spinbutton', { name: 'Time per division' }).focus();

    expect(onlyCall(actions)).toEqual([]);
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Roll mode');
  });
});

describe('trigger actions', () => {
  it('given the trigger group, when level is stepped and reset, then the level actions are called', async () => {
    const { actions, user } = setup();
    const group = within(dock('trigger'));
    await user.click(group.getByRole('button', { name: 'Increase' }));
    await user.dblClick(group.getByRole('spinbutton', { name: 'Trigger level' }));
    expect(actions.stepTriggerLevel).toHaveBeenCalledExactlyOnceWith(1);
    expect(actions.resetTriggerLevel).toHaveBeenCalledOnce();
  });

  it('given the trigger group, when source, slope and sweep are chosen, then each calls its action', async () => {
    const { actions, user } = setup();
    await user.click(within(screen.getByRole('group', { name: 'Trigger source' })).getByRole('button', { name: '2' }));
    await user.click(
      within(screen.getByRole('group', { name: 'Trigger slope' })).getByRole('button', { name: /Falling/ }),
    );
    await user.click(
      within(screen.getByRole('group', { name: 'Trigger sweep' })).getByRole('button', { name: 'Normal' }),
    );
    expect(actions.setTriggerSource).toHaveBeenCalledWith(2);
    expect(actions.setTriggerSlope).toHaveBeenCalledWith('falling');
    expect(actions.setTriggerSweep).toHaveBeenCalledWith('normal');
  });

  it('given the trigger group is unavailable, when a sweep is chosen, then nothing is called and the reason shows', async () => {
    const { actions, user } = setup((model) => ({
      ...model,
      trigger: { ...model.trigger, availability: { enabled: false, reason: 'No scope connected' } },
    }));
    const sweep = within(screen.getByRole('group', { name: 'Trigger sweep' })).getByRole('button', { name: 'Normal' });

    await user.click(sweep);
    await user.hover(sweep);

    expect(onlyCall(actions)).toEqual([]);
    expect(await screen.findByRole('tooltip')).toHaveTextContent('No scope connected');
  });
});

describe('marker drags', () => {
  it('given a ground marker, when it is nudged up, then setChannelPosition gets divisions from the centre', async () => {
    const { actions, user } = setup();
    const marker = screen.getByRole('slider', { name: 'Channel 1 ground' });
    const fraction = Number(marker.getAttribute('aria-valuenow'));

    marker.focus();
    await user.keyboard('{Alt>}{ArrowUp}{/Alt}');

    expect(actions.setChannelPosition).toHaveBeenCalledOnce();
    const [channel, divisions] = actions.setChannelPosition.mock.calls[0]!;
    expect(channel).toBe(1);
    expect(divisions).toBeGreaterThan((fraction - 0.5) * 8);
  });

  it('given the trigger level marker, when it is nudged to the top, then setTriggerLevel gets divisions from the centre', async () => {
    const { actions, user } = setup((model) => ({ ...model, trigger: { ...model.trigger, levelPosition: 1 } }));
    const marker = screen.getByRole('slider', { name: 'Trigger level' });

    marker.focus();
    await user.keyboard('{Alt>}{ArrowDown}{/Alt}');

    expect(actions.setTriggerLevel).toHaveBeenCalledOnce();
    expect(actions.setTriggerLevel.mock.calls[0]![0]).toBeLessThan(4);
  });

  it('given the trigger position marker, when it is nudged right, then setTriggerPosition gets the screen fraction', async () => {
    const { actions, user } = setup();
    const marker = screen.getByRole('slider', { name: 'Trigger position' });
    const fraction = Number(marker.getAttribute('aria-valuenow'));

    marker.focus();
    await user.keyboard('{Alt>}{ArrowRight}{/Alt}');

    expect(actions.setTriggerPosition).toHaveBeenCalledOnce();
    expect(actions.setTriggerPosition.mock.calls[0]![0]).toBeGreaterThan(fraction);
  });

  it('given the trigger level marker, when it is double-clicked, then resetTriggerLevel is called', async () => {
    const { actions, user } = setup();
    await user.dblClick(screen.getByRole('slider', { name: 'Trigger level' }));
    expect(actions.resetTriggerLevel).toHaveBeenCalledOnce();
  });

  it('given the trigger is unavailable, when its markers are nudged, then nothing is called and they carry the reason', async () => {
    const { actions, user } = setup((model) => ({
      ...model,
      trigger: { ...model.trigger, availability: { enabled: false, reason: 'No scope connected' } },
    }));
    const marker = screen.getByRole('slider', { name: 'Trigger level' });

    marker.focus();
    await user.keyboard('{Alt>}{ArrowUp}{/Alt}');

    expect(marker).toHaveAttribute('title', 'No scope connected');
    expect(onlyCall(actions)).toEqual([]);
  });

  it('given a channel is unavailable, when its ground marker is nudged, then nothing is called', async () => {
    const { actions, user } = setup((model) => ({
      ...model,
      channels: model.channels.map((c) => ({ ...c, availability: { enabled: false, reason: 'No scope connected' } })),
    }));
    const marker = screen.getByRole('slider', { name: 'Channel 1 ground' });

    marker.focus();
    await user.keyboard('{Alt>}{ArrowUp}{/Alt}');

    expect(onlyCall(actions)).toEqual([]);
  });

  it('given a ground marker, when it is double-clicked, then the channel position returns to the centre', async () => {
    const { actions, user } = setup();
    await user.dblClick(screen.getByRole('slider', { name: 'Channel 1 ground' }));
    expect(actions.setChannelPosition).toHaveBeenCalledExactlyOnceWith(1, 0);
  });
});

describe('controls without an action yet', () => {
  it.each(['Single', 'Autoset'])(
    'given the toolbar, when %s is focused, then it is disabled and says it arrives later',
    async (name) => {
      const { actions, user } = setup();
      const button = screen.getByRole('button', { name });

      await user.click(button);
      button.focus();

      expect(button).toHaveAttribute('aria-disabled', 'true');
      expect(await screen.findByRole('tooltip')).toHaveTextContent('Arrives in a later release');
      expect(onlyCall(actions)).toEqual([]);
    },
  );
});

describe('banners', () => {
  const banner =
    (actions: { id: 'copy-command' | 'retry' | 'use-simulator'; label: string }[]) => (model: MainWindowModel) => ({
      ...model,
      banners: [{ id: 'connection', kind: 'warning' as const, text: 'No scope found.', actions }],
    });

  it('given a banner with actions, when rendered, then each action is a button with its label', () => {
    setup(
      banner([
        { id: 'retry', label: 'Retry' },
        { id: 'use-simulator', label: 'Use simulator' },
      ]),
    );
    const buttons = within(screen.getByText('No scope found.').closest<HTMLElement>('.banner')!).getAllByRole('button');
    expect(buttons.map((b) => b.textContent?.trim())).toEqual(['Retry', 'Use simulator']);
  });

  it('given a banner action, when it is clicked, then bannerAction gets its id', async () => {
    const { actions, user } = setup(banner([{ id: 'copy-command', label: 'Copy command' }]));
    await user.click(screen.getByRole('button', { name: 'Copy command' }));
    expect(actions.bannerAction).toHaveBeenCalledExactlyOnceWith('copy-command');
  });
});

describe('status bar', () => {
  it('given a simulated connection, when rendered, then the connection item says Simulator', () => {
    setup((model) => ({
      ...model,
      status: { ...model.status, connection: { ...model.status.connection, simulated: true } },
    }));
    expect(document.querySelector('[data-item="connection"]')).toHaveTextContent('Simulator');
  });

  it('given a connection in error, when rendered, then the connection chip is an error chip', () => {
    setup((model) => ({
      ...model,
      status: {
        ...model.status,
        connection: { ...model.status.connection, state: 'Error', detail: 'Permission denied' },
      },
    }));
    expect(document.querySelector('[data-item="connection"]')).toHaveTextContent('Error');
    expect(document.querySelector('[data-item="connection"] .chip.error')).not.toBeNull();
  });
});
