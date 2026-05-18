# @chief-os/sdk

TypeScript bindings for Chief OS pack and app authors.

## Accessors

| Accessor | Since | Use for |
| --- | --- | --- |
| `ctx.ai()` | 0.2 | single-shot structured inference |
| `ctx.harness()` | 0.2 | multi-turn tool-using agents |
| `ctx.fs()` | 0.3.1 | grant-scoped filesystem (watch / read / list) |
| `new ChiefApp()` | 0.3.1 | headless app access to Work Objects, Chief-mediated AI, Ceremony, filesystem POCs |

Example — `ctx.fs()`:

```ts
const bytes   = await ctx.fs().read("/home/user/notes/today.md");
const entries = await ctx.fs().list("/home/user/notes");
const handle  = await ctx.fs().watch(["/home/user/notes"]);
```

## Hello Pack

```ts
import type { CapabilityContext } from "@chief-os/sdk";

export async function onTick(ctx: CapabilityContext): Promise<void> {
  const label = await ctx
    .ai()
    .prompt("Classify the latest inbox item")
    .input({ subject: "Quarterly renewal", body: "Please review." })
    .schema<string>()
    .tier("fast")
    .maxTokens(200)
    .call();
  const transcript = await ctx
    .harness()
    .goal(`Draft next action for ${label}`)
    .tools([ctx.memoryTool(["thought"]), ctx.aiTool("deep")])
    .maxTurns(4)
    .maxCostUsd(0.25)
    .run();
  console.log(transcript.finalOutput);
}
```

## Hello App

```ts
import { ChiefApp } from "@chief-os/sdk";

const chief = new ChiefApp({
  baseUrl: process.env.CHIEF_BASE_URL,
  principal: "app:acme-chief-ts",
});

const work = await chief.work("acme-follow-up").get();
const plan = await chief.ai.generate({
  prompt: "Return JSON with selected_agents, recommendation, and email.",
  input: { work_object: work },
});

await chief.work("acme-follow-up").contribute({
  node_type: "artifact",
  kind: "chief_app_followup_plan",
  body: plan.json,
  ceremony: {
    title: "Send follow-up",
    summary: "Approve exact outbound email payload.",
    category: "email.send",
    payload: (plan.json as any).email,
  },
});
```

## Filesystem App

```ts
const work = await chief.createWork({
  id: "downloads-steward-demo",
  title: "Downloads Steward Demo",
});
const scan = await chief.fs.scan({ root: "/tmp/Downloads Mess" });
const plan = await chief.ai.generate({
  prompt: "Return JSON move operations. Never delete files.",
  input: { files: scan.entries },
});
const contribution = await chief.work(work.id).contribute({
  node_type: "artifact",
  kind: "downloads_cleanup_plan",
  body: plan.json,
  ceremony: {
    title: "Apply Downloads cleanup",
    summary: "Approve exact file move manifest.",
    category: "fs.write",
    payload: { root: scan.root, operations: (plan.json as any).operations },
  },
});
const approved = await chief.ceremony.approve(contribution.ceremony!.id, {
  heldMs: 3000,
  payloadHash: contribution.ceremony!.payload_hash!,
});
void approved;
const applied = await chief.fs.apply({
  root: scan.root,
  operations: (plan.json as any).operations,
  ceremonyId: contribution.ceremony!.id,
});
await chief.fs.rewind(applied.receipt);
```
