import type { MainWindowActions } from '../../app/main-window';

type ShortcutAction = 'toggleCursors' | 'saveSetup' | 'exportCsv' | 'saveScreenshot';

const PLAIN: Record<string, ShortcutAction> = { c: 'toggleCursors' };
const WITH_COMMAND: Record<string, ShortcutAction> = { s: 'saveSetup', e: 'exportCsv', p: 'saveScreenshot' };

const TEXT_FIELDS = new Set(['INPUT', 'TEXTAREA', 'SELECT']);

function isTextField(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && (TEXT_FIELDS.has(target.tagName) || target.isContentEditable);
}

function lookup(event: KeyboardEvent): ShortcutAction | undefined {
  const key = event.key.toLowerCase();
  if (event.altKey || event.shiftKey) return undefined;
  if (event.ctrlKey || event.metaKey) return WITH_COMMAND[key];
  return PLAIN[key];
}

/** Runs the action a key press stands for; never while a text field has focus, so typing is never taken over. */
export function handleShortcut(event: KeyboardEvent, actions: Pick<MainWindowActions, ShortcutAction>): void {
  if (event.defaultPrevented || event.repeat || isTextField(event.target)) return;
  const action = lookup(event);
  if (!action) return;
  event.preventDefault();
  actions[action]();
}
