// Used by tests only: colour-vision-deficiency simulation (Machado et al. 2009, severity 1.0, linear RGB)
// and CIE76 distance in CIELAB (D65).
import { parseHex, toLinear, type Rgb } from './contrast';

type Matrix = readonly [Rgb, Rgb, Rgb];

export const deficiencies = {
  protanopia: [
    [0.152286, 1.052583, -0.204868],
    [0.114503, 0.786281, 0.099216],
    [-0.003882, -0.048116, 1.051998],
  ],
  deuteranopia: [
    [0.367322, 0.860646, -0.227968],
    [0.280085, 0.672501, 0.047413],
    [-0.01182, 0.04294, 0.968881],
  ],
  tritanopia: [
    [1.255528, -0.076749, -0.178779],
    [-0.078411, 0.930809, 0.147602],
    [0.004733, 0.691367, 0.3039],
  ],
} as const satisfies Record<string, Matrix>;

export type Deficiency = keyof typeof deficiencies;

const toRgbTriplet = (hex: string): Rgb => parseHex(hex).map(toLinear) as unknown as Rgb;
const clamp = (value: number) => Math.min(1, Math.max(0, value));
const multiply = (matrix: Matrix, [r, g, b]: Rgb): Rgb =>
  matrix.map((row) => clamp(row[0] * r + row[1] * g + row[2] * b)) as unknown as Rgb;

const toXyz = ([r, g, b]: Rgb): Rgb => [
  0.4124564 * r + 0.3575761 * g + 0.1804375 * b,
  0.2126729 * r + 0.7151522 * g + 0.072175 * b,
  0.0193339 * r + 0.119192 * g + 0.9503041 * b,
];

const labCurve = (t: number) => (t > 216 / 24389 ? Math.cbrt(t) : ((24389 / 27) * t + 16) / 116);

export function toLab(linear: Rgb): Rgb {
  const [x, y, z] = toXyz(linear);
  const [fx, fy, fz] = [labCurve(x / 0.95047), labCurve(y), labCurve(z / 1.08883)];
  return [116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)];
}

export function simulate(hex: string, deficiency: Deficiency | 'normal'): Rgb {
  const linear = toRgbTriplet(hex);
  return toLab(deficiency === 'normal' ? linear : multiply(deficiencies[deficiency], linear));
}

export function labDistance(a: Rgb, b: Rgb): number {
  return Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
}

export function minimumPairwiseDistance(colours: readonly string[], deficiency: Deficiency | 'normal'): number {
  const labs = colours.map((colour) => simulate(colour, deficiency));
  const distances = labs.flatMap((a, i) => labs.slice(i + 1).map((b) => labDistance(a, b)));
  return Math.min(...distances);
}
