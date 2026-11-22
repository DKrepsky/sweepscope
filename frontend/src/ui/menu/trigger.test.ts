import { describe, expect, it } from 'vitest';
import { menuTriggerAttributes } from './trigger';

describe('menuTriggerAttributes', () => {
  it('given a closed menu, when attributes are requested, then the trigger announces a collapsed menu popup', () => {
    expect(menuTriggerAttributes(false)).toEqual({ 'aria-haspopup': 'menu', 'aria-expanded': false });
  });

  it('given an open menu, when attributes are requested, then the trigger announces an expanded menu popup', () => {
    expect(menuTriggerAttributes(true)).toEqual({ 'aria-haspopup': 'menu', 'aria-expanded': true });
  });
});
