export interface StrokeRecord {
  style: string;
  width: number;
  alpha: number;
  dash: number[];
  points: [number, number][];
}

export interface FillRecord {
  style: string;
  alpha: number;
  kind: 'path' | 'rect';
  points: [number, number][];
}

export interface TextRecord {
  text: string;
  x: number;
  y: number;
  align: string;
  baseline: string;
  style: string;
}

export interface ImageRecord {
  source: unknown;
  alpha: number;
}

interface PaintState {
  strokeStyle: string;
  fillStyle: string;
  lineWidth: number;
  globalAlpha: number;
  lineJoin: string;
  lineCap: string;
}

/** A fake 2D context for tests: records what each stroke and fill looked like when it was issued. */
class RecordingContext implements Partial<PaintState> {
  strokeStyle = '';
  fillStyle = '';
  lineWidth = 1;
  globalAlpha = 1;
  lineJoin = 'miter';
  lineCap = 'butt';
  font = '';
  textAlign = 'start';
  textBaseline = 'alphabetic';
  readonly strokes: StrokeRecord[] = [];
  readonly fills: FillRecord[] = [];
  readonly texts: TextRecord[] = [];
  readonly images: ImageRecord[] = [];
  private dash: number[] = [];
  private path: [number, number][] = [];
  private readonly saved: PaintState[] = [];

  private snapshot(): PaintState {
    const { strokeStyle, fillStyle, lineWidth, globalAlpha, lineJoin, lineCap } = this;
    return { strokeStyle, fillStyle, lineWidth, globalAlpha, lineJoin, lineCap };
  }

  save() {
    this.saved.push(this.snapshot());
  }
  restore() {
    Object.assign(this, this.saved.pop());
  }
  setTransform() {}
  clearRect() {}
  closePath() {}
  setLineDash(dash: number[]) {
    this.dash = dash;
  }
  beginPath() {
    this.path = [];
  }
  moveTo(x: number, y: number) {
    this.path.push([x, y]);
  }
  lineTo(x: number, y: number) {
    this.path.push([x, y]);
  }
  stroke() {
    this.strokes.push({
      style: this.strokeStyle,
      width: this.lineWidth,
      alpha: this.globalAlpha,
      dash: this.dash,
      points: this.path,
    });
  }
  fill() {
    this.fills.push({ style: this.fillStyle, alpha: this.globalAlpha, kind: 'path', points: this.path });
  }
  fillText(text: string, x: number, y: number) {
    const { textAlign: align, textBaseline: baseline, fillStyle: style } = this;
    this.texts.push({ text, x, y, align, baseline, style });
  }
  drawImage(source: unknown) {
    this.images.push({ source, alpha: this.globalAlpha });
  }
  fillRect(x: number, y: number) {
    this.fills.push({ style: this.fillStyle, alpha: this.globalAlpha, kind: 'rect', points: [[x, y]] });
  }
}

export function createRecordingContext() {
  const recording = new RecordingContext();
  return {
    ctx: recording as unknown as CanvasRenderingContext2D,
    strokes: recording.strokes,
    fills: recording.fills,
    texts: recording.texts,
    images: recording.images,
  };
}
