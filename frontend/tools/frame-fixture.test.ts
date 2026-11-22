// @vitest-environment node
/// <reference types="node" />
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { decodeFrame } from '../src/app/waveform';

const FIXTURE = new URL('../../backend/features/display/tests/fixtures/frame-v1.bin', import.meta.url);

function fixtureBuffer(): ArrayBuffer {
  const bytes = readFileSync(FIXTURE);
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}

describe('decodeFrame against the backend golden fixture (ADR 0011)', () => {
  it('given the golden fixture of the backend, when decoded, then the header fields match the encoder test', () => {
    const frame = decodeFrame(fixtureBuffer());
    expect(frame).toMatchObject({
      sequence: 42,
      producedUs: 1_700_000_000_000_000,
      sentUs: 0,
      triggered: true,
      auto: false,
      samplesPerChannel: 5,
      sampleIntervalS: 4e-6,
      triggerIndex: 3,
      setupRevision: 7,
    });
  });

  it('given the golden fixture, when decoded, then channels 1 and 3 hold the encoded divisions', () => {
    const { channels } = decodeFrame(fixtureBuffer());
    expect(channels.map(({ channel, samples }) => [channel, Array.from(samples)])).toEqual([
      [1, [8, 6, 4, 2, 0]],
      [3, [2, 3, 1, 2, 2]],
    ]);
  });
});
