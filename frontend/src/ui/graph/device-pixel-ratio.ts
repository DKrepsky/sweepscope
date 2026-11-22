export interface RatioWindow {
  devicePixelRatio: number;
  matchMedia(query: string): Pick<MediaQueryList, 'addEventListener' | 'removeEventListener'>;
}

/** The only way to learn that the ratio changed is a one-shot media query for the current value, re-armed on each change. */
export function watchDevicePixelRatio(win: RatioWindow, onChange: (ratio: number) => void): () => void {
  let stop = () => {};
  const arm = () => {
    const query = win.matchMedia(`(resolution: ${win.devicePixelRatio}dppx)`);
    const listener = () => {
      query.removeEventListener('change', listener);
      onChange(win.devicePixelRatio);
      arm();
    };
    query.addEventListener('change', listener);
    stop = () => query.removeEventListener('change', listener);
  };
  arm();
  return () => stop();
}
