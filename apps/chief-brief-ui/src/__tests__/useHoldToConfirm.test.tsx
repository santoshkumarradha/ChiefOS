import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useHoldToConfirm } from "../hooks/useHoldToConfirm";

describe("useHoldToConfirm", () => {
  it("starts with zero progress and not holding", () => {
    const { result } = renderHook(() =>
      useHoldToConfirm({ thresholdMs: 3000, onConfirm: () => {} })
    );
    expect(result.current.isHolding).toBe(false);
    expect(result.current.heldMs).toBe(0);
    expect(result.current.progressPct).toBe(0);
    expect(result.current.confirmed).toBe(false);
  });

  it("fires onConfirm after threshold elapses", async () => {
    vi.useFakeTimers();
    // Mock performance.now so we can advance it with a counter.
    let t = 0;
    const originalNow = performance.now.bind(performance);
    vi.spyOn(performance, "now").mockImplementation(() => t);
    // Ensure rAF runs deterministically: it queues a setTimeout here, which
    // vitest's fake timers can drive.
    const rafOriginal = globalThis.requestAnimationFrame;
    globalThis.requestAnimationFrame = ((cb: FrameRequestCallback) =>
      setTimeout(() => cb(t), 16)) as unknown as typeof requestAnimationFrame;

    const onConfirm = vi.fn();
    const { result } = renderHook(() =>
      useHoldToConfirm({ thresholdMs: 300, onConfirm })
    );

    // Begin hold.
    act(() => {
      result.current.bind.onMouseDown({
        button: 0,
        preventDefault: () => {},
      } as unknown as React.MouseEvent);
    });

    // Advance both the clock and the rAF timers past the threshold.
    await act(async () => {
      t = 400;
      await vi.advanceTimersByTimeAsync(500);
    });

    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(onConfirm.mock.calls[0][0]).toBeGreaterThanOrEqual(300);

    // Restore.
    (performance.now as unknown as { mockRestore: () => void }).mockRestore();
    globalThis.requestAnimationFrame = rafOriginal;
    vi.useRealTimers();
    // Ensure we call originalNow at least once so the var stays referenced.
    expect(originalNow()).toBeGreaterThanOrEqual(0);
  });
});
