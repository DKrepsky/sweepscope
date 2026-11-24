import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ConnectionItem, { type ConnectionItemState } from './ConnectionItem.svelte';

const states: ConnectionItemState[] = ['ready', 'connecting', 'disconnected', 'error'];

describe('ConnectionItem', () => {
  it.each(states)('given the %s state, when rendered, then the word and the detail show', (state) => {
    const view = render(ConnectionItem, { state, word: 'Word', detail: 'Some detail' });

    expect(view.container).toHaveTextContent('Word');
    expect(view.container).toHaveTextContent('Some detail');
  });

  it('given a ready state, when rendered, then a dot marks it and no chip box is drawn', () => {
    const view = render(ConnectionItem, { state: 'ready', word: 'Ready', detail: '' });

    expect(view.container.querySelector('.dot')).not.toBeNull();
    expect(view.container.querySelector('.chip')).toBeNull();
  });

  it('given an error state, when rendered, then a warning icon replaces the dot', () => {
    const view = render(ConnectionItem, { state: 'error', word: 'Error', detail: 'No permission' });

    expect(view.container.querySelector('.dot')).toBeNull();
    expect(view.container.querySelector('svg')).not.toBeNull();
  });

  it('given a title, when rendered, then it is the hover text of the item', () => {
    const view = render(ConnectionItem, { state: 'ready', word: 'Ready', detail: '', title: 'Firmware 1' });

    expect(view.container.querySelector('.connection')).toHaveAttribute('title', 'Firmware 1');
  });

  it('given no detail, when rendered, then no empty detail is left', () => {
    const view = render(ConnectionItem, { state: 'connecting', word: 'Connecting', detail: '' });

    expect(view.container.querySelector('.detail')).toBeNull();
  });
});
