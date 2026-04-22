/**
 * InboxDrawer — HAX Inbox as a top-right anchored drawer.
 *
 * - Fetches /v1/inbox + subscribes to /v1/inbox/stream via useInboxStream.
 * - Three tab chips: All / Needs you / Ceremony (counts from live state).
 * - Per-row thin-line icon + title + snippet + agent + timestamp.
 * - Click a CeremonyPending item → onCeremonyClick(ceremony_id).
 * - Empty / offline states render without fake content.
 */

import { useMemo, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useInboxStream } from "../hooks/useInboxStream";
import type { InboxItem, InboxKind } from "../types";

type Tab = "all" | "needs" | "ceremony";

type InboxDrawerProps = {
  open: boolean;
  onClose: () => void;
  onCeremonyClick: (ceremonyId: string) => void;
};

function iconCharFor(kind: InboxKind): string {
  // SF-Symbol-like glyphs via Unicode.
  switch (kind) {
    case "CeremonyPending":
      return "◈";
    case "NeedsYou":
      return "●";
    case "Handled":
      return "✓";
    case "Anchor":
      return "⚓";
    case "System":
    default:
      return "·";
  }
}

function iconClassFor(kind: InboxKind): string {
  switch (kind) {
    case "CeremonyPending":
      return "inbox-icon kind-ceremony";
    case "NeedsYou":
      return "inbox-icon kind-needs";
    case "Handled":
      return "inbox-icon kind-handled";
    default:
      return "inbox-icon";
  }
}

function badgeFor(item: InboxItem): {
  label: string | null;
  tone: string;
} {
  if (item.badge) {
    const tone =
      item.kind === "CeremonyPending"
        ? "tone-ceremony"
        : item.kind === "NeedsYou"
          ? "tone-needs"
          : item.kind === "Handled"
            ? "tone-handled"
            : "";
    return { label: item.badge, tone };
  }
  switch (item.kind) {
    case "CeremonyPending":
      return { label: "Ceremony", tone: "tone-ceremony" };
    case "NeedsYou":
      return { label: "Needs you", tone: "tone-needs" };
    case "Handled":
      return { label: "Handled", tone: "tone-handled" };
    default:
      return { label: null, tone: "" };
  }
}

function formatTs(ts: string): string {
  try {
    const d = new Date(ts);
    if (Number.isNaN(d.getTime())) return "";
    return new Intl.DateTimeFormat("en", {
      hour: "numeric",
      minute: "2-digit",
    }).format(d);
  } catch {
    return "";
  }
}

export default function InboxDrawer({
  open,
  onClose,
  onCeremonyClick,
}: InboxDrawerProps) {
  const { items, status, error } = useInboxStream();
  const [tab, setTab] = useState<Tab>("all");

  const counts = useMemo(() => {
    return {
      all: items.length,
      needs: items.filter(
        (x) => x.kind === "NeedsYou" || x.kind === "CeremonyPending"
      ).length,
      ceremony: items.filter((x) => x.kind === "CeremonyPending").length,
    };
  }, [items]);

  const visible = useMemo(() => {
    if (tab === "all") return items;
    if (tab === "needs")
      return items.filter(
        (x) => x.kind === "NeedsYou" || x.kind === "CeremonyPending"
      );
    return items.filter((x) => x.kind === "CeremonyPending");
  }, [items, tab]);

  const handleRowClick = (item: InboxItem) => {
    if (item.kind === "CeremonyPending" && item.ceremony_id) {
      onCeremonyClick(item.ceremony_id);
    }
  };

  return (
    <AnimatePresence>
      {open ? (
        <>
          {/* Scrim: invisible layer that catches outside-clicks. */}
          <motion.div
            className="inbox-drawer-scrim"
            onClick={onClose}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            aria-hidden="true"
          />
          <motion.aside
            className="inbox-drawer"
            role="dialog"
            aria-modal="false"
            aria-labelledby="hax-inbox-title"
            initial={{ opacity: 0, x: 40, scale: 0.98 }}
            animate={{
              opacity: 1,
              x: 0,
              scale: 1,
              transition: { type: "spring", stiffness: 320, damping: 32 },
            }}
            exit={{
              opacity: 0,
              x: 40,
              scale: 0.98,
              transition: { type: "spring", stiffness: 360, damping: 36 },
            }}
          >
            <div className="inbox-header">
              <h2 id="hax-inbox-title">HAX Inbox</h2>
              <span className="inbox-status">
                {status === "live"
                  ? "Live"
                  : status === "loading"
                    ? "Loading"
                    : "Offline"}
              </span>
            </div>
            <div className="inbox-tabs" role="tablist">
              <button
                role="tab"
                className={"inbox-tab" + (tab === "all" ? " active" : "")}
                aria-selected={tab === "all"}
                onClick={() => setTab("all")}
                type="button"
              >
                All <span className="tab-count">{counts.all}</span>
              </button>
              <button
                role="tab"
                className={"inbox-tab" + (tab === "needs" ? " active" : "")}
                aria-selected={tab === "needs"}
                onClick={() => setTab("needs")}
                type="button"
              >
                Needs you <span className="tab-count">{counts.needs}</span>
              </button>
              <button
                role="tab"
                className={"inbox-tab" + (tab === "ceremony" ? " active" : "")}
                aria-selected={tab === "ceremony"}
                onClick={() => setTab("ceremony")}
                type="button"
              >
                Ceremony <span className="tab-count">{counts.ceremony}</span>
              </button>
            </div>
            <div className="inbox-list" role="list">
              {visible.length === 0 ? (
                <div className="inbox-empty">
                  {status === "offline" ? (
                    <>
                      <span className="empty-title">Inbox unavailable</span>
                      <span>chief-core is offline.</span>
                      {error ? (
                        <span style={{ fontSize: 11 }}>{error}</span>
                      ) : null}
                    </>
                  ) : status === "loading" ? (
                    <span>Loading inbox…</span>
                  ) : (
                    <>
                      <span className="empty-title">All clear</span>
                      <span>Nothing needs your attention right now.</span>
                    </>
                  )}
                </div>
              ) : (
                visible.map((item) => {
                  const badge = badgeFor(item);
                  return (
                    <button
                      key={item.id}
                      role="listitem"
                      type="button"
                      className="inbox-row"
                      onClick={() => handleRowClick(item)}
                      aria-label={`${item.title} — ${item.agent}`}
                    >
                      <span className={iconClassFor(item.kind)} aria-hidden>
                        {iconCharFor(item.kind)}
                      </span>
                      <span className="inbox-body">
                        <span className="inbox-title">{item.title}</span>
                        <span className="inbox-snippet">
                          {item.snippet}
                          {item.agent ? ` — ${item.agent}` : ""}
                        </span>
                      </span>
                      <span className="inbox-meta">
                        {badge.label ? (
                          <span className={`inbox-badge ${badge.tone}`}>
                            {badge.label}
                          </span>
                        ) : null}
                        {item.ts ? (
                          <span className="inbox-ts">{formatTs(item.ts)}</span>
                        ) : null}
                      </span>
                    </button>
                  );
                })
              )}
            </div>
            <div className="inbox-footer">
              <span>Chief · HAX Inbox</span>
              <span>⌘I · esc</span>
            </div>
          </motion.aside>
        </>
      ) : null}
    </AnimatePresence>
  );
}
