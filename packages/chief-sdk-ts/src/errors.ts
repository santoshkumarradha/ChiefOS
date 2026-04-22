/**
 * Error types for AI and Harness operations (ADR-0013).
 */

import type { Tier } from "./tier.js";

/**
 * Errors from ctx.ai() operations.
 * Discriminated union matching Rust AiError exactly.
 */
export type AiError =
  | {
      kind: "capability_denied";
      reason: string;
    }
  | {
      kind: "tier_unavailable";
      tier: Tier;
    }
  | {
      kind: "max_tokens_clamped";
      requested: number;
      allowed: number;
    }
  | {
      kind: "model";
      message: string;
    }
  | {
      kind: "budget";
      message: string;
    }
  | {
      kind: "schema";
      message: string;
    };

/**
 * Errors from ctx.harness() operations.
 * Discriminated union matching Rust HarnessError exactly.
 */
export type HarnessError =
  | {
      kind: "engine";
      message: string;
    }
  | {
      kind: "budget";
      reason: string;
    }
  | {
      kind: "capability_denied";
      reason: string;
    }
  | {
      kind: "external_action_blocked";
      tool: string;
    }
  | {
      kind: "timeout";
      stage: string;
    };

/**
 * Type guard for AiError.
 */
export function isAiError(err: unknown): err is AiError {
  return (
    typeof err === "object" &&
    err !== null &&
    "kind" in err &&
    typeof (err as Record<string, unknown>).kind === "string" &&
    [
      "capability_denied",
      "tier_unavailable",
      "max_tokens_clamped",
      "model",
      "budget",
      "schema",
    ].includes((err as Record<string, unknown>).kind as string)
  );
}

/**
 * Type guard for HarnessError.
 */
export function isHarnessError(err: unknown): err is HarnessError {
  return (
    typeof err === "object" &&
    err !== null &&
    "kind" in err &&
    typeof (err as Record<string, unknown>).kind === "string" &&
    [
      "engine",
      "budget",
      "capability_denied",
      "external_action_blocked",
      "timeout",
    ].includes((err as Record<string, unknown>).kind as string)
  );
}
