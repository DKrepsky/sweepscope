import assert from 'node:assert/strict';
import { test } from 'node:test';
import { checkMetadata, checkShellSources, LIB_ALLOWLIST } from './check-architecture.mjs';

const ROOT = '/repo';

function crate(name, folder, dependencies = []) {
  return { name, manifest_path: `${ROOT}/${folder}/Cargo.toml`, dependencies: dependencies.map((d) => ({ name: d })) };
}

const metadataOf = (...packages) => ({ workspace_root: ROOT, packages });
const rulesOf = (violations) => violations.map((v) => v.rule);

const kernel = crate('sweepscope-kernel', 'backend/kernel');
const device = crate('sweepscope-device', 'backend/features/device', ['sweepscope-kernel', 'hantek6xx4-lib']);
const shell = crate('sweepscope-app', 'src-tauri', ['tauri', 'tauri-build', 'sweepscope-kernel', 'sweepscope-settings']);

test('given a clean workspace, when check-architecture runs, then there are no violations', () => {
  const settings = crate('sweepscope-settings', 'backend/features/settings', ['sweepscope-kernel', 'serde']);
  const fft = crate('sweepscope-fft', 'backend/libs/fft', ['rustfft']);

  assert.deepEqual(checkMetadata(metadataOf(kernel, settings, device, fft, shell)), []);
});

test('given settings depends on device, when check-architecture runs, then it fails with rule feature-to-feature', () => {
  const settings = crate('sweepscope-settings', 'backend/features/settings', ['sweepscope-kernel', 'sweepscope-device']);

  const violations = checkMetadata(metadataOf(kernel, settings, device));

  assert.deepEqual(violations, [
    { rule: 'feature-to-feature', message: 'crate sweepscope-settings depends on sweepscope-device' },
  ]);
});

test('given a lib depends on the kernel, when check-architecture runs, then it fails with rule lib-depends-on-ours', () => {
  const lib = crate('sweepscope-fft', 'backend/libs/fft', ['sweepscope-kernel']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(kernel, lib))), ['lib-depends-on-ours']);
});

test('given a lib depends on a feature, when check-architecture runs, then it fails with rule lib-depends-on-ours', () => {
  const lib = crate('sweepscope-fft', 'backend/libs/fft', ['sweepscope-device']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(kernel, device, lib))), ['lib-depends-on-ours']);
});

test('given the allowlist is empty today, when the module is loaded, then no lib may depend on our crates', () => {
  assert.deepEqual(LIB_ALLOWLIST, []);
});

test('given tauri in a feature crate, when check-architecture runs, then it fails with rule tauri-outside-shell', () => {
  const settings = crate('sweepscope-settings', 'backend/features/settings', ['sweepscope-kernel', 'tauri']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(kernel, settings))), ['tauri-outside-shell']);
});

test('given a tauri plugin in the kernel, when check-architecture runs, then it fails with rule tauri-outside-shell', () => {
  const badKernel = crate('sweepscope-kernel', 'backend/kernel', ['tauri-plugin-dialog']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(badKernel))), ['tauri-outside-shell']);
});

test('given the cli lists tauri, when check-architecture runs, then it fails with rule cli-without-tauri', () => {
  const cli = crate('sweepscope-cli', 'cli', ['sweepscope-kernel', 'tauri']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(kernel, cli))), ['cli-without-tauri']);
});

test('given the cli lists a tauri plugin, when check-architecture runs, then it fails with rule cli-without-tauri', () => {
  const cli = crate('sweepscope-cli', 'cli', ['tauri-plugin-dialog']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(cli))), ['cli-without-tauri']);
});

test('given the cli depends on features and the kernel only, when check-architecture runs, then there are no violations', () => {
  const cli = crate('sweepscope-cli', 'cli', ['sweepscope-kernel', 'sweepscope-device']);

  assert.deepEqual(checkMetadata(metadataOf(kernel, device, cli)), []);
});

test('given the driver in a crate other than device, when check-architecture runs, then it fails with rule hantek-outside-device', () => {
  const acquisition = crate('sweepscope-acquisition', 'backend/features/acquisition', ['hantek6xx4-lib']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(acquisition))), ['hantek-outside-device']);
});

