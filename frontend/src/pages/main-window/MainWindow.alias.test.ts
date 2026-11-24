import { render, screen, within } from '@testing-library/svelte';
import { get, writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Notification } from '../../app/notifications';
import { createMockMainWindow, type MainWindowModel } from '../../app/main-window';
import type { TextScalePercent } from '../../app/text-scale';
import { createRecordingContext } from '../../ui/graph/recording-context';
import MainWindow from './MainWindow.svelte';
import { spyActions } from './test-support';

const MESSAGE = 'Possible aliasing on CH1. Its frequency (62.0 MHz) is more than one fifth of the sample rate.';
const alias = { short: 'Alias CH1', message: MESSAGE };

beforeEach(() => {
  window.matchMedia = (() => ({ addEventListener: vi.fn(), removeEventListener: vi.fn() })) as never;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => createRecordingContext() as never);
});

function withAlias(base: MainWindowModel, on: boolean): MainWindowModel {
  const column = (c: MainWindowModel['measurements']['columns'][number]) =>
    c.id === 'ch1' && on ? { ...c, alias } : c;
  return {
    ...base,
    channels: base.channels.map((c) => (c.number === 1 && on ? { ...c, enabled: true, alias } : c)),
    status: { ...base.status, sampleRate: { ...base.status.sampleRate, ...(on && { alias }) } },
    measurements: {
      ...base.measurements,
      columns: base.measurements.columns.map(column),
      footnotes: on ? [{ key: 1, text: MESSAGE }] : [],
    },
  };
}

function renderWith(on: boolean) {
  const base = get(createMockMainWindow());
  const model = writable(withAlias(base, on));
  const view = render(MainWindow, {
    props: {
      model: { subscribe: model.subscribe },
      tier: writable({ width: 'w1', height: 'h1', rootFontPx: 16 } as const),
      textScale: writable<TextScalePercent>(100),
      notifications: writable<Notification[]>([]),
      onnotificationdismiss: () => {},
      actions: spyActions(),
      ontextscale: () => {},
    },
  });
  return { ...view, update: (next: boolean) => model.set(withAlias(base, next)) };
}

describe('alias warning in the main window', () => {
  it('given a warning, when rendered, then the status bar, the scale chip and the column header show it', () => {
    const { container } = renderWith(true);

    const status = container.querySelector<HTMLElement>('[data-region="status"]')!;
    expect(within(status).getByText('Alias CH1')).toBeInTheDocument();
    const strip = container.querySelector<HTMLElement>('[data-region="scale-strip"]')!;
    expect(within(strip).getByText('Alias')).toBeInTheDocument();
    expect(screen.getAllByText('Alias').length).toBeGreaterThanOrEqual(2);
  });

  it('given a warning, when rendered, then every badge and the status item share the one message', () => {
    const { container } = renderWith(true);

    const badges = screen.getAllByText('Alias').map((badge) => badge.closest('[role="note"]')!);
    expect(badges.length).toBeGreaterThanOrEqual(2);
    badges.forEach((badge) => expect(badge).toHaveTextContent(MESSAGE));
    expect(container.querySelector(`[data-region="status"] [title="${MESSAGE}"]`)).toBeInTheDocument();
  });

  it('given the warning clears, when the model updates, then all places clear together', async () => {
    const { container, update } = renderWith(true);

    update(false);
    await Promise.resolve();

    expect(screen.queryByText('Alias')).toBeNull();
    expect(container.querySelector('[data-region="status"]')).not.toHaveTextContent('Alias');
  });
});
