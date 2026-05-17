/**
 * Shared TypeScript types for chief-core /v1/* API.
 *
 * These mirror the contract the frontend expects. When chief-core exposes
 * the /v1 surface, these shapes are what it must produce. If a real endpoint
 * returns a legacy shape (e.g. the current `/brief` returns string arrays
 * in `needs_you`), the client adapter in api.ts coerces it.
 */

/* ---------- Brief ---------- */

export type Card = {
  card_id: string;
  intent_id: string;
  action_type: string;
  summary: string;
  region: string;
  surface: string;
  friction_tier: string;
  mem_uri: string;
  created_at: string;
  approved_at: string | null;
};

export type Brief = {
  date: string;
  needs_you: Card[];
  handled: Card[];
  trust_ledger: Record<string, number>;
};

/* ---------- Inbox ---------- */

export type InboxKind =
  | "NeedsYou"
  | "Handled"
  | "CeremonyPending"
  | "Anchor"
  | "System";

export type InboxItem = {
  id: string;
  kind: InboxKind;
  title: string;
  snippet: string;
  agent: string;
  ts: string; // ISO-8601
  badge?: string | null; // e.g. "Needs you", "Handled", "Review"
  ceremony_id?: string | null;
  mem_uri?: string | null;
};

export type InboxPage = {
  items: InboxItem[];
  next_cursor?: string | null;
};

/**
 * SSE event frame shape emitted on /v1/inbox/stream.
 * We keep this permissive so stale fields don't crash the UI.
 */
export type InboxStreamEvent =
  | { type: "upsert"; item: InboxItem }
  | { type: "remove"; id: string }
  | { type: "heartbeat"; ts: string };

/* ---------- Omnibar ---------- */

export type OmnibarSource = "memory" | "event" | "inbox" | "agent" | "pack";

export type OmnibarHit = {
  id: string;
  source: OmnibarSource;
  title: string;
  snippet: string;
  ts?: string | null;
  ref?: string | null; // e.g. a mem://... URI
};

export type OmnibarSearchResult = {
  hits: OmnibarHit[];
  query: string;
  took_ms?: number;
};

/* ---------- Work Object ---------- */

export type WorkAuthorityState =
  | "handled"
  | "blocked"
  | "needs_ceremony"
  | "shipped";

export type WorkSourceRef = {
  uri: string;
  node_type: string;
  source: string;
  summary: string;
};

export type WorkContribution = {
  uri: string;
  node_type: string;
  source: string;
  pack: string;
  kind: string;
  title: string;
  summary: string;
  authority_state: WorkAuthorityState;
  source_refs: string[];
  body: Record<string, unknown>;
};

export type WorkProvenanceRow = {
  kind: string;
  uri?: string;
  source?: string;
  ts?: string;
};

export type WorkObject = {
  id: string;
  uri: string;
  title: string;
  source_refs: WorkSourceRef[];
  contributions: WorkContribution[];
  provenance: WorkProvenanceRow[];
};

/* ---------- Ceremony ---------- */

export type CeremonyEvidenceRow = {
  label: string;
  value: string;
  note?: string | null;
};

export type CeremonyProvenanceRow = {
  actor: string;
  action: string;
  ts: string;
  outcome?: string | null;
};

export type Ceremony = {
  id: string;
  region: string; // e.g. "Region 6"
  title: string;
  eyebrow: string; // e.g. "CEREMONY · REGION 6 · CO-SIGN REQUIRED"
  summary: string;
  evidence: CeremonyEvidenceRow[];
  provenance: CeremonyProvenanceRow[];
  rollback_window: string; // e.g. "72 hours · funds held in escrow"
  threshold_ms: number; // hold duration required, typically 3000
};

export type CeremonyApproveResponse = {
  approved: true;
  attestation_id: string;
  held_ms: number;
};

export type CeremonyDenyResponse = {
  approved: false;
  reason: string;
};

/* ---------- Trust ledger ---------- */

export type TrustRow = {
  id: string;
  label: string;
  score: number; // 0..100
  delta: number; // + or - since last epoch
};

export type TrustLedger = {
  rows: TrustRow[];
  generated_at: string;
};

/* ---------- Models / cost ---------- */

export type ModelCost = {
  currency: string; // "USD"
  spend_today: number; // e.g. 0.42
  spend_month?: number;
  last_updated: string;
};

/* ---------- Attestation / status (legacy; preserved) ---------- */

export type Attestation = {
  tier: string;
  ok: boolean;
};

export type ApiStatus = {
  services: Record<string, string>;
  uptime_s: number;
  version: string;
};
