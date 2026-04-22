/**
 * Vitest setup — jsdom shims + testing-library matchers.
 *
 * Shims:
 *   - EventSource (jsdom doesn't ship it).
 *   - matchMedia (framer-motion can touch it on mount).
 *   - requestAnimationFrame / cancelAnimationFrame (node provides these in
 *     recent versions, but we pin a simple polyfill so hold-to-confirm tests
 *     work deterministically).
 */

import "@testing-library/jest-dom/vitest";

// EventSource shim: minimal enough for our code paths (ctor + close() + onerror).
class EventSourceMock {
  url: string;
  readyState = 0;
  onopen: ((ev: Event) => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;
  constructor(url: string) {
    this.url = url;
    // Immediately report error so tests don't wait for data. The
    // useInboxStream hook treats this as "offline" and renders empty state.
    queueMicrotask(() => {
      this.onerror?.(new Event("error"));
    });
  }
  close() {
    this.readyState = 2;
  }
  addEventListener() {}
  removeEventListener() {}
  dispatchEvent() {
    return true;
  }
}
(globalThis as unknown as { EventSource: typeof EventSource }).EventSource =
  EventSourceMock as unknown as typeof EventSource;

// matchMedia shim (framer-motion reads prefers-reduced-motion).
if (typeof window !== "undefined" && !window.matchMedia) {
  window.matchMedia = (() =>
    ({
      matches: false,
      media: "",
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }) as unknown as MediaQueryList) as unknown as typeof window.matchMedia;
}
