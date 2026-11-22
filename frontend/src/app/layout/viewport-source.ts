import type { ViewportSize, ViewportSource } from './layout-tier';

type ObserverFactory = (callback: () => void) => { observe(target: Element): void; disconnect(): void };

const browserObserver: ObserverFactory = (callback) => new ResizeObserver(callback);

/** The only DOM-touching code in `app`: reports the size of `target` (normally the root element). */
export function createResizeObserverViewport(
  target: Element,
  createObserver: ObserverFactory = browserObserver,
): ViewportSource {
  const read = (): ViewportSize => ({ width: target.clientWidth, height: target.clientHeight });
  return {
    subscribe(listener) {
      listener(read());
      const observer = createObserver(() => listener(read()));
      observer.observe(target);
      return () => observer.disconnect();
    },
  };
}
