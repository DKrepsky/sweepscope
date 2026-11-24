import { readFileSync } from 'node:fs';
import { expect, test } from '@playwright/test';
import { COMMANDS } from '../src/api/generated/commands';
import { ACKNOWLEDGED_COMMANDS } from './support/tauri-stub';

const capability = JSON.parse(
  readFileSync(new URL('../../src-tauri/capabilities/main.json', import.meta.url), 'utf8'),
) as { permissions: string[] };

const permissionFor = (command: string) => `allow-${command.replaceAll('_', '-')}`;
const granted = new Set(capability.permissions);
const eventCommand = 'plugin:event|unlisten';

test.describe('main window capability against the e2e stub', () => {
  test('given every generated command, when compared with the capability, then each one is granted', () => {
    const missing = COMMANDS.filter((command) => !granted.has(permissionFor(command)));

    expect(missing).toEqual([]);
  });

  test('given the commands the stub acknowledges, when compared with the capability, then each one is granted', () => {
    const ungranted = ACKNOWLEDGED_COMMANDS.filter(
      (command) => command !== eventCommand && !granted.has(permissionFor(command)),
    );

    expect(ungranted).toEqual([]);
  });

  test('given the stub acknowledges unlisten, when compared with the capability, then the event permission is granted', () => {
    expect(ACKNOWLEDGED_COMMANDS).toContain(eventCommand);
    expect(granted.has('core:event:allow-unlisten')).toBe(true);
  });
});
