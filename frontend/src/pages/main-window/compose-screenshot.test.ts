import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import type { MainWindowModel } from '../../app/main-window';
import { createMockMainWindow } from '../../app/main-window/mock-main-window';
import { buildScreenshotSpec } from './compose-screenshot';

const model = (): MainWindowModel => get(createMockMainWindow());

describe('buildScreenshotSpec', () => {
  it('given enabled channels when built then the header names each of them', () => {
    const header = buildScreenshotSpec(model()).header;

    expect(header.filter((line) => /^CH\d/.test(line))).toHaveLength(3);
  });

  it('given an invalid cell when built then the table shows a dash with its footnote', () => {
    const { table } = buildScreenshotSpec(model());

    const texts = table.rows.flatMap((row) => row.cells.map((cell) => cell.text));
    expect(texts.some((text) => /^— \[\d\]$/.test(text))).toBe(true);
  });

  it('given cursors enabled when built then their lines become markers', () => {
    const current = model();
    const withCursors: MainWindowModel = {
      ...current,
      cursors: {
        ...current.cursors,
        enabled: true,
        lines: [{ id: 'x1', axis: 'x', position: 0.3, tag: 'X1', valueText: '' }],
      },
    };

    expect(buildScreenshotSpec(withCursors).cursors).toEqual([{ axis: 'x', position: 0.3, tag: 'X1' }]);
  });

  it('given a frozen graph when built then the picture says Frozen', () => {
    const current = model();

    const spec = buildScreenshotSpec({ ...current, graph: { ...current.graph, state: 'frozen' } });

    expect(spec.frozenLabel).toBe('Frozen');
  });
});
