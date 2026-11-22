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
const TS_TEST = /(?:^|[^\w.])(?:it|test)(?:\.(?:only|skip|concurrent))?\(\s*(['"`])((?:\\.|(?!\1)[^\\])*)\1/g;

const lineOf = (text, index) => text.slice(0, index).split('\n').length;

export function checkRustSource(path, text) {
  return [...text.matchAll(RUST_TEST)]
    .filter((m) => !RUST_NAME.test(m[2]))
    .map((m) => ({ path, line: lineOf(text, m.index), name: m[2], expected: RUST_SHAPE }));
}

export function checkTypeScriptSource(path, text) {
  return [...text.matchAll(TS_TEST)]
    .filter((m) => !TS_TITLE.test(m[2].replace(/,/g, '')))
    .map((m) => ({ path, line: lineOf(text, m.index), name: m[2], expected: TS_SHAPE }));
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
