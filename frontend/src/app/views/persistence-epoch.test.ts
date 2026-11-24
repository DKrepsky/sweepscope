import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createPersistenceEpoch } from './persistence-epoch';

describe('persistence epoch', () => {
  it('given a new epoch, when cleared twice, then it has advanced by two', () => {
    const epoch = createPersistenceEpoch();
    const start = get(epoch);

    epoch.clearPersistence();
    epoch.clearPersistence();

    expect(get(epoch)).toBe(start + 2);
  });
});
