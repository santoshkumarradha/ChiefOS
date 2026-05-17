/**
 * Typed fetch wrapper for chief-core HTTP API.
 *
 * Two base URLs in play:
 *
 * 1. `BASE_URL` — legacy chief-core routes (/status, /intent, /brief, …).
 *    Kept for backwards compatibility with existing tests and the pre-v1
 *    kernel surface. Defaults to http://127.0.0.1:4711.
 *
 * 2. `V1_BASE` — the forward-looking /v1 surface that this UI targets.
 *    Uses a RELATIVE path ("/v1") so the same bundle works under
 *    the vite dev proxy and under same-origin production serving.
 *    Override with VITE_CHIEF_CORE_URL=<origin> at build time.
 */

import type {
  Attestation,
  ApiStatus,
  Brief,
  Card,
  Ceremony,
  CeremonyApproveResponse,
  CeremonyDenyResponse,
  InboxItem,
  InboxPage,
  ModelCost,
  OmnibarSearchResult,
  TrustLedger,
  WorkAuthorityState,
  WorkObject,
} from "./types";

const LEGACY_BASE: string =
  ((import.meta as any).env &&
    (import.meta as any).env.VITE_CHIEF_CORE_URL) ||
  "http://127.0.0.1:4711";

// When VITE_CHIEF_CORE_URL is set, /v1 is also prefixed with that origin.
// When unset (the common case in dev + prod), we use relative /v1 so the
// vite proxy (dev) or same-origin server (prod) routes to chief-core.
const V1_ORIGIN: string =
  ((import.meta as any).env &&
    (import.meta as any).env.VITE_CHIEF_CORE_URL) ||
  "";
const V1_BASE = `${V1_ORIGIN}/v1`;

// Exported so tests can assert paths.
export const __bases = {
  legacy: LEGACY_BASE,
  v1: V1_BASE,
};

class ApiError extends Error {
  constructor(public statusCode: number, message: string) {
    super(message);
    this.name = "ApiError";
  }
}

export { ApiError };

async function fetchWithTimeout(
  url: string,
  options: RequestInit & { timeout?: number } = {}
): Promise<Response> {
  const { timeout = 5000, ...fetchOptions } = options;
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);

  try {
    const response = await fetch(url, {
      ...fetchOptions,
      signal: controller.signal,
    });
    return response;
  } finally {
    clearTimeout(timeoutId);
  }
}

async function jsonGet<T>(url: string, timeout = 5000): Promise<T> {
  const response = await fetchWithTimeout(url, { timeout });
  if (!response.ok) {
    throw new ApiError(response.status, `GET ${url} failed: ${response.statusText}`);
  }
  return response.json();
}

async function jsonPost<T>(
  url: string,
  body: unknown,
  timeout = 15000
): Promise<T> {
  const response = await fetchWithTimeout(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body ?? {}),
    timeout,
  });
  if (!response.ok) {
    throw new ApiError(response.status, `POST ${url} failed: ${response.statusText}`);
  }
  return response.json();
}

/* ---------- Legacy chief-core routes (no /v1 prefix) ---------- */

export async function getStatus(): Promise<ApiStatus> {
  const response = await fetchWithTimeout(`${LEGACY_BASE}/status`, {
    timeout: 5000,
  });
  if (!response.ok) {
    throw new ApiError(response.status, `Failed to fetch status: ${response.statusText}`);
  }
  return response.json();
}

export async function postIntent(text: string): Promise<{ intent_id: string; cards_queued: number }> {
  const response = await fetchWithTimeout(`${LEGACY_BASE}/intent`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ text }),
    timeout: 30000,
  });
  if (!response.ok) {
    throw new ApiError(response.status, `Failed to post intent: ${response.statusText}`);
  }
  return response.json();
}

export async function getBrief(): Promise<Brief> {
  const response = await fetchWithTimeout(`${LEGACY_BASE}/brief`, {
    timeout: 5000,
  });
  if (!response.ok) {
    throw new ApiError(response.status, `Failed to fetch brief: ${response.statusText}`);
  }
  return response.json();
}

