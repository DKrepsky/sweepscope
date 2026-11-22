import { fireEvent, render } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { tokenNames } from '../../design-system';
import Gallery from './Gallery.svelte';

const designComponents = [
  'Button',
  'RunButton',
  'Segmented',
  'Stepper',
  'NumberField',
  'Badge',
  'StatusChip',
  'Panel',
  'MeasurementMatrix',
  'Readout',
  'Graph',
  'Marker',
  'StatusBar',
  'Banner',
  'Toast',
  'Dialog',
  'Menu',
  'Tooltip',
];

beforeEach(() => {
  window.matchMedia = (() => ({
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })) as unknown as typeof window.matchMedia;
  const noopContext = new Proxy({}, { get: () => () => undefined, set: () => true });
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(noopContext as CanvasRenderingContext2D);
});

afterEach(() => vi.restoreAllMocks());

describe('Gallery', () => {
  it('given the gallery, when mounted, then every component of the design inventory has a section', () => {
    const { container } = render(Gallery);
    const sections = [...container.querySelectorAll('[data-component]')].map((s) => s.getAttribute('data-component'));
    expect(sections).toEqual(expect.arrayContaining(designComponents));
  });

  it('given the gallery, when mounted, then no rendered inline style contains a raw colour literal', () => {
    const { container } = render(Gallery);
    const styles = [...container.querySelectorAll('[style]')].map((e) => e.getAttribute('style') ?? '');
    const raw = styles.filter((style) => /#[0-9a-f]{3,8}\b|\brgba?\(|\bhsla?\(/i.test(style));
    expect(raw).toEqual([]);
  });

  it('given the Tokens section, when mounted, then it has one swatch per colour token', () => {
    const { container } = render(Gallery);
    const tokensSection = container.querySelector('[data-component="Tokens"]')!;
    const colourTokens = tokenNames.filter((name) => name.startsWith('color-'));
    expect(tokensSection.querySelectorAll('[data-swatch]')).toHaveLength(colourTokens.length);
  });

  it('given the Tooltip section, when mounted, then the tooltip is forced visible', async () => {
    const { container } = render(Gallery);
    const section = container.querySelector('[data-component="Tooltip"]')!;
    await vi.waitFor(() => expect(section.querySelector('[role="tooltip"]')).not.toBeNull());
  });

  it('given the gallery, when a theme is passed, then the root carries the theme attribute', () => {
    render(Gallery, { props: { theme: 'dark' } });
    expect(document.documentElement.dataset.theme).toBe('dark');
  });

  it('given the Menu section, when mounted, then the trigger has haspopup and is collapsed', () => {
    const { container } = render(Gallery);
    const trigger = container.querySelector('[data-component="Menu"] button')!;
    expect(trigger.getAttribute('aria-haspopup')).toBe('menu');
    expect(trigger.getAttribute('aria-expanded')).toBe('false');
  });

  it('given the Menu section, when the menu is opened, then the trigger is expanded', async () => {
    const { container } = render(Gallery);
    const trigger = container.querySelector('[data-component="Menu"] button')!;
    await fireEvent.click(trigger);
    expect(trigger.getAttribute('aria-expanded')).toBe('true');
  });
});
