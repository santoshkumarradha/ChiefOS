import type { CapabilityKind } from "./capability.js";

declare const ToolHandleBrand: unique symbol;
declare const OpaqueIdBrand: unique symbol;
declare const SessionIdBrand: unique symbol;

export type OpaqueId = string & { readonly [OpaqueIdBrand]: never };
export type SessionId = string & { readonly [SessionIdBrand]: never };

export interface ToolHandle {
  readonly [ToolHandleBrand]: never;
  readonly id: OpaqueId;
  readonly kind: CapabilityKind;
}

export function createToolHandle(kind: CapabilityKind): ToolHandle {
  return {
    id: randomOpaqueId(),
    kind,
  } as ToolHandle;
}

function randomOpaqueId(): OpaqueId {
  const id = Array.from({ length: 16 }, () =>
    Math.floor(Math.random() * 256)
      .toString(16)
      .padStart(2, "0"),
  ).join("");
  return id as OpaqueId;
}
