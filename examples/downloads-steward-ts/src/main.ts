import {
  ChiefApp,
  type FsEntry,
  type FsMoveOperation,
  type FsReceipt,
} from "@chief-os/sdk";

interface CleanupPlan {
  summary: string;
  operations: FsMoveOperation[];
  review: string[];
}

interface RunResult {
  app: string;
  work_object_id: string;
  root: string;
  scanned_files: number;
  llm: {
    provider: string;
    model: string;
  };
  plan: CleanupPlan;
  ceremony: {
    id: string;
    payload_hash?: string;
    status: string;
  };
  applied?: {
    count: number;
    receipt: FsReceipt;
  };
  rewind?: {
    count: number;
  };
}

const principal = process.env.CHIEF_APP_PRINCIPAL ?? "app:downloads-steward";
const baseUrl = process.env.CHIEF_BASE_URL ?? "http://127.0.0.1:18089";
const root = requiredEnv("CHIEF_DOWNLOADS_ROOT");
const workId = process.env.CHIEF_WORK_ID ?? "downloads-steward-demo";
const mode = process.env.CHIEF_DOWNLOADS_MODE ?? "plan";

const chief = new ChiefApp({ baseUrl, principal });

async function main(): Promise<void> {
  await ensureWorkObject();
  const scan = await chief.fs.scan({ root, maxEntries: 100 });
  const plan = await planCleanup(scan.entries);
  const manifest = { root: scan.root, operations: plan.operations };

  const contribution = await chief.work(workId).contribute({
    node_type: "artifact",
    kind: "downloads_cleanup_plan",
    title: "Downloads cleanup plan",
    summary: plan.summary,
    body: {
      root: scan.root,
      scanned_files: scan.entries.length,
      plan,
      proposed_action: "fs.move_manifest",
      chief_app: {
        principal,
        sdk: "@chief-os/sdk",
        runtime: "typescript",
      },
    },
    ceremony: {
      title: "Apply Downloads cleanup",
      summary: `Move ${plan.operations.length} files inside the selected folder.`,
      category: "fs.write",
      payload: manifest,
      trust_context: 3,
    },
  });

  const ceremony = contribution.ceremony;
  if (!ceremony) {
    throw new Error("Chief did not create a Ceremony for the cleanup plan");
  }

  const result: RunResult = {
    app: principal,
    work_object_id: workId,
    root: scan.root,
    scanned_files: scan.entries.length,
    llm: lastLlm,
    plan,
    ceremony: {
      id: ceremony.id,
      payload_hash: ceremony.payload_hash,
      status: ceremony.status,
    },
  };

  if (mode === "apply") {
    const approved = await chief.ceremony.approve(ceremony.id, {
      heldMs: 3000,
      payloadHash: requiredString(ceremony.payload_hash, "payload_hash"),
    });
    void approved;
    const applied = await chief.fs.apply({
      root: scan.root,
      operations: plan.operations,
      ceremonyId: ceremony.id,
    });
    result.applied = {
      count: applied.applied.length,
      receipt: applied.receipt,
    };

    if (process.env.CHIEF_DOWNLOADS_REWIND === "1") {
      const rewind = await chief.fs.rewind(applied.receipt);
      result.rewind = { count: rewind.rewound.length };
    }
  }

  console.log(JSON.stringify(result, null, 2));
}

let lastLlm: RunResult["llm"] = {
  provider: "unknown",
  model: "unknown",
};

async function ensureWorkObject(): Promise<void> {
  try {
    await chief.work(workId).get();
  } catch {
    await chief.createWork({
      id: workId,
      title: "Downloads Steward Demo",
      summary: "A real local folder cleanup work object.",
      body: {
        kind: "downloads_steward",
        root,
      },
    });
  }
}

async function planCleanup(entries: FsEntry[]): Promise<CleanupPlan> {
  const response = await chief.ai.generate({
    tier: "fast",
    temperature: 0.1,
    maxTokens: 512,
    schemaName: "DownloadsCleanupPlan",
    prompt: [
      "You are a Chief OS Downloads Steward app.",
      "Return only valid JSON with keys summary, operations, review.",
      "operations must be an array of move operations: {\"from\":\"relative/path\",\"to\":\"relative/path\"}.",
      "Never delete files. Only move files into these folders: Finance/Receipts, Finance/Statements, Work/Contracts, Images/Screenshots, Review/Unknown.",
      "Use only relative paths from the input. Do not invent files.",
    ].join("\n"),
    input: {
      files: entries.map((entry) => ({
        relative_path: entry.relative_path,
        extension: entry.extension,
        size_bytes: entry.size_bytes,
        preview: entry.preview,
      })),
    },
  });

  lastLlm = {
    provider: response.provider,
    model: response.model,
  };

  return sanitizePlan(response.json ?? parseJson(response.text), entries);
}

function sanitizePlan(value: unknown, entries: FsEntry[]): CleanupPlan {
  const existing = new Set(entries.map((entry) => entry.relative_path));
  const obj = typeof value === "object" && value !== null ? (value as Record<string, unknown>) : {};
  const rawOps = Array.isArray(obj.operations) ? obj.operations : [];
  const operations: FsMoveOperation[] = [];

  for (const item of rawOps) {
    if (typeof item !== "object" || item === null) continue;
    const op = item as Record<string, unknown>;
    const from = typeof op.from === "string" ? op.from : "";
    const to = typeof op.to === "string" ? op.to : "";
    if (!existing.has(from) || !safeRelative(to) || to === from) continue;
    operations.push({ from, to });
  }

  for (const entry of entries) {
    if (operations.some((op) => op.from === entry.relative_path)) continue;
    const to = fallbackDestination(entry);
    if (to !== entry.relative_path) {
      operations.push({ from: entry.relative_path, to });
    }
  }

  return {
    summary:
      typeof obj.summary === "string" && obj.summary.trim().length > 0
        ? obj.summary
        : `Move ${operations.length} files into reviewable folders without deleting anything.`,
    operations: operations.slice(0, 25),
    review: stringArray(obj.review, ["No files are deleted; unknown files go to Review/Unknown."]),
  };
}

function fallbackDestination(entry: FsEntry): string {
  const name = entry.relative_path.split("/").pop() ?? entry.relative_path;
  const lower = entry.relative_path.toLowerCase();
  if (lower.includes("receipt")) return `Finance/Receipts/${name}`;
  if (lower.includes("statement") || lower.includes("bank")) return `Finance/Statements/${name}`;
  if (lower.includes("contract") || lower.includes("agreement")) return `Work/Contracts/${name}`;
  if (lower.includes("screenshot") || lower.endsWith(".png") || lower.endsWith(".jpg")) {
    return `Images/Screenshots/${name}`;
  }
  return `Review/Unknown/${name}`;
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
    return {};
  }
}

function safeRelative(value: string): boolean {
  return value.length > 0 && !value.startsWith("/") && !value.includes("..");
}

function stringArray(value: unknown, fallback: string[]): string[] {
  if (!Array.isArray(value)) return fallback;
  const out = value.filter((item): item is string => typeof item === "string" && item.length > 0);
  return out.length > 0 ? out : fallback;
}

function requiredEnv(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
}

function requiredString(value: unknown, name: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${name} is required`);
  }
  return value;
}

main().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
