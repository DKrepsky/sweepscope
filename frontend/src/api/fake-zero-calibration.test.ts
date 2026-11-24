import { describe, expect, it } from 'vitest';
import { createFakeClient, type FakeClient } from './fake-client';
import { DEVICE_LOST_TEXT } from './fake-zero-calibration';

async function openAndStart(client: FakeClient) {
  await client.send('open_zero_calibration', {});
  await client.send('start_zero_calibration', {});
}

describe('fake zero calibration', () => {
  it('given the fake client, when a session is opened and started, then it is on channel 1 after the intro', async () => {
    const client = createFakeClient();
    const seen: string[] = [];
    client.subscribe('zero_calibration_changed', (state) => seen.push(state.session?.step ?? 'closed'));

    await openAndStart(client);

    expect(seen).toEqual(['intro', 'channel']);
    expect(client.zeroCalibration.state().session).toMatchObject({ step: 'channel', channel: 1 });
  });

  it('given shorted inputs, when all four channels are measured, then it is done with the table and the unit is calibrated', async () => {
    const client = createFakeClient();
    await openAndStart(client);

    for (let channel = 0; channel < 4; channel++) await client.send('measure_zero_calibration', {});

    const state = client.zeroCalibration.state();
    expect(state.session).toMatchObject({ step: 'done', doneChannels: [true, true, true, true] });
    expect(state.session?.table).toHaveLength(12);
    expect(state.unit?.calibrated).toBe(true);
  });

  it('given channel 2 is not shorted, when measured, then the session stops there and stores nothing', async () => {
    const client = createFakeClient();
    client.zeroCalibration.leaveUnshorted(2);
    await openAndStart(client);

    await client.send('measure_zero_calibration', {});
    await client.send('measure_zero_calibration', {});

    const state = client.zeroCalibration.state();
    expect(state.session).toMatchObject({ step: 'notShorted', channel: 2, doneChannels: [true, false, false, false] });
    expect(state.session?.notShorted).toHaveLength(1);
    expect(state.unit?.calibrated).toBe(false);
  });

  it('given a session, when it is closed, then the slice has no session', async () => {
    const client = createFakeClient();
    await openAndStart(client);

    await client.send('close_zero_calibration', {});

    expect(client.zeroCalibration.state().session).toBeNull();
  });

  it('given the device is lost mid session, when it happens, then the session fails and calibration is unavailable', async () => {
    const client = createFakeClient();
    await openAndStart(client);

    client.zeroCalibration.loseDevice();

    const state = client.zeroCalibration.state();
    expect(state.session).toMatchObject({ step: 'failed', failure: DEVICE_LOST_TEXT });
    expect(state.availability.enabled).toBe(false);
  });

  it('given no device, when a session is opened, then the command is refused', async () => {
    const client = createFakeClient();
    client.zeroCalibration.loseDevice();

    await expect(client.send('open_zero_calibration', {})).rejects.toThrow('cannot open');
  });

  it('given no session, when a channel is measured, then the command is refused', async () => {
    const client = createFakeClient();

    await expect(client.send('measure_zero_calibration', {})).rejects.toThrow('cannot measure');
  });

  it('given a measured channel, when the events are read, then the revisions strictly grow', async () => {
    const client = createFakeClient();
    const revisions: number[] = [];
    client.subscribe('zero_calibration_changed', (state) => revisions.push(state.revision));

    await openAndStart(client);
    await client.send('measure_zero_calibration', {});

    expect(revisions).toEqual([...revisions].sort((a, b) => a - b));
    expect(new Set(revisions).size).toBe(revisions.length);
  });

  it('given a rejection that can continue, when it is continued, then the channel is stored as overridden and the sweep moves on', async () => {
    const client = createFakeClient();
    client.zeroCalibration.leaveUnshorted(1);
    await openAndStart(client);
    await client.send('measure_zero_calibration', {});

    await client.send('continue_zero_calibration' as never, {} as never);

    expect(client.zeroCalibration.state().session).toMatchObject({
      step: 'channel',
      channel: 2,
      doneChannels: [true, false, false, false],
      overridden: [true, false, false, false],
      canContinue: false,
    });
  });

  it('given a rejection, when it is shown, then it offers continue', async () => {
    const client = createFakeClient();
    client.zeroCalibration.leaveUnshorted(1);
    await openAndStart(client);

    await client.send('measure_zero_calibration', {});

    expect(client.zeroCalibration.state().session).toMatchObject({ step: 'notShorted', canContinue: true });
  });

  it('given a clipped reading, when it is rejected, then continue is not offered and the request fails', async () => {
    const client = createFakeClient();
    client.zeroCalibration.leaveClipped(1);
    await openAndStart(client);
    await client.send('measure_zero_calibration', {});

    await expect(client.send('continue_zero_calibration' as never, {} as never)).rejects.toThrow('cannot continue');
    expect(client.zeroCalibration.state().session).toMatchObject({ step: 'notShorted', canContinue: false });
  });

  it('given every channel continued, when the last is stored, then the done step lists them all as overridden', async () => {
    const client = createFakeClient();
    client.zeroCalibration.leaveUnshorted(1, 2, 3, 4);
    await openAndStart(client);

    for (let channel = 0; channel < 4; channel++) {
      await client.send('measure_zero_calibration', {});
      await client.send('continue_zero_calibration' as never, {} as never);
    }

    expect(client.zeroCalibration.state().session).toMatchObject({
      step: 'done',
      overridden: [true, true, true, true],
    });
  });

  it('given no rejection, when continue is sent, then it is refused', async () => {
    const client = createFakeClient();
    await openAndStart(client);

    await expect(client.send('continue_zero_calibration' as never, {} as never)).rejects.toThrow('cannot continue');
  });
});
