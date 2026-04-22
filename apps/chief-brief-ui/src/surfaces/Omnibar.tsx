/**
 * Omnibar — floating palette opened via ⌘Space.
 *
 * - Debounced POST to /v1/omnibar/search (220ms).
 * - Results grouped by source (memory / event / inbox / agent / pack).
 * - Keyboard nav: ↑↓ to select, Enter to (for now) highlight, Esc to dismiss.
 * - Footer hints as specified.
 * - Empty states when no query or no backend.
 */

import { useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { v1OmnibarSearch } from "../api";
import type { OmnibarHit, OmnibarSource } from "../types";

type OmnibarProps = {
  open: boolean;
  onClose: () => void;
};

const GROUP_ORDER: OmnibarSource[] = [
  "inbox",
  "memory",
  "event",
  "agent",
  "pack",
];

const GROUP_LABEL: Record<OmnibarSource, string> = {
  inbox: "Inbox",
  memory: "Memory",
  event: "Event log",
  agent: "Agents",
  pack: "Packs",
};

const GROUP_ICON: Record<OmnibarSource, string> = {
  inbox: "▤",
  memory: "◇",
  event: "●",
  agent: "◎",
  pack: "▣",
};

export default function Omnibar({ open, onClose }: OmnibarProps) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<OmnibarHit[]>([]);
  const [loading, setLoading] = useState(false);
  const [selected, setSelected] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const reqIdRef = useRef(0);

  // Reset + focus on open; blank state on close.
  useEffect(() => {
    if (open) {
      window.setTimeout(() => inputRef.current?.focus(), 30);
      return;
    }
    setQuery("");
    setHits([]);
    setSelected(0);
    setError(null);
  }, [open]);

  // Debounced search.
  useEffect(() => {
    if (!open) return;
    const q = query.trim();
    if (!q) {
      setHits([]);
      setError(null);
      setLoading(false);
      return;
    }

    const thisReq = ++reqIdRef.current;
    setLoading(true);
    const timer = window.setTimeout(async () => {
      try {
        const result = await v1OmnibarSearch(q);
        if (thisReq !== reqIdRef.current) return;
        setHits(result.hits ?? []);
        setSelected(0);
        setError(null);
      } catch (err) {
        if (thisReq !== reqIdRef.current) return;
        setHits([]);
        setError(err instanceof Error ? err.message : "Search failed");
      } finally {
        if (thisReq === reqIdRef.current) setLoading(false);
      }
    }, 220);

    return () => window.clearTimeout(timer);
  }, [query, open]);

  const grouped = useMemo(() => {
    const g: Record<OmnibarSource, OmnibarHit[]> = {
      inbox: [],
      memory: [],
      event: [],
      agent: [],
      pack: [],
    };
    for (const h of hits) {
      const key = (g[h.source] ? h.source : "memory") as OmnibarSource;
      g[key].push(h);
    }
    return g;
  }, [hits]);

  // Flatten preserving group order, for arrow-key traversal.
  const flat = useMemo(() => {
    const rows: OmnibarHit[] = [];
    for (const src of GROUP_ORDER) {
      for (const h of grouped[src]) rows.push(h);
    }
    return rows;
  }, [grouped]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (flat.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelected((s) => Math.min(flat.length - 1, s + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelected((s) => Math.max(0, s - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      // No-op for now — opening a result is out of scope for v0.
      // Selected index is highlighted visually.
    }
  };

  const indexOfHit = (hit: OmnibarHit): number =>
    flat.findIndex((x) => x.id === hit.id);

  return (
    <AnimatePresence>
      {open ? (
        <motion.div
          className="omnibar-scrim"
          role="presentation"
          onMouseDown={(e) => {
            if (e.target === e.currentTarget) onClose();
          }}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
        >
          <motion.section
            className="omnibar-panel"
            role="dialog"
            aria-modal="true"
            aria-label="Chief Omnibar"
            initial={{ opacity: 0, y: -10, scale: 0.98 }}
            animate={{
              opacity: 1,
              y: 0,
              scale: 1,
              transition: { type: "spring", stiffness: 320, damping: 30 },
            }}
            exit={{
              opacity: 0,
              y: -6,
              scale: 0.98,
              transition: { type: "spring", stiffness: 360, damping: 34 },
            }}
          >
            <div className="omnibar-input-row">
              <span className="omnibar-icon" aria-hidden>
                ⌘
              </span>
              <input
                ref={inputRef}
                className="omnibar-input"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Search memory, events, inbox…"
                aria-label="Search"
                autoCorrect="off"
                autoCapitalize="off"
                spellCheck={false}
              />
              <span
                className={"omnibar-pulse" + (loading ? " active" : "")}
                aria-hidden
              />
            </div>

            <div className="omnibar-results">
              {query.trim() === "" ? (
                <div className="omnibar-empty">
                  <p style={{ margin: 0 }}>Search across your memory graph.</p>
                </div>
              ) : error ? (
                <div className="omnibar-empty">
                  <p style={{ margin: 0 }}>Search unavailable: {error}</p>
                </div>
              ) : flat.length === 0 && !loading ? (
                <div className="omnibar-empty">
                  <p style={{ margin: 0 }}>No results.</p>
                </div>
              ) : (
                GROUP_ORDER.map((src) => {
                  const group = grouped[src];
                  if (group.length === 0) return null;
                  return (
                    <div className="omnibar-group" key={src}>
                      <div className="omnibar-group-label">
                        {GROUP_LABEL[src]}
                      </div>
                      {group.map((hit) => {
                        const idx = indexOfHit(hit);
                        const isSel = idx === selected;
                        return (
                          <button
                            key={hit.id}
                            type="button"
                            className={
                              "omnibar-hit" + (isSel ? " selected" : "")
                            }
                            onMouseEnter={() => setSelected(idx)}
                            onClick={() => {
                              setSelected(idx);
                            }}
                          >
                            <span className="omnibar-hit-icon" aria-hidden>
                              {GROUP_ICON[src]}
                            </span>
                            <span className="omnibar-hit-body">
                              <span className="omnibar-hit-title">
                                {hit.title}
                              </span>
                              <span className="omnibar-hit-snippet">
                                {hit.snippet}
                              </span>
                            </span>
                            {hit.ts ? (
                              <span className="omnibar-hit-ts">{hit.ts}</span>
                            ) : null}
                          </button>
                        );
                      })}
                    </div>
                  );
                })
              )}
            </div>

            <div className="omnibar-hints">
              <span>
                <kbd>↵</kbd>open
              </span>
              <span>
                <kbd>⌘K</kbd>recent
              </span>
              <span>
                <kbd>⎋</kbd>dismiss
              </span>
            </div>
          </motion.section>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
