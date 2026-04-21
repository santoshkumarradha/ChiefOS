import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useBriefState } from "../hooks/useBriefState";

// Mock the API module
vi.mock("../api", () => ({
  getBrief: vi.fn(),
}));

import { getBrief } from "../api";

describe("useBriefState Hook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.clearAllTimers();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  it("should return initial brief state (mock fallback)", async () => {
    (getBrief as any).mockRejectedValue(new Error("Offline"));

    const { result } = renderHook(() => useBriefState());

    // Initial state should be loading, then offline with mock data
    expect(result.current.brief).toBeDefined();
    expect(result.current.status).toBe("loading");

    await waitFor(() => {
      expect(result.current.status).toBe("offline");
    });

    // Mock data should still be present
    expect(result.current.brief.needs_you).toBeDefined();
  });

  it("should transition to live status on successful fetch", async () => {
    const mockBrief = {
      date: "2026-04-21T00:00:00Z",
      needs_you: [],
      handled: [],
      trust_ledger: { comms: 74 },
    };

    (getBrief as any).mockResolvedValue(mockBrief);

    const { result } = renderHook(() => useBriefState());

    expect(result.current.status).toBe("loading");

    // Advance timers to trigger the fetch
    await vi.runAllTimersAsync();

    await waitFor(() => {
      expect(result.current.status).toBe("live");
    });

    expect(result.current.error).toBeNull();
  });

  it("should set error on fetch failure", async () => {
    const error = new Error("Connection refused");
    (getBrief as any).mockRejectedValue(error);

    const { result } = renderHook(() => useBriefState());

    // Advance timers
    await vi.runAllTimersAsync();

    await waitFor(() => {
      expect(result.current.status).toBe("offline");
      expect(result.current.error).toBeDefined();
    });
  });

  it("should poll every 3 seconds", async () => {
    const mockBrief = {
      date: "2026-04-21T00:00:00Z",
      needs_you: [],
      handled: [],
      trust_ledger: {},
    };

    (getBrief as any).mockResolvedValue(mockBrief);

    const { result } = renderHook(() => useBriefState());

    // First fetch (initial)
    await vi.advanceTimersByTimeAsync(0);
    expect(getBrief).toHaveBeenCalledTimes(1);

    // Advance by 3 seconds
    await vi.advanceTimersByTimeAsync(3000);
    expect(getBrief).toHaveBeenCalledTimes(2);

    // Advance by 3 more seconds
    await vi.advanceTimersByTimeAsync(3000);
    expect(getBrief).toHaveBeenCalledTimes(3);
  });

  it("should reset backoff on successful fetch after error", async () => {
    // First call fails
    (getBrief as any)
      .mockRejectedValueOnce(new Error("Offline"))
      .mockResolvedValueOnce({
        date: "2026-04-21T00:00:00Z",
        needs_you: [],
        handled: [],
        trust_ledger: {},
      });

    const { result } = renderHook(() => useBriefState());

    // First fetch fails
    await vi.advanceTimersByTimeAsync(0);
    await waitFor(() => {
      expect(result.current.status).toBe("offline");
    });

    // Second fetch succeeds
    await vi.advanceTimersByTimeAsync(3000);
    await waitFor(() => {
      expect(result.current.status).toBe("live");
      expect(result.current.error).toBeNull();
    });
  });

  it("should cleanup interval on unmount", async () => {
    (getBrief as any).mockResolvedValue({
      date: "2026-04-21T00:00:00Z",
      needs_you: [],
      handled: [],
      trust_ledger: {},
    });

    const { unmount } = renderHook(() => useBriefState());

    const clearIntervalSpy = vi.spyOn(global, "clearInterval");

    unmount();

    expect(clearIntervalSpy).toHaveBeenCalled();

    clearIntervalSpy.mockRestore();
  });
});
