import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { iconNames, type IconName } from '../../design-system';
import Icon from './Icon.svelte';

describe('Icon', () => {
  it.each(iconNames)('given the icon %s, when rendered, then an aria-hidden svg is present', (name) => {
    const { container } = render(Icon, { name });
    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('aria-hidden', 'true');
    expect(svg?.querySelectorAll('path').length).toBeGreaterThan(0);
  });

  it('given the icon set, when counted, then it has 25 names', () => {
    expect(iconNames).toHaveLength(25);
  });

  it('given a name outside the set, when passed to Icon, then TypeScript rejects it', () => {
    // @ts-expect-error not an icon name
    const unknown: IconName = 'no-such-icon';
    expect(iconNames).not.toContain(unknown);
  });

  it('given a filled shape, when rendered, then its path has the filled class', () => {
    const { container } = render(Icon, { name: 'play' });
    expect(container.querySelector('path')?.getAttribute('class')).toContain('filled');
  });
});
