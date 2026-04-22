/**
 * Ceremony — full-screen co-sign takeover.
 *
 * - Fetches /v1/ceremony/{id} on mount.
 * - Renders eyebrow, title, summary, evidence table, provenance rows,
 *   rollback-window note.
 * - Hold-to-confirm ring at bottom; 3s hold calls /v1/ceremony/{id}/approve
 *   with { held_ms }. Short release snaps back.
 * - "Deny" link below ring calls /v1/ceremony/{id}/deny.
 * - On success: fade out + onDone().
 */

import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import {
  v1ApproveCeremony,
  v1DenyCeremony,
  v1GetCeremony,
} from "../api";
import type { Ceremony as CeremonyType } from "../types";
import { useHoldToConfirm } from "../hooks/useHoldToConfirm";

type CeremonyProps = {
  id: string;
  onDone: () => void;
};

const RING_RADIUS = 82;
const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS;

function HoldRing({
  thresholdMs,
  onApprove,
  onError,
}: {
  thresholdMs: number;
  onApprove: (heldMs: number) => Promise<void>;
  onError: (msg: string) => void;
}) {
  const [inFlight, setInFlight] = useState(false);

  const { isHolding, progressPct, confirmed, heldMs, bind } = useHoldToConfirm({
    thresholdMs,
    onConfirm: async (held) => {
      if (inFlight) return;
      setInFlight(true);
      try {
        await onApprove(held);
      } catch (err) {
        onError(err instanceof Error ? err.message : "Approval failed");
      } finally {
        setInFlight(false);
      }
    },
  });

  // When not holding but heldMs > 0 → recoil animation.
  const recoiling = !isHolding && !confirmed && heldMs > 0;
  const displayPct = recoiling ? 0 : progressPct;
  const strokeOffset =
    RING_CIRCUMFERENCE * (1 - Math.min(1, Math.max(0, displayPct / 100)));

  return (
    <div
      className="hold-ring-wrap"
      role="button"
      tabIndex={0}
      aria-label={confirmed ? "Approved" : "Hold to approve"}
      aria-pressed={isHolding}
      {...bind}
    >
      <svg
        className="hold-ring-svg"
        viewBox="0 0 180 180"
        aria-hidden="true"
      >
        <circle className="hold-ring-track" cx="90" cy="90" r={RING_RADIUS} />
        <circle
          className={
            "hold-ring-fill" +
            (recoiling ? " recoiling" : "") +
            (confirmed ? " confirmed" : "")
          }
          cx="90"
          cy="90"
          r={RING_RADIUS}
          strokeDasharray={RING_CIRCUMFERENCE}
          strokeDashoffset={strokeOffset}
        />
      </svg>
      <span className={"hold-ring-label" + (confirmed ? " confirmed" : "")}>
        {confirmed
          ? "APPROVED"
          : isHolding
            ? "HOLDING…"
            : "HOLD TO\nAPPROVE"}
      </span>
    </div>
  );
}

export default function Ceremony({ id, onDone }: CeremonyProps) {
  const [ceremony, setCeremony] = useState<CeremonyType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [denying, setDenying] = useState(false);
  const fadingRef = useRef(false);

  // Fetch.
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    v1GetCeremony(id)
      .then((c) => {
        if (cancelled) return;
        setCeremony(c);
        setLoading(false);
      })
      .catch((err) => {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : "Ceremony fetch failed");
        setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [id]);

  const handleApprove = async (heldMs: number) => {
    await v1ApproveCeremony(id, heldMs);
    // Fade out, then close.
    if (fadingRef.current) return;
    fadingRef.current = true;
    window.setTimeout(() => onDone(), 480);
  };

  const handleDeny = async () => {
    if (denying) return;
    setDenying(true);
    try {
      await v1DenyCeremony(id);
      onDone();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Deny failed");
    } finally {
      setDenying(false);
    }
  };

  const threshold = ceremony?.threshold_ms ?? 3000;

  return (
    <AnimatePresence>
      <motion.div
        key="scrim"
        className="ceremony-scrim"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1, transition: { duration: 0.28 } }}
        exit={{ opacity: 0, transition: { duration: 0.4 } }}
      >
        <motion.section
          className="ceremony-card"
          role="dialog"
          aria-modal="true"
          aria-labelledby="ceremony-title"
          initial={{ opacity: 0, y: 18, scale: 0.98 }}
          animate={{
            opacity: 1,
            y: 0,
            scale: 1,
            transition: { type: "spring", stiffness: 300, damping: 30 },
          }}
          exit={{
            opacity: 0,
            y: 12,
            scale: 0.98,
            transition: { duration: 0.3 },
          }}
        >
          {loading ? (
            <>
              <p className="ceremony-eyebrow">Ceremony · Loading</p>
              <h2 id="ceremony-title" className="ceremony-title">
                Preparing evidence…
              </h2>
            </>
          ) : error ? (
            <>
              <p className="ceremony-eyebrow">Ceremony · Error</p>
              <h2 id="ceremony-title" className="ceremony-title">
                Unable to load ceremony
              </h2>
              <p className="ceremony-summary">{error}</p>
              <div className="ceremony-footer">
                <button
                  type="button"
                  className="ceremony-deny"
                  onClick={onDone}
                >
                  Close
                </button>
              </div>
            </>
          ) : ceremony ? (
            <>
              <p className="ceremony-eyebrow">{ceremony.eyebrow}</p>
              <h2 id="ceremony-title" className="ceremony-title">
                {ceremony.title}
              </h2>
              <p className="ceremony-summary">{ceremony.summary}</p>

              <div className="ceremony-body">
                {ceremony.evidence.length > 0 && (
                  <section>
                    <p className="ceremony-section-label">Evidence</p>
                    <dl className="ceremony-evidence">
                      {ceremony.evidence.map((row) => (
                        <div className="ceremony-evidence-row" key={row.label}>
                          <dt>{row.label}</dt>
                          <dd>
                            {row.value}
                            {row.note ? (
                              <span className="note">— {row.note}</span>
                            ) : null}
                          </dd>
                        </div>
                      ))}
                    </dl>
                  </section>
                )}

                {ceremony.provenance.length > 0 && (
                  <section>
                    <p className="ceremony-section-label">Provenance</p>
                    <ul className="ceremony-provenance">
                      {ceremony.provenance.map((row, i) => (
                        <li key={`${row.actor}-${i}`}>
                          <span />
                          <span>
                            <span className="actor">{row.actor}</span>
                            {row.action}
                            {row.outcome ? ` — ${row.outcome}` : ""}
                          </span>
                          <span className="ts">{row.ts}</span>
                        </li>
                      ))}
                    </ul>
                  </section>
                )}

                {ceremony.rollback_window ? (
                  <section>
                    <p className="ceremony-section-label">Rollback window</p>
                    <p className="ceremony-rollback">
                      {ceremony.rollback_window}
                    </p>
                  </section>
                ) : null}
              </div>

              <div className="ceremony-footer">
                <HoldRing
                  thresholdMs={threshold}
                  onApprove={handleApprove}
                  onError={setError}
                />
                <div className="ceremony-hints">
                  <span>or tap Spacebar · {Math.round(threshold / 1000)}s hold</span>
                </div>
                <button
                  type="button"
                  className="ceremony-deny"
                  onClick={handleDeny}
                  disabled={denying}
                >
                  {denying ? "Denying…" : "Deny"}
                </button>
                {error ? <div className="ceremony-error">{error}</div> : null}
              </div>
            </>
          ) : null}
        </motion.section>
      </motion.div>
    </AnimatePresence>
  );
}
