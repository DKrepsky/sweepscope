export interface Size {
  width: number;
  height: number;
}

export const DEFAULT_ROOT_FONT_PX = 16;

/** Design 3.3: the screen never gets smaller than 32 x 25.6 rem, whatever the text scale. */
export const MIN_SCREEN_REM: Size = { width: 32, height: 25.6 };

const ASPECT = 1.25;

export function minScreen(rootFontPx: number = DEFAULT_ROOT_FONT_PX): Size {
  return { width: MIN_SCREEN_REM.width * rootFontPx, height: MIN_SCREEN_REM.height * rootFontPx };
}

export function fitScreen(stage: Size, rootFontPx: number = DEFAULT_ROOT_FONT_PX): Size {
  const minimum = minScreen(rootFontPx);
  const width = Math.min(stage.width, ASPECT * stage.height);
  const height = width / ASPECT;
  if (width < minimum.width || height < minimum.height) return minimum;
  return { width: Math.floor(width), height: Math.floor(height) };
}
