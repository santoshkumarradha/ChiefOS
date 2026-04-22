export const SOURCE_COLORS = [
  "amber",
  "blue",
  "green",
  "copper",
  "gray",
  "teal",
  "violet",
  "peach",
] as const;

export const BADGE_VARIANTS = ["needs-you", "review", "handled", "ceremony", "anchor"] as const;

export const PANE_VARIANTS = ["content", "widget", "overlay"] as const;
export const PANE_TINTS = ["warm", "dark", "neutral"] as const;

export const MOTION_PRESETS = {
  "card-reveal": { stiffness: 220, damping: 28, mass: 1 },
  "trust-grow": { stiffness: 180, damping: 22, mass: 1 },
  "omnibar-summon": { stiffness: 260, damping: 30, mass: 0.9 },
  "ceremony-begin": { stiffness: 120, damping: 20, mass: 1.2 },
} as const;

export type SourceColor = (typeof SOURCE_COLORS)[number];
export type BadgeVariant = (typeof BADGE_VARIANTS)[number];
export type PaneVariant = (typeof PANE_VARIANTS)[number];
export type PaneTint = (typeof PANE_TINTS)[number];
export type MotionPresetName = keyof typeof MOTION_PRESETS;

export function isSourceColor(value: unknown): value is SourceColor {
  return typeof value === "string" && (SOURCE_COLORS as readonly string[]).includes(value);
}

export function isBadgeVariant(value: unknown): value is BadgeVariant {
  return typeof value === "string" && (BADGE_VARIANTS as readonly string[]).includes(value);
}

export function isPaneVariant(value: unknown): value is PaneVariant {
  return typeof value === "string" && (PANE_VARIANTS as readonly string[]).includes(value);
}

export function isPaneTint(value: unknown): value is PaneTint {
  return typeof value === "string" && (PANE_TINTS as readonly string[]).includes(value);
}

export function clampProgress(value: number): number {
  if (Number.isNaN(value)) {
    return 0;
  }

  return Math.min(1, Math.max(0, value));
}
