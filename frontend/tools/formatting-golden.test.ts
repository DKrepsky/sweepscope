// @vitest-environment node
/// <reference types="node" />
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { formatQuantity } from '../src/app/formatting';
import type { Quantity } from '../src/app/formatting';

const GOLDEN = new URL('../../backend/libs/engineering-format/tests/golden.tsv', import.meta.url);

const rows = readFileSync(GOLDEN, 'utf8')
  .split('\n')
  .filter((line) => line.length > 0)
  .map((line) => line.split('\t'));

describe('formatQuantity golden vectors shared with the Rust crate', () => {
  it('given the shared golden file, when it is read, then every row has four columns', () => {
    expect(rows.length).toBeGreaterThan(0);
    expect(rows.every((row) => row.length === 4)).toBe(true);
  });

  it.each(rows)('given %s as %s, when formatted, then it reads %s and %s', (value, quantity, text, unit) => {
    expect(formatQuantity(Number(value), quantity as Quantity)).toEqual({ text, unit });
  });
});
