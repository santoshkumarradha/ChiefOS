/**
 * MorningBrief — primary demo surface.
 *
 * Binds directly to /v1/brief via the v1GetBrief API. Renders three panels:
 *   - Trust ledger (from brief.trust_ledger)
 *   - Needs you column (from brief.needs_you: Card[])
 *   - Handled column (from brief.handled: Card[])
 *
 * The previous prototype used a rich synthetic BriefState shape; since this
 * task forbids mock data, we use only fields that chief-core actually produces
 * (Card: summary, region, surface, friction_tier, created_at, …). Anything
 * else is an empty state.
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion, type Variants } from "framer-motion";
import { v1GetBrief } from "./api";
import type { Brief, Card } from "./types";

type MorningBriefProps = {
  onCeremonyClick?: (id: string) => void;
  onOffline?: (offline: boolean) => void;
};

const cardContainer: Variants = {
  hidden: {},
  shown: {
    transition: { staggerChildren: 0.06, delayChildren: 0.08 },
  },
};

const cardReveal: Variants = {
  hidden: { opacity: 0, y: 18, scale: 0.99 },
  shown: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: { type: "spring", stiffness: 360, damping: 32 },
  },
};

function coerceBrief(raw: Brief): Brief {
  // Some legacy shapes return strings instead of Card objects; coerce to Card
  // with enough fields that the UI can render. We do NOT fabricate contents —
  // we render exactly what the server said, just in the structural envelope
  // the UI needs.
  const needs = Array.isArray(raw.needs_you)
    ? raw.needs_you.map((x, i) => coerceCard(x, `needs-${i}`))
    : [];
  const handled = Array.isArray(raw.handled)
    ? raw.handled.map((x, i) => coerceCard(x, `handled-${i}`))
    : [];
  return {
    date: raw.date ?? "",
    needs_you: needs,
    handled,
    trust_ledger: raw.trust_ledger ?? {},
  };
}

function coerceCard(value: unknown, fallbackId: string): Card {
  if (typeof value === "string") {
    // Server produced a plain string; treat it as the summary.
    return {
      card_id: fallbackId,
      intent_id: "",
      action_type: "",
      summary: value,
      region: "",
      surface: "",
      friction_tier: "",
      mem_uri: "",
      created_at: "",
      approved_at: null,
    };
  }
  const v = (value ?? {}) as Partial<Card>;
  return {
    card_id: v.card_id ?? fallbackId,
    intent_id: v.intent_id ?? "",
    action_type: v.action_type ?? "",
    summary: v.summary ?? "",
    region: v.region ?? "",
    surface: v.surface ?? "",
    friction_tier: v.friction_tier ?? "",
    mem_uri: v.mem_uri ?? "",
    created_at: v.created_at ?? "",
    approved_at: v.approved_at ?? null,
  };
}

function formatTime(iso: string): string {
  try {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "";
    return new Intl.DateTimeFormat("en", {
      weekday: "short",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(d);
  } catch {
    return "";
  }
}

export default function MorningBrief({
  onCeremonyClick: _onCeremonyClick,
  onOffline,
}: MorningBriefProps) {
  const [brief, setBrief] = useState<Brief | null>(null);
  const [offline, setOffline] = useState(false);
  const [revealed, setRevealed] = useState(true);

  const load = useCallback(async () => {
    try {
      const raw = await v1GetBrief();
      setBrief(coerceBrief(raw));
      setOffline(false);
      onOffline?.(false);
    } catch {
      setBrief(null);
      setOffline(true);
      onOffline?.(true);
    }
  }, [onOffline]);

  useEffect(() => {
    void load();
    const id = setInterval(load, 3000);
    return () => clearInterval(id);
  }, [load]);

  const generatedTime = useMemo(
    () => (brief?.date ? formatTime(brief.date) : ""),
    [brief?.date]
  );

  const needs = brief?.needs_you ?? [];
  const handled = brief?.handled ?? [];
  const trustEntries = Object.entries(brief?.trust_ledger ?? {}).sort(
    (a, b) => (b[1] as number) - (a[1] as number)
  );

  return (
    <main className="brief-shell" aria-labelledby="brief-title">
      <section className="brief-hero" aria-label="Morning reveal">
        <div className="brief-title-block">
          <p className="eyebrow">Morning ritual</p>
          <h1 id="brief-title">Morning Brief</h1>
          <p className="brief-meta">
            {offline
              ? "Backend offline — showing empty state"
              : generatedTime
                ? `Synced · ${generatedTime}`
                : "Synced"}
          </p>
        </div>
        <button
          className="reveal-button"
          type="button"
          onClick={() => setRevealed((v) => !v)}
          aria-pressed={revealed}
        >
          {revealed ? "Revealed" : "Reveal Brief"}
        </button>
      </section>

      {trustEntries.length > 0 && (
        <section className="trust-ledger" aria-labelledby="trust-ledger-title">
          <div className="section-heading compact">
            <p className="section-kicker">Trust ledger</p>
            <h2 id="trust-ledger-title">Delegation health</h2>
          </div>
          <div className="trust-bars">
            {trustEntries.map(([label, score]) => {
              const pct = Math.max(
                0,
                Math.min(100, (Number(score) / 5) * 100)
              );
              return (
                <article className="trust-row" key={label}>
                  <div className="trust-label">
                    <span>{label}</span>
                    <strong>{score}</strong>
                  </div>
                  <div className="trust-track" aria-hidden>
                    <motion.div
                      className="trust-fill"
                      initial={{ width: 0 }}
                      animate={{ width: `${pct}%` }}
                      transition={{
                        type: "spring",
                        stiffness: 170,
                        damping: 24,
                        mass: 0.72,
                      }}
                    />
                  </div>
                </article>
              );
            })}
          </div>
        </section>
      )}

      <div className="brief-grid">
        <section className="needs-panel" aria-labelledby="needs-title">
          <div className="section-heading">
            <p className="section-kicker">Needs you</p>
            <h2 id="needs-title">
              {needs.length} {needs.length === 1 ? "decision" : "decisions"}
            </h2>
          </div>
          <AnimatePresence>
            {needs.length === 0 ? (
              <motion.p
                className="brief-meta"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
              >
                {offline
                  ? "Nothing to show while backend is offline."
                  : "Nothing to do."}
              </motion.p>
            ) : (
              <motion.div
                className="card-stack"
                variants={cardContainer}
                initial="hidden"
                animate={revealed ? "shown" : "hidden"}
              >
                {needs.map((card) => (
                  <motion.article
                    key={card.card_id}
                    className="need-card"
                    variants={cardReveal}
                  >
                    <div className="card-topline">
                      {card.friction_tier ? (
                        <span className={`priority-chip medium`}>
                          {card.friction_tier}
                        </span>
                      ) : (
                        <span />
                      )}
                      <span>{card.action_type || "action"}</span>
                    </div>
                    <h3>{card.summary || card.card_id}</h3>
                    <div className="card-footer">
                      <span>{card.region || "—"}</span>
                      <span>{card.surface || "—"}</span>
                    </div>
                  </motion.article>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </section>

        <section className="handled-panel" aria-labelledby="handled-title">
          <div className="section-heading">
            <p className="section-kicker">Handled</p>
            <h2 id="handled-title">
              {handled.length} {handled.length === 1 ? "receipt" : "receipts"}
            </h2>
          </div>
          <AnimatePresence>
            {handled.length === 0 ? (
              <motion.p
                className="brief-meta"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
              >
                {offline
                  ? "No recent activity — backend offline."
                  : "No recent activity."}
              </motion.p>
            ) : (
              <motion.div
                className="handled-list"
                variants={cardContainer}
                initial="hidden"
                animate={revealed ? "shown" : "hidden"}
              >
                {handled.map((card) => (
                  <motion.article
                    key={card.card_id}
                    className="handled-card-shell"
                    variants={cardReveal}
                  >
                    <div className="handled-card" aria-label={card.summary}>
                      <span className="handled-category">
                        {card.action_type || "action"}
                      </span>
                      <strong>{card.summary || card.card_id}</strong>
                      <span>{card.region || "—"}</span>
                      <span className="handled-footer">
                        <span>{card.surface || "—"}</span>
                        <span>
                          {card.approved_at ? formatTime(card.approved_at) : ""}
                        </span>
                      </span>
                    </div>
                  </motion.article>
                ))}
              </motion.div>
            )}
          </AnimatePresence>
        </section>
      </div>
    </main>
  );
}