test('given the kernel depends on a feature, when check-architecture runs, then it fails with rule upward-dependency', () => {
  const badKernel = crate('sweepscope-kernel', 'backend/kernel', ['sweepscope-device']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(badKernel, device))), ['upward-dependency']);
});

test('given a crate outside the known folders, when check-architecture runs, then it fails with rule unknown-crate-location', () => {
  const stray = crate('sweepscope-stray', 'misc/stray');

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(stray))), ['unknown-crate-location']);
});

test('given use tauri in composition.rs, when the shell sources are checked, then it fails with rule tauri-outside-ipc', () => {
  const files = [{ path: 'composition.rs', text: 'use sweepscope_kernel::Bus;\nuse tauri::AppHandle;\n' }];

  const violations = checkShellSources(files);

  assert.deepEqual(violations, [
    { rule: 'tauri-outside-ipc', message: 'src-tauri/src/composition.rs:2 uses tauri:: (allowed only in main.rs and ipc/)' },
  ]);
});

test('given tauri in main.rs and ipc, when the shell sources are checked, then it passes', () => {
  const files = [
    { path: 'main.rs', text: 'fn main() { tauri::Builder::default(); }' },
    { path: 'ipc/commands.rs', text: 'use tauri::State;' },
  ];

  assert.deepEqual(checkShellSources(files), []);
});

test('given tauri only in a comment of composition.rs, when the shell sources are checked, then it passes', () => {
  const files = [{ path: 'composition.rs', text: '// built by tauri::Builder in main\nfn compose() {}' }];

  assert.deepEqual(checkShellSources(files), []);
});

test('given tauri inside a block comment, when the shell sources are checked, then it passes', () => {
  const files = [{ path: 'composition.rs', text: '/* old:\n   tauri::Builder\n*/\nfn compose() {}' }];

  assert.deepEqual(checkShellSources(files), []);
});

test('given tauri used after a nested block comment, when the shell sources are checked, then the line number is kept', () => {
  const files = [{ path: 'composition.rs', text: '/* a /* b */ still comment tauri:: */\nuse tauri::State;' }];

  assert.deepEqual(checkShellSources(files).map((v) => v.message.split(' ')[0]), ['src-tauri/src/composition.rs:2']);
});

test('given tauri only inside a string literal, when the shell sources are checked, then it passes', () => {
  const files = [{ path: 'composition.rs', text: 'const NOTE: &str = "tauri::Builder";\nconst Q: char = \'"\';' }];

  assert.deepEqual(checkShellSources(files), []);
});

test('given use tauri alone, when the shell sources are checked, then it fails with rule tauri-outside-ipc', () => {
  const files = [{ path: 'composition.rs', text: 'use tauri;\nuse tauri as t;' }];

  assert.deepEqual(checkShellSources(files).map((v) => v.rule), ['tauri-outside-ipc', 'tauri-outside-ipc']);
});

test('given code after a line comment on the same line, when the shell sources are checked, then tauri in code is found', () => {
  const files = [{ path: 'composition.rs', text: 'let a = 1; tauri::run(); // done' }];

  assert.equal(checkShellSources(files).length, 1);
});

test('given the shell depends on the cli, when check-architecture runs, then there are no violations', () => {
  const cli = crate('sweepscope-cli', 'cli', ['sweepscope-kernel']);
  const app = crate('sweepscope-app', 'src-tauri', ['sweepscope-cli']);

  assert.deepEqual(checkMetadata(metadataOf(kernel, cli, app)), []);
});

test('given a feature depends on the cli, when check-architecture runs, then it fails with rule depends-on-cli', () => {
  const cli = crate('sweepscope-cli', 'cli');
  const settings = crate('sweepscope-settings', 'backend/features/settings', ['sweepscope-cli']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(cli, settings))), ['upward-dependency', 'depends-on-cli']);
});

test('given the kernel depends on the cli, when check-architecture runs, then it fails with rule depends-on-cli', () => {
  const cli = crate('sweepscope-cli', 'cli');
  const badKernel = crate('sweepscope-kernel', 'backend/kernel', ['sweepscope-cli']);

  assert.deepEqual(rulesOf(checkMetadata(metadataOf(cli, badKernel))), ['upward-dependency', 'depends-on-cli']);
});
