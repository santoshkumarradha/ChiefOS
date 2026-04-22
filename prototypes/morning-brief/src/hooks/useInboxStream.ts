/**
 * useInboxStream — React hook binding the HAX Inbox to /v1/inbox and
 * /v1/inbox/stream.
 *
 * Fetches the initial page via v1GetInbox(), then opens an EventSource to
 * the stream endpoint. Events with JSON body shape InboxStreamEvent are
 * applied to the in-memory item list (upsert / remove / heartbeat).
 *
 * When the backend is unreachable we:
 *   - leave the items list empty (no fake data)
 *   - set status to "offline"
 *   - attempt reconnect via EventSource's native backoff
 *
 * The hook also exposes a `ceremonyPending` counter derived from items with
 * kind === "CeremonyPending", so App can observe a new pending ceremony and
 * auto-open the Ceremony surface.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { adaptInboxItemExternal, v1GetInbox, v1InboxStreamUrl } from "../api";
import type { InboxItem, InboxStreamEvent } from "../types";

export type InboxStreamStatus = "loading" | "live" | "offline";

export type InboxStream = {
  items: InboxItem[];
  status: InboxStreamStatus;
  error: string | null;
  ceremonyPending: InboxItem | null;
  refresh: () => Promise<void>;
};

export function useInboxStream(): InboxStream {
  const [items, setItems] = useState<InboxItem[]>([]);
  const [status, setStatus] = useState<InboxStreamStatus>("loading");
  const [error, setError] = useState<string | null>(null);
  const esRef = useRef<EventSource | null>(null);
  const mountedRef = useRef(true);

  const applyEvent = useCallback((evt: InboxStreamEvent) => {
    if (evt.type === "upsert") {
      setItems((prev) => {
        const idx = prev.findIndex((x) => x.id === evt.item.id);
        if (idx === -1) return [evt.item, ...prev];
        const next = prev.slice();
        next[idx] = evt.item;
        return next;
      });
    } else if (evt.type === "remove") {
      setItems((prev) => prev.filter((x) => x.id !== evt.id));
    }
    // "heartbeat": no-op.
  }, []);

  const refresh = useCallback(async () => {
    try {
      const page = await v1GetInbox();
      if (!mountedRef.current) return;
      setItems(page.items ?? []);
      setStatus("live");
      setError(null);
    } catch (err) {
      if (!mountedRef.current) return;
      setStatus("offline");
      setError(err instanceof Error ? err.message : "Inbox fetch failed");
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    void refresh();

    // Open SSE.
    let es: EventSource | null = null;
    try {
      es = new EventSource(v1InboxStreamUrl());
      esRef.current = es;

      es.onopen = () => {
        if (!mountedRef.current) return;
        setStatus("live");
        setError(null);
      };

      es.onmessage = (msg) => {
        if (!mountedRef.current) return;
        try {
          const parsed = JSON.parse(msg.data);
          // Backend ships raw InboxItem JSON per frame; the frontend's internal
          // discriminated InboxStreamEvent is a richer shape we don't yet
          // receive. Treat every frame as an upsert of the adapted item.
          if (parsed && typeof parsed === "object" && "kind" in parsed && "source_agent" in parsed) {
            applyEvent({
              type: "upsert",
              item: adaptInboxItemExternal(parsed as Parameters<typeof adaptInboxItemExternal>[0]),
            });
          } else {
            // Tolerate pre-wrapped events too, for future-compat.
            applyEvent(parsed as InboxStreamEvent);
          }
        } catch {
          // Ignore malformed frames. Do not fake data.
        }
      };

      es.onerror = () => {
        if (!mountedRef.current) return;
        setStatus("offline");
        // EventSource auto-reconnects; we just reflect it in status.
      };
    } catch (err) {
      setStatus("offline");
      setError(err instanceof Error ? err.message : "Stream init failed");
    }

    return () => {
      mountedRef.current = false;
      if (es) es.close();
      esRef.current = null;
    };
  }, [refresh, applyEvent]);

  // Surface the most-recent pending ceremony for the App shell.
  const ceremonyPending =
    items.find((x) => x.kind === "CeremonyPending" && x.ceremony_id) ?? null;

  return { items, status, error, ceremonyPending, refresh };
}
