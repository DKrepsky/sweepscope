import { createRawSnippet } from 'svelte';
import { fireEvent, render } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Dialog from './Dialog.svelte';

const text = (value: string) => createRawSnippet(() => ({ render: () => `<span>${value}</span>` }));
const props = { title: text('Save'), body: text('Body'), onclose: () => {} };

// jsdom does not implement the modal state: record the calls the browser would act on.
beforeEach(() => {
  HTMLDialogElement.prototype.showModal = vi.fn(function (this: HTMLDialogElement) {
    this.setAttribute('open', '');
  });
  HTMLDialogElement.prototype.show = vi.fn(function (this: HTMLDialogElement) {
    this.setAttribute('open', '');
  });
  HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
    this.removeAttribute('open');
  });
});
afterEach(() => vi.restoreAllMocks());

describe('Dialog', () => {
  it('AC-D1: given a modal Dialog, when open is true, then the native dialog is opened in modal mode', () => {
    const { container } = render(Dialog, { ...props, open: true, modal: true });

    expect(container.querySelector('dialog')).toHaveAttribute('open');
    expect(HTMLDialogElement.prototype.showModal).toHaveBeenCalledOnce();
    expect(HTMLDialogElement.prototype.show).not.toHaveBeenCalled();
  });

  it('given a non-modal Dialog, when open is true, then it is shown without modal mode', () => {
    render(Dialog, { ...props, open: true, modal: false });

    expect(HTMLDialogElement.prototype.show).toHaveBeenCalledOnce();
    expect(HTMLDialogElement.prototype.showModal).not.toHaveBeenCalled();
  });

  it('given open is false, when rendered, then the dialog stays closed', () => {
    const { container } = render(Dialog, { ...props, open: false, modal: true });

    expect(container.querySelector('dialog')).not.toHaveAttribute('open');
  });

  it('given an open Dialog, when open becomes false, then the dialog is closed', async () => {
    const { container, rerender } = render(Dialog, { ...props, open: true, modal: true });

    await rerender({ ...props, open: false, modal: true });

    expect(container.querySelector('dialog')).not.toHaveAttribute('open');
  });

  it('AC-D1: given an open Dialog, when Escape cancels it, then onclose is called and the native close is prevented', async () => {
    const onclose = vi.fn();
    const { container } = render(Dialog, { ...props, open: true, modal: true, onclose });

    const cancel = new Event('cancel', { cancelable: true });
    await fireEvent(container.querySelector('dialog') as HTMLDialogElement, cancel);

    expect(onclose).toHaveBeenCalledOnce();
    expect(cancel.defaultPrevented).toBe(true);
  });

  it('given a Dialog with a title, when rendered, then the dialog is labelled by the title', () => {
    const { container } = render(Dialog, { ...props, open: true, modal: true });

    const dialog = container.querySelector('dialog') as HTMLDialogElement;
    const labelId = dialog.getAttribute('aria-labelledby') as string;
    expect(container.querySelector(`[id="${labelId}"]`)).toHaveTextContent('Save');
  });
});
