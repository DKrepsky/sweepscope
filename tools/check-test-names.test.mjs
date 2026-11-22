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

const titlesOf = (text) => checkTypeScriptSource('a.test.ts', text).map((f) => f.name);

test('given it.each with a bad title, when check-test-names runs, then it fails', () => {
  assert.deepEqual(titlesOf("it.each([[1, 2], [3, (4)]])('works %s', () => {});"), ['works %s']);
});

test('given it.each with a given when then title, when check-test-names runs, then it passes', () => {
  assert.deepEqual(titlesOf("it.each([['a', 'b']])('given %s, when linted, then it fails', () => {});"), []);
});

test('given it.each as a tagged template with a bad title, when check-test-names runs, then it fails', () => {
  assert.deepEqual(titlesOf("it.each`\n a | b\n ${1} | ${2}\n`('bad', () => {});"), ['bad']);
});

test('given test.skip.each and it.concurrent.each, when check-test-names runs, then their titles are checked', () => {
  assert.deepEqual(titlesOf("test.skip.each([1])('bad one', f);\nit.concurrent.each([1])('bad two', f);"), ['bad one', 'bad two']);
});

test('given it.fails and it.todo with bad titles, when check-test-names runs, then both fail', () => {
  assert.deepEqual(titlesOf("it.fails('known', () => {});\nit.todo('later');"), ['known', 'later']);
});

test('given it.todo with a good title, when check-test-names runs, then it passes', () => {
  assert.deepEqual(titlesOf("it.todo('given x, when y, then z');"), []);
});

test('given a Playwright test.fail annotation inside a test, when check-test-names runs, then it is not a declaration', () => {
  const text = "test('given a, when b, then c', async () => {\n  test.fail(true, 'BUG-3: reason');\n  test.fail('BUG-4: other');\n});";

  assert.deepEqual(titlesOf(text), []);
});

test('given test.fixme and test.skip conditions with reasons, when check-test-names runs, then they are ignored', () => {
  assert.deepEqual(titlesOf("test.fixme(browserName === 'webkit', 'flaky');\ntest.skip('BUG-5');"), []);
});

test('given test.fail declaring a test with a bad title and a body, when check-test-names runs, then it fails', () => {
  assert.deepEqual(titlesOf("test.fail('BUG-3 popover', async ({ page }) => {});"), ['BUG-3 popover']);
});

test('given test.describe and describe.each, when check-test-names runs, then they are not tests', () => {
  assert.deepEqual(titlesOf("test.describe('suite', () => {});\ndescribe.each([1])('suite %s', f);"), []);
});

test('given a bad title in frontend e2e, when check-test-names scans a tree, then the spec file is reported', () => {
  const root = mkdtempSync(join(tmpdir(), 'ss-names-'));
  try {
    mkdirSync(join(root, 'frontend', 'e2e'), { recursive: true });
    writeFileSync(join(root, 'frontend', 'e2e', 'x.spec.ts'), "test('bad', async () => {});");

    assert.deepEqual(checkTree(root).map((f) => f.path), ['frontend/e2e/x.spec.ts']);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("given test.fail with a description that has no BUG number, when check-test-names runs, then it fails", () => {
  const failures = checkTypeScriptSource('a.spec.ts', "test('given a, when b, then c', async () => {\n  test.fail(true, 'flaky');\n});");

  assert.deepEqual(failures.map((f) => [f.name, f.line]), [['flaky', 2]]);
});

test('given test.fail with a single description without the BUG prefix, when check-test-names runs, then it fails', () => {
  const failures = checkTypeScriptSource('a.spec.ts', "test.fail('BUG-4 no colon');");

  assert.deepEqual(failures.map((f) => f.name), ['BUG-4 no colon']);
});

test('given test.fail with a BUG description over several lines, when check-test-names runs, then it passes', () => {
  const text = "test.fail(\n  width === 1,\n  'BUG-2: the stage is small',\n);";

  assert.deepEqual(checkTypeScriptSource('a.spec.ts', text), []);
});

test('given test.fail declaring a test with a body, when check-test-names runs, then its description rule does not apply', () => {
  const text = "test.fail('given a, when b, then c', async ({ page }) => {});";

  assert.deepEqual(checkTypeScriptSource('a.spec.ts', text), []);
});
