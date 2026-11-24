import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { MeasurementsModel } from '../../app/main-window';
import MeasurementsPanel from './MeasurementsPanel.svelte';

const model = (overrides: Partial<MeasurementsModel> = {}): MeasurementsModel => ({
  columns: [{ id: 'ch1', label: 'CH1', color: 'color-ch1' }],
  rows: [{ id: 'vpp', label: 'Vpp', cells: { ch1: { state: 'valid', text: '1.91', unit: 'V' } } }],
  footnotes: [],
  accuracyNote: 'Volts accurate to 2 to 3 counts.',
  pinnable: [
    { id: 'vpp', label: 'Vpp', pinned: true },
    { id: 'period', label: 'Period', pinned: false },
  ],
  ...overrides,
});

describe('MeasurementsPanel', () => {
  it('given an accuracy note, when rendered, then it shows under the table', () => {
    render(MeasurementsPanel, { props: { measurements: model(), onpin: vi.fn() } });

    expect(screen.getByText('Volts accurate to 2 to 3 counts.')).toBeInTheDocument();
  });

  it('given an unavailable note, when rendered, then the reason shows as text', () => {
    const measurements = model({
      columns: [],
      rows: [],
      unavailableNote: 'Measurements are not available in roll mode.',
    });

    render(MeasurementsPanel, { props: { measurements, onpin: vi.fn() } });

    expect(screen.getByText('Measurements are not available in roll mode.')).toBeInTheDocument();
  });

  it('given an alias column, when rendered, then its header carries the Alias badge and the footnote shows', () => {
    const alias = { short: 'Alias CH1', message: 'Possible aliasing on CH1.' };
    const measurements = model({
      columns: [{ id: 'ch1', label: 'CH1', color: 'color-ch1', alias }],
      footnotes: [{ key: 1, text: alias.message }],
    });

    render(MeasurementsPanel, { props: { measurements, onpin: vi.fn() } });

    expect(screen.getByText('Alias')).toBeInTheDocument();
    expect(screen.getByRole('listitem')).toHaveTextContent(alias.message);
  });

  it('given a frozen model, when rendered, then the Frozen label shows', () => {
    render(MeasurementsPanel, { props: { measurements: model({ frozenLabel: 'Frozen' }), onpin: vi.fn() } });

    expect(screen.getByText('Frozen')).toBeInTheDocument();
  });

  it('given the pin menu, when opened, then the pinned row is checked and the other is not', async () => {
    render(MeasurementsPanel, { props: { measurements: model(), onpin: vi.fn() } });

    await userEvent.click(screen.getByRole('button', { name: 'Edit measurements' }));

    expect(screen.getByRole('menuitemcheckbox', { name: /Vpp/ })).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('menuitemcheckbox', { name: /Period/ })).toHaveAttribute('aria-checked', 'false');
  });

  it('given the pin menu, when an unpinned row is chosen, then onpin gets its id and true', async () => {
    const onpin = vi.fn();
    render(MeasurementsPanel, { props: { measurements: model(), onpin } });

    await userEvent.click(screen.getByRole('button', { name: 'Edit measurements' }));
    await userEvent.click(screen.getByRole('menuitemcheckbox', { name: /Period/ }));

    expect(onpin).toHaveBeenCalledWith('period', true);
  });

  it('given no data, when rendered, then the pin button is disabled', () => {
    render(MeasurementsPanel, { props: { measurements: model({ pinnable: [] }), onpin: vi.fn() } });

    expect(screen.getByRole('button', { name: 'Edit measurements' })).toHaveAttribute('aria-disabled', 'true');
  });
});
