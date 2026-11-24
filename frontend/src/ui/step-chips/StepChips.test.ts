import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import StepChips from './StepChips.svelte';

const items = [
  { label: '1', state: 'done' as const, color: 'var(--c1)', name: 'Channel 1, done' },
  { label: '2', state: 'current' as const, color: 'var(--c2)', name: 'Channel 2, current' },
  { label: '3', state: 'todo' as const, color: 'var(--c3)', name: 'Channel 3, to do' },
];

describe('StepChips', () => {
  it('given three steps, when rendered, then each has its accessible name', () => {
    render(StepChips, { props: { caption: 'Channels', items } });

    expect(screen.getByLabelText('Channel 1, done')).toBeInTheDocument();
    expect(screen.getByLabelText('Channel 2, current')).toBeInTheDocument();
    expect(screen.getByLabelText('Channel 3, to do')).toBeInTheDocument();
  });

  it('given a current step, when rendered, then only it is marked as the current step', () => {
    const { container } = render(StepChips, { props: { caption: 'Channels', items } });

    const current = [...container.querySelectorAll('[aria-current="step"]')];

    expect(current.map((chip) => chip.getAttribute('aria-label'))).toEqual(['Channel 2, current']);
  });

  it('given a done step, when rendered, then it shows a check and a todo step does not', () => {
    render(StepChips, { props: { caption: 'Channels', items } });

    expect(screen.getByLabelText('Channel 1, done').querySelector('svg')).not.toBeNull();
    expect(screen.getByLabelText('Channel 3, to do').querySelector('svg')).toBeNull();
  });

  it('given a done chip without check, when rendered, then it shows only its label', () => {
    const quiet = [{ ...items[0]!, check: false }];

    render(StepChips, { props: { caption: 'Channels', items: quiet } });

    expect(screen.getByLabelText('Channel 1, done').querySelector('svg')).toBeNull();
  });

  it('given a caption, when rendered, then it is visible', () => {
    render(StepChips, { props: { caption: 'Channels', items } });

    expect(screen.getByText('Channels')).toBeVisible();
  });
});
