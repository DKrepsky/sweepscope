// Placeholder with the shape the Rust export test writes (WP3 overwrites this file). Do not edit by hand.
import type { UiStateEntry } from './UiStateEntry';

export const COMMANDS = ['load_ui_state', 'save_ui_state'] as const;
export type CommandName = (typeof COMMANDS)[number];

export type CommandArgs = {
  load_ui_state: { namespace: string };
  save_ui_state: { entry: UiStateEntry };
};

export type CommandResult = {
  load_ui_state: UiStateEntry | null;
  save_ui_state: null;
};

export type EventName = never;
export type EventPayload = Record<EventName, never>;
