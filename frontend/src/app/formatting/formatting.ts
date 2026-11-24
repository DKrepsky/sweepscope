export type Quantity = 'volts' | 'seconds' | 'hertz' | 'frequency' | 'decibel' | 'volts_squared_per_div';
export type Formatted = { text: string; unit: string };

const GROUP = 3;
const LOWEST = -12;
const HIGHEST = 9;
const DEFAULT_DIGITS = 3;
const FREQUENCY_DIGITS = 4;
const DECIBEL_DECIMALS = 1;
const EXACT_DECIMALS = 6;
const MAX_DECIMALS = 15;
const GROUP_TOLERANCE = 1e-12;
const NEXT_GROUP_AT = 1000;
const TRUE_MINUS = '−';
const NO_VALUE = '-';
const DBV_UNIT = 'dBV';

const SYMBOLS: Record<Quantity, string> = {
  volts: 'V',
  seconds: 's',
  hertz: 'Hz',
  frequency: 'Hz',
  decibel: 'dB',
  volts_squared_per_div: 'V²/div',
};
const PREFIXES: Record<number, string> = { [-12]: 'p', [-9]: 'n', [-6]: 'u', [-3]: 'm', 0: '', 3: 'k', 6: 'M', 9: 'G' };

type Rule = { lowest: number; highest: number; number: (scaled: number) => string };

const significant = (digits: number) => (scaled: number) => significantDigits(scaled, digits);
const RULES: Record<Quantity, Rule> = {
  volts: { lowest: LOWEST, highest: HIGHEST, number: significant(DEFAULT_DIGITS) },
  seconds: { lowest: LOWEST, highest: HIGHEST, number: significant(DEFAULT_DIGITS) },
  hertz: { lowest: LOWEST, highest: HIGHEST, number: significant(DEFAULT_DIGITS) },
  frequency: { lowest: LOWEST, highest: HIGHEST, number: significant(FREQUENCY_DIGITS) },
  decibel: { lowest: 0, highest: 0, number: (scaled) => scaled.toFixed(DECIBEL_DECIMALS) },
  volts_squared_per_div: { lowest: 0, highest: 0, number: exact },
};

export function formatQuantity(value: number, quantity: Quantity): Formatted {
  if (!Number.isFinite(value)) return { text: NO_VALUE, unit: unit(quantity, 0) };
  const { text, exponent } = render(value, RULES[quantity]);
  return { text: text.replace('-', TRUE_MINUS), unit: unit(quantity, exponent) };
}

export function formatFrequency(hz: number): string {
  const { text, unit: symbol } = formatQuantity(hz, 'frequency');
  return `${text} ${symbol}`;
}

export function formatDbv(db: number): string {
  const { text } = formatQuantity(db, 'decibel');
  return `${text} ${DBV_UNIT}`;
}

function unit(quantity: Quantity, exponent: number): string {
  return `${PREFIXES[exponent] ?? ''}${SYMBOLS[quantity]}`;
}

function render(value: number, rule: Rule): { text: string; exponent: number } {
  const magnitude = Math.abs(value);
  let exponent = groupFor(magnitude, rule);
  let text = rule.number(magnitude / 10 ** exponent);
  if (exponent + GROUP <= rule.highest && Number(text) >= NEXT_GROUP_AT) {
    exponent += GROUP;
    text = rule.number(magnitude / 10 ** exponent);
  }
  const sign = value < 0 && Number(text) !== 0 ? '-' : '';
  return { text: `${sign}${text}`, exponent };
}

function groupFor(magnitude: number, rule: Rule): number {
  if (magnitude === 0) return 0;
  for (let group = rule.highest; group >= rule.lowest; group -= GROUP) {
    if (magnitude >= 10 ** group * (1 - GROUP_TOLERANCE)) return group;
  }
  return rule.lowest;
}

function exact(scaled: number): string {
  return scaled.toFixed(EXACT_DECIMALS).replace(/0+$/, '').replace(/\.$/, '');
}

function significantDigits(scaled: number, digits: number): string {
  const first = scaled.toFixed(decimalsFor(scaled, digits));
  const rounded = Number(first);
  const decimals = decimalsFor(rounded, digits);
  return decimals === decimalsFor(scaled, digits) ? first : rounded.toFixed(decimals);
}

function decimalsFor(magnitude: number, digits: number): number {
  if (magnitude === 0) return digits - 1;
  let decimals = digits - 1;
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
