import type { MeasurementsState } from '../../api';
import type { AliasNotice } from './types';

type Warning = MeasurementsState['warnings'][number];

const isAliasWarning = (warning: Warning) => warning.id.startsWith('alias-');

const warningsOf = (measurements: MeasurementsState | undefined): Warning[] =>
  measurements?.warnings.filter(isAliasWarning) ?? [];

export function aliasOf(measurements: MeasurementsState | undefined, channel: number): AliasNotice | undefined {
  const warning = warningsOf(measurements).find((candidate) => candidate.channel === channel);
  return warning && { short: warning.short, message: warning.message };
}

/** The status bar has room for one badge: the first warning, with a count when more channels alias. */
export function aliasSummary(measurements: MeasurementsState | undefined): AliasNotice | undefined {
  const [first, ...others] = warningsOf(measurements);
  if (!first) return undefined;
  const short = others.length > 0 ? `${first.short} +${others.length}` : first.short;
  return { short, message: [first, ...others].map((warning) => warning.message).join(' ') };
}
