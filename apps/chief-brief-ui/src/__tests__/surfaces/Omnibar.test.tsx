import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import Omnibar from "../../surfaces/Omnibar";

vi.mock("../../api", () => ({
  v1OmnibarSearch: vi.fn(async (q: string) => ({
    hits: q.trim()
      ? [
          {
            id: "m-1",
            source: "memory",
            title: `Match for "${q}"`,
            snippet: "snippet",
          },
        ]
      : [],
    query: q,
  })),
}));

describe("Omnibar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not render when closed", () => {
    render(<Omnibar open={false} onClose={() => {}} />);
    expect(screen.queryByLabelText("Search")).toBeNull();
  });

  it("renders input + footer hints when open", () => {
    render(<Omnibar open={true} onClose={() => {}} />);
    expect(screen.getByLabelText("Search")).toBeInTheDocument();
    expect(screen.getByText(/open/)).toBeInTheDocument();
    expect(screen.getByText(/dismiss/)).toBeInTheDocument();
  });

  it("shows results after typing a query", async () => {
    vi.useFakeTimers();
    render(<Omnibar open={true} onClose={() => {}} />);
    const input = screen.getByLabelText("Search") as HTMLInputElement;

    await act(async () => {
      fireEvent.change(input, { target: { value: "alice" } });
      // Advance past the 220ms debounce.
      await vi.advanceTimersByTimeAsync(260);
    });

    expect(screen.getByText(/Match for "alice"/)).toBeInTheDocument();
    vi.useRealTimers();
  });

  it("shows an empty prompt when query is blank", () => {
    render(<Omnibar open={true} onClose={() => {}} />);
    expect(
      screen.getByText(/Search across your memory graph/)
    ).toBeInTheDocument();
  });
});
