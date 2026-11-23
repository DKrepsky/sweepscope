export type Quantity = 'volts' | 'seconds' | 'hertz';
export type Formatted = { text: string; unit: string };

const GROUP = 3;
const LOWEST = -12;
const HIGHEST = 9;
const SIGNIFICANT_DIGITS = 3;
const MAX_DECIMALS = 15;
const GROUP_TOLERANCE = 1e-12;
const NEXT_GROUP_AT = 1000;
const TRUE_MINUS = '−';
const NO_VALUE = '-';

const SYMBOLS: Record<Quantity, string> = { volts: 'V', seconds: 's', hertz: 'Hz' };
const PREFIXES: Record<number, string> = { [-12]: 'p', [-9]: 'n', [-6]: 'u', [-3]: 'm', 0: '', 3: 'k', 6: 'M', 9: 'G' };

export function formatQuantity(value: number, quantity: Quantity): Formatted {
  if (!Number.isFinite(value)) return { text: NO_VALUE, unit: unit(quantity, 0) };
  const { text, exponent } = render(value);
  return { text: text.replace('-', TRUE_MINUS), unit: unit(quantity, exponent) };
}

function unit(quantity: Quantity, exponent: number): string {
  return `${PREFIXES[exponent] ?? ''}${SYMBOLS[quantity]}`;
}

function render(value: number): { text: string; exponent: number } {
  const magnitude = Math.abs(value);
  let exponent = groupFor(magnitude);
  let text = threeSignificant(magnitude / 10 ** exponent);
  if (exponent + GROUP <= HIGHEST && Number(text) >= NEXT_GROUP_AT) {
    exponent += GROUP;
    text = threeSignificant(magnitude / 10 ** exponent);
  }
  const sign = value < 0 && magnitude !== 0 ? '-' : '';
  return { text: `${sign}${text}`, exponent };
}

function groupFor(magnitude: number): number {
  if (magnitude === 0) return 0;
  for (let group = HIGHEST; group >= LOWEST; group -= GROUP) {
    if (magnitude >= 10 ** group * (1 - GROUP_TOLERANCE)) return group;
  }
  return LOWEST;
}

function threeSignificant(scaled: number): string {
  const first = scaled.toFixed(decimalsFor(scaled));
  const rounded = Number(first);
  const decimals = decimalsFor(rounded);
  return decimals === decimalsFor(scaled) ? first : rounded.toFixed(decimals);
}

function decimalsFor(magnitude: number): number {
  if (magnitude === 0) return SIGNIFICANT_DIGITS - 1;
  let decimals = SIGNIFICANT_DIGITS - 1;
  let probe = magnitude;
  while (probe >= 10 && decimals > 0) {
    probe /= 10;
    decimals -= 1;
  }
  while (probe < 1 && decimals < MAX_DECIMALS) {
    probe *= 10;
    decimals += 1;
  }
  return decimals;
}
