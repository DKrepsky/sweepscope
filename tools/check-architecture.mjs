#!/usr/bin/env node
// Enforces the crate rules of docs/architecture.md section 8.2 (ADR 0004). No dependencies.
import { spawnSync } from 'node:child_process';
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const DEVICE_CRATE = 'sweepscope-device';
const DRIVER_CRATE = 'hantek6xx4-lib';
// A lib may depend on a crate of ours only when listed here as "<lib>-><dependency>".
export const LIB_ALLOWLIST = [];

const LOCATIONS = [
  ['kernel', 'backend/kernel/'],
  ['lib', 'backend/libs/'],
  ['feature', 'backend/features/'],
  ['shell', 'src-tauri/'],
  ['shell', 'cli/'],
];

const CLI_LOCATION = 'cli/';

const LAYER = { lib: 0, kernel: 1, feature: 2, shell: 3 };

/** Rules over (crate, dependency) pairs. `broken` returns true when the pair violates the rule. */
export const DEPENDENCY_RULES = [
  {
    id: 'feature-to-feature',
    broken: (from, to) => from.kind === 'feature' && to.workspace && to.kind === 'feature',
  },
  {
    id: 'lib-depends-on-ours',
    broken: (from, to) => from.kind === 'lib' && to.workspace && !LIB_ALLOWLIST.includes(`${from.name}->${to.name}`),
  },
  {
    id: 'upward-dependency',
    broken: (from, to) => from.kind !== 'lib' && to.workspace && LAYER[to.kind] > LAYER[from.kind],
  },
  {
    id: 'hantek-outside-device',
    broken: (from, to) => to.name === DRIVER_CRATE && from.name !== DEVICE_CRATE,
  },
  {
    id: 'tauri-outside-shell',
    broken: (from, to) => /^tauri(-|$)/.test(to.name) && from.kind !== 'shell',
  },
  {
    id: 'cli-without-tauri',
    broken: (from, to) => from.location.startsWith(CLI_LOCATION) && /^tauri(-|$)/.test(to.name),
  },
];

export const TAURI_ALLOWED_SOURCES = [/^main\.rs$/, /^ipc\//];

function locationOf(manifestPath, workspaceRoot) {
  return relative(workspaceRoot, manifestPath).split('\\').join('/');
}

function kindOf(manifestPath, workspaceRoot) {
  const path = locationOf(manifestPath, workspaceRoot);
  const match = LOCATIONS.find(([, prefix]) => path.startsWith(prefix));
  return match ? match[0] : 'unknown';
}

export function checkMetadata(metadata) {
  const crates = metadata.packages.map((p) => ({
    name: p.name,
    kind: kindOf(p.manifest_path, metadata.workspace_root),
    location: locationOf(p.manifest_path, metadata.workspace_root),
    dependencies: p.dependencies.map((d) => d.name),
  }));
  const ours = new Map(crates.map((c) => [c.name, c]));
  const violations = [];
  for (const crate of crates) {
    if (crate.kind === 'unknown') {
      const message = `crate ${crate.name} is outside kernel, libs, features, cli and src-tauri`;
      violations.push({ rule: 'unknown-crate-location', message });
    }
    for (const dependency of new Set(crate.dependencies)) {
      const target = ours.get(dependency);
      const to = { name: dependency, kind: target?.kind, workspace: Boolean(target) };
      for (const rule of DEPENDENCY_RULES.filter((r) => r.broken(crate, to))) {
        violations.push({ rule: rule.id, message: `crate ${crate.name} depends on ${dependency}` });
      }
    }
  }
  return violations;
}

const TAURI_USE = [/(^|[^\w])tauri\s*::/, /\buse\s+(::)?tauri\s*(;|as\b)/, /\bextern\s+crate\s+tauri\b/];

/** Blanks comments and string literals but keeps every newline, so line numbers stay valid. */
export function stripCommentsAndStrings(text) {
  const blank = (match) => match.replace(/[^\n]/g, ' ');
  let result = '';
  const token = /\/\/[^\n]*|"(?:\\.|[^"\\])*"|r(#*)"[\s\S]*?"\1|'(?:\\.|[^'\\])'|\/\*/y;
  for (let i = 0; i < text.length; ) {
    token.lastIndex = i;
    const match = token.exec(text);
    if (!match) {
      result += text[i++];
    } else if (match[0] === '/*') {
      const end = blockCommentEnd(text, i);
      result += blank(text.slice(i, end));
      i = end;
    } else {
      result += blank(match[0]);
      i += match[0].length;
    }
  }
  return result;
}

function blockCommentEnd(text, start) {
  let depth = 0;
  for (let i = start; i < text.length - 1; i++) {
    const pair = text.slice(i, i + 2);
    if (pair === '/*') depth++;
    else if (pair === '*/' && --depth === 0) return i + 2;
  }
  return text.length;
}

/** `files` are `{ path, text }` with `path` relative to `src-tauri/src`, using forward slashes. */
export function checkShellSources(files) {
  const violations = [];
  for (const file of files.filter((f) => !TAURI_ALLOWED_SOURCES.some((allowed) => allowed.test(f.path)))) {
    stripCommentsAndStrings(file.text)
      .split('\n')
      .forEach((line, index) => {
        if (TAURI_USE.some((pattern) => pattern.test(line))) {
          violations.push({
            rule: 'tauri-outside-ipc',
            message: `src-tauri/src/${file.path}:${index + 1} uses tauri:: (allowed only in main.rs and ipc/)`,
          });
        }
      });
  }
  return violations;
}

function readRustSources(directory, base = directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const full = join(directory, entry.name);
    if (entry.isDirectory()) return readRustSources(full, base);
    if (!entry.name.endsWith('.rs')) return [];
    return [{ path: relative(base, full).split('\\').join('/'), text: readFileSync(full, 'utf8') }];
  });
}

function loadMetadata(root, metadataFile) {
  if (metadataFile) return JSON.parse(readFileSync(metadataFile, 'utf8'));
  const run = spawnSync('cargo', ['metadata', '--format-version', '1', '--no-deps'], {
    cwd: root,
    encoding: 'utf8',
    maxBuffer: 256 * 1024 * 1024,
  });
  if (run.status !== 0) throw new Error(`cargo metadata failed:\n${run.stderr}`);
  return JSON.parse(run.stdout);
}

function parseArguments(argv) {
  const options = { root: resolve(dirname(fileURLToPath(import.meta.url)), '..') };
  for (let i = 0; i < argv.length; i += 2) {
    if (argv[i] === '--root') options.root = resolve(argv[i + 1]);
    else if (argv[i] === '--metadata') options.metadataFile = resolve(argv[i + 1]);
    else throw new Error(`unknown argument ${argv[i]}`);
  }
  return options;
}

function main() {
  const { root, metadataFile } = parseArguments(process.argv.slice(2));
  const violations = checkMetadata(loadMetadata(root, metadataFile));
  try {
    violations.push(...checkShellSources(readRustSources(join(root, 'src-tauri', 'src'))));
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  for (const v of violations) console.error(`${v.rule}: ${v.message}`);
  if (violations.length > 0) process.exit(1);
  console.log('check-architecture: ok');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main();
  } catch (error) {
    console.error(`check-architecture: ${error.message}`);
    process.exit(2);
  }
}
