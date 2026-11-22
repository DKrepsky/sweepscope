export interface Size {
  width: number;
  height: number;
}

export const MIN_SCREEN: Size = { width: 512, height: 410 };

const ASPECT = 1.25;

export function fitScreen(stage: Size): Size {
  const width = Math.min(stage.width, ASPECT * stage.height);
  const height = width / ASPECT;
  if (width < MIN_SCREEN.width || height < MIN_SCREEN.height) return MIN_SCREEN;
  return { width: Math.floor(width), height: Math.floor(height) };
}
