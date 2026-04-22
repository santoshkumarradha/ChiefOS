/**
 * Integration tests for `ctx.fs()` accessor and `InMemoryFsConnector`.
 *
 * Mirrors Rust tests in `crates/chief-sdk/tests/fs_ctx.rs` — any change here
 * must land in both.
 */

import { describe, it, expect } from "vitest";
import {
  CapabilityContext,
  FsErrorException,
  InMemoryFsConnector,
  isFsError,
  type FsConnector,
  type FsError,
} from "../src/index.js";

function makeCtx(fs: FsConnector): CapabilityContext {
  return new CapabilityContext({ fs });
}

describe("ctx.fs() accessor", () => {
  it("returns a connector without panicking", () => {
    const fs = new InMemoryFsConnector(["/tmp"]);
    const ctx = makeCtx(fs);
    expect(ctx.fs()).toBe(fs);
  });

  it("read allowed path succeeds", async () => {
    const stub = new InMemoryFsConnector(["/tmp"]);
    stub.put("/tmp/hello.txt", new TextEncoder().encode("hello world"));
    const ctx = makeCtx(stub);

    const bytes = await ctx.fs().read("/tmp/hello.txt");
    expect(new TextDecoder().decode(bytes)).toBe("hello world");
  });

  it("read denied path errors with grant_denied", async () => {
    const stub = new InMemoryFsConnector(["/tmp"]);
    const ctx = makeCtx(stub);

    await expect(async () => {
      await ctx.fs().read("/etc/passwd");
    }).rejects.toMatchObject({
      error: {
        kind: "grant_denied",
        path: "/etc/passwd",
      },
    });
  });

  it("read missing file surfaces not_found", async () => {
    const stub = new InMemoryFsConnector(["/tmp"]);
    const ctx = makeCtx(stub);

    try {
      await ctx.fs().read("/tmp/missing.txt");
      expect.fail("expected throw");
    } catch (err) {
      expect(err).toBeInstanceOf(FsErrorException);
      const fsErr = (err as FsErrorException).error;
      expect(isFsError(fsErr)).toBe(true);
      expect(fsErr.kind).toBe("not_found");
      if (fsErr.kind === "not_found") {
        expect(fsErr.path).toBe("/tmp/missing.txt");
      }
    }
  });

  it("watch returns a handle tagged fs.watch", async () => {
    const stub = new InMemoryFsConnector(["/work"]);
    const ctx = makeCtx(stub);

    const handle = await ctx.fs().watch(["/work/inbox", "/work/todo"]);
    expect(handle.kind).toBe("fs.watch");
    expect(handle.id).toMatch(/^[0-9a-f]{32}$/);
  });

  it("watch denied path rejects", async () => {
    const stub = new InMemoryFsConnector(["/work"]);
    const ctx = makeCtx(stub);

    await expect(
      ctx.fs().watch(["/work/ok", "/etc/passwd"]),
    ).rejects.toBeInstanceOf(FsErrorException);
  });

  it("list dir returns non-recursive entries", async () => {
    const stub = new InMemoryFsConnector(["/work"]);
    stub.put("/work/a.md", new TextEncoder().encode("a"));
    stub.put("/work/b.md", new TextEncoder().encode("b"));
    stub.put("/work/sub/c.md", new TextEncoder().encode("c"));
    const ctx = makeCtx(stub);

    const entries = await ctx.fs().list("/work");
    expect(entries).toEqual(["/work/a.md", "/work/b.md"]);
  });

  it("list denied path rejects", async () => {
    const stub = new InMemoryFsConnector(["/work"]);
    const ctx = makeCtx(stub);

    await expect(ctx.fs().list("/root")).rejects.toBeInstanceOf(
      FsErrorException,
    );
  });

  it("default ctx with no fs override denies every path", async () => {
    const ctx = new CapabilityContext();
    await expect(ctx.fs().read("/anything")).rejects.toMatchObject({
      error: { kind: "grant_denied" },
    });
  });

  it("FsError is a type guard-friendly discriminated union", () => {
    const denied: FsError = {
      kind: "grant_denied",
      path: "/x",
      reason: "no grant",
    };
    expect(isFsError(denied)).toBe(true);
    expect(isFsError({ kind: "bogus" })).toBe(false);
    expect(isFsError(null)).toBe(false);
  });
});
