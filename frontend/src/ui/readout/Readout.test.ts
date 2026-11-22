import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import Readout from './Readout.svelte';

const label = createRawSnippet(() => ({ render: () => '<span>Vpp</span>' }));

describe('Readout', () => {
  it('given value, unit and label, when rendered, then all three show', () => {
    render(Readout, { value: '3.30', unit: 'V', label });
    expect(screen.getByText('3.30')).toBeInTheDocument();
    expect(screen.getByText('V')).toBeInTheDocument();
    expect(screen.getByText('Vpp')).toBeInTheDocument();
  });

  it('given muted, when rendered, then the muted class applies', () => {
    render(Readout, { value: '-', muted: true });
    expect(screen.getByText('-').parentElement).toHaveClass('muted');
  });

  it('given minWidthCh, when rendered, then the value reserves that width', () => {
    render(Readout, { value: '1', minWidthCh: 8 });
    expect(screen.getByText('1')).toHaveAttribute('style', expect.stringContaining('min-width: 8ch'));
  });
});
