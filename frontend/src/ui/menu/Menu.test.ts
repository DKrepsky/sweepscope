import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Menu from './Menu.svelte';
import { nextIndex } from './navigation';

const items = [
  { id: 'open', label: 'Open' },
  { id: 'export', label: 'Export', disabled: true, disabledReason: 'Nothing to export' },
  { id: 'save', label: 'Save', badge: 'Exp' },
];

const setup = (overrides = {}) => {
  const onselect = vi.fn();
  const onclose = vi.fn();
  render(Menu, { props: { items, onselect, onclose, open: true, anchor: { x: 10, y: 20 }, ...overrides } });
  return { onselect, onclose };
};

describe('Menu', () => {
  it('given Menu open, when rendered, then the first item has focus', () => {
    setup();

    expect(screen.getByRole('menuitem', { name: 'Open' })).toHaveFocus();
  });

  it('given Menu closed, when rendered, then nothing is shown', () => {
    setup({ open: false });

    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
  });

  it('given a disabled item, when ArrowDown moves over it, then it is focused, its reason shows and Enter does not select', async () => {
    const { onselect } = setup();

    await userEvent.keyboard('{ArrowDown}');
    const disabled = screen.getByRole('menuitem', { name: 'Export' });
    expect(disabled).toHaveFocus();
    expect(disabled).toHaveAttribute('aria-disabled', 'true');
    expect(screen.getByRole('tooltip')).toHaveTextContent('Nothing to export');

    await userEvent.keyboard('{Enter}');
    expect(onselect).not.toHaveBeenCalled();
  });

  it('given an enabled item focused, when Enter is pressed, then onselect receives its id', async () => {
    const { onselect } = setup();

    await userEvent.keyboard('{Enter}');

    expect(onselect).toHaveBeenCalledExactlyOnceWith('open');
  });

  it('given a disabled item, when clicked, then onselect is not called', async () => {
    const { onselect } = setup();

    await userEvent.click(screen.getByRole('menuitem', { name: 'Export' }));

    expect(onselect).not.toHaveBeenCalled();
  });

  it('given Menu open, when Escape is pressed, then onclose is called', async () => {
    const { onclose } = setup();

    await userEvent.keyboard('{Escape}');

    expect(onclose).toHaveBeenCalledOnce();
  });

  it('given an item with a badge, when rendered, then the badge text is shown', () => {
    setup();

    expect(screen.getByRole('menuitem', { name: /Save/ })).toHaveTextContent('Exp');
  });

  it('given Menu open, when the pointer is pressed outside, then onclose is called', async () => {
    const { onclose } = setup();

    await userEvent.pointer({ keys: '[MouseLeft>]', target: document.querySelector('.scrim') as Element });

    expect(onclose).toHaveBeenCalledOnce();
  });
});

describe('nextIndex', () => {
  it.each([
    ['ArrowDown', 2, 0],
    ['ArrowUp', 0, 2],
    ['Home', 1, 0],
    ['End', 0, 2],
  ])('given key %s at index %i of 3, when navigating, then the target is %i', (key, current, expected) => {
    expect(nextIndex(key, current, 3)).toBe(expected);
  });

  it('given a non navigation key, when navigating, then there is no target', () => {
    expect(nextIndex('a', 0, 3)).toBeNull();
  });

  it('given an open menu, when Tab is pressed, then it closes and focus does not stay inside', async () => {
    const { onclose } = setup();

    await userEvent.keyboard('{Tab}');

    expect(onclose).toHaveBeenCalledOnce();
  });

  it('given a menu opened from a button, when it closes, then focus returns to that button', async () => {
    const opener = document.createElement('button');
    document.body.append(opener);
    opener.focus();
    const view = render(Menu, {
      props: { items, onselect: vi.fn(), onclose: vi.fn(), open: true, anchor: { x: 0, y: 0 } },
    });
    expect(screen.getByRole('menuitem', { name: 'Open' })).toHaveFocus();

    await view.rerender({ open: false });

    expect(opener).toHaveFocus();
    opener.remove();
  });
});
