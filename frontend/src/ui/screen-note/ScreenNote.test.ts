import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import ScreenNote from './ScreenNote.svelte';

const text = (value: string) => createRawSnippet(() => ({ render: () => `<span>${value}</span>` }));

describe('ScreenNote', () => {
  it('given a title and text, when rendered, then a status region names the cause and the next step', () => {
    render(ScreenNote, { kind: 'warning', title: text('XY cannot draw'), children: text('CH2 (Y) is off.') });

    expect(screen.getByRole('status')).toHaveTextContent('XY cannot draw');
    expect(screen.getByRole('status')).toHaveTextContent('CH2 (Y) is off.');
  });

  it.each(['info', 'warning'] as const)(
    'given kind %s, when rendered, then the note carries that kind class',
    (kind) => {
      render(ScreenNote, { kind, title: text('Title') });

      expect(screen.getByRole('status')).toHaveClass(kind);
    },
  );

  it('given an actions snippet, when rendered, then the actions are shown inside the note', () => {
    const actions = createRawSnippet(() => ({ render: () => '<button type="button">Enable CH2</button>' }));
    render(ScreenNote, { kind: 'warning', title: text('Title'), actions });

    expect(screen.getByRole('status')).toContainElement(screen.getByRole('button', { name: 'Enable CH2' }));
  });

  it('given no actions and no text, when rendered, then neither region is present', () => {
    const { container } = render(ScreenNote, { kind: 'info', title: text('Title') });

    expect(container.querySelector('.actions')).toBeNull();
    expect(container.querySelector('.text')).toBeNull();
  });
});
