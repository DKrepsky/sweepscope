import type { MeasurementsState } from '../../api';
import type { GraphModel, MeasurementsModel } from './types';

const EMPTY: MeasurementsModel = { columns: [], rows: [], footnotes: [], accuracyNote: '', pinnable: [] };

/** STUB: the rows, footnotes and pin menu arrive with WP13; until then the model is empty. */
export const selectMeasurements: (
  measurements: MeasurementsState | undefined,
  pinned: readonly string[],
  graph: GraphModel['state'],
) => MeasurementsModel = () => EMPTY;
