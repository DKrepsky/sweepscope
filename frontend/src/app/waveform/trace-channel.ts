import type { ChannelNumber } from '../main-window/types';

/** The four inputs and the math trace (5, column `M`). */
export type TraceChannel = ChannelNumber | 5;

const TRACES: readonly TraceChannel[] = [1, 2, 3, 4, 5];

/** The channels a wire mask selects, ascending (bit 0 = CH1, bit 4 = M). */
export function channelsInMask(mask: number): TraceChannel[] {
  return TRACES.filter((_, bit) => (mask & (1 << bit)) !== 0);
}
