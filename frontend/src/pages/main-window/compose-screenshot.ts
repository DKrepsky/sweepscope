import type { MainWindowModel } from '../../app/main-window';

/** STUB: WP16 draws the window into a canvas of css size times `ratio` and returns the PNG bytes. */
export const composeScreenshot: (model: MainWindowModel, ratio: number) => Promise<Uint8Array> = () =>
  Promise.reject(new Error('Screenshots are not available yet'));
