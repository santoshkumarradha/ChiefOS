import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, act } from "@testing-library/react";
import Menubar from "../../surfaces/Menubar";

vi.mock("../../api", () => ({
  v1GetModelCost: vi.fn(),
}));

describe("Menubar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders brand + menu labels", async () => {
    const { v1GetModelCost } = await import("../../api");
    (v1GetModelCost as unknown as { mockRejectedValue: Function })
      .mockRejectedValue(new Error("offline"));

    render(<Menubar />);
    expect(screen.getByText("Chief")).toBeInTheDocument();
    expect(screen.getByText("Brief")).toBeInTheDocument();
    expect(screen.getByText("Approvals")).toBeInTheDocument();
  });

  it("shows em-dash when cost endpoint fails (no fake numbers)", async () => {
    const { v1GetModelCost } = await import("../../api");
    (v1GetModelCost as unknown as { mockRejectedValue: Function })
      .mockRejectedValue(new Error("offline"));

    render(<Menubar />);
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(screen.getByTitle("Today's model spend")).toHaveTextContent("—");
  });

  it("shows real spend when cost endpoint succeeds", async () => {
    const { v1GetModelCost } = await import("../../api");
    (v1GetModelCost as unknown as { mockResolvedValue: Function })
      .mockResolvedValue({
        currency: "USD",
        spend_today: 0.42,
        last_updated: "2026-04-21T00:00:00Z",
      });

    render(<Menubar />);
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(screen.getByTitle("Today's model spend")).toHaveTextContent(
      "$0.42"
    );
  });
});
