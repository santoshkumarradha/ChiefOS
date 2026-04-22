import type { Tier } from "./tier.js";

export interface AiRequest {
  readonly prompt: string;
  readonly input: unknown;
  readonly schemaName: string;
  readonly tier: Tier;
  readonly maxTokens: number;
}

export interface AiBackend {
  call<T>(request: AiRequest): Promise<T>;
  stream<T>(request: AiRequest): AsyncIterable<T>;
}

export class InMemoryAiBackend implements AiBackend {
  readonly #output: unknown;

  constructor(output: unknown = "test response") {
    this.#output = output;
  }

  async call<T>(_request: AiRequest): Promise<T> {
    return this.#output as T;
  }

  async *stream<T>(_request: AiRequest): AsyncIterable<T> {
    yield this.#output as T;
  }
}

export class AiBuilder<T = unknown> {
  readonly #backend: AiBackend;
  #prompt?: string;
  #input: unknown;
  #schemaName?: string;
  #tier: Tier;
  #maxTokens: number;

  constructor(backend: AiBackend = new InMemoryAiBackend()) {
    this.#backend = backend;
    this.#tier = "fast";
    this.#maxTokens = 1024;
  }

  prompt(p: string): this {
    this.#prompt = p;
    return this;
  }

  input<I>(v: I): this {
    this.#input = v;
    return this;
  }

  schema<Schema>(): AiBuilder<Schema> {
    this.#schemaName = "schema";
    return this as unknown as AiBuilder<Schema>;
  }

  tier(t: Tier): this {
    this.#tier = t;
    return this;
  }

  maxTokens(n: number): this {
    this.#maxTokens = n;
    return this;
  }

  call(): Promise<T> {
    return this.#backend.call<T>(this.#request());
  }

  stream(): AsyncIterable<T> {
    return this.#backend.stream<T>(this.#request());
  }

  #request(): AiRequest {
    return {
      prompt: this.#prompt ?? "",
      input: this.#input ?? null,
      schemaName: this.#schemaName ?? "",
      tier: this.#tier,
      maxTokens: this.#maxTokens,
    };
  }

}
