import type { Tier } from "./tier.js";
import type { ToolHandle } from "./tool_handle.js";

export interface HarnessRequest {
  readonly goal: string;
  readonly tools: readonly ToolHandle[];
  readonly tier: Tier;
  readonly maxTurns: number;
  readonly maxCostUsd: number;
  readonly maxWallSecs: number;
}

export interface HarnessBackend {
  runSession(request: HarnessRequest): Promise<HarnessTranscript>;
}

export interface HarnessToolScope {
  readonly maxTurns: number;
  readonly toolKinds: readonly string[];
}

export interface HarnessTranscript {
  readonly sessionId: string;
  readonly turns: readonly Turn[];
  readonly finalOutput: unknown;
  readonly costUsd: number;
  readonly wallSecs: number;
  readonly attestation: SignedAttestation;
  readonly children: readonly HarnessTranscript[];
}

export interface Turn {
  readonly index: number;
  readonly role: Role;
  readonly content: TurnContent;
  readonly attestation: SignedAttestation;
  readonly wallMs: number;
  readonly costUsd: number;
}

export type Role = "system" | "user" | "assistant" | "tool";

export type TurnContent =
  | { readonly kind: "text"; readonly text: string }
  | {
      readonly kind: "tool_call";
      readonly callId: string;
      readonly toolName: string;
      readonly arguments: unknown;
    }
  | { readonly kind: "tool_result"; readonly callId: string; readonly result: unknown };

export interface SignedAttestation {
  readonly signer: string;
  readonly signature: string;
  readonly ts: string;
}

export class InMemoryHarnessBackend implements HarnessBackend {
  async runSession(request: HarnessRequest): Promise<HarnessTranscript> {
    const attestation: SignedAttestation = {
      signer: "in-memory-stub",
      signature: "00".repeat(64),
      ts: new Date().toISOString(),
    };

    return {
      sessionId: randomHexId(),
      turns: [
        {
          index: 0,
          role: "user",
          content: { kind: "text", text: request.goal },
          attestation,
          wallMs: 100,
          costUsd: 0.001,
        },
        {
          index: 1,
          role: "assistant",
          content: { kind: "text", text: "Test response" },
          attestation,
          wallMs: 200,
          costUsd: 0.002,
        },
      ],
      finalOutput: { result: "completed" },
      costUsd: 0.003,
      wallSecs: 0.3,
      attestation,
      children: [],
    };
  }
}

export class HarnessBuilder {
  readonly #backend: HarnessBackend;
  #goal?: string;
  #tools: readonly ToolHandle[];
  #tier: Tier;
  #maxTurns: number;
  #maxCostUsd: number;
  #maxWallSecs: number;

  constructor(backend: HarnessBackend = new InMemoryHarnessBackend()) {
    this.#backend = backend;
    this.#tools = [];
    this.#tier = "deep";
    this.#maxTurns = 10;
    this.#maxCostUsd = 1.0;
    this.#maxWallSecs = 120;
  }

  goal(g: string): this {
    this.#goal = g;
    return this;
  }

  tools(tools: readonly ToolHandle[]): this {
    this.#tools = tools;
    return this;
  }

  tier(t: Tier): this {
    this.#tier = t;
    return this;
  }

  maxTurns(n: number): this {
    this.#maxTurns = n;
    return this;
  }

  maxCostUsd(usd: number): this {
    this.#maxCostUsd = usd;
    return this;
  }

  maxWallSecs(s: number): this {
    this.#maxWallSecs = s;
    return this;
  }

  run(): Promise<HarnessTranscript> {
    return this.#backend.runSession({
      goal: this.#goal ?? "",
      tools: this.#tools,
      tier: this.#tier,
      maxTurns: this.#maxTurns,
      maxCostUsd: this.#maxCostUsd,
      maxWallSecs: this.#maxWallSecs,
    });
  }

}

function randomHexId(): string {
  return Array.from({ length: 16 }, () =>
    Math.floor(Math.random() * 256)
      .toString(16)
      .padStart(2, "0"),
  ).join("");
}
