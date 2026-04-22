/**
 * CapabilityContext — the ONLY object through which a pack touches the kernel.
 *
 * Mirrors `crates/chief-sdk/src/context.rs`. Any change to the Rust SDK surface
 * MUST land in the same PR as a matching change here (version lockstep).
 */

import { AiBuilder, InMemoryAiBackend } from "./ai.js";
import { InMemoryFsConnector, type FsConnector } from "./fs.js";
import {
  HarnessBuilder,
  InMemoryHarnessBackend,
  type HarnessToolScope,
} from "./harness.js";
import { createToolHandle, type ToolHandle } from "./tool_handle.js";
import type { CapabilityKind } from "./capability.js";
import type { Tier } from "./tier.js";

/**
 * The capability context passed to all pack code.
 * This is the gate: every I/O must go through here, and the broker enforces grants.
 */
export class CapabilityContext {
  readonly #fs: FsConnector;

  /**
   * @param overrides Optional connector overrides. Currently only `fs` is
   *   injectable; the rest are wired by `chief-core` in production and stubbed
   *   in tests via the `InMemory*Backend` classes the builders pick up by
   *   default.
   */
  constructor(overrides: { readonly fs?: FsConnector } = {}) {
    // Default fs: empty-allowlist stub — accessor returns it, but every
    // read/watch/list resolves to a `grant_denied` FsError. This keeps the
    // zero-arg `new CapabilityContext()` backwards-compatible across 0.3.0 → 0.3.1.
    this.#fs = overrides.fs ?? new InMemoryFsConnector([]);
  }

  /**
   * Access the grant-scoped filesystem connector.
   *
   * Every read/watch/list call is checked against the pack's `fs.read` /
   * `fs.watch` grants by the Capability Broker (or the in-memory allowlist,
   * under tests).
   *
   * @example
   * ```ts
   * const handle  = await ctx.fs().watch(["/home/user/notes"]);
   * const bytes   = await ctx.fs().read("/home/user/notes/today.md");
   * const entries = await ctx.fs().list("/home/user/notes");
   * ```
   */
  fs(): FsConnector {
    return this.#fs;
  }

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
  netTool(_hosts: readonly string[]): ToolHandle {
    const kind: CapabilityKind = "net.http";
    return createToolHandle(kind);
  }

  /**
   * Create a tool handle for memory access (scoped to specific types).
   */
  memoryTool(_types: readonly string[]): ToolHandle {
    const kind: CapabilityKind = "mem.read";
    return createToolHandle(kind);
  }

  /**
   * Create a tool handle for filesystem access (scoped to specific paths).
   */
  fsTool(_paths: readonly string[]): ToolHandle {
    const kind: CapabilityKind = "fs.read";
    return createToolHandle(kind);
  }

  /**
   * Create a tool handle for nested AI calls (e.g., for meta-prompting).
   */
  aiTool(_tier: Tier): ToolHandle {
    const kind: CapabilityKind = "llm.ai";
    return createToolHandle(kind);
  }

  /**
   * Create a tool handle for nested harness calls (meta-prompting).
   */
  harnessTool(_scope: HarnessToolScope): ToolHandle {
    const kind: CapabilityKind = "llm.harness";
    return createToolHandle(kind);
  }
}
