import { describe, expect, it } from 'vitest';
import { connectionSnapshot } from '../instrument/test-data';
import { selectConnection } from './select-status';

const identity = { model: 'Hantek 6254BC', firmware: '1.2', fpgaVersion: '3.4', calibration: 'valid' as const };

describe('selectConnection', () => {
  it('given a ready real scope, when selected, then the detail has model, firmware and FPGA', () => {
    const connection = connectionSnapshot({
      simulatorActive: false,
      identity: { ...identity, simulated: false },
    });

    expect(selectConnection(connection)).toEqual({
      state: 'Ready',
      detail: 'Hantek 6254BC · FW 1.2 · FPGA 3.4',
      tooltip: 'Firmware 1.2',
    });
  });

  it('given a ready simulator, when selected, then Simulator leads the detail', () => {
    const connection = connectionSnapshot({ simulatorActive: true, identity: { ...identity, simulated: true } });

    expect(selectConnection(connection).detail).toBe('Simulator · Hantek 6254BC');
  });

  it('given a connecting scope, when selected, then the detail says it is settling', () => {
    expect(selectConnection(connectionSnapshot({ state: 'connecting', identity: null }))).toMatchObject({
      state: 'Connecting',
      detail: 'Waiting for the scope to settle',
    });
  });

  it('given no connection slice yet, when selected, then it is Connecting', () => {
    expect(selectConnection(undefined).state).toBe('Connecting');
  });

  it('given no scope, when selected, then it is Disconnected with the connect hint', () => {
    expect(selectConnection(connectionSnapshot({ state: 'disconnected', identity: null }))).toMatchObject({
      state: 'Disconnected',
      detail: 'No scope found. Connect the scope',
    });
  });

  it('given an error with an issue, when selected, then the issue message is the detail', () => {
    const issue = { kind: 'permissionDenied' as const, message: 'No permission to open the scope', help: null };

    expect(selectConnection(connectionSnapshot({ state: 'error', issue }))).toMatchObject({
      state: 'Error',
      detail: 'No permission to open the scope',
    });
  });

  it('given an error without an issue, when selected, then a generic detail shows', () => {
    expect(selectConnection(connectionSnapshot({ state: 'error', issue: null })).detail).toBe('Connection failed');
  });
});
