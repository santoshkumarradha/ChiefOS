export interface ChiefAppOptions {
  baseUrl?: string;
  principal: string;
  fetchImpl?: typeof fetch;
}

export interface WorkObject {
  id: string;
  uri: string;
  title: string;
  source_refs: WorkSourceRef[];
  contributions: WorkContribution[];
  provenance: unknown[];
}

export interface WorkSourceRef {
  uri: string;
  node_type: string;
  source: string;
  summary: string;
}

export interface WorkContribution {
  uri: string;
  node_type: string;
  source: string;
  pack: string;
  kind: string;
  title: string;
  summary: string;
  authority_state: string;
  source_refs: string[];
  body: Record<string, unknown>;
}

export interface AiGenerateRequest {
  prompt: string;
  input?: unknown;
  schemaName?: string;
  tier?: string;
  maxTokens?: number;
  temperature?: number;
  topP?: number;
}

export interface AiGenerateResponse {
  text: string;
  json?: unknown;
  provider: string;
  model: string;
  tier: string;
  max_tokens?: number;
  attestation: unknown;
}

export interface ContributionWriteRequest {
  node_type?: "finding" | "artifact" | "decision";
  kind: string;
  title?: string;
  summary?: string;
  authority_state?: string;
  source_refs?: string[];
  body?: unknown;
  ceremony?: ContributionCeremonyRequest;
}

export interface ContributionCeremonyRequest {
  title: string;
  summary: string;
  category: string;
  payload: unknown;
  trust_context?: number;
  target_principal?: string;
}

export interface ContributionWriteResponse {
  work_object_id: string;
  contribution: WorkContribution;
  ceremony?: CeremonyItem;
}

export interface CeremonyItem {
  id: string;
  title: string;
  source_agent: string;
  status: string;
  payload_hash?: string;
  target_principal: string;
  [key: string]: unknown;
}

export interface ApproveCeremonyRequest {
  heldMs: number;
  payloadHash: string;
}

export interface RewindRequest {
  contributionUri: string;
  reason?: string;
}

export interface RewindResponse {
  rewound: boolean;
  work_object_id: string;
  contribution_uri: string;
  event_id: string;
  stale_marked: number;
}

export class ChiefApiError extends Error {
  readonly status: number;
  readonly body: unknown;

  constructor(message: string, status: number, body: unknown) {
    super(message);
    this.name = "ChiefApiError";
    this.status = status;
    this.body = body;
  }
}

export class ChiefApp {
  readonly baseUrl: string;
  readonly principal: string;
  private readonly fetchImpl: typeof fetch;

  constructor(options: ChiefAppOptions) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:4711").replace(/\/$/, "");
    this.principal = options.principal;
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  work(id: string): WorkClient {
    return new WorkClient(this, id);
  }

  get ai(): AiClient {
    return new AiClient(this);
  }

  get ceremony(): CeremonyClient {
    return new CeremonyClient(this);
  }

  async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
      method,
      headers: {
        accept: "application/json",
        "content-type": "application/json",
        "x-chief-principal": this.principal,
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });

    const text = await response.text();
    const parsed = text.length === 0 ? undefined : JSON.parse(text);
    if (!response.ok) {
      throw new ChiefApiError(
        `${method} ${path} failed with ${response.status}`,
        response.status,
        parsed,
      );
    }
    return parsed as T;
  }
}

export class WorkClient {
  constructor(
    private readonly app: ChiefApp,
    private readonly id: string,
  ) {}

  get(): Promise<WorkObject> {
    return this.app.request<WorkObject>("GET", `/v1/work/${encodeURIComponent(this.id)}`);
  }

  contribute(request: ContributionWriteRequest): Promise<ContributionWriteResponse> {
    return this.app.request<ContributionWriteResponse>(
      "POST",
      `/v1/work/${encodeURIComponent(this.id)}/contributions`,
      request,
    );
  }

  provenance(): Promise<unknown[]> {
    return this.app.request<unknown[]>(
      "GET",
      `/v1/work/${encodeURIComponent(this.id)}/provenance`,
    );
  }

  rewind(request: RewindRequest): Promise<RewindResponse> {
    return this.app.request<RewindResponse>("POST", `/v1/work/${encodeURIComponent(this.id)}/rewind`, {
      contribution_uri: request.contributionUri,
      reason: request.reason,
    });
  }
}

export class AiClient {
  constructor(private readonly app: ChiefApp) {}

  generate(request: AiGenerateRequest): Promise<AiGenerateResponse> {
    return this.app.request<AiGenerateResponse>("POST", "/v1/ai/generate", {
      prompt: request.prompt,
      input: request.input ?? null,
      schema_name: request.schemaName,
      tier: request.tier,
      max_tokens: request.maxTokens,
      temperature: request.temperature,
      top_p: request.topP,
    });
  }
}

export class CeremonyClient {
  constructor(private readonly app: ChiefApp) {}

  list(): Promise<CeremonyItem[]> {
    return this.app.request<CeremonyItem[]>("GET", "/v1/ceremony");
  }

  approve(id: string, request: ApproveCeremonyRequest): Promise<unknown> {
    return this.app.request<unknown>("POST", `/v1/ceremony/${encodeURIComponent(id)}/approve`, {
      held_ms: request.heldMs,
      payload_hash: request.payloadHash,
    });
  }
}
