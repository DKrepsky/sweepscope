import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { mockChannels } from '../../app/main-window/mock-model';
import ChannelRow from './ChannelRow.svelte';
import { spyActions } from './test-support';

const rowFor = (container: HTMLElement) => container.querySelector<HTMLElement>('[data-channel]')!;

describe('ChannelRow', () => {
  it('AC-CH1: given a channel enabled and then disabled, when rendered, then the row keeps its classes and child count, controls are aria-disabled and the number toggle is outline only', () => {
    const channel = mockChannels()[1]!;
    const on = render(ChannelRow, { props: { channel: { ...channel, enabled: true }, actions: spyActions() } });
    const enabledRow = rowFor(on.container);
    const enabledShape = { classes: enabledRow.className, children: enabledRow.children.length };
    on.unmount();

    const off = render(ChannelRow, { props: { channel: { ...channel, enabled: false }, actions: spyActions() } });
    const disabledRow = rowFor(off.container);

    expect({ classes: disabledRow.className, children: disabledRow.children.length }).toEqual(enabledShape);
    expect(screen.getByRole('button', { name: 'Channel 2 enabled' })).toHaveClass('outline');
    expect(screen.getByRole('button', { name: 'BW, Channel 2 bandwidth limit' })).toHaveAttribute(
      'aria-disabled',
      'true',
    );
    expect(screen.getByRole('button', { name: 'Inv, Channel 2 invert' })).toHaveAttribute('aria-disabled', 'true');
    expect(screen.getByRole('textbox', { name: /vertical position/ })).toHaveAttribute('aria-disabled', 'true');
  });

  it('given an enabled channel, when rendered, then the number toggle is filled and controls are operable', () => {
    render(ChannelRow, { props: { channel: mockChannels()[0]!, actions: spyActions() } });

    expect(screen.getByRole('button', { name: 'Channel 1 enabled' })).not.toHaveClass('outline');
    expect(screen.getByRole('button', { name: 'Inv, Channel 1 invert' })).not.toHaveAttribute('aria-disabled');
  });
});
