import { createRawSnippet } from 'svelte';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import MeasurementMatrix from './MeasurementMatrix.svelte';
import type { MeasurementCell } from './types';

const first = { id: 'ch1', label: '1', color: 'color-ch1' };
const columns = [first, { id: 'ch2', label: '2', color: 'color-ch2' }];

const rowWith = (cells: Record<string, MeasurementCell>) => [{ id: 'vpp', label: 'Vpp', cells }];
const valid: MeasurementCell = { state: 'valid', text: '1.20', unit: 'V' };

describe('MeasurementMatrix', () => {
  it('AC-M1: given an invalid cell with footnote 2, when rendered, then it shows a muted dash followed by [2] and lists the footnote', () => {
    const cells = { ch1: { state: 'invalid', text: '', footnote: 2 } as MeasurementCell, ch2: valid };
    render(MeasurementMatrix, {
      columns,
      rows: rowWith(cells),
      footnotes: [{ key: 2, text: 'Signal too short' }],
    });

    const cell = screen.getAllByRole('cell')[0];
    expect(cell).toHaveTextContent('- [2]');
    expect(cell).toHaveClass('muted');
    expect(screen.getByText('Signal too short').closest('li')).toHaveTextContent('[2]');
  });

  it('AC-M1: given an invalid cell with a tooltip, when it is focused, then the tooltip carries the reason', async () => {
    const cells = { ch1: { state: 'invalid', text: '', footnote: 2, tooltip: 'No edge found' } as MeasurementCell };
    render(MeasurementMatrix, { columns: [first], rows: rowWith(cells), footnotes: [] });

    await userEvent.tab();

    expect(screen.getByRole('tooltip')).toHaveTextContent('No edge found');
  });

  it('AC-M1: given an unverified cell, when rendered, then the value is followed by ? and the cell has the unverified class', () => {
    const cells = { ch1: { state: 'unverified', text: '1.20', unit: 'V' } as MeasurementCell };
    render(MeasurementMatrix, { columns: [first], rows: rowWith(cells), footnotes: [] });

    const cell = screen.getByRole('cell');
    expect(cell).toHaveTextContent('1.20 ? V');
    expect(cell).toHaveClass('unverified');
  });

  it('AC-M1: given a column for a disabled channel is not passed, when rendered, then no header or cell exists for it', () => {
    render(MeasurementMatrix, { columns: [first], rows: rowWith({ ch1: valid, ch2: valid }), footnotes: [] });

    expect(screen.queryByRole('columnheader', { name: '2' })).not.toBeInTheDocument();
    expect(screen.getAllByRole('cell')).toHaveLength(1);
  });

  it('AC-M1: given frozenLabel Frozen, when rendered, then the header shows the label and value cells are muted', () => {
    render(MeasurementMatrix, {
      columns,
      rows: rowWith({ ch1: valid, ch2: valid }),
      footnotes: [],
      frozenLabel: 'Frozen',
    });

    expect(screen.getByRole('columnheader', { name: 'Frozen' })).toBeInTheDocument();
    screen.getAllByRole('cell').forEach((cell) => expect(cell).toHaveClass('muted'));
  });

  it('AC-M1: given columns and rows, when rendered, then headers use th with a scope', () => {
    render(MeasurementMatrix, { columns, rows: rowWith({ ch1: valid, ch2: valid }), footnotes: [] });

    expect(screen.getByRole('columnheader', { name: '1' })).toHaveAttribute('scope', 'col');
    expect(screen.getByRole('rowheader', { name: 'Vpp' })).toHaveAttribute('scope', 'row');
  });

  it('given a cell snippet, when rendered, then it replaces the default cell content', () => {
    const cell = createRawSnippet<[MeasurementCell]>((c) => ({ render: () => `<b>custom ${c().text}</b>` }));
    render(MeasurementMatrix, { columns: [first], rows: rowWith({ ch1: valid }), footnotes: [], cell });

    expect(screen.getByRole('cell')).toHaveTextContent('custom 1.20');
    expect(screen.getByRole('cell')).not.toHaveTextContent('V');
  });
});
