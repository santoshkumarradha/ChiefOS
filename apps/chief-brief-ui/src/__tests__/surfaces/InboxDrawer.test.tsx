import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, act } from "@testing-library/react";
import InboxDrawer from "../../surfaces/InboxDrawer";

// Mock the API module so the hook doesn't try to hit a real backend.
vi.mock("../../api", () => ({
  v1GetInbox: vi.fn().mockRejectedValue(new Error("offline")),
  v1InboxStreamUrl: vi.fn(() => "http://localhost/v1/inbox/stream"),
}));

describe("InboxDrawer", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    // Flush microtasks so the EventSource mock's queued onerror fires in
    // isolation before the next test.
  });

  it("does not render its dialog when closed", () => {
    render(
      <InboxDrawer
        open={false}
        onClose={() => {}}
        onCeremonyClick={() => {}}
      />
    );
    expect(screen.queryByText("HAX Inbox")).toBeNull();
  });

  it("renders the empty state when the API is offline", async () => {
    render(
      <InboxDrawer
        open={true}
        onClose={() => {}}
        onCeremonyClick={() => {}}
      />
    );

    // Let the rejected promise + EventSource onerror run.
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(screen.getByText("HAX Inbox")).toBeInTheDocument();
    // Empty-state title + body message from the offline branch.
    expect(screen.getByText("Inbox unavailable")).toBeInTheDocument();
    expect(screen.getByText(/chief-core is offline/i)).toBeInTheDocument();
  });

  it("renders tab chips with zero counts when empty", async () => {
    render(
      <InboxDrawer
        open={true}
        onClose={() => {}}
        onCeremonyClick={() => {}}
      />
    );
    await act(async () => {
      await Promise.resolve();
    });

    // Tab labels should all be present.
    expect(screen.getByRole("tab", { name: /All 0/ })).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: /Needs you 0/ })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("tab", { name: /Ceremony 0/ })
    ).toBeInTheDocument();
  });
});
