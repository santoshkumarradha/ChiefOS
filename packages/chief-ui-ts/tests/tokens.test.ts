import { equal, ok } from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const css = readFileSync(`${process.cwd()}/src/tokens.css`, "utf8");

test("token snapshot contains the binding color values", () => {
  const expected = [
    "--chief-bg-0: #0c0e11;",
    "--chief-bg-1: #14171b;",
    "--chief-bg-2: #1b1f24;",
    "--chief-fg-primary: #e8ecf1;",
    "--chief-fg-secondary: #98a2b0;",
    "--chief-fg-tertiary: #5c6470;",
    "--chief-amber: #d4a574;",
    "--chief-blue: #5294e2;",
    "--chief-green: #73c991;",
    "--chief-copper: #c68858;",
    "--chief-gray: #8a8f99;",
    "--chief-src-teal: #8ec0b8;",
    "--chief-src-violet: #9d89b8;",
    "--chief-src-peach: #e0b090;",
  ];

  for (const token of expected) {
    ok(css.includes(token), token);
  }
});

test("public entry does not mention Radix or shadcn", () => {
  const index = readFileSync(`${process.cwd()}/src/index.ts`, "utf8");
  equal(index.includes("radix"), false);
  equal(index.includes("shadcn"), false);
});

test("font faces are package-local and runtime network-free", () => {
  ok(css.includes("url(\"../assets/fonts/InterTight-Regular.ttf\")"));
  ok(css.includes("url(\"../assets/fonts/Inter-Regular.ttf\")"));
  ok(css.includes("url(\"../assets/fonts/JetBrainsMono-Regular.ttf\")"));
  equal(css.includes("fonts.googleapis.com"), false);
});
