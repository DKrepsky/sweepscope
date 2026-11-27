import { createRawSnippet } from 'svelte';
import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import StatusBar from './StatusBar.svelte';
import type { StatusBarItem } from './types';

const item = (id: string, priority: number, warning = false): StatusBarItem => ({
  id,
  priority,
  warning,
  content: createRawSnippet(() => ({ render: () => `<span>${id}</span>` })),
});

const items = [item('connection', 0), item('rate', 1), item('capture', 2), item('renderer', 3)];
const rowOf = (id: string) => screen.getByText(id).closest('.row');

describe('StatusBar', () => {
  it('AC-SB1: given items and a wrap threshold, when rendered, then lower priority items wrap to a second row and the two key items stay in the first', () => {
    render(StatusBar, { items, wrapPriorityAtLeast: 2 });

    expect(rowOf('connection')).toBe(rowOf('rate'));
    expect(rowOf('capture')).toBe(rowOf('renderer'));
    expect(rowOf('capture')).not.toBe(rowOf('connection'));
  });

  it('AC-SB1: given a threshold of 0, when rendered, then connection and sample rate are still present in the first row', () => {
    render(StatusBar, { items, wrapPriorityAtLeast: 0 });

    expect(screen.getByText('connection')).toBeVisible();
    expect(rowOf('connection')).toBe(rowOf('rate'));
  });

  it('given no threshold, when rendered, then all items share one row in priority order', () => {
    render(StatusBar, { items: [item('renderer', 3), item('connection', 0)] });

    expect(document.querySelectorAll('.row')).toHaveLength(1);
    expect(document.querySelector('.item')).toHaveTextContent('connection');
  });

  it('AC-SB1: given an item with warning, when rendered, then it has the warning class and an icon', () => {
    render(StatusBar, { items: [item('accuracy', 2, true)] });

    const element = screen.getByText('accuracy').closest('.item');
    expect(element).toHaveClass('warning');
    expect(element?.querySelector('svg')).toBeInTheDocument();
  });

  it('given an end item of low priority, when rendered with a wrap threshold, then it closes the first row and never wraps', () => {
    const tag = { ...item('tag', 1), end: true };
    render(StatusBar, { items: [tag, ...items], wrapPriorityAtLeast: 2 });

    const first = document.querySelector('.row');
    expect(rowOf('tag')).toBe(first);
    expect(first?.lastElementChild).toHaveTextContent('tag');
    expect(screen.getByText('tag').closest('.item')).toHaveClass('end');
  });
});
