/** One drawn element of an icon on the 16 px grid. Stroke is `currentColor`; `filled` shapes are solid. */
export interface IconShape {
  readonly d: string;
  readonly filled?: boolean;
  readonly strokeWidth?: number;
  readonly dashArray?: string;
}

export interface IconDefinition {
  readonly viewBox: '0 0 16 16';
  readonly shapes: readonly IconShape[];
}

const grid = (...shapes: IconShape[]): IconDefinition => ({ viewBox: '0 0 16 16', shapes });
const line = (d: string): IconShape => ({ d });
const solid = (d: string): IconShape => ({ d, filled: true });
const circle = (cx: number, cy: number, r: number): string =>
  `M${cx - r} ${cy}a${r} ${r} 0 1 0 ${2 * r} 0a${r} ${r} 0 1 0 ${-2 * r} 0`;

export const icons = {
  play: grid(solid('M4 2.5v11l9-5.5z')),
  stop: grid(solid('M3 3h10v10H3z')),
  single: grid(solid('M2.5 3v10l7-5z'), line('M12 4v8')),
  force: grid(line('M9 1.5L3.5 9H8l-1 5.5L12.5 7H8z')),
  autoset: grid(line('M2 12l3-8 3 8M3.2 9.5h3.6M10 4v8M10 8h4')),
  cursors: grid(line('M4.5 2v12M11.5 2v12M4.5 8h7M6.5 6l-2 2 2 2M9.5 6l2 2-2 2')),
  math: grid(line('M3 8h10M8 3v10')),
  reference: grid({ d: 'M2 11c2-6 3-6 5 0s3 6 5 0M13 4h1', dashArray: '2 2' }),
  save: grid(line('M2.5 2.5h9l2 2v9h-11zM5 2.5v4h5v-4M5 13.5v-4h6v4')),
  screenshot: grid(line('M2 5h3l1-2h4l1 2h3v8H2z'), line(circle(8, 8.5, 2.2))),
  settings: grid(line('M2 4.5h12M2 8h12M2 11.5h12'), { d: 'M5 3v3M10 6.5v3M7 10v3', strokeWidth: 2.5 }),
  minus: grid(line('M4 8h8')),
  plus: grid(line('M4 8h8M8 4v8')),
  chevron: grid(line('M4 6l4 4 4-4')),
  'rising-edge': grid(line('M2 12h4V4h8')),
  'falling-edge': grid(line('M2 4h4v8h8')),
  warning: grid(line('M8 2l6.5 11.5h-13zM8 6.5v3.2M8 11.4v.4')),
  info: grid(line(circle(8, 8, 6)), line('M8 7.2v4.3M8 4.6v.4')),
  record: grid(solid(circle(8, 8, 4.5))),
  'pulse-positive': grid(line('M2 12h3.5V4h5v8H14')),
  'pulse-negative': grid(line('M2 4h3.5v8h5V4H14')),
  peak: grid(solid('M3 5h10L8 12z'), line('M8 2v1.5')),
  close: grid(line('M4 4l8 8M12 4l-8 8')),
  disconnected: grid(line('M7 5l1-1a2.5 2.5 0 0 1 3.5 3.5l-1 1M9 11l-1 1a2.5 2.5 0 0 1-3.5-3.5l1-1M3 13L13 3')),
} as const satisfies Record<string, IconDefinition>;

export type IconName = keyof typeof icons;
export const iconNames = Object.keys(icons) as IconName[];
