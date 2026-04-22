/**
 * Menubar — thin top strip matching the macOS-style reference mockups.
 *
 * Left: brand stub + app name.
 * Center: first-level menus (static labels; non-interactive in demo).
 * Right: system glyphs + spend-today (pulled from /v1/models/cost) + time.
 *
 * No fake numbers. If cost is unavailable we render "—". If time is
 * unavailable (we derive from local clock), we render placeholder "--:--".
 */

import { useEffect, useState } from "react";
import { v1GetModelCost } from "../api";

const MENUS = ["Chief", "Brief", "Approvals", "Timeline", "Window", "Help"];

function formatSpend(amount: number | null, currency: string | null): string {
  if (amount === null || amount === undefined || Number.isNaN(amount)) return "—";
  const c = currency === "USD" || !currency ? "$" : `${currency} `;
  return `${c}${amount.toFixed(2)}`;
}

function formatClock(now: Date): string {
  try {
    return new Intl.DateTimeFormat("en", {
      weekday: "short",
      hour: "numeric",
      minute: "2-digit",
    }).format(now);
  } catch {
    return "--:--";
  }
}

export default function Menubar() {
  const [spend, setSpend] = useState<{
    amount: number | null;
    currency: string | null;
  }>({ amount: null, currency: null });
  const [now, setNow] = useState<Date>(() => new Date());

  // Poll cost every 30s; failures leave the previous value (or null if we never had one).
  useEffect(() => {
    let cancelled = false;
    const tick = async () => {
      try {
        const cost = await v1GetModelCost();
        if (cancelled) return;
        setSpend({ amount: cost.spend_today, currency: cost.currency });
      } catch {
        // Leave as-is; no fake data.
      }
    };
    void tick();
    const id = setInterval(tick, 30000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  // Clock tick every 30s (we show minute granularity).
  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 30000);
    return () => clearInterval(id);
  }, []);

  return (
    <header className="menubar" role="banner" aria-label="Chief OS menubar">
      <div className="menubar-left">
        <span className="menubar-glyph" aria-hidden="true">
          ◉
        </span>
        {MENUS.map((m, i) => (
          <span
            key={m}
            className={
              i === 0 ? "menubar-item brand" : "menubar-item menubar-menu"
            }
          >
            {m}
          </span>
        ))}
      </div>
      <div className="menubar-center" aria-hidden="true" />
      <div className="menubar-right">
        <span className="menubar-glyph" aria-hidden="true">
          ◐
        </span>
        <span className="menubar-glyph" aria-hidden="true">
          ⌘
        </span>
        <span
          className={
            "menubar-cost" +
            (spend.amount === null ? " cost-unknown" : "")
          }
          title="Today's model spend"
        >
          {formatSpend(spend.amount, spend.currency)}
        </span>
        <span className="menubar-time">{formatClock(now)}</span>
      </div>
    </header>
  );
}
