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
  Ceremony,
  CeremonyApproveResponse,
  CeremonyDenyResponse,
  InboxItem,
  InboxPage,
  ModelCost,
  OmnibarSearchResult,
  TrustLedger,
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

/**
 * Morning Brief (/v1/brief).
 */
export async function v1GetBrief(): Promise<Brief> {
  return jsonGet<Brief>(`${V1_BASE}/brief`);
}

/**
 * HAX Inbox — initial page (/v1/inbox).
 */
export async function v1GetInbox(): Promise<InboxPage> {
  return jsonGet<InboxPage>(`${V1_BASE}/inbox`);
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
 */
export async function v1OmnibarSearch(query: string): Promise<OmnibarSearchResult> {
  if (!query.trim()) {
    return { hits: [], query };
  }
  return jsonPost<OmnibarSearchResult>(
    `${V1_BASE}/omnibar/search`,
    { query },
    5000
  );
}

/**
 * Fetch a single ceremony by id (/v1/ceremony/{id}).
 */
export async function v1GetCeremony(id: string): Promise<Ceremony> {
  return jsonGet<Ceremony>(`${V1_BASE}/ceremony/${encodeURIComponent(id)}`);
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
 * Trust ledger (/v1/trust).
 */
export async function v1GetTrust(): Promise<TrustLedger> {
  return jsonGet<TrustLedger>(`${V1_BASE}/trust`);
}

/**
 * Model cost tracker (/v1/models/cost).
 * The menubar shows spend_today. If this 404s, the UI shows "—".
 */
export async function v1GetModelCost(): Promise<ModelCost> {
  return jsonGet<ModelCost>(`${V1_BASE}/models/cost`);
}
