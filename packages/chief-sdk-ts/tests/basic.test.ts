/**
 * Basic runtime surface tests.
 */

import { describe, it, expect } from "vitest";
import { CapabilityContext, AiBuilder, HarnessBuilder, ChiefApp } from "../src/index.js";

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

  it("should call Chief OS app APIs with principal attribution", async () => {
    const calls: Array<{ url: string; init: RequestInit }> = [];
    const fetchImpl = async (url: RequestInfo | URL, init?: RequestInit) => {
      calls.push({ url: String(url), init: init ?? {} });
      return new Response(JSON.stringify({ id: "acme-follow-up", source_refs: [] }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    };

    const app = new ChiefApp({
      baseUrl: "http://chief.test/",
      principal: "app:acme-chief-ts",
      fetchImpl,
    });

    await app.work("acme-follow-up").get();

    expect(calls[0].url).toBe("http://chief.test/v1/work/acme-follow-up");
    expect((calls[0].init.headers as Record<string, string>)["x-chief-principal"]).toBe(
      "app:acme-chief-ts",
    );
  });
});
