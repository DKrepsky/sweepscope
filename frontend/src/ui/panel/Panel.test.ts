import { createRawSnippet } from 'svelte';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Panel from './Panel.svelte';

const text = (value: string) => createRawSnippet(() => ({ render: () => `<span>${value}</span>` }));

describe('Panel', () => {
  it('given Panel collapsed, when rendered, then the body is not rendered and the header keeps aria-expanded false', () => {
    render(Panel, { collapsed: true, header: text('Cursors'), body: text('Body text') });

    expect(screen.queryByText('Body text')).not.toBeInTheDocument();
    expect(screen.getByText('Cursors')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle panel' })).toHaveAttribute('aria-expanded', 'false');
  });

  it('given Panel expanded, when rendered, then the body and the actions are shown', () => {
    render(Panel, { header: text('Cursors'), body: text('Body text'), actions: text('Action') });

    expect(screen.getByText('Body text')).toBeInTheDocument();
    expect(screen.getByText('Action')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle panel' })).toHaveAttribute('aria-expanded', 'true');
  });

  it('given a toggle handler, when the header button is clicked, then ontoggle is called once', async () => {
    const ontoggle = vi.fn();
    render(Panel, { header: text('Cursors'), body: text('Body'), ontoggle });

    await userEvent.click(screen.getByRole('button', { name: 'Toggle panel' }));

    expect(ontoggle).toHaveBeenCalledOnce();
  });
});
