/**
 * Shared TypeScript types for Morning Brief and chief-core API.
 */

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

export type Attestation = {
  tier: string;
  ok: boolean;
};

export type ApiStatus = {
  services: Record<string, string>;
  uptime_s: number;
  version: string;
};
