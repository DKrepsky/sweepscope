import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import NumberField from './NumberField.svelte';

const setup = (extra = {}) => {
  const commit = vi.fn();
  const view = render(NumberField, { value: '-2.00', commit, ariaLabel: 'Position', ...extra });
  return { commit, view, input: () => screen.getByRole('textbox') as HTMLInputElement };
};

describe('NumberField', () => {
  it('given an error shown by the caller, when rendered, then the field is invalid and draws no message itself', () => {
    const { input } = setup({ error: 'bad width', errorText: 'external' });

    expect(input()).toBeInvalid();
    expect(screen.queryByText('bad width')).toBeNull();
  });

  it('AC-N1: given editing text, when Enter is pressed, then commit fires once with the text', async () => {
    const { commit, input } = setup();
    await userEvent.clear(input());
    await userEvent.type(input(), '1.5{Enter}');
    expect(commit).toHaveBeenCalledExactlyOnceWith('1.5');
  });

  it('AC-N1: given editing text, when Escape is pressed, then the original value shows and commit is not called', async () => {
    const { commit, input } = setup();
    await userEvent.type(input(), '99{Escape}');
    expect(input().value).toBe('-2.00');
    expect(commit).not.toHaveBeenCalled();
  });

  it('given editing text, when the field loses focus, then the original value shows and commit is not called', async () => {
    const { commit, input } = setup();
    await userEvent.type(input(), '99');
    await userEvent.tab();
    expect(input().value).toBe('-2.00');
    expect(commit).not.toHaveBeenCalled();
  });

  it('AC-N1: given error, when rendered, then aria-invalid is true and the message is shown', () => {
    const { input } = setup({ error: 'Range -4 to +4 div' });
    expect(input()).toHaveAttribute('aria-invalid', 'true');
    expect(input()).toHaveAccessibleDescription('Range -4 to +4 div');
  });

  it('AC-N1: given error, when it appears, then the message slot exists in both states and the field class set is unchanged', () => {
    const valid = setup();
    const slot = () => document.querySelector('.msg');
    expect(slot()).toBeInTheDocument();
    valid.view.unmount();
    setup({ error: 'bad' });
    expect(slot()).toHaveTextContent('bad');
    expect(document.querySelector('.field')?.children).toHaveLength(2);
  });

  it('given a disabled field with a reason, when typing and focusing, then commit is not called and the reason shows', async () => {
    const { commit, input } = setup({ disabled: true, disabledReason: 'Channel off' });
    await userEvent.type(input(), '5{Enter}');
    expect(commit).not.toHaveBeenCalled();
    expect(input()).toHaveAttribute('aria-disabled', 'true');
    expect(screen.getByRole('tooltip')).toHaveTextContent('Channel off');
  });
});
