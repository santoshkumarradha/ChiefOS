import { deepEqual, equal } from "node:assert/strict";
import { test } from "node:test";
import {
  BADGE_VARIANTS,
  MOTION_PRESETS,
  SOURCE_COLORS,
  clampProgress,
  isBadgeVariant,
  isSourceColor,
} from "../src/guards.js";

test("source color guard accepts only the closed source-agent palette", () => {
  deepEqual(SOURCE_COLORS, ["amber", "blue", "green", "copper", "gray", "teal", "violet", "peach"]);
  equal(isSourceColor("teal"), true);
  equal(isSourceColor("red"), false);
});

test("badge variant guard accepts only HAX badge variants", () => {
  deepEqual(BADGE_VARIANTS, ["needs-you", "review", "handled", "ceremony", "anchor"]);
  equal(isBadgeVariant("ceremony"), true);
  equal(isBadgeVariant("urgent"), false);
});

test("hold progress clamps to the legal ring range", () => {
  equal(clampProgress(-0.4), 0);
  equal(clampProgress(0.42), 0.42);
  equal(clampProgress(2), 1);
  equal(clampProgress(Number.NaN), 0);
});

test("motion presets match the binding spring values", () => {
  deepEqual(MOTION_PRESETS["card-reveal"], { stiffness: 220, damping: 28, mass: 1 });
  deepEqual(MOTION_PRESETS["trust-grow"], { stiffness: 180, damping: 22, mass: 1 });
  deepEqual(MOTION_PRESETS["omnibar-summon"], { stiffness: 260, damping: 30, mass: 0.9 });
  deepEqual(MOTION_PRESETS["ceremony-begin"], { stiffness: 120, damping: 20, mass: 1.2 });
});
