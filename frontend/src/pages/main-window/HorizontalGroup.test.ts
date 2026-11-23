import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createMockMainWindow, type HorizontalModel } from '../../app/main-window';
import HorizontalGroup from './HorizontalGroup.svelte';
import { spyActions } from './test-support';

const baseline = (): HorizontalModel => get(createMockMainWindow()).horizontal;

function setup(change: Partial<HorizontalModel> = {}) {
  const actions = spyActions();
  render(HorizontalGroup, { props: { horizontal: { ...baseline(), ...change }, actions } });
  return { actions, user: userEvent.setup() };
}

const option = (name: string) => screen.getByRole('button', { name });

describe('HorizontalGroup interpolation', () => {
  it('given the backend selects sin(x)/x, when rendered, then sin(x)/x is enabled and pressed', () => {
    setup({ interpolation: 'sinc' });
    expect(option('sin(x)/x')).toHaveAttribute('aria-pressed', 'true');
    expect(option('sin(x)/x')).not.toHaveAttribute('aria-disabled');
  });

  it('given sin(x)/x is selected, when Dots is clicked, then setInterpolation gets dots', async () => {
    const { actions, user } = setup({ interpolation: 'sinc' });
    await user.click(option('Dots'));
    expect(actions.setInterpolation).toHaveBeenCalledExactlyOnceWith('dots');
  });

  it('given linear is selected, when sin(x)/x is clicked, then setInterpolation gets sinc', async () => {
    const { actions, user } = setup({ interpolation: 'linear' });
    await user.click(option('sin(x)/x'));
    expect(actions.setInterpolation).toHaveBeenCalledExactlyOnceWith('sinc');
  });

  it('given dots is selected, when Linear is clicked, then setInterpolation gets linear', async () => {
    const { actions, user } = setup({ interpolation: 'dots' });
    await user.click(option('Linear'));
    expect(actions.setInterpolation).toHaveBeenCalledExactlyOnceWith('linear');
  });

  it('given the automatic mode, when rendered, then the sin(x)/x tooltip states the automatic rule', () => {
    setup({ interpolation: 'sinc', automaticInterpolation: true });
    expect(screen.getByText('sin(x)/x')).toHaveAttribute(
      'title',
      expect.stringContaining('Automatic: sin(x)/x below 250'),
    );
  });

  it('given the user chose a mode, when rendered, then the sin(x)/x tooltip does not mention automatic', () => {
    setup({ interpolation: 'sinc', automaticInterpolation: false });
    expect(screen.getByText('sin(x)/x').getAttribute('title')).not.toContain('Automatic');
  });

  it('given the horizontal group is unavailable, when sin(x)/x is clicked, then nothing is called', async () => {
    const { actions, user } = setup({ interpolation: 'linear', availability: { enabled: false, reason: 'Roll mode' } });
    await user.click(option('sin(x)/x'));
    expect(actions.setInterpolation).not.toHaveBeenCalled();
  });
});
