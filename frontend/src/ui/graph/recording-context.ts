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

interface PaintState {
  strokeStyle: string;
  fillStyle: string;
  lineWidth: number;
  globalAlpha: number;
  lineJoin: string;
}

/** A fake 2D context for tests: records what each stroke and fill looked like when it was issued. */
class RecordingContext implements Partial<PaintState> {
  strokeStyle = '';
  fillStyle = '';
  lineWidth = 1;
  globalAlpha = 1;
  lineJoin = 'miter';
  readonly strokes: StrokeRecord[] = [];
  readonly fills: FillRecord[] = [];
  private dash: number[] = [];
  private path: [number, number][] = [];
  private readonly saved: PaintState[] = [];

  private snapshot(): PaintState {
    const { strokeStyle, fillStyle, lineWidth, globalAlpha, lineJoin } = this;
    return { strokeStyle, fillStyle, lineWidth, globalAlpha, lineJoin };
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
  fillRect(x: number, y: number) {
    this.fills.push({ style: this.fillStyle, alpha: this.globalAlpha, kind: 'rect', points: [[x, y]] });
  }
}

export function createRecordingContext() {
  const recording = new RecordingContext();
  return { ctx: recording as unknown as CanvasRenderingContext2D, strokes: recording.strokes, fills: recording.fills };
}
