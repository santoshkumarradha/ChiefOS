import React from "react";
import {
  Badge,
  Pane,
  Row,
  SectionHeader,
  Select,
  SourceAvatar,
  TabBar,
  Widget,
} from "../../index.js";

export interface TierBindingView {
  tier: "fast" | "deep";
  kind: "local" | "cloud";
  provider?: string;
  modelId: string;
  lastUsed?: string;
}

export interface ProviderView {
  id: string;
  displayName: string;
  models: string[];
}

export interface ModelsSurfaceProps {
  fastBinding: TierBindingView | null;
  deepBinding: TierBindingView | null;
  providers: ProviderView[];
  defaults: {
    localOnly: boolean;
    dailyCostCapUsd: number | null;
  };
  deviceProbe: {
    ramGb: number;
    canRunDeepLocally: boolean;
    reason?: string;
  };
  onBindTier: (tier: "fast" | "deep", target: BindTarget) => Promise<void>;
  onAddProvider: (id: string) => Promise<void>;
  onRevokeProvider: (id: string) => Promise<void>;
  onSetLocalOnly: (v: boolean) => Promise<void>;
  onSetDailyCostCap: (usd: number | null) => Promise<void>;
}

export type BindTarget =
  | { kind: "local"; modelId: string }
  | { kind: "cloud"; provider: string; modelId: string };

export function ModelsSurface(props: ModelsSurfaceProps): JSX.Element {
  const [activeTab, setActiveTab] = React.useState<string>("bindings");
  const [localOnly, setLocalOnly] = React.useState(props.defaults.localOnly);
  const [costCap, setCostCap] = React.useState(
    props.defaults.dailyCostCapUsd?.toString() ?? "",
  );

  const handleSetLocalOnly = async (v: boolean) => {
    setLocalOnly(v);
    await props.onSetLocalOnly(v);
  };

  const handleSetCostCap = async (value: string) => {
    setCostCap(value);
    const usd = value === "" ? null : parseFloat(value);
    if (usd === null || !isNaN(usd)) {
      await props.onSetDailyCostCap(usd);
    }
  };

  return (
    <Pane id="models-surface" variant="content" tint="warm">
      <div style={{ padding: "1rem" }}>
        <h1 style={{ margin: "0 0 1.5rem 0" }}>Models</h1>

        <TabBar
          chips={[
            { id: "bindings", label: "Tier Bindings", highlighted: true },
            { id: "providers", label: "Cloud Providers" },
            { id: "preferences", label: "Preferences" },
          ]}
          selected={activeTab}
          onSelect={setActiveTab}
        />

        {activeTab === "bindings" && (
          <div style={{ marginTop: "1.5rem" }}>
            <SectionHeader>Fast Tier</SectionHeader>
            {props.fastBinding ? (
              <TierBindingCard
                binding={props.fastBinding}
                onBind={(target) => props.onBindTier("fast", target)}
              />
            ) : (
              <Widget title="Not Bound">
                <div style={{ padding: "1rem" }}>
                  Fast tier is not configured. Bind to a local model or cloud provider.
                </div>
              </Widget>
            )}

            <SectionHeader style={{ marginTop: "2rem" }}>Deep Tier</SectionHeader>
            {props.deepBinding ? (
              <TierBindingCard
                binding={props.deepBinding}
                onBind={(target) => props.onBindTier("deep", target)}
              />
            ) : (
              <Widget title="Not Bound">
                <div style={{ padding: "1rem" }}>
                  <p>
                    Deep tier is not configured. Your device:
                    {props.deviceProbe.canRunDeepLocally
                      ? " can run Deep locally."
                      : ` cannot run Deep locally (${props.deviceProbe.reason || "insufficient resources"}).`}
                  </p>
                  {!props.deviceProbe.canRunDeepLocally && (
                    <p>
                      Bind to a cloud provider or upgrade your device.
                    </p>
                  )}
                </div>
              </Widget>
            )}
          </div>
        )}

        {activeTab === "providers" && (
          <div style={{ marginTop: "1.5rem" }}>
            <SectionHeader>Configured Providers</SectionHeader>
            {props.providers.length === 0 ? (
              <Widget title="No providers">
                <div style={{ padding: "1rem" }}>
                  No cloud providers are configured.
                </div>
              </Widget>
            ) : (
              props.providers.map((provider) => (
                <ProviderCard
                  key={provider.id}
                  provider={provider}
                  onRevoke={() => props.onRevokeProvider(provider.id)}
                />
              ))
            )}

            <div style={{ marginTop: "1.5rem" }}>
              <button
                onClick={() => props.onAddProvider("anthropic")}
                style={{
                  padding: "0.75rem 1.5rem",
                  borderRadius: "4px",
                  border: "1px solid var(--neutral-400)",
                  cursor: "pointer",
                }}
              >
                Add Provider
              </button>
            </div>
          </div>
        )}

        {activeTab === "preferences" && (
          <div style={{ marginTop: "1.5rem" }}>
            <Widget title="Settings">
              <div style={{ padding: "1.5rem" }}>
                <div style={{ marginBottom: "1.5rem" }}>
                  <label style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
                    <input
                      type="checkbox"
                      checked={localOnly}
                      onChange={(e) => handleSetLocalOnly(e.target.checked)}
                    />
                    <span>Local-only mode (no cloud models)</span>
                  </label>
                </div>

                <div>
                  <label style={{ display: "block", marginBottom: "0.5rem" }}>
                    Daily cost cap (USD)
                  </label>
                  <input
                    type="number"
                    min="0"
                    step="0.01"
                    value={costCap}
                    onChange={(e) => handleSetCostCap(e.target.value)}
                    placeholder="No limit"
                    style={{
                      padding: "0.5rem",
                      borderRadius: "4px",
                      border: "1px solid var(--neutral-400)",
                      width: "100%",
                    }}
                  />
                </div>
              </div>
            </Widget>

            <Widget title="Device Capability" style={{ marginTop: "1.5rem" }}>
              <div style={{ padding: "1.5rem" }}>
                <p>
                  RAM: {props.deviceProbe.ramGb}GB
                </p>
                <p>
                  Can run Deep tier locally: {props.deviceProbe.canRunDeepLocally ? "Yes" : "No"}
                </p>
                {props.deviceProbe.reason && (
                  <p style={{ fontSize: "0.875rem", color: "var(--neutral-500)" }}>
                    {props.deviceProbe.reason}
                  </p>
                )}
              </div>
            </Widget>
          </div>
        )}
      </div>
    </Pane>
  );
}

