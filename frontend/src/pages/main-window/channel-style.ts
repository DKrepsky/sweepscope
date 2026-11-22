import type { ChannelNumber } from '../../app/main-window';

/** Channel colour custom properties, set inline so the components keep using token variables only. */
export const channelStyle = (number: ChannelNumber) =>
  `--chc: var(--ss-color-ch${number}); --cht: var(--ss-color-ch${number}-tint)`;
