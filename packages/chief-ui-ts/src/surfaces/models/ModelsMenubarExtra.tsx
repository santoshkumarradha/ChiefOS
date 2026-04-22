import React from "react";
import { MenubarExtra } from "../../index.js";

export interface ModelsMenubarExtraProps {
  costDisplay?: string; // e.g., "$0.42"
  tierStatus?: {
    fast: "bound" | "unbound";
    deep: "bound" | "unbound";
  };
  onSummon: () => void;
}

/**
 * Menubar entry point for Models surface.
 * Displays as a brain glyph icon in the status strip.
 * Tap shows a summary popover, click opens the full surface.
 */
export function ModelsMenubarExtra({
  costDisplay = "$0.00",
  tierStatus,
  onSummon,
}: ModelsMenubarExtraProps): JSX.Element {
  const deepStatus = tierStatus?.deep ?? "unbound";
  const isActive = deepStatus === "unbound";

  return (
    <MenubarExtra
      id="models-menubar"
      label="Models"
      active={isActive}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "0.5rem",
          padding: "0.75rem",
          minWidth: "200px",
          fontSize: "0.875rem",
        }}
      >
        <div>
          <div style={{ fontWeight: "500", marginBottom: "0.25rem" }}>Model Status</div>
          <div style={{ fontSize: "0.75rem", color: "var(--neutral-600)" }}>
            Fast: <span style={{ color: tierStatus?.fast === "bound" ? "var(--emerald-500)" : "var(--coral-500)" }}>
              {tierStatus?.fast ?? "unknown"}
            </span>
          </div>
          <div style={{ fontSize: "0.75rem", color: "var(--neutral-600)" }}>
            Deep: <span style={{ color: tierStatus?.deep === "bound" ? "var(--emerald-500)" : "var(--coral-500)" }}>
              {tierStatus?.deep ?? "unknown"}
            </span>
          </div>
        </div>

        <div style={{ marginLeft: "auto" }}>
          <div style={{ fontSize: "0.75rem", color: "var(--neutral-600)" }}>Cost today</div>
          <div style={{ fontWeight: "500" }}>{costDisplay}</div>
        </div>

        <button
          onClick={onSummon}
          style={{
            marginTop: "0.75rem",
            width: "100%",
            padding: "0.5rem",
            borderRadius: "4px",
            border: "1px solid var(--neutral-400)",
            cursor: "pointer",
            backgroundColor: "var(--neutral-50)",
          }}
        >
          Open Models
        </button>
      </div>
    </MenubarExtra>
  );
}
