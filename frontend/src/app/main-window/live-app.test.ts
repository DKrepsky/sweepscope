import { get } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import { createFakeClient } from '../../api';
import { instrumentState, recordingState, stateSnapshot } from '../instrument/test-data';
import type { FrameBudgetProbe } from '../waveform';
import { buildFrame } from '../waveform/test-frame';
import { createLiveApp } from './live-app';

function setup() {
  const client = createFakeClient();
  client.respond('get_snapshot', () => stateSnapshot());
  const animationFrames: (() => void)[] = [];
  const order: string[] = [];
  const probe: FrameBudgetProbe = {
    beginFrame: () => order.push('begin'),
    endFrame: () => order.push('end'),
  };
  const drawn = Promise.resolve();
  const notify = vi.fn();
  const app = createLiveApp({
    client,
    notify,
    copyText: vi.fn(),
    composeScreenshot: () => Promise.resolve(new Uint8Array()),
    now: () => 0,
    requestAnimationFrame: (callback) => void animationFrames.push(callback),
    afterRender: () => drawn,
    probe,
  });
  const runAnimationFrame = () => animationFrames.splice(0).forEach((callback) => callback());
  return { client, app, order, runAnimationFrame, notify };
}

describe('createLiveApp', () => {
  it('given an app that has not started, when read, then no stream is open and the model has no instrument', () => {
    const { client, app } = setup();

    expect(client.openStreamCount('frames')).toBe(0);
    expect(get(app.model).toolbar.chips[0]?.text).toBe('Connecting');
  });

  it('given an app, when started, then the snapshot is applied and the frame stream is open', async () => {
    const { client, app } = setup();

    await app.start();

    expect(client.openStreamCount('frames')).toBe(1);
    expect(get(app.model).channels[0]?.voltsPerDiv.value).not.toBe('-');
  });

  it('given a started app, when a frame arrives and the animation frame fires, then the graph holds its samples', async () => {
    const { client, app, runAnimationFrame } = setup();
    await app.start();

    client.pushStream('frames', buildFrame({ sequence: 1 }));
    runAnimationFrame();

    expect(get(app.model).graph.channels.length).toBeGreaterThan(0);
  });

  it('given a started app, when an animation frame fires, then the probe sees begin before the draw and end after it', async () => {
    const { client, app, order, runAnimationFrame } = setup();
    await app.start();
    client.pushStream('frames', buildFrame({ sequence: 1 }));

    runAnimationFrame();
    expect(order).toEqual(['begin']);
    await Promise.resolve();

    expect(order).toEqual(['begin', 'end']);
  });

  it('given a started app, when it is stopped, then the frame stream is closed', async () => {
    const { client, app } = setup();
    await app.start();

    app.stop();

    expect(client.openStreamCount('frames')).toBe(0);
  });

  describe('analysis wiring', () => {
    it('given the live app, when Single, Autoset, Persist and Record are used, then each sends its backend command', async () => {
      const { client, app } = setup();
      for (const name of ['arm_single', 'start_autoset', 'set_persistence', 'start_recording'] as const) {
        client.respond(name, () => null);
      }
      await app.start();

      app.model.armSingle();
      app.model.startAutoset();
      app.model.setPersistence('infinite');
      app.model.startRecording();

      expect(client.calls.map((call) => call.name)).toEqual(
        expect.arrayContaining(['arm_single', 'start_autoset', 'set_persistence', 'start_recording']),
      );
    });

    it('given the live app, when the model is read, then the roll and spectrum streams open', async () => {
      const { client, app } = setup();
      await app.start();

      const stop = app.model.subscribe(() => {});

      expect(client.openStreamCount('roll')).toBe(1);
      expect(client.openStreamCount('spectrum')).toBe(1);
      stop();
    });

    it('given a recording state from the backend, when it arrives, then the model shows it', async () => {
      const { client, app } = setup();
      await app.start();
      const stop = app.model.subscribe(() => {});

      client.emit('recording_changed', recordingState({ revision: 5, active: true }));

      expect(get(app.model).recording.active).toBe(true);
      stop();
    });
  });

  describe('setup notice', () => {
    const noticeState = (revision: number, id: number) =>
      instrumentState({ revision, setupNotice: { id, text: `notice ${id}` } });

    it('given a setup notice with a new id, when it arrives, then one warning toast shows its text', async () => {
      const { client, app, notify } = setup();
      await app.start();

      client.emit('instrument_changed', noticeState(2, 1));

      expect(notify).toHaveBeenCalledExactlyOnceWith('warning', 'notice 1');
    });

    it('given a shown notice, when later state updates carry the same id, then it is not shown again', async () => {
      const { client, app, notify } = setup();
      await app.start();
      client.emit('instrument_changed', noticeState(2, 1));

      client.emit('instrument_changed', noticeState(3, 1));
      client.emit('instrument_changed', instrumentState({ revision: 4 }));

      expect(notify).toHaveBeenCalledTimes(1);
    });

    it('given a shown notice, when the app restarts and the snapshot carries the same id, then it is not shown again', async () => {
      const { client, app, notify } = setup();
      client.respond('get_snapshot', () => stateSnapshot({ instrument: noticeState(2, 1) }));
      await app.start();
      app.stop();

      await app.start();

      expect(notify).toHaveBeenCalledTimes(1);
    });

    it('given a shown notice, when a notice with another id arrives, then it is shown too', async () => {
      const { client, app, notify } = setup();
      await app.start();
      client.emit('instrument_changed', noticeState(2, 1));

      client.emit('instrument_changed', noticeState(3, 2));

      expect(notify).toHaveBeenLastCalledWith('warning', 'notice 2');
    });
  });
});

describe('calibration wiring in the live app', () => {
  it('given the live app, when the user runs the wizard, then the model walks tag, dialog, chip and back', async () => {
    const { client, app } = setup();
    await app.start();
    const model = () => get(app.model);

    app.model.openCalibration();
    expect(model().calibration.dialog?.stepNumber).toBe(1);
    expect(model().toolbar.chips.map((chip) => chip.text)).toEqual(['Paused (calibration)']);

    app.model.startCalibration();
    for (let channel = 1; channel <= 4; channel++) app.model.calibrateChannel();
    expect(model().calibration.dialog).toMatchObject({ step: 'done', stepNumber: 6 });
    expect(model().calibration.tag.state).toBe('calibrated');

    app.model.closeCalibration();
    expect(model().calibration.dialog).toBeUndefined();
    expect(model().toolbar.chips[0]?.text).toBe('Run');
    expect(client.calls.filter((call) => call.name.endsWith('zero_calibration')).length).toBe(7);
  });

  it('given a recording is active, when the user opens calibration, then the tag says why and no command is sent', async () => {
    const { client, app } = setup();
    client.respond('get_snapshot', () => stateSnapshot({ recording: recordingState({ active: true }) }));
    await app.start();

    app.model.openCalibration();

    expect(get(app.model).calibration.tag).toEqual({ state: 'unavailable', reason: 'Stop recording first' });
    expect(client.calls.some((call) => call.name === 'open_zero_calibration')).toBe(false);
  });
});
