/**
 * Basic runtime surface tests.
 */

import { describe, it, expect } from "vitest";
import { CapabilityContext, AiBuilder, HarnessBuilder } from "../src/index.js";

describe("Basic SDK surface", () => {
  it("should create AI builders", () => {
    const ctx = new CapabilityContext();
    const builder = ctx.ai();
    expect(builder).toBeInstanceOf(AiBuilder);
  });

  it("should create harness builders", () => {
    const ctx = new CapabilityContext();
    const builder = ctx.harness();
    expect(builder).toBeInstanceOf(HarnessBuilder);
  });

  it("should create tool handles", () => {
    const ctx = new CapabilityContext();
    const netTool = ctx.netTool(["example.com"]);
    const memTool = ctx.memoryTool(["notes"]);
    expect(netTool).toBeDefined();
    expect(memTool).toBeDefined();
  });

  it("should support fluent AI API", async () => {
    const ctx = new CapabilityContext();
    const result = await ctx.ai()
      .prompt("test")
      .tier("fast")
      .maxTokens(500)
      .call();
    expect(result).toBeTruthy();
  });

  it("should support fluent harness API", async () => {
    const ctx = new CapabilityContext();
    const result = await ctx.harness()
      .goal("test")
      .tier("deep")
      .maxTurns(5)
      .run();
    expect(result).not.toBeNull();
  });
});
