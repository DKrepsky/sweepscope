import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import tseslint from 'typescript-eslint';

const tauriImports = { group: ['@tauri-apps/*'], message: 'Only src/api talks to Tauri.' };
const RAW_COLOUR = '(^|[^\\w&])#[0-9a-fA-F]{3,8}\\b|\\b(rgb|hsl|hwb|lab|lch|oklab|oklch)a?\\(';
const rawColourMessage = 'Raw colour literal: use a design token.';
// Strings, template strings and markup attribute values (`style="color: #fff"`, `fill="rgb(...)"`).
const rawColour = [
  { selector: `Literal[value=/${RAW_COLOUR}/]`, message: rawColourMessage },
  { selector: `TemplateElement[value.cooked=/${RAW_COLOUR}/]`, message: rawColourMessage },
  { selector: `SvelteLiteral[value=/${RAW_COLOUR}/]`, message: rawColourMessage },
];

const restrictImports = (...patterns) => ({ 'no-restricted-imports': ['error', { patterns }] });
const layer = (name) => ({ group: [`**/${name}`, `**/${name}/**`], message: `This layer must not import ${name}.` });

const layerRules = [
  {
    files: ['src/ui/**'],
    rules: {
      ...restrictImports(layer('app'), layer('pages'), layer('api'), tauriImports),
      'no-restricted-syntax': ['error', ...rawColour],
    },
  },
  {
    files: ['src/pages/**'],
    rules: {
      ...restrictImports(layer('api'), tauriImports),
      'no-restricted-syntax': ['error', ...rawColour],
    },
  },
  {
    files: ['src/app/**'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          paths: [{ name: 'svelte', message: 'app is plain TypeScript: only svelte/store is allowed.' }],
          patterns: [
            { group: ['**/*.svelte'], message: 'app never imports components.' },
            { group: ['svelte/*', '!svelte/store'], message: 'app is plain TypeScript: only svelte/store is allowed.' },
            layer('pages'),
            layer('ui'),
            tauriImports,
          ],
        },
      ],
    },
  },
  { files: ['src/api/**'], rules: restrictImports(layer('app'), layer('pages'), layer('ui')) },
  { files: ['src/design-system/**'], rules: restrictImports(layer('app'), layer('pages'), layer('ui'), layer('api')) },
];

export default tseslint.config(
  {
    ignores: ['dist', 'node_modules', 'test-results', 'playwright-report', 'tools/lint-fixtures', 'src/api/generated'],
  },
  js.configs.recommended,
  tseslint.configs.recommended,
  svelte.configs.recommended,
  {
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      complexity: ['error', 10],
      'max-lines-per-function': ['error', { max: 40, skipBlankLines: true, skipComments: true }],
    },
  },
  { files: ['**/*.svelte', '**/*.svelte.ts'], languageOptions: { parserOptions: { parser: tseslint.parser } } },
  ...layerRules,
  { files: ['**/*.test.ts', 'e2e/**/*.ts'], rules: { 'max-lines-per-function': 'off', complexity: 'off' } },
);
