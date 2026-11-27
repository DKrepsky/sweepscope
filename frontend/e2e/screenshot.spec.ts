import { expect, test } from '@playwright/test';
import { openMain } from './support/app';

const RATIOS = [1, 1.25, 1.5, 2];

interface Decoded {
  expectedWidth: number;
  expectedHeight: number;
  width: number;
  height: number;
  graphLeft: number;
  graphWidth: number;
  row: number;
  runs: number[];
  isPng: boolean;
}

/** Composes a screenshot of the empty-graph mock window, decodes the PNG and scans one graticule row. */
async function composeAndScan(page: import('@playwright/test').Page): Promise<Decoded> {
  return page.evaluate(async () => {
    const composePath = '/src/pages/main-window/compose-screenshot.ts';
    const mockPath = '/src/app/main-window/mock-main-window.ts';
    const layoutPath = '/src/ui/screenshot/screenshot-layout.ts';
    const compose = (await import(
      /* @vite-ignore */ composePath
    )) as typeof import('../src/pages/main-window/compose-screenshot');
    const mock = (await import(
      /* @vite-ignore */ mockPath
    )) as typeof import('../src/app/main-window/mock-main-window');
    const layout = (await import(
      /* @vite-ignore */ layoutPath
    )) as typeof import('../src/ui/screenshot/screenshot-layout');

    let model!: Parameters<typeof compose.composeScreenshot>[0];
    mock.createMockMainWindow().subscribe((value) => (model = value))();
    model = { ...model, graph: { ...model.graph, state: 'empty' } };

    const ratio = window.devicePixelRatio;
    const bytes = await compose.composeScreenshot(model, ratio);
    const isPng = [0x89, 0x50, 0x4e, 0x47].every((byte, i) => bytes[i] === byte);
    const bitmap = await createImageBitmap(new Blob([bytes as BlobPart], { type: 'image/png' }));
    const canvas = document.createElement('canvas');
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const ctx = canvas.getContext('2d')!;
    ctx.drawImage(bitmap, 0, 0);

    const spec = compose.buildScreenshotSpec(model);
    const noteLines = layout.screenshotNotes(spec.footnotes).length;
    const content = { header: spec.header, tableRows: spec.table.rows.length, noteLines };
    const size = layout.screenshotSize(content);
    const { graph } = layout.screenshotLayout(content);
    const graphLeft = Math.round(graph.x * ratio);
    const graphWidth = layout.GRAPH_SIZE.width * ratio;
    const row = Math.round(graph.y * ratio) + Math.floor(layout.GRAPH_SIZE.height * ratio * 0.0625);
    const data = ctx.getImageData(graphLeft, row, Math.round(graphWidth), 1).data;
    const runs: number[] = [];
    let run = 0;
    for (let i = 0; i < data.length; i += 4) {
      const background = data[i] === 16 && data[i + 1] === 18 && data[i + 2] === 20; // color-bg-scope
      if (!background) run += 1;
      else if (run) {
        runs.push(run);
        run = 0;
      }
    }
    if (run) runs.push(run);
    return {
      expectedWidth: Math.round(size.width * ratio),
      expectedHeight: Math.round(size.height * ratio),
      width: bitmap.width,
      height: bitmap.height,
      graphLeft,
      graphWidth,
      row,
      runs,
      isPng,
    };
  });
}

for (const deviceScaleFactor of RATIOS) {
  test.describe(`device pixel ratio ${deviceScaleFactor}`, () => {
    test.use({ deviceScaleFactor });

    test('given a model when composed then the PNG is css size times the ratio', async ({ page }) => {
      await openMain(page);

      const result = await composeAndScan(page);

      expect(result.isPng).toBe(true);
      expect(result.width).toBe(result.expectedWidth);
      expect(result.height).toBe(result.expectedHeight);
    });

    test('given the empty screen when composed then every graticule line is one device pixel wide', async ({
      page,
    }) => {
      await openMain(page);

      const { runs } = await composeAndScan(page);

      // Nine major lines and the two frame edges cross the row (the axis replaces the centre major line).
      expect(runs.length).toBeGreaterThanOrEqual(11);
      expect(runs.every((length) => length === 1)).toBe(true);
    });
  });
}
