/**
 * Filesystem connector via `ctx.fs()` (ADR-0013 continuation).
 *
 * TypeScript mirror of `crates/chief-sdk/src/fs.rs`. Any change to the Rust
 * trait MUST land in the same PR as a matching change here (version lockstep).
 *
 * Closes the SDK gap flagged by the file-watcher briefer pack (PR #42):
 * `CapabilityKind::Fs*` variants existed but there was no accessor for packs
 * to actually use them. Before this, packs worked around the gap with
 * `ctx.ai()` prompt hacks; now they get a typed, grant-scoped connector.
 */

import type { CapabilityKind } from "./capability.js";
import { createToolHandle, type ToolHandle } from "./tool_handle.js";

/**
 * Errors from `ctx.fs()` operations.
 *
 * Discriminated union — mirrors Rust `FsError` variants exactly. Pattern the
 * same as {@link import("./errors.js").AiError}.
 */
export type FsError =
  | {
      readonly kind: "grant_denied";
      readonly path: string;
      readonly reason: string;
    }
  | {
      readonly kind: "not_found";
      readonly path: string;
    }
  | {
      readonly kind: "io";
      readonly message: string;
    };

/** Type guard for {@link FsError}. */
export function isFsError(err: unknown): err is FsError {
  return (
    typeof err === "object" &&
    err !== null &&
    "kind" in err &&
    typeof (err as Record<string, unknown>).kind === "string" &&
    ["grant_denied", "not_found", "io"].includes(
      (err as Record<string, unknown>).kind as string,
    )
  );
}

/**
 * Class wrapper so {@link FsError} can be `throw`n (plain object unions can't
 * extend `Error`). Callers may either `throw new FsErrorException(err)` or
 * reject promises with the raw `FsError`; both are supported.
 */
export class FsErrorException extends Error {
  readonly error: FsError;
  constructor(error: FsError) {
    super(formatFsError(error));
    this.error = error;
    this.name = "FsErrorException";
  }
}

function formatFsError(err: FsError): string {
  switch (err.kind) {
    case "grant_denied":
      return `grant denied for ${err.path}: ${err.reason}`;
    case "not_found":
      return `not found: ${err.path}`;
    case "io":
      return `io error: ${err.message}`;
  }
}

/**
 * Narrow filesystem facade used by packs.
 *
 * All paths passed to methods on this interface must be covered by the pack's
 * `fs.read` / `fs.watch` grants. The real `chief-core` implementation enforces
 * that via the Capability Broker; the in-memory stub enforces it via a simple
 * allowlist (see {@link InMemoryFsConnector}).
 */
export interface FsConnector {
  /**
   * Subscribe to file-system events on `paths`.
   *
   * Returns a {@link ToolHandle} tagged with `fs.watch`. The handle is
   * session-bound; packs pass it to `ctx.harness().tools([h])` or let it
   * fall out of scope to stop watching.
   *
   * Rejects with {@link FsError} (kind `grant_denied`) if any path falls
   * outside the allowlist.
   */
  watch(paths: readonly string[]): Promise<ToolHandle>;

  /**
   * Read a file's bytes.
   *
   * Rejects with {@link FsError} (`grant_denied` | `not_found` | `io`).
   */
  read(path: string): Promise<Uint8Array>;

  /**
   * Enumerate entries in a directory (non-recursive).
   *
   * Rejects with {@link FsError} (`grant_denied` | `io`).
   */
  list(dir: string): Promise<readonly string[]>;
}

/**
 * In-memory stub for tests and examples.
 *
 * Stores `(path, bytes)` pairs in memory and enforces a simple allowlist: a
 * path is accessible iff it is equal to or is a descendant of (i.e. starts
 * with `"{allowed}/"`) an entry in `allowed`.
 *
 * Mirrors {@link import("chief_sdk::fs::InMemoryFsConnector")} from Rust.
 */
export class InMemoryFsConnector implements FsConnector {
  readonly #allowed: readonly string[];
  readonly #files: Map<string, Uint8Array>;

  /**
   * @param allowed Paths this stub is allowed to touch. Empty denies everything.
   */
  constructor(allowed: readonly string[] = []) {
    this.#allowed = [...allowed];
    this.#files = new Map();
  }

  /**
   * Test helper: populate the stub with a file's bytes.
   *
   * Does NOT check the allowlist — the allowlist is enforced on
   * read/watch/list, not on ingestion (mirrors real FS semantics).
   */
  put(path: string, bytes: Uint8Array): void {
    this.#files.set(path, bytes);
  }

  async watch(paths: readonly string[]): Promise<ToolHandle> {
    for (const p of paths) {
      this.#assertAllowed(p);
    }
    const kind: CapabilityKind = "fs.watch";
    return createToolHandle(kind);
  }

  async read(path: string): Promise<Uint8Array> {
    this.#assertAllowed(path);
    const bytes = this.#files.get(path);
    if (bytes === undefined) {
      throw new FsErrorException({ kind: "not_found", path });
    }
    return bytes;
  }

  async list(dir: string): Promise<readonly string[]> {
    this.#assertAllowed(dir);
    const prefix = dir.endsWith("/") ? dir : `${dir}/`;
    const entries: string[] = [];
    for (const p of this.#files.keys()) {
      if (!p.startsWith(prefix)) continue;
      const rest = p.slice(prefix.length);
      // Non-recursive: skip nested paths.
      if (rest.includes("/")) continue;
      entries.push(p);
    }
    entries.sort();
    return entries;
  }

  #assertAllowed(path: string): void {
    const ok = this.#allowed.some(
      (a) => path === a || path.startsWith(`${a}/`),
    );
    if (!ok) {
      throw new FsErrorException({
        kind: "grant_denied",
        path,
        reason: "path not covered by fs grant allowlist",
      });
    }
  }
}
