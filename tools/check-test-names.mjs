#!/usr/bin/env node
// Test names follow given / when / then (docs/architecture.md section 6). No dependencies.
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SKIPPED_DIRECTORIES = new Set(['node_modules', 'target', 'dist', '.git', '.claude', 'docs', 'gen']);
const RUST_SHAPE = 'given_<state>_when_<action>_then_<result>';
const TS_SHAPE = 'given <state>, when <action>, then <result>';
const RUST_NAME = /^given_.+_when_.+_then_.+$/;
const TS_TITLE = /^(AC-[A-Z]+\d+: )?given .+ when .+ then .+$/;
const RUST_TEST = /#\[(?:\w+::)*test(?:\([^\]]*\))?\]((?:\s*#\[[^\]]*\])*)\s*(?:async\s+)?fn\s+(\w+)/g;
const TS_HEAD = /(?:^|[^\w.])(it|test)((?:\.\w+)*)\s*/g;
const TS_MODIFIERS = new Set(['only', 'skip', 'concurrent', 'fails', 'todo', 'fixme', 'fail', 'sequential', 'each']);
// Playwright annotations (`test.fail('BUG-3')`, `test.fixme(cond, 'why')`) are not declarations: only a title followed by a body is.
const PLAYWRIGHT_ANNOTATIONS = new Set(['skip', 'fixme', 'fail']);
const BODY_START = /^\s*,\s*(?:async\b|function\b|\(|\{|\w+\s*=>)/;
const STRING_LITERAL = /^(['"`])((?:\\.|(?!\1)[^\\])*)\1/;

const lineOf = (text, index) => text.slice(0, index).split('\n').length;

export function checkRustSource(path, text) {
  return [...text.matchAll(RUST_TEST)]
    .filter((m) => !RUST_NAME.test(m[2]))
    .map((m) => ({ path, line: lineOf(text, m.index), name: m[2], expected: RUST_SHAPE }));
}

function skipBalanced(text, start) {
  const open = text[start];
  if (open === '`') return text.indexOf('`', start + 1) + 1;
  let depth = 0;
  for (let i = start; i < text.length; i++) {
    if (text[i] === '(' || text[i] === '[' || text[i] === '{') depth++;
    else if (text[i] === ')' || text[i] === ']' || text[i] === '}') depth--;
    if (depth === 0) return i + 1;
  }
  return -1;
}

/** Position just after `(` that opens the test call, or -1 when this is not a test declaration. */
function titleStart(text, afterHead, modifiers) {
  let position = afterHead;
  if (modifiers.includes('each')) {
    if (text[position] !== '(' && text[position] !== '`') return -1;
    position = skipBalanced(text, position);
    while (position > 0 && /\s/.test(text[position] ?? '')) position++;
  }
  return position > 0 && text[position] === '(' ? position + 1 : -1;
}

function declaredTitles(text) {
  const titles = [];
  for (const head of text.matchAll(TS_HEAD)) {
    const modifiers = head[2].split('.').filter(Boolean);
    if (!modifiers.every((m) => TS_MODIFIERS.has(m))) continue;
    const start = titleStart(text, head.index + head[0].length, modifiers);
    const literal = start < 0 ? null : STRING_LITERAL.exec(text.slice(start).replace(/^\s+/, ''));
    if (!literal) continue;
    const isAnnotation =
      head[1] === 'test' && PLAYWRIGHT_ANNOTATIONS.has(modifiers.at(-1) ?? '') &&
      !BODY_START.test(text.slice(start).replace(/^\s+/, '').slice(literal[0].length));
    if (!isAnnotation) titles.push({ index: head.index, title: literal[2] });
  }
  return titles;
}

export function checkTypeScriptSource(path, text) {
  return declaredTitles(text)
    .filter(({ title }) => !TS_TITLE.test(title.replace(/,/g, '')))
    .map(({ index, title }) => ({ path, line: lineOf(text, index + 1), name: title, expected: TS_SHAPE }));
}

function* walk(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (SKIPPED_DIRECTORIES.has(entry.name)) continue;
    const full = join(directory, entry.name);
    if (entry.isDirectory()) yield* walk(full);
    else yield full;
  }
}

export function checkTree(root) {
  const failures = [];
  for (const file of walk(root)) {
    const path = relative(root, file).split('\\').join('/');
    if (file.endsWith('.rs')) failures.push(...checkRustSource(path, readFileSync(file, 'utf8')));
    else if (/\.(test|spec)\.ts$/.test(file)) failures.push(...checkTypeScriptSource(path, readFileSync(file, 'utf8')));
  }
  return failures;
}

function main() {
  const root = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
  const failures = checkTree(root);
  for (const f of failures) console.error(`test-name: ${f.path}:${f.line} "${f.name}" should be ${f.expected}`);
  if (failures.length > 0) process.exit(1);
  console.log('check-test-names: ok');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
