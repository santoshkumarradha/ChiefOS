declare module "node:assert/strict" {
  export function equal(actual: unknown, expected: unknown, message?: string): void;
  export function deepEqual(actual: unknown, expected: unknown, message?: string): void;
  export function ok(value: unknown, message?: string): void;
}

declare module "node:fs" {
  export function readFileSync(path: string, encoding: BufferEncoding): string;
}

declare module "node:test" {
  export function test(name: string, fn: () => void | Promise<void>): void;
}

declare type BufferEncoding = "utf8" | "utf-8";

declare const process: {
  cwd(): string;
};
