import type { MainWindowModel, MeasurementCellModel } from '../../app/main-window';
import { drawScreenshot, specSize, type ScreenshotCell, type ScreenshotSpec } from '../../ui/screenshot';

const NO_VALUE = '—';

function headerLines(model: MainWindowModel): string[] {
  const { horizontal, trigger, cursors } = model;
  const channels = model.channels
    .filter((channel) => channel.enabled)
    .map((c) => {
      const coupling = c.coupling.toUpperCase();
      return `CH${c.number}  ${c.voltsPerDiv.value} ${c.voltsPerDiv.unit}  ${coupling}  probe ${c.probe}  position ${c.position.value} ${c.position.unit}`;
    });
  const time = `Time  ${horizontal.timePerDiv.value} ${horizontal.timePerDiv.unit}  ${horizontal.rateNote}`;
  const level = `${trigger.level.value} ${trigger.level.unit}`;
  const edge = `Trigger  CH${trigger.source}  ${trigger.slope}  ${level}  ${trigger.sweep}`;
  const readouts = cursors.enabled ? [`Cursors  ${cursors.values.map((v) => `${v.label} ${v.value}`).join('  ')}`] : [];
  return ['SweepScope', time, ...channels, edge, ...readouts];
}

function cellOf(cell: MeasurementCellModel | undefined): ScreenshotCell {
  if (!cell) return { text: NO_VALUE, muted: true };
  const mark = cell.footnote === undefined ? '' : ` [${cell.footnote}]`;
  if (cell.state === 'invalid' || cell.text === '') return { text: `${NO_VALUE}${mark}`, muted: true };
  const value = cell.unit ? `${cell.text} ${cell.unit}` : cell.text;
  return { text: `${value}${mark}`, muted: cell.state === 'stale' };
}

function markers(model: MainWindowModel): Pick<ScreenshotSpec, 'cursors' | 'triggerMarkers'> {
  const { trigger, cursors } = model;
  const lines = cursors.enabled ? cursors.lines : [];
  const triggerShown = trigger.availability?.enabled !== false;
  return {
    cursors: lines.map(({ axis, position, tag }) => ({ axis, position, tag })),
    triggerMarkers: triggerShown
      ? [
          { axis: 'y', position: 1 - trigger.levelPosition, tag: trigger.levelTag },
          { axis: 'x', position: trigger.timePosition, tag: trigger.positionTag },
        ]
      : [],
  };
}

export function buildScreenshotSpec(model: MainWindowModel): ScreenshotSpec {
  const { measurements, graph, horizontal } = model;
  const frozenLabel = graph.state === 'frozen' ? 'Frozen' : measurements.frozenLabel;
  return {
    graph: {
      state: graph.state,
      channels: graph.channels,
      selected: graph.selected,
      dots: horizontal.interpolation === 'dots',
    },
    header: headerLines(model),
    ...markers(model),
    table: {
      columns: measurements.columns.map(({ label, color }) => ({ label, color })),
      rows: measurements.rows.map((row) => ({
        label: row.label,
        cells: measurements.columns.map((column) => cellOf(row.cells[column.id])),
      })),
    },
    footnotes: measurements.footnotes.map(({ key, text }) => `[${key}] ${text}`),
    frozenLabel,
  };
}

interface PngCanvas {
  context: CanvasRenderingContext2D;
  toPng: () => Promise<Blob>;
}

function createCanvas(width: number, height: number): PngCanvas {
  if (typeof OffscreenCanvas !== 'undefined') {
    const canvas = new OffscreenCanvas(width, height);
    const context = canvas.getContext('2d') as unknown as CanvasRenderingContext2D | null;
    if (context) return { context, toPng: () => canvas.convertToBlob({ type: 'image/png' }) };
  }
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('Screenshots are not available: no 2D canvas');
  const toPng = () =>
    new Promise<Blob>((resolve, reject) => {
      canvas.toBlob(
        (blob) => (blob ? resolve(blob) : reject(new Error('The screenshot could not be encoded'))),
        'image/png',
      );
    });
  return { context, toPng };
}

/** Draws the window into a canvas of css size times `ratio` and returns the PNG bytes. */
export async function composeScreenshot(model: MainWindowModel, ratio: number): Promise<Uint8Array> {
  const spec = buildScreenshotSpec(model);
  const size = specSize(spec);
  const { context, toPng } = createCanvas(Math.round(size.width * ratio), Math.round(size.height * ratio));
  drawScreenshot(context, spec, ratio);
  return new Uint8Array(await (await toPng()).arrayBuffer());
}
