import { describe, expect, it } from 'vitest';
import { createPausableClock } from './pausable-clock';

function setup() {
  let time = 1000;
  const clock = createPausableClock(() => time);
  return { clock, advance: (ms: number) => (time += ms) };
}

describe('pausable clock', () => {
  it('given it runs, when time passes, then it follows the source', () => {
    const { clock, advance } = setup();
    const start = clock.now();
    advance(250);
    expect(clock.now() - start).toBe(250);
  });

  it('given it is paused, when time passes, then it stands still', () => {
    const { clock, advance } = setup();
    clock.setPaused(true);
    const held = clock.now();
    advance(5000);
    expect(clock.now()).toBe(held);
  });

  it('given a pause, when it resumes, then the paused time is not counted', () => {
    const { clock, advance } = setup();
    const start = clock.now();
    advance(100);
    clock.setPaused(true);
    advance(5000);
    clock.setPaused(false);
    advance(50);
    expect(clock.now() - start).toBe(150);
  });

  it('given it is paused, when paused again, then the first pause point is kept', () => {
    const { clock, advance } = setup();
    clock.setPaused(true);
    const held = clock.now();
    advance(10);
    clock.setPaused(true);
    expect(clock.now()).toBe(held);
  });
});
