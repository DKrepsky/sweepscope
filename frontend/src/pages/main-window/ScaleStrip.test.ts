import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import type { MathModel } from '../../app/main-window';
import { mockChannels } from '../../app/main-window/mock-model';
import ScaleStrip from './ScaleStrip.svelte';

describe('ScaleStrip', () => {
  it('given a clipped enabled channel, when rendered, then its chip shows the Clip badge', () => {
    const channels = mockChannels().map((channel) => ({ ...channel, enabled: true, clipped: channel.number === 1 }));

    render(ScaleStrip, { props: { channels, cursorSource: 1 } });

    expect(screen.getAllByText('Clip')).toHaveLength(1);
  });

  it('given XY roles, when rendered, then the two chips carry X and Y and the name says so', () => {
    const channels = mockChannels().map((channel) => ({
      ...channel,
      enabled: true,
      ...(channel.number === 1 && { role: 'x' as const }),
      ...(channel.number === 2 && { role: 'y' as const }),
    }));

    render(ScaleStrip, { props: { channels, cursorSource: 1 } });

    expect(screen.getByText('X')).toBeInTheDocument();
    expect(screen.getByText('Y')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /^2 .* Y\b.*channel 2 scale$/ })).toBeInTheDocument();
  });

  it('given an alias notice, when rendered, then the chip shows the Alias badge with the message', () => {
    const alias = { short: 'Alias CH1', message: 'Possible aliasing on CH1.' };
    const channels = mockChannels().map((channel) => ({
      ...channel,
      enabled: true,
      ...(channel.number === 1 && { alias }),
    }));

    render(ScaleStrip, { props: { channels, cursorSource: 1 } });

    expect(screen.getAllByText('Alias')).toHaveLength(1);
    expect(screen.getByText('Alias').closest('[role="note"]')).toHaveTextContent('Possible aliasing on CH1.');
  });

  it('given a mixed math model, when rendered, then the M chip shows its text and the Mixed badge', () => {
    const math = {
      chip: 'M  A−B  1 V/div',
      row: { status: 'mixed', statusDetail: 'CH1 500 mV/div, CH2 20 mV/div.' },
    } as MathModel;

    render(ScaleStrip, { props: { channels: mockChannels(), cursorSource: 1, math } });

    expect(screen.getByText('M A−B 1 V/div')).toBeInTheDocument();
    expect(screen.getByText('Mixed')).toBeInTheDocument();
  });

  it('given an unavailable math model, when rendered, then the chip states the reason and is dashed', () => {
    const math = { chip: 'M  A−B  unavailable: CH2 is off', row: { status: 'unavailable' } } as MathModel;

    const { container } = render(ScaleStrip, { props: { channels: mockChannels(), cursorSource: 1, math } });

    expect(screen.getByText('M A−B unavailable: CH2 is off')).toBeInTheDocument();
    expect(container.querySelector('[data-math-chip]')).toHaveClass('unavailable');
  });

  it('given math is off, when rendered, then there is no M chip', () => {
    const math = { chip: '', row: { status: 'ok' } } as MathModel;

    const { container } = render(ScaleStrip, { props: { channels: mockChannels(), cursorSource: 1, math } });

    expect(container.querySelector('[data-math-chip]')).toBeNull();
  });

  it('given no clipped channel, when rendered, then there is no Clip badge', () => {
    render(ScaleStrip, { props: { channels: mockChannels(), cursorSource: 1 } });

    expect(screen.queryByText('Clip')).not.toBeInTheDocument();
  });
});
