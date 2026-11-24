import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createMockMainWindow, type TriggerModel } from '../../app/main-window';
import TriggerGroup from './TriggerGroup.svelte';
import { spyActions } from './test-support';

const pulse = (change: Partial<NonNullable<TriggerModel['pulse']>> = {}): Partial<TriggerModel> => ({
  type: 'pulse',
  pulse: { condition: 'greater', width: { value: '500', unit: 'µs' }, verified: true, ...change },
});

function setup(change: Partial<TriggerModel> = {}) {
  const { trigger, channels } = get(createMockMainWindow());
  const actions = spyActions();
  render(TriggerGroup, { props: { trigger: { ...trigger, ...change }, channels, actions } });
  return { actions, user: userEvent.setup() };
}

describe('TriggerGroup pulse', () => {
  it('given an edge trigger, when rendered, then there is no width row', () => {
    setup();

    expect(screen.queryByRole('textbox', { name: 'Pulse width' })).toBeNull();
  });

  it('given an edge trigger, when Pulse is chosen, then the kind is pulse', async () => {
    const { actions, user } = setup();

    await user.click(screen.getByRole('button', { name: 'Pulse' }));

    expect(actions.setTriggerKind).toHaveBeenCalledExactlyOnceWith('pulse');
  });

  it('given a pulse trigger, when rendered, then the polarity options are named positive and negative pulse', () => {
    setup(pulse());

    expect(screen.getByRole('button', { name: 'Positive pulse' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Negative pulse' })).toBeInTheDocument();
  });

  it('given a pulse trigger, when the shorter condition is chosen, then the condition is less', async () => {
    const { actions, user } = setup(pulse());

    await user.click(screen.getByRole('button', { name: 'Shorter than' }));

    expect(actions.setPulseCondition).toHaveBeenCalledExactlyOnceWith('less');
  });

  it('given a pulse trigger, when a width is typed and committed, then the text goes to the backend as typed', async () => {
    const { actions, user } = setup(pulse());
    const field = screen.getByRole('textbox', { name: 'Pulse width' });

    await user.clear(field);
    await user.type(field, '0.5 ms{Enter}');

    expect(actions.commitPulseWidth).toHaveBeenCalledExactlyOnceWith('0.5 ms');
  });

  it('given a rejected width, when rendered, then the field is invalid and the message shows in the reserved slot', () => {
    setup(pulse({ error: 'Invalid width. Enter a time such as 500 µs. Applied: 500 µs.' }));

    expect(screen.getByRole('textbox', { name: 'Pulse width' })).toBeInvalid();
    expect(screen.getByRole('status')).toHaveTextContent(
      'Invalid width. Enter a time such as 500 µs. Applied: 500 µs.',
    );
  });

  it('given a shown error, when Escape is pressed in the field, then the error is cleared', async () => {
    const { actions, user } = setup(pulse({ error: 'Invalid width.' }));

    await user.type(screen.getByRole('textbox', { name: 'Pulse width' }), 'x{Escape}');

    expect(actions.clearPulseWidthError).toHaveBeenCalled();
  });

  it('given a width outside the verified range, when rendered, then the question badge shows', () => {
    setup(pulse({ verified: false }));

    expect(screen.getByText('?')).toBeInTheDocument();
  });

  it('given a pulse trigger without error, when rendered, then the two-line slot is still present and empty', () => {
    setup(pulse());

    expect(screen.getByRole('status').textContent?.trim()).toBe('');
  });

  it('given the trigger blocked in roll, when rendered, then the reason line shows the backend text', () => {
    const reason = 'Trigger is not available in roll mode (500 ms/div and slower). Use a faster time base to trigger.';
    setup({ availability: { enabled: false, reason } });

    expect(screen.getByTestId('trigger-reason')).toHaveTextContent(reason);
  });

  it('given a trigger that is available, when rendered, then there is no reason line', () => {
    setup();

    expect(screen.queryByTestId('trigger-reason')).toBeNull();
  });

  it('given the video type, when rendered, then it stays disabled', () => {
    setup();

    expect(screen.getByRole('button', { name: 'Video' })).toHaveAttribute('aria-disabled', 'true');
  });
});
