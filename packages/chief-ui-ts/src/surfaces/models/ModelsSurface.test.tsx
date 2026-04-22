import { describe, it, expect, vi } from "vitest";
import { render } from "@testing-library/react";
import { ModelsSurface, type ModelsSurfaceProps } from "./ModelsSurface.js";

describe("ModelsSurface", () => {
  const defaultProps: ModelsSurfaceProps = {
    fastBinding: {
      tier: "fast",
      kind: "local",
      modelId: "qwen-2.5-3b",
      lastUsed: "today",
    },
    deepBinding: null,
    providers: [
      {
        id: "anthropic",
        displayName: "Anthropic",
        models: ["claude-sonnet-4.6"],
      },
    ],
    defaults: {
      localOnly: false,
      dailyCostCapUsd: null,
    },
    deviceProbe: {
      ramGb: 16,
      canRunDeepLocally: false,
      reason: "Insufficient RAM (need 24GB+)",
    },
    onBindTier: vi.fn(),
    onAddProvider: vi.fn(),
    onRevokeProvider: vi.fn(),
    onSetLocalOnly: vi.fn(),
    onSetDailyCostCap: vi.fn(),
  };

  it("renders without throwing", () => {
    const { container } = render(<ModelsSurface {...defaultProps} />);
    expect(container).toBeTruthy();
  });

  it("renders with fast binding", () => {
    const { getByText } = render(<ModelsSurface {...defaultProps} />);
    expect(getByText("qwen-2.5-3b")).toBeTruthy();
  });

  it("shows unbound deep tier message", () => {
    const { getByText } = render(<ModelsSurface {...defaultProps} />);
    expect(getByText(/Deep tier is not configured/)).toBeTruthy();
  });

  it("displays device capability info", () => {
    const { getByText } = render(<ModelsSurface {...defaultProps} />);
    // Navigate to preferences tab
    const prefsTab = getByText("Preferences").closest("div");
    prefsTab?.click();
    // Should show device info
    expect(getByText(/RAM: 16GB/)).toBeTruthy();
  });
});
