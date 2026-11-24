import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  createMockMainWindow,
  type GraphModel,
  type PeakModel,
  type RollModel,
  type ViewsModel,
  type XyModel,
} from '../../app/main-window';
import { createRecordingContext } from '../../ui/graph/recording-context';
import ScopeStage from './ScopeStage.svelte';
import { spyActions } from './test-support';

beforeEach(() => {
  window.matchMedia = (() => ({ addEventListener: vi.fn(), removeEventListener: vi.fn() })) as never;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => createRecordingContext() as never);
});

interface Patch {
  graph?: Partial<GraphModel>;
  views?: Partial<ViewsModel>;
  xy?: Partial<XyModel>;
  peak?: Partial<PeakModel>;
  roll?: Partial<RollModel>;
}

function renderStage(patch: Patch = {}) {
  const model = get(createMockMainWindow());
  const actions = spyActions();
  const views = { ...model.views, ...patch.views, xy: { ...model.views.xy, ...patch.xy } };
  const { container } = render(ScopeStage, {
    props: {
      graph: { ...model.graph, ...patch.graph },
      channels: model.channels,
      trigger: model.trigger,
      cursors: model.cursors,
      views,
      peak: { ...model.peak, ...patch.peak },
      roll: { ...model.roll, ...patch.roll },
      dots: false,
      actions,
      rootFontPx: 16,
    },
  });
  return { actions, container, user: userEvent.setup() };
}

const marker = (position: number, step: number) => ({ position, level: 0.8, step, valueText: '3.000 kHz −6.9 dBV' });

describe('ScopeStage in Yt', () => {
  it('given Yt, when rendered, then the ground and trigger markers are shown', () => {
    renderStage();
    expect(screen.getByRole('slider', { name: 'Channel 1 ground' })).toBeInTheDocument();
    expect(screen.getByRole('slider', { name: 'Trigger level' })).toBeInTheDocument();
    expect(screen.getByRole('slider', { name: 'Trigger position' })).toBeInTheDocument();
  });

  it('given persistence on, when rendered, then the screen carries the persistence tag', () => {
    renderStage({ graph: { tags: [{ id: 'persistence', corner: 'top-right', text: 'PERSIST 1 s' }] } });
    expect(screen.getByText('PERSIST 1 s')).toBeInTheDocument();
  });
});

describe('ScopeStage in XY', () => {
  const xy = { graph: { view: 'xy' as const } };

  it('given XY, when rendered, then only the Y ground marker is in the left gutter and no trigger marker is drawn', () => {
    renderStage(xy);
    expect(screen.getByRole('slider', { name: 'Channel 2 ground' })).toBeInTheDocument();
    expect(screen.queryByRole('slider', { name: 'Channel 1 ground' })).toBeNull();
    expect(screen.queryByRole('slider', { name: 'Trigger level' })).toBeNull();
    expect(screen.queryByRole('slider', { name: 'Trigger position' })).toBeNull();
  });

  it('given XY, when rendered, then the X ground marker is in the bottom gutter and moves in tenths of the width', async () => {
    const { actions, user } = renderStage(xy);
    const x = screen.getByRole('slider', { name: 'Channel 1 ground (X axis)' });
    expect(x).toHaveAttribute('aria-orientation', 'horizontal');
    x.focus();
    await user.keyboard('{Alt>}{ArrowRight}{/Alt}');
    expect(actions.setChannelPosition).toHaveBeenCalledTimes(1);
    const [channel, divisions] = actions.setChannelPosition.mock.calls[0]!;
    expect(channel).toBe(1);
    const ground = get(createMockMainWindow()).channels[0]!.groundPosition;
    expect(divisions).toBeCloseTo((ground - 0.5) * 8 + 0.0125 * 10, 9);
  });

  it('given XY, when the axes are tagged, then both scale tags are shown', () => {
    renderStage({
      graph: {
        view: 'xy',
        tags: [
          { id: 'xy-y', corner: 'top-left', text: 'Y  CH2 200 mV/div', channel: 2 },
          { id: 'xy-x', corner: 'bottom-right', text: 'X  CH1 500 mV/div', channel: 1 },
        ],
      },
    });
    expect(screen.getByText('Y CH2 200 mV/div')).toBeInTheDocument();
    expect(screen.getByText('X CH1 500 mV/div')).toBeInTheDocument();
  });

  it('given Y is off, when rendered, then the note names it and Enable CH2 turns it on', async () => {
    const problem = { title: 'XY cannot draw', text: 'CH2 (Y) is off.', enable: 2 as const };
    const { actions, user } = renderStage({ ...xy, xy: { problem } });
    expect(screen.getByRole('status')).toHaveTextContent('XY cannot draw');
    expect(screen.getByRole('status')).toHaveTextContent('CH2 (Y) is off.');
    await user.click(screen.getByRole('button', { name: 'Enable CH2' }));
    expect(actions.setChannelEnabled).toHaveBeenCalledExactlyOnceWith(2, true);
  });

  it('given fewer than two channels, when rendered, then the note has no action button', () => {
    const problem = { title: 'XY needs two enabled channels', text: 'Only CH1 is on. Enable another channel.' };
    renderStage({ ...xy, xy: { problem } });
    expect(screen.getByRole('status')).toHaveTextContent('XY needs two enabled channels');
    expect(screen.queryByRole('button', { name: /Enable/ })).toBeNull();
  });
});

