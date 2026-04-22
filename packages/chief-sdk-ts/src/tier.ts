/**
 * Tier — Two-tier LLM access control (closed enum, ADR-0013).
 *
 * Packs declare which tier of model they need: Fast for cheap inference
 * (classification, formatting), Deep for capable reasoning (synthesis, tool-using).
 * The user's Models configuration maps each tier to a concrete model.
 */

/**
 * Closed enumeration of LLM capability tiers.
 * Adding new variants requires an ADR amendment.
 */
export type Tier = "fast" | "deep";

/**
 * All available tier variants (used for exhaustiveness checks).
 */
export const ALL_TIERS: readonly Tier[] = ["fast", "deep"] as const;
