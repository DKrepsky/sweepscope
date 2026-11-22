import { createRawSnippet } from 'svelte';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Banner from './Banner.svelte';
import BannerStack from './BannerStack.svelte';

const text = (value: string) => createRawSnippet(() => ({ render: () => `<span>${value}</span>` }));

describe('Banner', () => {
  it('AC-BN1: given an error Banner with actions, when rendered, then it shows the actions and stays until ondismiss is called', async () => {
    const ondismiss = vi.fn();
    render(Banner, { kind: 'error', text: text('Scope lost'), actions: text('Reconnect'), ondismiss });

    expect(screen.getByText('Reconnect')).toBeInTheDocument();
    expect(ondismiss).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(ondismiss).toHaveBeenCalledOnce();
  });

  it('AC-BN1: given an error Banner, when rendered, then it has role alert', () => {
    render(Banner, { kind: 'error', text: text('Scope lost') });

    expect(screen.getByRole('alert')).toHaveTextContent('Scope lost');
  });

  it.each(['info', 'warning'] as const)('AC-BN1: given a %s Banner, when rendered, then it has role status', (kind) => {
    render(Banner, { kind, text: text('Heads up') });

    expect(screen.getByRole('status')).toHaveTextContent('Heads up');
  });

  it('given no ondismiss, when rendered, then there is no dismiss button', () => {
    render(Banner, { kind: 'info', text: text('Heads up') });

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});

describe('BannerStack', () => {
  it('AC-BN1: given three banners, when rendered, then two are shown and the newest comes first', () => {
    const banners = ['oldest', 'middle', 'newest'].map((id) => ({ id, node: text(id) }));
    render(BannerStack, { banners });

    const shown = screen.getAllByText(/oldest|middle|newest/).map((node) => node.textContent);
    expect(shown).toEqual(['newest', 'middle']);
  });
});
