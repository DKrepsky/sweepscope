import { describe, expect, it } from 'vitest';
import { heightCollapse } from './collapse';

describe('heightCollapse', () => {
  it('given h1, when collapsed, then nothing is taken away except that the overview shows', () => {
    expect(heightCollapse('h1')).toEqual({
      recordOverview: true,
      smallRows: false,
      channelTabs: false,
      essentialStatusOnly: false,
      groupTabs: false,
    });
  });

  it('given h2, when collapsed, then the overview goes and the rows get small', () => {
    expect(heightCollapse('h2')).toMatchObject({ recordOverview: false, smallRows: true, channelTabs: false });
  });

  it('given h3, when collapsed, then channel tabs and the essential status items apply but the dock keeps its groups', () => {
    expect(heightCollapse('h3')).toMatchObject({ channelTabs: true, essentialStatusOnly: true, groupTabs: false });
  });

  it.each(['h4', 'scroll'] as const)(
    'given %s, when collapsed, then every step applies including dock tabs',
    (tier) => {
      expect(heightCollapse(tier)).toEqual({
        recordOverview: false,
        smallRows: true,
        channelTabs: true,
        essentialStatusOnly: true,
        groupTabs: true,
      });
    },
  );
});
