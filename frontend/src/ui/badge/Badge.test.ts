import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import Badge, { type BadgeProps } from './Badge.svelte';

const kinds: BadgeProps['kind'][] = ['warning', 'software', 'offline', 'error', 'experimental'];
const text = createRawSnippet(() => ({ render: () => '<span>Label</span>' }));
const classesOf = (kind: BadgeProps['kind']) => {
  const { unmount } = render(Badge, { kind, tooltip: `tip ${kind}`, children: text });
  const note = screen.getByRole('note');
  const result = {
    label: note.textContent,
    classes: note.className,
    description: note.getAttribute('aria-describedby'),
  };
  unmount();
  return result;
};

describe('Badge', () => {
  it.each(kinds)('AC-BD1: given kind %s, when rendered, then label and tooltip text are present', (kind) => {
    render(Badge, { kind, tooltip: `tip ${kind}`, children: text });
    expect(screen.getByRole('note')).toHaveTextContent('Label');
    expect(screen.getByRole('note')).toHaveAccessibleDescription(`tip ${kind}`);
  });

  it('AC-BD1: given each kind, when rendered, then the kind classes differ', () => {
    const classes = kinds.map((kind) => classesOf(kind).classes);
    expect(new Set(classes).size).toBe(kinds.length);
  });

  it('AC-BD1: given warning, when rendered, then it has the dashed modifier and others do not', () => {
    expect(classesOf('warning').classes).toContain('dashed');
    expect(classesOf('software').classes).not.toContain('dashed');
  });
});