export async function postApprove(
  cardId: string,
  ceremony: boolean
): Promise<{ shipped: boolean; attestation_id: string }> {
  const response = await fetchWithTimeout(`${LEGACY_BASE}/approve`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ card_id: cardId, ceremony }),
    timeout: 30000,
  });
  if (!response.ok) {
    throw new ApiError(response.status, `Failed to approve card: ${response.statusText}`);
  }
  return response.json();
}

export async function postVerify(attestation: Attestation): Promise<Attestation> {
  const response = await fetchWithTimeout(`${LEGACY_BASE}/verify`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(attestation),
    timeout: 5000,
  });
  if (!response.ok) {
    throw new ApiError(response.status, `Failed to verify attestation: ${response.statusText}`);
  }
  return response.json();
}

export async function postRewind(duration: string): Promise<{ reverted_count: number }> {
  const response = await fetchWithTimeout(`${LEGACY_BASE}/rewind`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ duration }),
    timeout: 5000,
  });
  if (!response.ok) {
    throw new ApiError(response.status, `Failed to rewind: ${response.statusText}`);
  }
  return response.json();
}

/* ---------- /v1 surface ---------- */

/* ---------- Adapters: normalize real backend shapes → frontend types ----- */

type RawBriefNeed = {
  id: string;
  title: string;
  snippet: string;
  source_agent: string;
  ceremony_id?: string | null;
  created_at: string;
};
type RawBriefHandled = {
  id: string;
  title: string;
  summary?: string | null;
  snippet?: string | null;
  source_agent: string;
  completed_at: string;
};
type RawBriefTrustRow = {
  category: string;
  score: number;
  raw: number;
  approvals: number;
  setbacks: number;
};
type RawBrief = {
  greeting: string;
  needs_you: RawBriefNeed[];
  handled: RawBriefHandled[];
  provenance: unknown[];
  trust: RawBriefTrustRow[];
  signed_by: string;
  generated_at: string;
};

function adaptBrief(raw: RawBrief): Brief {
  const needs_you: Card[] = (raw.needs_you ?? []).map((n) => ({
    card_id: n.id,
    intent_id: n.ceremony_id ?? "",
    action_type: n.ceremony_id ? "ceremony" : "needs_you",
    summary: n.title,
    region: n.snippet,
    surface: n.source_agent,
    friction_tier: n.ceremony_id ? "Hold 3s to approve" : "",
    mem_uri: "",
    created_at: n.created_at,
    approved_at: null,
  }));
  const handled: Card[] = (raw.handled ?? []).map((h) => ({
    card_id: h.id,
    intent_id: "",
    action_type: "handled",
    summary: h.title,
    region: h.snippet ?? h.summary ?? "",
    surface: h.source_agent,
    friction_tier: "",
    mem_uri: "",
    created_at: h.completed_at,
    approved_at: h.completed_at,
  }));
  const trust_ledger: Record<string, number> = {};
  for (const row of raw.trust ?? []) {
    trust_ledger[row.category] = row.score;
  }
  return {
    date: raw.generated_at,
    needs_you,
    handled,
    trust_ledger,
  };
}

type RawInboxItem = {
  id: string;
  kind: string; // "ceremony_pending" | "informational" | "needs_attention" | ...
  title: string;
  snippet: string;
  source_agent: string;
  badge: string;
  timestamp: string;
  ceremony_id?: string | null;
};

function adaptInboxKind(raw: string): InboxItem["kind"] {
  switch (raw) {
    case "ceremony_pending":
      return "CeremonyPending";
    case "needs_attention":
      return "NeedsYou";
    case "informational":
      return "Handled";
    case "anchor":
      return "Anchor";
    default:
      return "System";
  }
}

function adaptInboxItem(raw: RawInboxItem): InboxItem {
  return {
    id: raw.id,
    kind: adaptInboxKind(raw.kind),
    title: raw.title,
    snippet: raw.snippet,
    agent: raw.source_agent,
    ts: raw.timestamp,
    badge: raw.badge ?? null,
    ceremony_id: raw.ceremony_id ?? null,
    mem_uri: null,
  };
}

/* ---------- Public /v1 API ---------- */

