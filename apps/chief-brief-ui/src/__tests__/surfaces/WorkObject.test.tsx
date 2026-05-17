import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import WorkObjectView from "../../surfaces/WorkObjectView";
import { v1GetWorkObject } from "../../api";
import type { WorkObject } from "../../types";

vi.mock("../../api", () => ({
  v1GetWorkObject: vi.fn(),
}));

const getWork = vi.mocked(v1GetWorkObject);

describe("WorkObjectView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders from the /v1/work projection", async () => {
    getWork.mockResolvedValueOnce(workObject());

    render(<WorkObjectView id="acme-follow-up" />);

    await waitFor(() =>
      expect(screen.getByText("Prepare the Acme follow-up")).toBeInTheDocument()
    );
    expect(getWork).toHaveBeenCalledWith("acme-follow-up");
    expect(screen.getByText("document-pack")).toBeInTheDocument();
    expect(screen.getByText("calendar-pack")).toBeInTheDocument();
    expect(screen.getByText("email-pack")).toBeInTheDocument();
    expect(screen.getAllByText("Needs Ceremony").length).toBeGreaterThan(0);
    expect(screen.getByText("fixtures/platform_demo/acme-contract.md")).toBeInTheDocument();
  });

  it("covers the empty state", async () => {
    getWork.mockResolvedValueOnce(workObject({ contributions: [] }));

    render(<WorkObjectView id="acme-follow-up" />);

    expect(
      await screen.findByText("No pack contributions yet")
    ).toBeInTheDocument();
  });

  it("covers a partial Work Object", async () => {
    getWork.mockResolvedValueOnce(
      workObject({ contributions: [contribution("document-pack", "handled")] })
    );

    render(<WorkObjectView id="acme-follow-up" />);

    expect(await screen.findByText("document-pack")).toBeInTheDocument();
    expect(screen.queryByText("email-pack")).toBeNull();
  });

  it("covers blocked and completed contribution states", async () => {
    getWork.mockResolvedValueOnce(
      workObject({
        contributions: [
          contribution("risk-pack", "blocked"),
          contribution("email-pack", "shipped"),
        ],
      })
    );

    render(<WorkObjectView id="acme-follow-up" />);

    await waitFor(() =>
      expect(screen.getAllByText("Blocked").length).toBeGreaterThan(0)
    );
    expect(screen.getAllByText("Shipped").length).toBeGreaterThan(0);
  });
});

function workObject(
  overrides: Partial<WorkObject> = {}
): WorkObject {
  return {
    id: "acme-follow-up",
    uri: "mem://artifact/acme-follow-up",
    title: "Prepare the Acme follow-up",
    source_refs: [
      {
        uri: "mem://file/acme-contract",
        node_type: "file",
        source: "fixture:document-pack",
        summary: "fixtures/platform_demo/acme-contract.md",
      },
    ],
    contributions: [
      contribution("document-pack", "handled"),
      contribution("calendar-pack", "handled"),
      contribution("email-pack", "needs_ceremony"),
    ],
    provenance: [
      {
        kind: "memory_node",
        uri: "mem://artifact/acme-follow-up",
        source: "chief-core:pack-interop-e2e",
      },
    ],
    ...overrides,
  };
}

function contribution(
  pack: string,
  authority_state: WorkObject["contributions"][number]["authority_state"]
): WorkObject["contributions"][number] {
  return {
    uri: `mem://artifact/${pack}`,
    node_type: pack === "email-pack" ? "artifact" : "finding",
    source: `pack:${pack}/agent`,
    pack,
    kind: pack === "calendar-pack" ? "candidate_slot" : "draft_reply",
    title: `${pack} contribution`,
    summary: "Projected by chief-core.",
    authority_state,
    source_refs: ["mem://file/acme-contract"],
    body: {},
  };
}
