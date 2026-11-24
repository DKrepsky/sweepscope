/** A point whose x or y is NaN is a missing sample. */
const isMissing = ([x, y]: [number, number]) => Number.isNaN(x) || Number.isNaN(y);

/** One path that restarts after every missing point; a lone point is drawn as a dot by the round cap. */
export function tracePath(ctx: CanvasRenderingContext2D, count: number, at: (index: number) => [number, number]) {
  let run = 0;
  ctx.beginPath();
  for (let i = 0; i < count; i++) {
    const p = at(i);
    if (isMissing(p)) {
      run = 0;
      continue;
    }
    if (run === 0) ctx.moveTo(...p);
    else ctx.lineTo(...p);
    run += 1;
    if (run === 1 && (i + 1 === count || isMissing(at(i + 1)))) ctx.lineTo(...p);
  }
}

export function strokeStyled(ctx: CanvasRenderingContext2D, colour: string, width: number, dash: number[]) {
  ctx.strokeStyle = colour;
  ctx.lineWidth = width;
  ctx.lineJoin = 'round';
  ctx.lineCap = 'round';
  ctx.setLineDash(dash);
}