interface TierBindingCardProps {
  binding: TierBindingView;
  onBind: (target: BindTarget) => Promise<void>;
}

function TierBindingCard({ binding, onBind }: TierBindingCardProps): JSX.Element {
  const kindLabel = binding.kind === "local" ? "Local" : "Cloud";
  const providerLabel = binding.provider ? ` (${binding.provider})` : "";

  return (
    <div
      style={{
        padding: "1rem",
        border: "1px solid var(--neutral-300)",
        borderRadius: "4px",
        marginTop: "0.75rem",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "1rem", marginBottom: "0.75rem" }}>
        <SourceAvatar
          color={binding.kind === "local" ? "teal" : "blue"}
          label={kindLabel}
        />
        <div>
          <div style={{ fontWeight: "500" }}>{binding.modelId}</div>
          <div style={{ fontSize: "0.875rem", color: "var(--neutral-600)" }}>
            {kindLabel}
            {providerLabel}
          </div>
        </div>
        {binding.lastUsed && (
          <Badge variant="neutral">Last used: {binding.lastUsed}</Badge>
        )}
      </div>
    </div>
  );
}

interface ProviderCardProps {
  provider: ProviderView;
  onRevoke: () => Promise<void>;
}

function ProviderCard({ provider, onRevoke }: ProviderCardProps): JSX.Element {
  return (
    <div
      style={{
        padding: "1rem",
        border: "1px solid var(--neutral-300)",
        borderRadius: "4px",
        marginTop: "0.75rem",
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
      }}
    >
      <div>
        <div style={{ fontWeight: "500" }}>{provider.displayName}</div>
        <div style={{ fontSize: "0.875rem", color: "var(--neutral-600)" }}>
          {provider.models.length} model{provider.models.length !== 1 ? "s" : ""} available
        </div>
      </div>
      <button
        onClick={onRevoke}
        style={{
          padding: "0.5rem 1rem",
          borderRadius: "4px",
          border: "1px solid var(--danger-400)",
          color: "var(--danger-600)",
          cursor: "pointer",
          backgroundColor: "transparent",
        }}
      >
        Revoke
      </button>
    </div>
  );
}