describe('ScopeStage in FFT', () => {
  const fft = (peak: Partial<PeakModel> = {}) => ({
    graph: { view: 'fft' as const },
    peak: { bins: 101, marker: marker(0.3, 0.01), ...peak },
  });

  it('given FFT, when rendered, then no ground or trigger marker is drawn', () => {
    renderStage(fft());
    expect(screen.queryByRole('slider', { name: /ground|Trigger/ })).toBeNull();
  });

  it('given a peak marker, when rendered, then it is a slider with the shared value text', () => {
    renderStage(fft());
    expect(screen.getByRole('slider', { name: 'Peak marker' })).toHaveAttribute('aria-valuetext', '3.000 kHz −6.9 dBV');
    expect(screen.getAllByText('3.000 kHz −6.9 dBV').length).toBeGreaterThan(0);
  });

  it('given the focused marker, when Right is pressed, then the marker is placed one bin higher', async () => {
    const { actions, user } = renderStage(fft());
    screen.getByRole('slider', { name: 'Peak marker' }).focus();
    await user.keyboard('{ArrowRight}');
    expect(actions.placePeakMarker).toHaveBeenCalledExactlyOnceWith(31);
  });

  it('given the focused marker, when Shift and Right are pressed, then the marker moves ten bins', async () => {
    const { actions, user } = renderStage(fft());
    screen.getByRole('slider', { name: 'Peak marker' }).focus();
    await user.keyboard('{Shift>}{ArrowRight}{/Shift}');
    expect(actions.placePeakMarker).toHaveBeenCalledExactlyOnceWith(40);
  });

  it('given a click on the spectrum, when it lands half way, then the marker is placed on the middle bin', async () => {
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
      left: 0,
      top: 0,
      width: 200,
      height: 100,
      right: 200,
      bottom: 100,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    });
    const { actions, container, user } = renderStage(fft());
    await user.pointer({
      target: container.querySelector('.peak-layer')!,
      coords: { clientX: 100 },
      keys: '[MouseLeft]',
    });
    expect(actions.placePeakMarker).toHaveBeenCalledExactlyOnceWith(50);
  });

  it('given Off with no marker, when the spectrum is clicked, then the marker is still placed', async () => {
    const { actions, container, user } = renderStage({
      graph: { view: 'fft' },
      peak: { bins: 101, mode: 'off', available: { enabled: true } },
    });
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({ left: 0, width: 100 } as DOMRect);
    await user.pointer({
      target: container.querySelector('.peak-layer')!,
      coords: { clientX: 20 },
      keys: '[MouseLeft]',
    });
    expect(actions.placePeakMarker).toHaveBeenCalledExactlyOnceWith(20);
  });

  it('given no channel is on, when rendered, then the note asks for one and no marker layer exists', () => {
    const problem = { title: 'No spectrum', text: 'Enable a channel to see its spectrum.' };
    const { container } = renderStage({
      ...fft(),
      views: { fft: { ...get(createMockMainWindow()).views.fft, problem } },
    });
    expect(screen.getByRole('status')).toHaveTextContent('No spectrum');
    expect(container.querySelector('.peak-layer')).toBeNull();
  });
});

describe('ScopeStage in roll', () => {
  const gap = { from: 0.2, to: 0.3, label: 'GAP 1.20 s' };

  it('given roll gaps, when rendered, then each band shows its label', () => {
    renderStage({ roll: { active: true, gaps: [gap], gapCount: 1, announcement: 'Data gap of 1.20 s' } });
    expect(screen.getByText('GAP 1.20 s')).toBeInTheDocument();
  });

  it('given a gap, when rendered, then it is announced politely', () => {
    renderStage({ roll: { active: true, gaps: [gap], gapCount: 1, announcement: 'Data gap of 1.20 s' } });
    const live = screen.getByText('Data gap of 1.20 s');
    expect(live).toHaveAttribute('aria-live', 'polite');
  });

  it('given roll is not active, when rendered, then there is no gap overlay', () => {
    renderStage({ roll: { active: false, gaps: [gap] } });
    expect(screen.queryByText('GAP 1.20 s')).toBeNull();
  });
});
