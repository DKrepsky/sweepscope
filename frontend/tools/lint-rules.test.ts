// @vitest-environment node
import { ESLint } from 'eslint';
import stylelint from 'stylelint';
import { describe, expect, it } from 'vitest';

const frontend = new URL('..', import.meta.url).pathname;
const fixtures = `${frontend}tools/lint-fixtures`;
const resolve = (...parts: string[]) => parts.join('/').replace(/\/+/g, '/');

async function eslintRules(file: string): Promise<string[]> {
  const eslint = new ESLint({ cwd: fixtures, overrideConfigFile: resolve(frontend, 'eslint.config.js') });
  const [result] = await eslint.lintFiles([file]);
  return (result?.messages ?? []).map((message) => message.ruleId ?? 'parse-error');
}

async function stylelintWarnings(file: string): Promise<string[]> {
  const { results } = await stylelint.lint({
    files: resolve(fixtures, file),
    configFile: resolve(frontend, '.stylelintrc.json'),
  });
  return (results[0]?.warnings ?? []).map((warning) => warning.rule);
}

describe('architecture lint rules', () => {
  it.each([
    ['ui importing app', 'src/ui/imports-app.ts'],
    ['pages importing api', 'src/pages/imports-api.ts'],
    ['app importing a svelte component', 'src/app/imports-component.ts'],
    ['app importing svelte', 'src/app/imports-svelte.ts'],
    ['ui with a hex colour string', 'src/ui/hex-literal.ts'],
  ])('given %s, when linted, then the boundary rule fails', async (_name, file) => {
    const rules = await eslintRules(file);
    expect(rules).not.toHaveLength(0);
    expect(new Set(rules)).toSatisfy((found: Set<string>) => [...found].every((rule) => /^no-restricted-/.test(rule)));
  });

  it('given app importing svelte/store, when linted, then there is no finding', async () => {
    expect(await eslintRules('src/app/imports-svelte-store.ts')).toEqual([]);
  });

  it('given raw hex and px in a ui svelte style block, when linted, then both rules fail', async () => {
    expect((await stylelintWarnings('src/ui/raw-literals.svelte')).sort()).toEqual([
      'color-no-hex',
      'unit-disallowed-list',
    ]);
  });

  it('given a ui svelte style block that uses tokens and rem, when linted, then there is no finding', async () => {
    expect(await stylelintWarnings('src/ui/tokens-only.svelte')).toEqual([]);
  });
});
