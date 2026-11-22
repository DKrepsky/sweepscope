import type { ConnectionSnapshot, InstrumentState } from '../../api';
import type { Availability, ChannelNumber, Quantity } from './types';

export type BackendAvailability = InstrumentState['controls']['run'];
export type FormattedValue = InstrumentState['channels'][number]['position'];

export const SCREEN_VERTICAL_DIVISIONS = 8;

export const channelNumber = (number: number): ChannelNumber =>
  Math.min(4, Math.max(1, Math.round(number))) as ChannelNumber;

export const toAvailability = ({ enabled, reason }: BackendAvailability): Availability =>
  reason ? { enabled, reason } : { enabled };

export const toQuantity = ({ text, unit }: FormattedValue): Quantity => ({ value: text, unit });

/** Divisions from the screen centre (up positive) to a 0 (bottom) to 1 (top) screen fraction. */
export const fractionFromCentre = (divisions: number): number =>
  Math.min(1, Math.max(0, 0.5 + divisions / SCREEN_VERTICAL_DIVISIONS));

/** Why nothing can be changed while no instrument state exists, or none is ready. */
export function unavailableReason(connection: ConnectionSnapshot | undefined): string {
  if (connection?.state === 'error') return connection.issue?.message ?? 'Connection failed';
  if (connection?.state === 'connecting' || connection === undefined) return 'Connecting';
  return 'No scope connected';
}

export const NOT_MEASURED = '-';
