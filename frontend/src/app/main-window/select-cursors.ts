import type { InstrumentState } from '../../api';
import type { CursorsState } from '../cursors';
import type { ChannelNumber, CursorsModel, GraphModel } from './types';

const DISABLED: CursorsModel = {
  enabled: false,
  available: { enabled: false, reason: 'No waveform' },
  source: 1,
  values: [],
  originNote: '',
  lines: [],
};

/** STUB: the cursor readouts arrive with WP12; until then the model is the disabled one. */
export const selectCursors: (
  cursors: CursorsState,
  instrument: InstrumentState | undefined,
  graph: GraphModel['state'],
  selected: ChannelNumber,
) => CursorsModel = (_cursors, _instrument, graph, selected) => ({
  ...DISABLED,
  available: graph === 'empty' ? DISABLED.available : { enabled: true },
  source: selected,
});
