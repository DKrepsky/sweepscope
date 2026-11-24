import type { AcquisitionStatus, ConnectionSnapshot, InstrumentState } from '../../api';
import type { ChipModel } from '../main-window/types';

export interface ConnectionChip {
  state: 'Connected' | 'Connecting' | 'Disconnected' | 'Error';
  detail: string;
  tooltip: string;
  simulated: boolean;
}

type Connection = ConnectionSnapshot | undefined;

export function isAcquiring(connection: Connection, instrument: InstrumentState | undefined): boolean {
  return connection?.state === 'ready' && instrument?.running === true;
}

export type GraphState = 'live' | 'frozen' | 'empty';

export function graphState(
  connection: Connection,
  instrument: InstrumentState | undefined,
  hasFrame: boolean,
): GraphState {
  if (!hasFrame) return 'empty';
  return isAcquiring(connection, instrument) ? 'live' : 'frozen';
}

const AUTO_TOOLTIP = 'Shown while the sweep is auto-triggering without a trigger event';

const ROLL_CHIP: ChipModel = { id: 'mode', kind: 'roll', text: 'Roll' };

const DONE_TOOLTIP = 'The single capture finished';

const runChips: Record<AcquisitionStatus['phase'], ChipModel | undefined> = {
  stopped: undefined,
  armed: { id: 'trigger', kind: 'armed', text: 'Armed' },
  waitingForTrigger: { id: 'trigger', kind: 'armed', text: 'Waiting for trigger' },
  triggered: { id: 'trigger', kind: 'triggered', text: '▲ Trig’d', filled: true },
  auto: { id: 'trigger', kind: 'auto', text: 'Auto', dim: true, tooltip: AUTO_TOOLTIP },
  done: { id: 'trigger', kind: 'triggered', text: 'Done', tooltip: DONE_TOOLTIP },
};

function unavailableChip(connection: Connection): ChipModel {
  if (connection?.state === 'error') return { id: 'state', kind: 'error', text: 'Error' };
  if (connection?.state === 'disconnected') return { id: 'state', kind: 'offline', text: 'Offline' };
  return { id: 'state', kind: 'offline', text: 'Connecting' };
}

const rollChips = (instrument: InstrumentState | undefined): ChipModel[] =>
  instrument?.horizontal.roll ? [ROLL_CHIP] : [];

export function statusChips(
  connection: Connection,
  instrument: InstrumentState | undefined,
  status: AcquisitionStatus | undefined,
): ChipModel[] {
  if (connection?.state !== 'ready') return [unavailableChip(connection)];
  const phase = status ? runChips[status.phase] : undefined;
  if (!isAcquiring(connection, instrument)) {
    const stopped: ChipModel = { id: 'state', kind: 'stop', text: 'Stop' };
    return status?.phase === 'done' && phase ? [stopped, phase] : [stopped];
  }
  return [{ id: 'state', kind: 'run', text: 'Run' }, ...rollChips(instrument), ...(phase ? [phase] : [])];
}

function readyChip(connection: ConnectionSnapshot): ConnectionChip {
  const identity = connection.identity;
  const firmware = identity?.firmware ? `Firmware ${identity.firmware}` : '';
  return {
    state: 'Connected',
    detail: identity?.model ?? '',
    tooltip: firmware,
    simulated: connection.simulatorActive,
  };
}

export function connectionChip(connection: Connection): ConnectionChip {
  const simulated = connection?.simulatorActive ?? false;
  switch (connection?.state) {
    case 'ready':
      return readyChip(connection);
    case 'disconnected':
      return { state: 'Disconnected', detail: 'No scope found', tooltip: '', simulated };
    case 'error':
      return { state: 'Error', detail: connection.issue?.message ?? 'Connection failed', tooltip: '', simulated };
    default:
      return { state: 'Connecting', detail: '', tooltip: '', simulated };
  }
}
