/**
 * Hold-to-confirm hook. Used by the Ceremony ring.
 *
 * Returns:
 *   - isHolding: true while the user is pressing
 *   - progressPct: 0..100 linear fill ratio
 *   - heldMs: elapsed ms since press start (resets on release)
 *   - bind: spread on the target element to wire events
 *
 * Call onConfirm(heldMs) exactly once when progress crosses thresholdMs.
 * Short release (<thresholdMs) fires nothing — caller can animate the
 * recoil via the `isHolding === false && heldMs > 0` transition state.
 */

import { useCallback, useEffect, useRef, useState } from "react";

export type HoldToConfirmOptions = {
  thresholdMs: number;
  onConfirm: (heldMs: number) => void;
  onCancel?: (heldMs: number) => void;
};

type HoldBindings = {
  onMouseDown: (e: React.MouseEvent) => void;
  onMouseUp: (e: React.MouseEvent) => void;
  onMouseLeave: (e: React.MouseEvent) => void;
  onTouchStart: (e: React.TouchEvent) => void;
  onTouchEnd: (e: React.TouchEvent) => void;
  onTouchCancel: (e: React.TouchEvent) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onKeyUp: (e: React.KeyboardEvent) => void;
};

export type HoldToConfirmState = {
  isHolding: boolean;
  heldMs: number;
  progressPct: number;
  confirmed: boolean;
  bind: HoldBindings;
  reset: () => void;
};

export function useHoldToConfirm({
  thresholdMs,
  onConfirm,
  onCancel,
}: HoldToConfirmOptions): HoldToConfirmState {
  const [isHolding, setIsHolding] = useState(false);
  const [heldMs, setHeldMs] = useState(0);
  const [confirmed, setConfirmed] = useState(false);

  const startTsRef = useRef<number | null>(null);
  const rafRef = useRef<number | null>(null);
  const firedRef = useRef(false);

  const stopAnim = useCallback(() => {
    if (rafRef.current !== null) {
      cancelAnimationFrame(rafRef.current);
      rafRef.current = null;
    }
  }, []);

  const reset = useCallback(() => {
    stopAnim();
    startTsRef.current = null;
    firedRef.current = false;
    setIsHolding(false);
    setHeldMs(0);
    setConfirmed(false);
  }, [stopAnim]);

  const tick = useCallback(() => {
    if (startTsRef.current === null) {
      return;
    }
    const elapsed = performance.now() - startTsRef.current;
    setHeldMs(elapsed);

    if (elapsed >= thresholdMs && !firedRef.current) {
      firedRef.current = true;
      setConfirmed(true);
      stopAnim();
      onConfirm(elapsed);
      return;
    }

    rafRef.current = requestAnimationFrame(tick);
  }, [thresholdMs, onConfirm, stopAnim]);

  const begin = useCallback(() => {
    if (firedRef.current) return;
    if (startTsRef.current !== null) return;
    startTsRef.current = performance.now();
    setIsHolding(true);
    setHeldMs(0);
    rafRef.current = requestAnimationFrame(tick);
  }, [tick]);

  const end = useCallback(() => {
    stopAnim();
    setIsHolding(false);
    const elapsed =
      startTsRef.current === null ? 0 : performance.now() - startTsRef.current;
    startTsRef.current = null;
    if (!firedRef.current && elapsed > 0) {
      onCancel?.(elapsed);
    }
    // Snap back smoothly — leave heldMs untouched for one frame so the
    // consumer can animate from current progress → 0.
    if (!firedRef.current) {
      window.setTimeout(() => {
        // Only clear if we didn't start a new press in the meantime.
        if (startTsRef.current === null) {
          setHeldMs(0);
        }
      }, 280);
    }
  }, [onCancel, stopAnim]);

  // Cleanup on unmount.
  useEffect(() => {
    return () => {
      stopAnim();
      startTsRef.current = null;
    };
  }, [stopAnim]);

  const bind: HoldBindings = {
    onMouseDown: (e) => {
      // Left mouse button only.
      if (e.button !== 0) return;
      e.preventDefault();
      begin();
    },
    onMouseUp: () => end(),
    onMouseLeave: () => {
      if (isHolding) end();
    },
    onTouchStart: (e) => {
      e.preventDefault();
      begin();
    },
    onTouchEnd: () => end(),
    onTouchCancel: () => end(),
    onKeyDown: (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        if (!e.repeat) begin();
      }
    },
    onKeyUp: (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        end();
      }
    },
  };

  const progressPct = Math.min(100, (heldMs / thresholdMs) * 100);

  return { isHolding, heldMs, progressPct, confirmed, bind, reset };
}
