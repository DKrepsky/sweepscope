import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get, writable, type Readable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { LayoutTier } from '../../app/layout';
import type { Notification } from '../../app/notifications';
import { createMockMainWindow, type MainWindowModel } from '../../app/main-window';
import { createTextScaleStore, setTextScale, type TextScalePercent } from '../../app/text-scale';
import { createRecordingContext } from '../../ui/graph/recording-context';
import MainWindow from './MainWindow.svelte';
import { spyActions } from './test-support';

const tiers = {
  w1: { width: 'w1', height: 'h1', rootFontPx: 16 },
  w2: { width: 'w2', height: 'h1', rootFontPx: 16 },
  w3: { width: 'w3', height: 'h1', rootFontPx: 16 },
  h2: { width: 'w1', height: 'h2', rootFontPx: 16 },
  h3: { width: 'w1', height: 'h3', rootFontPx: 16 },
  h4: { width: 'w3', height: 'h4', rootFontPx: 16 },
} satisfies Record<string, LayoutTier>;

beforeEach(() => {
  window.matchMedia = (() => ({ addEventListener: vi.fn(), removeEventListener: vi.fn() })) as never;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => createRecordingContext() as never);
});

function renderShell(
  tier: LayoutTier,
  overrides: {
    ontextscale?: (p: TextScalePercent) => void;
    notifications?: Readable<Notification[]>;
    model?: Readable<MainWindowModel>;
  } = {},
) {
  const props = {
    model: createMockMainWindow(),
    tier: writable(tier),
    textScale: writable<TextScalePercent>(100),
    notifications: writable<Notification[]>([]),
    onnotificationdismiss: () => {},
    actions: spyActions(),
    ontextscale: () => {},
    ...overrides,
  };
  return { ...render(MainWindow, { props }), tier: props.tier };
}

const drawerButton = () => screen.queryByRole('button', { name: 'Measurements drawer' });

describe('MainWindow tiers', () => {
  it('given a tier, when rendered, then the root carries the width and height tier attributes', () => {
    renderShell(tiers.w2);

    const root = screen.getByTestId('main-window');
    expect(root).toHaveAttribute('data-width-tier', 'w2');
    expect(root).toHaveAttribute('data-height-tier', 'h1');
  });

  it('given tier w1, when rendered, then the sidebar is docked and there is no drawer button', () => {
    renderShell(tiers.w1);

    expect(screen.getByRole('complementary', { name: /Cursors and measurements/ })).toBeInTheDocument();
    expect(drawerButton()).toBeNull();
  });

  it('given tier w3, when the drawer button is activated, then the sidebar drawer is shown and Escape closes it', async () => {
    const user = userEvent.setup();
    renderShell(tiers.w3);
    expect(screen.queryByRole('complementary')).toBeNull();

    await user.click(drawerButton()!);
    expect(screen.getByRole('region', { name: /drawer/ })).toBeInTheDocument();

    await user.keyboard('{Escape}');
    expect(screen.queryByRole('region', { name: /drawer/ })).toBeNull();
  });

  it('AC-SB1: given tier w3, when rendered, then connection and effective sample rate stay in the status bar', () => {
    const { container } = renderShell(tiers.w3);

    const status = container.querySelector<HTMLElement>('[data-region="status"]')!;
    expect(status.querySelector('[data-item="connection"]')).toBeInTheDocument();
    expect(status.querySelector('[data-item="sample-rate"]')).toBeInTheDocument();
  });

  it('given tier h2, when rendered, then the record overview is hidden but the readout stays', () => {
    renderShell(tiers.h2);

    expect(screen.queryByRole('img', { name: /Record overview/ })).toBeNull();
    expect(screen.getByText('Window')).toBeInTheDocument();
  });

  it('given tier h3, when rendered, then the Vertical group shows one channel row with channel tabs', () => {
    const { container } = renderShell(tiers.h3);

    expect(container.querySelectorAll('[data-channel]')).toHaveLength(1);
    expect(screen.getByRole('group', { name: 'Channel tabs' })).toBeInTheDocument();
  });

  it('given tier w1, when rendered, then the dock shows four channel rows', () => {
    const { container } = renderShell(tiers.w1);

    expect(container.querySelectorAll('[data-channel]')).toHaveLength(4);
  });
});

