import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import StatusChip, { type StatusChipKind } from './StatusChip.svelte';

const kinds: StatusChipKind[] = [
  'run',
  'stop',
  'armed',
  'triggered',
  'auto',
  'disconnected',
  'error',
  'warning',
  'info',
  'offline',
  'ok',
  'record',
];
const text = (value: string) => createRawSnippet(() => ({ render: () => `<span>${value}</span>` }));
const chip = () => screen.getByText('State').closest('.chip') as HTMLElement;

describe('StatusChip', () => {
  it.each(kinds)('AC-C1: given kind %s, when rendered, then text, glyph and kind class are present', (kind) => {
    render(StatusChip, { kind, children: text('State') });
    expect(chip()).toHaveClass(kind);
    expect(chip().querySelector('.glyph')?.children.length).toBeGreaterThan(0);
  });

  it('AC-C1: given dim, when rendered, then the dim class applies and no other class is lost', () => {
    render(StatusChip, { kind: 'run', dim: true, children: text('State') });
    expect(chip()).toHaveClass('dim', 'run');
  });

  it('AC-C1: given the kind changes, when rerendered, then the live region text updates', async () => {
    const view = render(StatusChip, { kind: 'run', children: text('Run') });
    const region = screen.getByText('Run').closest('[aria-live="polite"]');
    await view.rerender({ kind: 'stop', children: text('Stop') });
    expect(region).toHaveTextContent('Stop');
    expect(region).toHaveClass('stop');
  });

  it('given kind record, when rendered, then it shows a dot glyph and the text', () => {
    render(StatusChip, { kind: 'record', children: text('State') });
    expect(chip()).toHaveClass('record');
    expect(chip().querySelector('.dot')).not.toBeNull();
  });

  it('given kind triggered and the text Done, when rendered, then the live region announces Done', () => {
    render(StatusChip, { kind: 'triggered', children: text('Done') });
    expect(screen.getByText('Done').closest('[aria-live="polite"]')).toHaveClass('triggered');
  });

  it('given a left snippet, when rendered, then it replaces the default glyph', () => {
    render(StatusChip, { kind: 'run', children: text('State'), left: text('X') });
    expect(chip().querySelector('.dot')).toBeNull();
    expect(chip().querySelector('.glyph')).toHaveTextContent('X');
  });
});
