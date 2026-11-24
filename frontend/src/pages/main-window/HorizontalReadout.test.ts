import { render, screen } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import {
  createMockMainWindow,
  type MainWindowModel,
  type PersistenceModel,
  type RecordingModel,
  type RollModel,
  type ViewsModel,
} from '../../app/main-window';
import HorizontalReadout from './HorizontalReadout.svelte';

interface Patch {
  tab?: ViewsModel['tab'];
  roll?: Partial<RollModel>;
  persistence?: Partial<PersistenceModel>;
  recording?: Partial<RecordingModel>;
  showOverview?: boolean;
}

function setup(patch: Patch = {}) {
  const model: MainWindowModel = get(createMockMainWindow());
  render(HorizontalReadout, {
    props: {
      horizontal: model.horizontal,
      views: { ...model.views, tab: patch.tab ?? 'yt' },
      roll: { ...model.roll, ...patch.roll },
      persistence: { ...model.persistence, ...patch.persistence },
      recording: { ...model.recording, ...patch.recording },
      showOverview: patch.showOverview ?? true,
    },
  });
}

describe('HorizontalReadout', () => {
  it('given Yt, when rendered, then the horizontal, trigger position and window readouts and the overview are shown', () => {
    setup();
    expect(screen.getByText('Horizontal')).toBeInTheDocument();
    expect(screen.getByText('Trig pos')).toBeInTheDocument();
    expect(screen.getByRole('img', { name: /Record overview/ })).toBeInTheDocument();
  });

  it('given Yt with persistence on, when rendered, then the persistence setting is read out', () => {
    setup({ persistence: { value: '1s' } });
    expect(screen.getByText('Persistence')).toBeInTheDocument();
    expect(screen.getByText('1 s')).toBeInTheDocument();
  });

  it('given XY, when rendered, then the axes and the time base are read out with the trigger hint', () => {
    setup({ tab: 'xy' });
    expect(screen.getByText('CH1 500 mV/div')).toBeInTheDocument();
    expect(screen.getByText('CH2 200 mV/div')).toBeInTheDocument();
    expect(screen.getByText('Time base')).toBeInTheDocument();
    expect(screen.getByText('Trigger markers hidden in XY; the Trigger group stays active')).toBeInTheDocument();
  });

  it('given FFT, when rendered, then span, bin, window and scale come from the backend texts', () => {
    setup({ tab: 'fft' });
    expect(screen.getByText('0 to 25 kHz')).toBeInTheDocument();
    expect(screen.getByText('12.21 Hz')).toBeInTheDocument();
    expect(screen.getByText('Hann')).toBeInTheDocument();
    expect(screen.getByText('10 dB/div')).toBeInTheDocument();
  });

  it('given FFT, when rendered, then the record overview slot says it is not shown', () => {
    setup({ tab: 'fft' });
    expect(screen.getByText('Time domain overview not shown in FFT')).toBeInTheDocument();
    expect(screen.queryByRole('img', { name: /Record overview/ })).toBeNull();
  });

  it('given roll mode, when rendered, then the roll label and window are read out and the overview is hidden', () => {
    setup({ roll: { active: true, window: '10 s' } });
    expect(screen.getByText('Roll')).toBeInTheDocument();
    expect(screen.getByText('10 s')).toBeInTheDocument();
    expect(screen.queryByText('Trig pos')).toBeNull();
    expect(screen.queryByRole('img', { name: /Record overview/ })).toBeNull();
  });

  it('given a recording in roll mode, when rendered, then the record chip takes the overview slot', () => {
    setup({ roll: { active: true, window: '10 s' }, recording: { active: true, chip: 'Rec 00:00:03 · 12 kB' } });
    expect(screen.getByText('Rec 00:00:03 · 12 kB')).toBeInTheDocument();
  });
});
