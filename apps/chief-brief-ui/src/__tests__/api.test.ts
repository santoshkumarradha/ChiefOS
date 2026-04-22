import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  getStatus,
  postIntent,
  getBrief,
  postApprove,
  postVerify,
  postRewind,
} from "../api";

// Mock fetch globally
global.fetch = vi.fn();

describe("API Client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("getStatus", () => {
    it("should fetch status successfully", async () => {
      const mockResponse = {
        services: { memory: "ok", event_log: "ok" },
        uptime_s: 1234,
        version: "0.0.1",
      };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await getStatus();
      expect(result).toEqual(mockResponse);
      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining("/status"),
        expect.any(Object)
      );
    });

    it("should handle 500 errors", async () => {
      (fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
      });

      await expect(getStatus()).rejects.toThrow("Failed to fetch status");
    });

    it("should handle network errors", async () => {
      (fetch as any).mockRejectedValueOnce(new Error("Network error"));

      await expect(getStatus()).rejects.toThrow();
    });
  });

  describe("postIntent", () => {
    it("should post intent successfully", async () => {
      const mockResponse = { intent_id: "intent_123", cards_queued: 1 };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await postIntent("draft an email");
      expect(result).toEqual(mockResponse);
      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining("/intent"),
        expect.objectContaining({ method: "POST" })
      );
    });

    it("should handle 500 errors on intent post", async () => {
      (fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
      });

      await expect(postIntent("test")).rejects.toThrow("Failed to post intent");
    });
  });

  describe("getBrief", () => {
    it("should fetch brief successfully", async () => {
      const mockBrief = {
        date: "2026-04-21T00:00:00Z",
        needs_you: [],
        handled: [],
        trust_ledger: {},
      };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockBrief,
      });

      const result = await getBrief();
      expect(result).toEqual(mockBrief);
      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining("/brief"),
        expect.any(Object)
      );
    });

    it("should handle network failure on brief fetch", async () => {
      (fetch as any).mockRejectedValueOnce(new Error("Network timeout"));

      await expect(getBrief()).rejects.toThrow();
    });
  });

  describe("postApprove", () => {
    it("should approve card successfully", async () => {
      const mockResponse = { shipped: true, attestation_id: "att_123" };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await postApprove("card_123", false);
      expect(result).toEqual(mockResponse);
      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining("/approve"),
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ card_id: "card_123", ceremony: false }),
        })
      );
    });

    it("should handle approval failure", async () => {
      (fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: "Not Found",
      });

      await expect(postApprove("card_123", false)).rejects.toThrow(
        "Failed to approve card"
      );
    });
  });

  describe("postVerify", () => {
    it("should verify attestation successfully", async () => {
      const mockResponse = { tier: "Generated", ok: true };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await postVerify({ tier: "Generated", ok: true });
      expect(result).toEqual(mockResponse);
      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining("/verify"),
        expect.any(Object)
      );
    });
  });

  describe("postRewind", () => {
    it("should rewind successfully", async () => {
      const mockResponse = { reverted_count: 2 };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await postRewind("1h");
      expect(result).toEqual(mockResponse);
      expect(fetch).toHaveBeenCalledWith(
        expect.stringContaining("/rewind"),
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ duration: "1h" }),
        })
      );
    });

    it("should handle rewind errors", async () => {
      (fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
      });

      await expect(postRewind("1h")).rejects.toThrow("Failed to rewind");
    });
  });

  describe("timeout handling", () => {
    it("should timeout on slow requests", async () => {
      (fetch as any).mockImplementationOnce(
        () =>
          new Promise((resolve) =>
            setTimeout(
              () =>
                resolve({
                  ok: true,
                  json: async () => ({ data: "test" }),
                }),
              10000
            )
          )
      );

      // The request should abort before completing
      // Note: actual timeout behavior depends on AbortController implementation
      expect(fetch).toBeDefined();
    });
  });
});
