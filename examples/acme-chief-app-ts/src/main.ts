import { ChiefApp, type WorkObject } from "@chief-os/sdk";

interface FollowupPlan {
  selected_agents: string[];
  steps: string[];
  risk_notes: string[];
  recommendation: string;
  email: {
    to: string;
    subject: string;
    body: string;
  };
}

interface AppResult {
  app: string;
  work_object_id: string;
  llm: {
    provider: string;
    model: string;
    attestation_id: unknown;
  };
  selected_agents: string[];
  contribution_uri: string;
  ceremony?: {
    id: string;
    payload_hash?: string;
    status: string;
  };
  output: unknown;
}

const principal = process.env.CHIEF_APP_PRINCIPAL ?? "app:acme-chief-ts";
const baseUrl = process.env.CHIEF_BASE_URL ?? "http://127.0.0.1:18088";
const workId = process.env.CHIEF_WORK_ID ?? "acme-follow-up";

const app = new ChiefApp({ baseUrl, principal });

async function main(): Promise<void> {
  const work = await app.work(workId).get();
  const plan = await planFollowup(work);
  const sourceRefs = work.source_refs.map((ref) => ref.uri).filter(Boolean).slice(0, 4);

  const contribution = await app.work(workId).contribute({
    node_type: "artifact",
    kind: "chief_app_followup_plan",
    title: "Acme follow-up plan from SDK app",
    summary: plan.recommendation,
    source_refs: sourceRefs,
    body: {
      selected_agents: plan.selected_agents,
      steps: plan.steps,
      risk_notes: plan.risk_notes,
      recommendation: plan.recommendation,
      proposed_action: "email.send",
      payload: plan.email,
      chief_app: {
        principal,
        sdk: "@chief-os/sdk",
        runtime: "typescript",
      },
    },
    ceremony: {
      title: "Send Acme follow-up from TypeScript Chief app",
      summary: "acme-chief-ts asks Chief Ceremony to approve the exact outbound email payload.",
      category: "email.send",
      payload: plan.email,
      trust_context: 3,
    },
  });

  const result: AppResult = {
    app: principal,
    work_object_id: workId,
    llm: {
      provider: lastLlm.provider,
      model: lastLlm.model,
      attestation_id: lastLlm.attestation_id,
    },
    selected_agents: plan.selected_agents,
    contribution_uri: contribution.contribution.uri,
    ceremony: contribution.ceremony
      ? {
          id: contribution.ceremony.id,
          payload_hash: contribution.ceremony.payload_hash,
          status: contribution.ceremony.status,
        }
      : undefined,
    output: contribution,
  };

  console.log(JSON.stringify(result, null, 2));
}

let lastLlm: AppResult["llm"] = {
  provider: "unknown",
  model: "unknown",
  attestation_id: undefined,
};

async function planFollowup(work: WorkObject): Promise<FollowupPlan> {
  const response = await app.ai.generate({
    tier: "fast",
    temperature: 0.1,
    maxTokens: 512,
    schemaName: "FollowupPlan",
    prompt: [
      "You are a small user-built Chief OS app.",
      "Chief OS is the substrate; this app owns planning and agent selection.",
      "Return only valid JSON with keys selected_agents, steps, risk_notes, recommendation, email.",
      "The email must be a draft that still needs human Ceremony before external send.",
      "Use these exact candidate agent names when useful:",
      "contract-context-agent, calendar-window-agent, reply-drafter-agent, risk-checker-agent.",
    ].join("\n"),
    input: {
      work_object: {
        id: work.id,
        title: work.title,
        source_refs: work.source_refs.map((ref) => ({
          source: ref.source,
          summary: ref.summary.slice(0, 480),
        })),
        existing_contributions: work.contributions.map((contribution) => ({
          source: contribution.source,
          kind: contribution.kind,
          summary: contribution.summary.slice(0, 360),
        })),
      },
    },
  });

  lastLlm = {
    provider: response.provider,
    model: response.model,
    attestation_id:
      typeof response.attestation === "object" && response.attestation !== null
        ? (response.attestation as Record<string, unknown>).id
        : undefined,
  };

  return normalizePlan(response.json ?? parseJson(response.text));
}

function parseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    const start = text.indexOf("{");
    const end = text.lastIndexOf("}");
    if (start >= 0 && end > start) {
      return JSON.parse(text.slice(start, end + 1));
    }
    throw new Error(`LLM did not return JSON: ${text.slice(0, 240)}`);
  }
}

function normalizePlan(value: unknown): FollowupPlan {
  if (typeof value !== "object" || value === null) {
    throw new Error("FollowupPlan must be an object");
  }
  const obj = value as Record<string, unknown>;
  const email = typeof obj.email === "object" && obj.email !== null
    ? (obj.email as Record<string, unknown>)
    : {};

  return {
    selected_agents: stringArray(obj.selected_agents ?? obj.selectedAgents, [
      "contract-context-agent",
      "reply-drafter-agent",
      "risk-checker-agent",
    ]),
    steps: stringArray(obj.steps ?? obj.plan, [
      "Read the work object sources.",
      "Draft a bounded follow-up.",
      "Ask Chief Ceremony before send.",
    ]),
    risk_notes: stringArray(obj.risk_notes ?? obj.riskNotes, [
      "External email send remains behind Ceremony.",
    ]),
    recommendation: stringValue(
      obj.recommendation,
      "Send a concise follow-up after human Ceremony confirms the exact payload.",
    ),
    email: {
      to: stringValue(email.to, "maya@acme.example"),
      subject: stringValue(email.subject, "Acme follow-up"),
      body: stringValue(
        email.body,
        "Thanks for the discussion. I want to confirm clause 4 and the Tuesday follow-up window before we finalize the next step.",
      ),
    },
  };
}

function stringArray(value: unknown, fallback: string[]): string[] {
  if (!Array.isArray(value)) {
    return fallback;
  }
  const out = value.filter((item): item is string => typeof item === "string" && item.length > 0);
  return out.length > 0 ? out : fallback;
}

function stringValue(value: unknown, fallback: string): string {
  return typeof value === "string" && value.trim().length > 0 ? value : fallback;
}

main().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
