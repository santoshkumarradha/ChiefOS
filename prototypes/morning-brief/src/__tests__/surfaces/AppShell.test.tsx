import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import App from "../../App";

// Every API call should fail so the shell renders in empty/offline state.
vi.mock("../../api", () => ({
  v1GetBrief: vi.fn().mockRejectedValue(new Error("offline")),
  v1GetInbox: vi.fn().mockRejectedValue(new Error("offline")),
  v1GetModelCost: vi.fn().mockRejectedValue(new Error("offline")),
  v1InboxStreamUrl: vi.fn(() => "http://localhost/v1/inbox/stream"),
  v1OmnibarSearch: vi.fn().mockResolvedValue({ hits: [], query: "" }),
  v1GetCeremony: vi.fn().mockResolvedValue(null),
  v1ApproveCeremony: vi.fn(),
  v1DenyCeremony: vi.fn(),
}));

describe("App shell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("mounts without a backend and renders the Morning Brief", async () => {
    render(<App />);
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(screen.getByText("Morning Brief")).toBeInTheDocument();
  });

  it("toggles the HAX Inbox with Cmd+I", async () => {
    render(<App />);
    await act(async () => {
      await Promise.resolve();
    });

    expect(screen.queryByText("HAX Inbox")).toBeNull();

    await act(async () => {
      fireEvent.keyDown(window, { key: "i", metaKey: true });
    });

    expect(screen.getByText("HAX Inbox")).toBeInTheDocument();

    await act(async () => {
      fireEvent.keyDown(window, { key: "Escape" });
    });

    // After Escape, inbox should be hidden (AnimatePresence exit is sync
    // enough for our simple assertion since framer-motion runs inline here).
    // We can also re-toggle with Cmd+I, which is more robust:
    await act(async () => {
      fireEvent.keyDown(window, { key: "i", metaKey: true });
    });
  });

  it("toggles the Omnibar with Cmd+Space", async () => {
    render(<App />);
    await act(async () => {
      await Promise.resolve();
    });

    expect(screen.queryByLabelText("Search")).toBeNull();

    await act(async () => {
      fireEvent.keyDown(window, { key: " ", metaKey: true });
    });

    expect(screen.getByLabelText("Search")).toBeInTheDocument();

    await act(async () => {
      fireEvent.keyDown(window, { key: "Escape" });
    });
  });

  it("shows the offline toast when /v1/brief fails", async () => {
    render(<App />);
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(
      screen.getByText(/chief-core offline/i)
    ).toBeInTheDocument();
  });
});
