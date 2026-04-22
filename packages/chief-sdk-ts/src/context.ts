/**
 * CapabilityContext — the ONLY object through which a pack touches the kernel.
 */

import { AiBuilder, InMemoryAiBackend } from "./ai.js";
import { HarnessBuilder, InMemoryHarnessBackend, type HarnessToolScope } from "./harness.js";
import type { Tier } from "./tier.js";
import type { ToolHandle } from "./tool_handle.js";
import type { CapabilityKind } from "./capability.js";

/**
 * The capability context passed to all pack code.
 * This is the gate: every I/O must go through here, and the broker enforces grants.
 */
export class CapabilityContext {
  /**
   * Create a single-shot structured LLM call.
   */
  ai(): AiBuilder {
    return new AiBuilder(new InMemoryAiBackend());
  }

  /**
   * Create a multi-turn harness session with tools.
   */
  harness(): HarnessBuilder {
    return new HarnessBuilder(new InMemoryHarnessBackend());
  }

  /**
   * Create a tool handle for network access (scoped to specific hosts).
   */
  netTool(hosts: string[]): ToolHandle {
    const capKind: CapabilityKind = {
      kind: "net.http",
      hosts,
      methods: ["GET", "POST"],
    };
    return createToolHandle(capKind);
  }

  /**
   * Create a tool handle for memory access (scoped to specific types).
   */
  memoryTool(types: string[]): ToolHandle {
    const capKind: CapabilityKind = {
      kind: "mem.read",
      types,
      horizon: "7d",
    };
    return createToolHandle(capKind);
  }

  /**
   * Create a tool handle for filesystem access (scoped to specific paths).
   */
  fsTool(paths: string[]): ToolHandle {
    const capKind: CapabilityKind = {
      kind: "fs.read",
      paths,
    };
    return createToolHandle(capKind);
  }

  /**
   * Create a tool handle for nested AI calls (e.g., for meta-prompting).
   */
  aiTool(tier: Tier): ToolHandle {
    const capKind: CapabilityKind = {
      kind: "llm.ai",
      maxTokens: 1024,
      tier,
    };
    return createToolHandle(capKind);
  }

  /**
   * Create a tool handle for nested harness calls (meta-prompting).
   */
  harnessTool(scope: HarnessToolScope): ToolHandle {
    const capKind: CapabilityKind = {
      kind: "llm.harness",
      maxTurns: scope.maxTurns,
      maxCostUsd: 0.5,
      maxWallSecs: 60,
      tier: "deep",
    };
    return createToolHandle(capKind);
  }
}

/**
 * Internal helper: create a branded ToolHandle (only callable by context methods).
 */
function createToolHandle(kind: CapabilityKind): ToolHandle {
  return {
    id: randomId(),
    kind,
  } as ToolHandle;
}

/**
 * Generate a random opaque ID for a tool handle.
 */
function randomId(): string {
  const buf = new Uint8Array(16);
  if (typeof window !== "undefined" && window.crypto) {
    window.crypto.getRandomValues(buf);
  } else {
    for (let i = 0; i < buf.length; i++) {
      buf[i] = Math.floor(Math.random() * 256);
    }
  }
  return Array.from(buf)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
