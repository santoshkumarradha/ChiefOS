/**
 * Global keybind hook.
 *
 * Usage:
 *   useKeyboard({
 *     "Meta+i": () => setInboxOpen(v => !v),
 *     "Meta+Space": () => setOmnibarOpen(v => !v),
 *     "Escape": () => closeAll(),
 *   });
 *
 * Binding syntax:
 *   - Modifier prefixes: "Meta+", "Ctrl+", "Alt+", "Shift+".
 *   - Case-insensitive on the key; "Meta+I" == "meta+i".
 *   - Spaces allowed in the key token: "Meta+Space".
 *
 * The handler is called with the KeyboardEvent. Return false from the handler
 * to allow default browser behavior; any other return (incl. void) preventsDefault.
 */

import { useEffect, useRef } from "react";

export type KeyHandler = (event: KeyboardEvent) => void | boolean;
export type KeyMap = Record<string, KeyHandler>;

type ParsedBinding = {
  key: string; // normalized lowercase
  meta: boolean;
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
};

function parseBinding(binding: string): ParsedBinding {
  const parts = binding.split("+").map((p) => p.trim());
  const out: ParsedBinding = {
    key: "",
    meta: false,
    ctrl: false,
    alt: false,
    shift: false,
  };
  for (const raw of parts) {
    const p = raw.toLowerCase();
    if (p === "meta" || p === "cmd" || p === "command") out.meta = true;
    else if (p === "ctrl" || p === "control") out.ctrl = true;
    else if (p === "alt" || p === "option") out.alt = true;
    else if (p === "shift") out.shift = true;
    else out.key = p;
  }
  return out;
}

function matches(event: KeyboardEvent, p: ParsedBinding): boolean {
  if (p.meta !== event.metaKey) return false;
  if (p.ctrl !== event.ctrlKey) return false;
  if (p.alt !== event.altKey) return false;
  // Shift is tricky: for "Meta+Space" we want shift:false, but many keys carry
  // shift implicitly (e.g. "?"). We only enforce shift if the binding declared it.
  if (p.shift && !event.shiftKey) return false;
  const k = event.key.toLowerCase();
  // Normalize " " → "space" so both "Space" and " " work.
  const normalizedK = k === " " ? "space" : k;
  return normalizedK === p.key;
}

export function useKeyboard(bindings: KeyMap): void {
  // Keep a stable ref so consumers don't need useCallback.
  const bindingsRef = useRef(bindings);
  bindingsRef.current = bindings;

  useEffect(() => {
    const parsed: Array<{ parsed: ParsedBinding; name: string }> = Object.keys(
      bindingsRef.current
    ).map((name) => ({ name, parsed: parseBinding(name) }));

    const onKey = (event: KeyboardEvent) => {
      for (const { parsed: p, name } of parsed) {
        if (matches(event, p)) {
          const handler = bindingsRef.current[name];
          if (!handler) continue;
          const result = handler(event);
          if (result !== false) {
            event.preventDefault();
          }
          return;
        }
      }
    };

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}