/**
 * Morning Brief (/v1/brief).
 * Transforms the backend shape into the frontend's internal Brief shape so
 * downstream components (MorningBrief.tsx, trust ledger panel) don't need
 * to know about the chief-core wire format.
 */
export async function v1GetBrief(): Promise<Brief> {
  const raw = await jsonGet<RawBrief>(`${V1_BASE}/brief`);
  return adaptBrief(raw);
}

/**
 * HAX Inbox — initial page (/v1/inbox).
 * Backend returns a bare array; we wrap it in an InboxPage for the UI.
 */
export async function v1GetInbox(): Promise<InboxPage> {
  const raw = await jsonGet<RawInboxItem[]>(`${V1_BASE}/inbox`);
  return {
    items: raw.map(adaptInboxItem),
    next_cursor: null,
  };
}

// Exposed for the SSE hook so stream frames can be parsed with the same adapter.
export function adaptInboxItemExternal(raw: RawInboxItem): InboxItem {
  return adaptInboxItem(raw);
}

/**
 * Stream URL for /v1/inbox/stream. The hook consumes this with EventSource.
 * Exposed as a string so the hook can construct the EventSource itself
 * (EventSource doesn't play well with our fetch wrapper).
 */
export function v1InboxStreamUrl(): string {
  return `${V1_BASE}/inbox/stream`;
}

/**
 * Omnibar search (/v1/omnibar/search). Debounced at the call site.
 * Backend returns bare array of hits; UI expects a wrapped result.
 */
export async function v1OmnibarSearch(query: string): Promise<OmnibarSearchResult> {
  if (!query.trim()) {
    return { hits: [], query };
  }
  const started = Date.now();
  type RawHit = {
    id: string;
    source: string;
    title: string;
    snippet: string;
    timestamp?: string | null;
    ref?: string | null;
  };
  const raw = await jsonPost<RawHit[]>(
    `${V1_BASE}/omnibar/search`,
    { q: query },
    5000,
  );
  return {
    query,
    took_ms: Date.now() - started,
    hits: raw.map((h) => ({
      id: h.id,
      source: (h.source as OmnibarSearchResult["hits"][number]["source"]) ?? "memory",
      title: h.title,
      snippet: h.snippet,
      ts: h.timestamp ?? null,
      ref: h.ref ?? null,
    })),
  };
}

/**
 * Work Object projection (/v1/work/{id}).
 *
 * The backend owns the graph projection, contribution summaries, and authority
 * state. The UI only groups and renders the returned rows.
 */
export async function v1GetWorkObject(id: string): Promise<WorkObject> {
  type RawContribution = {
    uri: string;
    node_type: string;
    source: string;
    pack?: string | null;
    kind?: string | null;
    title?: string | null;
    summary?: string | null;
    authority_state?: string | null;
    source_refs?: string[] | null;
    body?: Record<string, unknown> | null;
  };
  type RawWorkObject = {
    id: string;
    uri: string;
    title: string;
    source_refs?: WorkObject["source_refs"] | null;
    contributions?: RawContribution[] | null;
    provenance?: WorkObject["provenance"] | null;
  };

  const raw = await jsonGet<RawWorkObject>(
    `${V1_BASE}/work/${encodeURIComponent(id)}`,
  );
  return {
    id: raw.id,
    uri: raw.uri,
    title: raw.title,
    source_refs: raw.source_refs ?? [],
    contributions: (raw.contributions ?? []).map((c) => ({
      uri: c.uri,
      node_type: c.node_type,
      source: c.source,
      pack: c.pack ?? packName(c.source),
      kind: c.kind ?? String(c.body?.kind ?? "unknown"),
      title: c.title ?? String(c.body?.title ?? c.body?.subject ?? "Contribution"),
      summary: c.summary ?? contributionFallbackSummary(c.body),
      authority_state: normalizeAuthority(c.authority_state),
      source_refs: c.source_refs ?? [],
      body: c.body ?? {},
    })),
    provenance: raw.provenance ?? [],
  };
}

function packName(source: string): string {
  return source.replace(/^pack:/, "").split("/")[0] || source;
}

