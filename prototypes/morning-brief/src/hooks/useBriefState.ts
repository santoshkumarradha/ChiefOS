/**
 * React hook for polling /brief endpoint from chief-core.
 * Returns live brief data when available; falls back to mock when offline.
 */

import { useEffect, useState, useRef, useCallback } from "react";
import { getBrief } from "../api";
import briefStateMock from "../../mock/brief-state.json";
import type { BriefState } from "../MorningBrief";

type BriefStatus = "loading" | "live" | "offline";

export function useBriefState() {
  const [brief, setBrief] = useState<BriefState>(briefStateMock as BriefState);
  const [status, setStatus] = useState<BriefStatus>("loading");
  const [error, setError] = useState<string | null>(null);
  const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const backoffRef = useRef<number>(1000);

  const fetchBrief = useCallback(async () => {
    try {
      const result = await getBrief();
      // Transform chief-core Brief format to BriefState format for now
      // Once mock is fully deprecated, we'll adapt the UI types
      setBrief({
        ...briefStateMock,
        generated_at: result.date || briefStateMock.generated_at,
        trust_ledger: Object.entries(result.trust_ledger || {}).map(([label, current]) => ({
          id: label,
          label,
          current: typeof current === "number" ? current : 0,
          previous: typeof current === "number" ? current - 1 : 0,
        })),
      } as BriefState);
      setStatus("live");
      setError(null);
      backoffRef.current = 1000; // Reset backoff on success
    } catch (err) {
      const message = err instanceof Error ? err.message : "Unknown error";
      setError(message);
      setStatus("offline");
      // Exponential backoff capped at 30s
      backoffRef.current = Math.min(backoffRef.current * 1.5, 30000);
    }
  }, []);

  useEffect(() => {
    // Initial fetch
    fetchBrief();

    // Poll every 3 seconds (or according to backoff when offline)
    pollIntervalRef.current = setInterval(() => {
      fetchBrief();
    }, 3000);

    return () => {
      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current);
      }
    };
  }, [fetchBrief]);

  return { brief, status, error };
}
