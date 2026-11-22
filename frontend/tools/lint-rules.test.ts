// @vitest-environment node
import { ESLint } from 'eslint';
import stylelint from 'stylelint';
import { describe, expect, it } from 'vitest';

const frontend = new URL('..', import.meta.url).pathname;
const fixtures = `${frontend}tools/lint-fixtures`;
const resolve = (...parts: string[]) => parts.join('/').replace(/\/+/g, '/');

interface Finding {
  rule: string;
  message: string;
}

async function eslintFindings(file: string): Promise<Finding[]> {
  const eslint = new ESLint({ cwd: fixtures, overrideConfigFile: resolve(frontend, 'eslint.config.js') });
  const [result] = await eslint.lintFiles([file]);
  return (result?.messages ?? []).map((m) => ({ rule: m.ruleId ?? 'parse-error', message: m.message }));
}

const eslintRules = async (file: string) => (await eslintFindings(file)).map((finding) => finding.rule);

async function stylelintWarnings(file: string): Promise<string[]> {
  const { results } = await stylelint.lint({
    files: resolve(fixtures, file),
    configFile: resolve(frontend, '.stylelintrc.json'),
  });
  return (results[0]?.warnings ?? []).map((warning) => warning.rule);
}

const imports = 'no-restricted-imports';
const syntax = 'no-restricted-syntax';
const mustNotImport = (layer: string) =>
  new RegExp(`^'.*' import is restricted.*This layer must not import ${layer}\\.`);
const onlyApi = /Only src\/api talks to Tauri\./;
const rawColour = /Raw colour literal: use a design token\./;

describe('architecture lint rules', { timeout: 30_000 }, () => {
  it.each([
    ['ui importing app', 'src/ui/imports-app.ts', imports, mustNotImport('app')],
    ['ui importing pages', 'src/ui/imports-pages.ts', imports, mustNotImport('pages')],
    ['ui importing api', 'src/ui/imports-api.ts', imports, mustNotImport('api')],
    ['ui importing @tauri-apps', 'src/ui/imports-tauri.ts', imports, onlyApi],
    ['pages importing api', 'src/pages/imports-api.ts', imports, mustNotImport('api')],
    ['pages importing @tauri-apps', 'src/pages/imports-tauri.ts', imports, onlyApi],
    ['app importing ui', 'src/app/imports-ui.ts', imports, mustNotImport('ui')],
    ['app importing pages', 'src/app/imports-pages.ts', imports, mustNotImport('pages')],
    ['app importing a svelte component', 'src/app/imports-component.ts', imports, /app never imports components\./],
    ['app importing svelte', 'src/app/imports-svelte.ts', imports, /only svelte\/store is allowed/],
    ['api importing ui', 'src/api/imports-ui.ts', imports, mustNotImport('ui')],
    ['api importing app', 'src/api/imports-app.ts', imports, mustNotImport('app')],
    ['api importing pages', 'src/api/imports-pages.ts', imports, mustNotImport('pages')],
    ['design-system importing ui', 'src/design-system/imports-ui.ts', imports, mustNotImport('ui')],
    ['design-system importing api', 'src/design-system/imports-api.ts', imports, mustNotImport('api')],
    ['ui importing a node: module', 'src/ui/imports-node-prefixed.ts', imports, /no Node imports\./],
    ['app importing the path module', 'src/app/imports-node-path.ts', imports, /no Node imports\./],
    ['ui using the process global', 'src/ui/uses-process.ts', 'no-restricted-globals', /no Node globals\./],
    ['ui with a hex colour string', 'src/ui/hex-literal.ts', syntax, rawColour],
    ['ui with an rgb() string', 'src/ui/rgb-literal.ts', syntax, rawColour],
    ['ui with an hsla() template string', 'src/ui/hsl-template.ts', syntax, rawColour],
    ['ui with hex inside a style string', 'src/ui/hex-in-style-string.ts', syntax, rawColour],
    ['pages with a hex colour string', 'src/pages/hex-literal.ts', syntax, rawColour],
    ['pages with a hex colour in a markup attribute', 'src/pages/hex-attribute.svelte', syntax, rawColour],
    ['pages with an rgb() markup attribute', 'src/pages/rgb-attribute.svelte', syntax, rawColour],
  ])(
    'given %s, when linted, then only that rule fails and one message names the layer pair',
    async (_name, file, rule, message) => {
      const findings = await eslintFindings(file);

      expect(findings.map((finding) => finding.rule)).toSatisfy((rules: string[]) => rules.every((r) => r === rule));
      expect(findings.map((finding) => finding.message)).toSatisfy((all: string[]) => all.some((m) => message.test(m)));
    },
  );

  it('given a design-system file, when it imports from a layer, then the other layers stay unrestricted', async () => {
    expect(await eslintRules('src/design-system/imports-ui.ts')).toEqual([imports]);
  });

  it('given a pages markup attribute with a token and a hash in text, when linted, then there is no finding', async () => {
    expect(await eslintRules('src/pages/tokens-attribute.svelte')).toEqual([]);
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