function normalizeAuthority(value?: string | null): WorkAuthorityState {
  if (
    value === "blocked" ||
    value === "needs_ceremony" ||
    value === "shipped"
  ) {
    return value;
  }
  return "handled";
}

function contributionFallbackSummary(
  body?: Record<string, unknown> | null
): string {
  if (!body) return "";
  for (const key of ["content", "rationale", "summary", "subject", "body"]) {
    const value = body[key];
    if (typeof value === "string") return value;
  }
  return "";
}

/**
 * Fetch a single ceremony by id (/v1/ceremony/{id}).
 */
export async function v1GetCeremony(id: string): Promise<Ceremony> {
  type RawCeremony = {
    id: string;
    title: string;
    evidence: {
      summary: string;
      details: Record<string, unknown>;
    };
    source_agent: string;
    trust_context: number;
    proposed_grant: Record<string, unknown>;
    target_principal: string;
    rollback_window_secs: number;
    created_at: string;
    expires_at: string;
    status: string;
  };
  const raw = await jsonGet<RawCeremony>(
    `${V1_BASE}/ceremony/${encodeURIComponent(id)}`,
  );
  // Flatten the evidence.details object into labeled rows the UI can render.
  const evidence = [
    { label: "Principal", value: String(raw.evidence.details.principal ?? raw.source_agent) },
    { label: "Requested op", value: String(raw.evidence.details.requested_op ?? "") },
    { label: "Denial", value: String(raw.evidence.details.denial_reason ?? raw.evidence.details.denial_kind ?? "") },
  ].filter((r) => r.value.length > 0);
  const rollbackHours = Math.round(raw.rollback_window_secs / 3600);
  return {
    id: raw.id,
    region: `Region ${raw.trust_context}`,
    title: raw.title,
    eyebrow: `CEREMONY · REGION ${raw.trust_context} · CO-SIGN REQUIRED`,
    summary: raw.evidence.summary,
    evidence,
    provenance: [
      { actor: raw.source_agent, action: "requested", ts: raw.created_at },
    ],
    rollback_window: `${rollbackHours} hours · grant held in escrow`,
    threshold_ms: 3000,
  };
}

/**
 * Approve a ceremony with the held duration in ms (/v1/ceremony/{id}/approve).
 */
export async function v1ApproveCeremony(
  id: string,
  heldMs: number
): Promise<CeremonyApproveResponse> {
  return jsonPost<CeremonyApproveResponse>(
    `${V1_BASE}/ceremony/${encodeURIComponent(id)}/approve`,
    { held_ms: heldMs },
    10000
  );
}

/**
 * Deny a ceremony (/v1/ceremony/{id}/deny).
 */
export async function v1DenyCeremony(
  id: string,
  reason?: string
): Promise<CeremonyDenyResponse> {
  return jsonPost<CeremonyDenyResponse>(
    `${V1_BASE}/ceremony/${encodeURIComponent(id)}/deny`,
    { reason: reason ?? "" },
    5000
  );
}

/**
 * Trust ledger (/v1/trust-ledger).
 */
export async function v1GetTrust(): Promise<TrustLedger> {
  type RawRow = {
    category: string;
    score: number;
    raw: number;
    approvals: number;
    setbacks: number;
  };
  const raw = await jsonGet<RawRow[]>(`${V1_BASE}/trust-ledger`);
  return {
    rows: raw.map((r) => ({
      id: r.category,
      label: r.category,
      score: r.score,
      delta: r.approvals - r.setbacks,
    })),
    generated_at: new Date().toISOString(),
  };
}

/**
 * Model cost tracker. Backend exposes /v1/models; it currently returns
 * {tiers: {fast, deep}, bindings: []} without a cost figure. The menubar
 * falls back to "—" when this is missing.
 */
export async function v1GetModelCost(): Promise<ModelCost> {
  type RawModels = {
    tiers: { fast: unknown; deep: unknown };
    bindings: unknown[];
  };
  try {
    await jsonGet<RawModels>(`${V1_BASE}/models`);
  } catch {
    // Fall through to zero-state response.
  }
  return {
    currency: "USD",
    spend_today: 0,
    last_updated: new Date().toISOString(),
  };
}
