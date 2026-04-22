import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, act } from "@testing-library/react";
import Ceremony from "../../surfaces/Ceremony";

vi.mock("../../api", () => {
  // Factory must be self-contained — top-level vars are hoisted later.
  const ceremony = {
    id: "cer-1",
    region: "Region 6",
    title: "Approve $4,200 payment to Acme Corp.",
    eyebrow: "CEREMONY · REGION 6 · CO-SIGN REQUIRED",
    summary: "Counter-offer negotiated by Chief on Monday.",
    evidence: [
      { label: "Recipient", value: "Acme Corp — routing verified" },
      { label: "Amount", value: "$4,200.00 USD", note: "one-time" },
    ],
    provenance: [
      {
        actor: "Negotiator Agent",
        action: "drafted counter",
        ts: "08:02",
      },
    ],
    rollback_window: "72 hours — funds held in escrow",
    threshold_ms: 3000,
  };
  return {
    v1GetCeremony: vi.fn().mockResolvedValue(ceremony),
    v1ApproveCeremony: vi.fn().mockResolvedValue({
      approved: true,
      attestation_id: "att_1",
      held_ms: 3100,
    }),
    v1DenyCeremony: vi.fn().mockResolvedValue({
      approved: false,
      reason: "denied",
    }),
  };
});

describe("Ceremony", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders loading state before the fetch resolves", () => {
    render(<Ceremony id="cer-1" onDone={() => {}} />);
    expect(screen.getByText(/Preparing evidence/i)).toBeInTheDocument();
  });

  it("renders ceremony content after fetch", async () => {
    render(<Ceremony id="cer-1" onDone={() => {}} />);
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(
      screen.getByText(/CEREMONY · REGION 6 · CO-SIGN REQUIRED/i)
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Approve \$4,200 payment to Acme Corp\./)
    ).toBeInTheDocument();
    expect(screen.getByText("Recipient")).toBeInTheDocument();
    expect(screen.getByText(/\$4,200\.00 USD/)).toBeInTheDocument();
  });

  it("renders the hold-to-approve ring and deny button", async () => {
    render(<Ceremony id="cer-1" onDone={() => {}} />);
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(
      screen.getByRole("button", { name: /Hold to approve/i })
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Deny/i })).toBeInTheDocument();
  });

  it("renders error state when fetch fails", async () => {
    const { v1GetCeremony } = await import("../../api");
    (v1GetCeremony as unknown as { mockRejectedValueOnce: Function })
      .mockRejectedValueOnce(new Error("404 not found"));

    render(<Ceremony id="bad" onDone={() => {}} />);
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(
      screen.getByText(/Unable to load ceremony/i)
    ).toBeInTheDocument();
  });
});
