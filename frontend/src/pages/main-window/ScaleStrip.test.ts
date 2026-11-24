import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { mockChannels } from '../../app/main-window/mock-model';
import ScaleStrip from './ScaleStrip.svelte';

describe('ScaleStrip', () => {
  it('given a clipped enabled channel, when rendered, then its chip shows the Clip badge', () => {
    const channels = mockChannels().map((channel) => ({ ...channel, enabled: true, clipped: channel.number === 1 }));

    render(ScaleStrip, { props: { channels, cursorSource: 1 } });

    expect(screen.getAllByText('Clip')).toHaveLength(1);
  });

  it('given no clipped channel, when rendered, then there is no Clip badge', () => {
    render(ScaleStrip, { props: { channels: mockChannels(), cursorSource: 1 } });

    expect(screen.queryByText('Clip')).not.toBeInTheDocument();
  });
});
