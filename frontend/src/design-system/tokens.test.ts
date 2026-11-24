// @vitest-environment node
import { describe, expect, it } from 'vitest';
import {
  generate,
  main,
  parseReducedMotionTokens,
  parseTokenBlock,
  readTokenSources,
} from '../../tools/gen-tokens.mjs';
import { tokenNames, tokens } from './index';

const sources = readTokenSources();
const dark = ':root[data-theme="dark"]';

describe('design tokens', () => {
  it('given the prototype tokens and the frontend tokens, when names and values are compared, then they are identical', () => {
    const prototype = parseTokenBlock(sources.prototypeCss, ':root');
    const frontend = parseTokenBlock(sources.frontendCss, dark);
    expect(frontend).toEqual(prototype);
  });

  it('given the committed generated module, when compared with a fresh generation, then they are equal', () => {
    expect(sources.generatedTs).toBe(generate(sources.frontendCss));
  });

  it('given the generated module, when token names are listed, then they match the keys of the dark theme', () => {
    expect([...tokenNames]).toEqual(Object.keys(tokens));
  });

  it('given a token file with var(--x) as a value, when the generator runs, then it exits non-zero naming the token', () => {
    const css = `${dark} { --ss-color-a: #ffffff; --ss-color-b: var(--x); }`;
    const errors: string[] = [];
    const io = { read: () => css, write: () => undefined, error: (message: string) => errors.push(message) };

    const code = main(['in.css', 'out.ts'], io);

    expect(code).not.toBe(0);
    expect(errors.join()).toContain('color-b');
  });

  it('given a valid token file, when the generator runs, then it writes the generated module and exits zero', () => {
    const written: Record<string, string> = {};
    const io = {
      read: () => sources.frontendCss,
      write: (path: string, content: string) => (written[path] = content),
      error: () => undefined,
    };

    const code = main(['in.css', 'out.ts'], io);

    expect(code).toBe(0);
    expect(written['out.ts']).toBe(sources.generatedTs);
  });

  it('given prefers-reduced-motion, when the block is parsed, then the motion tokens are 0 ms', () => {
    const overrides = parseReducedMotionTokens(sources.frontendCss);

    expect(overrides).toEqual({ 'motion-fast': '0ms', 'motion-normal': '0ms' });
  });

  it('given the frontend tokens, when the theme block is read, then the dark colour scheme is declared inside it', () => {
    expect(sources.frontendCss).toMatch(/:root\[data-theme=["']dark["']\]\s*\{\s*color-scheme:\s*dark;/);
  });
});

describe('M3 tokens', () => {
  it('given the persistence opacity tokens, when read, then they hold the graded design values', () => {
    expect(tokens['opacity-persist-newest']).toBe('0.85');
    expect(tokens['opacity-persist-hold']).toBe('0.6');
    expect(tokens['opacity-persist-oldest']).toBe('0.3');
  });

  it('given the record and gap tokens, when read, then they hold the design values', () => {
    expect(tokens['color-status-record']).toBe('#ff8fb1');
    expect(tokens['color-gap-fill']).toBe('rgb(224 160 48 / 0.14)');
  });
});
