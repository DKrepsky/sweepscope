import type { ConnectionSnapshot } from '../../api';
import { failureText } from '../recording';
import type { ModelInputs } from './select-common';
import type { BannerAction, BannerModel } from './types';

const RETRY: BannerAction = { id: 'retry', label: 'Retry' };
const USE_SIMULATOR: BannerAction = { id: 'use-simulator', label: 'Use simulator' };
const COPY_COMMAND: BannerAction = { id: 'copy-command', label: 'Copy command' };
const CHOOSE_NEW_FILE: BannerAction = { id: 'choose-new-file', label: 'Choose new file' };
const DISMISS: BannerAction = { id: 'dismiss', label: 'Dismiss' };

function errorBanner(connection: ConnectionSnapshot): BannerModel {
  const issue = connection.issue;
  const actions = issue?.help ? [COPY_COMMAND, RETRY] : [RETRY, USE_SIMULATOR];
  return { id: 'connection', kind: 'error', text: issue?.message ?? 'The scope connection failed.', actions };
}

function connectionBanners(connection: ConnectionSnapshot | undefined): BannerModel[] {
  switch (connection?.state) {
    case 'disconnected':
      return [
        {
          id: 'connection',
          kind: 'warning',
          text: 'No scope found. Plug in the scope or use the simulator.',
          actions: [RETRY, USE_SIMULATOR],
        },
      ];
    case 'error':
      return [errorBanner(connection)];
    default:
      return [];
  }
}

/** A recording that stopped by itself stays until dismissed; a failure shows once per id. */
function recordingBanners({ recording, dismissedRecordingFailure }: ModelInputs): BannerModel[] {
  const failure = recording?.failure;
  if (!failure || failure.id <= dismissedRecordingFailure) return [];
  return [{ id: 'recording', kind: 'error', text: failureText(failure), actions: [CHOOSE_NEW_FILE, DISMISS] }];
}

export function selectBanners(inputs: ModelInputs): BannerModel[] {
  return [...connectionBanners(inputs.connection), ...recordingBanners(inputs)];
}
