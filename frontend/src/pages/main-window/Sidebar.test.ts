import { render, screen } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import { createMockMainWindow, type ViewTab } from '../../app/main-window';
import Sidebar from './Sidebar.svelte';

function setup(view: ViewTab) {
  const model = get(createMockMainWindow());
  render(Sidebar, {
    props: {
      view,
      cursors: model.cursors,
      measurements: model.measurements,
      peak: model.peak,
      onclearcursors: vi.fn(),
      onpin: vi.fn(),
      onpeakmode: vi.fn(),
    },
  });
}

describe('Sidebar', () => {
  it('given Yt, when rendered, then the Cursors panel is shown', () => {
    setup('yt');
    expect(screen.getByRole('heading', { name: 'Cursors' })).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: 'Peak marker' })).toBeNull();
  });

  it('given FFT, when rendered, then the Peak marker panel replaces the Cursors panel', () => {
    setup('fft');
    expect(screen.getByRole('heading', { name: 'Peak marker' })).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: 'Cursors' })).toBeNull();
  });

  it('given XY, when rendered, then the Cursors body says they are not available in XY', () => {
    setup('xy');
    expect(
      screen.getByText('Cursors are not available in XY. Return to Yt to measure time and voltage.'),
    ).toBeInTheDocument();
  });

  it.each(['yt', 'xy', 'fft'] as const)(
    'given the %s view, when rendered, then the Measurements panel stays',
    (view) => {
      setup(view);
      expect(screen.getByRole('heading', { name: /Measurements/ })).toBeInTheDocument();
    },
  );
});
