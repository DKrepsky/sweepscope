import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { checkRustSource, checkTree, checkTypeScriptSource } from './check-test-names.mjs';

const rust = (name, attributes = '') => `#[cfg(test)]\nmod t {\n    #[test]\n${attributes}    fn ${name}() {}\n}\n`;

test('given fn it_works marked test, when check-test-names runs, then it fails and prints the file and name', () => {
  const failures = checkRustSource('backend/kernel/src/bus.rs', rust('it_works'));

  assert.equal(failures.length, 1);
  assert.equal(failures[0].path, 'backend/kernel/src/bus.rs');
  assert.equal(failures[0].name, 'it_works');
  assert.equal(failures[0].line, 3);
});

test('given a given_x_when_y_then_z test, when check-test-names runs, then it passes', () => {
  assert.deepEqual(checkRustSource('a.rs', rust('given_x_when_y_then_z')), []);
});

test('given a bad name after another attribute, when check-test-names runs, then it still fails', () => {
  assert.equal(checkRustSource('a.rs', rust('works', '    #[should_panic]\n')).length, 1);
});

test('given a function without the test attribute, when check-test-names runs, then it is ignored', () => {
  assert.deepEqual(checkRustSource('a.rs', 'fn helper() {}\n#[derive(Debug)]\nstruct S;'), []);
});

test("given it('works') in a test file, when check-test-names runs, then it fails", () => {
  const failures = checkTypeScriptSource('a.test.ts', "describe('x', () => {\n  it('works', () => {});\n});");

  assert.deepEqual(failures.map((f) => [f.name, f.line]), [['works', 2]]);
});

test('given a given when then title, when check-test-names runs, then it passes', () => {
  const text = "it('given no entry, when load runs, then the store is 100', () => {});";

  assert.deepEqual(checkTypeScriptSource('a.test.ts', text), []);
});

test('given a title on the next line and an AC prefix, when check-test-names runs, then it passes', () => {
  const text = "it(\n  'AC-B1: given a Button, when pressed, then onclick fires',\n  () => {},\n);";

  assert.deepEqual(checkTypeScriptSource('a.test.ts', text), []);
});

test('given it.skip with a bad title, when check-test-names runs, then it fails', () => {
  assert.equal(checkTypeScriptSource('a.test.ts', "it.skip('later', () => {});").length, 1);
});

test('given a repository tree, when check-test-names runs, then it scans rust and ts tests and skips node_modules', () => {
  const root = mkdtempSync(join(tmpdir(), 'ss-names-'));
  try {
    mkdirSync(join(root, 'backend'));
    mkdirSync(join(root, 'node_modules'));
    writeFileSync(join(root, 'backend', 'a.rs'), rust('bad'));
    writeFileSync(join(root, 'b.test.ts'), "it('bad', () => {});");
    writeFileSync(join(root, 'node_modules', 'c.test.ts'), "it('ignored', () => {});");

    assert.deepEqual(checkTree(root).map((f) => f.path).sort(), ['b.test.ts', 'backend/a.rs']);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
