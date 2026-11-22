import type { ConnectionSnapshot } from '../../api';
import type { BannerAction, BannerModel } from './types';

const RETRY: BannerAction = { id: 'retry', label: 'Retry' };
const USE_SIMULATOR: BannerAction = { id: 'use-simulator', label: 'Use simulator' };
const COPY_COMMAND: BannerAction = { id: 'copy-command', label: 'Copy command' };

function errorBanner(connection: ConnectionSnapshot): BannerModel {
  const issue = connection.issue;
  const actions = issue?.help ? [COPY_COMMAND, RETRY] : [RETRY, USE_SIMULATOR];
  return { id: 'connection', kind: 'error', text: issue?.message ?? 'The scope connection failed.', actions };
}

export function selectBanners(connection: ConnectionSnapshot | undefined): BannerModel[] {
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
