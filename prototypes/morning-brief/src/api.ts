/**
 * Typed fetch wrapper for chief-core HTTP API.
 * Base URL from import.meta.env.VITE_CHIEF_CORE_URL (default http://127.0.0.1:4711)
 */

import type { Brief, Attestation, ApiStatus } from "./types";

const BASE_URL: string = (import.meta as any).env.VITE_CHIEF_CORE_URL || "http://127.0.0.1:4711";

class ApiError extends Error {
  constructor(public statusCode: number, message: string) {
    super(message);
    this.name = "ApiError";
  }
}

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

export async function getStatus(): Promise<ApiStatus> {
  const response = await fetchWithTimeout(`${BASE_URL}/status`, {
    timeout: 5000,
  });
  if (!response.ok) {
    throw new ApiError(response.status, `Failed to fetch status: ${response.statusText}`);
  }
  return response.json();
}

export async function postIntent(text: string): Promise<{ intent_id: string; cards_queued: number }> {
  const response = await fetchWithTimeout(`${BASE_URL}/intent`, {
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
  const response = await fetchWithTimeout(`${BASE_URL}/brief`, {
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
  const response = await fetchWithTimeout(`${BASE_URL}/approve`, {
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
  const response = await fetchWithTimeout(`${BASE_URL}/verify`, {
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
  const response = await fetchWithTimeout(`${BASE_URL}/rewind`, {
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