describe('MainWindow behaviour', () => {
  it('given the text-scale control, when 150 is selected, then SaveUiState is sent and the root font is applied', async () => {
    const send = vi.fn(async () => undefined);
    const client = { send } as never;
    const applyRootFontPercent = vi.fn();
    const store = createTextScaleStore({ client, applyRootFontPercent });
    const user = userEvent.setup();
    const { container } = render(MainWindow, {
      props: {
        model: createMockMainWindow(),
        tier: writable(tiers.w1),
        textScale: store,
        notifications: writable<Notification[]>([]),
        onnotificationdismiss: () => {},
        actions: spyActions(),
        ontextscale: (percent: TextScalePercent) => void setTextScale(store, percent),
      },
    });

    await user.click(within(container).getByRole('button', { name: 'Settings' }));
    await user.click(screen.getByRole('button', { name: '150 %' }));

    await vi.waitFor(() => expect(applyRootFontPercent).toHaveBeenCalledWith(150));
    expect(send).toHaveBeenCalledWith('save_ui_state', expect.anything());
    expect(get(store)).toBe(150);
  });

  it('given the Settings button, when it is activated, then focus moves into the popover', async () => {
    const user = userEvent.setup();
    renderShell(tiers.w1);

    await user.click(screen.getByRole('button', { name: 'Settings' }));

    expect(screen.getByRole('dialog', { name: 'Settings' })).toHaveFocus();
  });

  it('given the Settings popover open with focus on the opener, when Escape is pressed, then it closes', async () => {
    const user = userEvent.setup();
    renderShell(tiers.w1);
    const opener = screen.getByRole('button', { name: 'Settings' });
    await user.click(opener);
    opener.focus();

    await user.keyboard('{Escape}');

    expect(screen.queryByRole('dialog', { name: 'Settings' })).toBeNull();
  });

  it('given the Settings popover with focus inside, when Escape closes it, then focus returns to the Settings button', async () => {
    const user = userEvent.setup();
    renderShell(tiers.w1);
    await user.click(screen.getByRole('button', { name: 'Settings' }));

    await user.keyboard('{Escape}');

    expect(screen.getByRole('button', { name: 'Settings' })).toHaveFocus();
  });

  it('given an error notification, when rendered, then it is announced as an alert', () => {
    renderShell(tiers.w1, {
      notifications: writable<Notification[]>([{ id: 1, kind: 'error', text: 'Could not save text size' }]),
    });

    expect(screen.getByRole('alert')).toHaveTextContent('Could not save text size');
  });

  it('given two notifications, when rendered, then both show with the first one on top', () => {
    renderShell(tiers.w1, {
      notifications: writable<Notification[]>([
        { id: 2, kind: 'warning', text: 'newest' },
        { id: 1, kind: 'info', text: 'older' },
      ]),
    });

    const texts = screen.getAllByRole('status').map((element) => element.textContent?.trim());
    expect(texts.indexOf('newest')).toBeLessThan(texts.indexOf('older'));
  });

  it('given no notifications, when rendered, then there is no alert', () => {
    renderShell(tiers.w1);

    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('given tier h4, when rendered, then the dock shows one group at a time and the Trigger group is one click away', async () => {
    const { container } = renderShell(tiers.h4);
    expect(container.querySelector('[data-region="trigger"]')).toBeNull();

    await userEvent.setup().click(screen.getByRole('button', { name: 'Trigger' }));

    expect(container.querySelector('[data-region="trigger"]')).not.toBeNull();
    expect(container.querySelector('[data-region="vertical"]')).toBeNull();
  });

  it('given tier h4, when rendered, then the scale strip carries the trigger summary', () => {
    const { container } = renderShell(tiers.h4);

    expect(container.querySelector('[data-trigger-summary]')).toHaveTextContent(/^Trigger CH\d/);
  });

  it('given tier h3, when rendered, then only connection, sample rate and the calibration tag remain in the status bar', () => {
    const { container } = renderShell({ ...tiers.h3, width: 'w3' });

    const items = [...container.querySelectorAll('[data-item]')].map((item) => item.getAttribute('data-item'));
    expect(items).toEqual(['connection', 'sample-rate', 'calibration']);
  });

  it('given the mock model, when rendered, then no placeholder note sits next to the numbers', () => {
    renderShell(tiers.w1);

    expect(screen.queryByText('Illustrative numbers (placeholders)')).toBeNull();
  });
});

describe('MainWindow verification markers', () => {
  it('given the shell, when rendered, then no Unverified badge is on screen', () => {
    renderShell(tiers.w1);

    expect(screen.queryByText('Unverified')).not.toBeInTheDocument();
  });

  it('given the component sources, when scanned, then none reads a verified or unverified field', () => {
    const sources = import.meta.glob('../../**/*.svelte', { query: '?raw', import: 'default', eager: true });

    const readers = Object.entries(sources)
      .filter(([, text]) => /\.(un)?verified\b/i.test(text as string))
      .map(([path]) => path);

    expect(Object.keys(sources).length).toBeGreaterThan(50);
    expect(readers).toEqual([]);
  });
});

describe('MainWindow keyboard', () => {
  it('AC-I12: given the rendered shell, when Tab is pressed repeatedly, then focus visits the regions in front-panel order', async () => {
    const user = userEvent.setup();
    renderShell(tiers.w1);
    const visited: string[] = [];

    for (let press = 0; press < 120; press += 1) {
      await user.tab();
      const region = document.activeElement?.closest<HTMLElement>('[data-region]')?.dataset.region;
      if (region && visited.at(-1) !== region) visited.push(region);
    }

    expect(visited.slice(0, 8)).toEqual([
      'toolbar',
      'scope',
      'scale-strip',
      'vertical',
      'horizontal',
      'trigger',
      'sidebar',
      'status',
    ]);
  });
});
