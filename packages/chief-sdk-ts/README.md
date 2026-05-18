# @chief-os/sdk

TypeScript bindings for Chief OS pack and app authors.

## Accessors

| Accessor | Since | Use for |
| --- | --- | --- |
| `ctx.ai()` | 0.2 | single-shot structured inference |
| `ctx.harness()` | 0.2 | multi-turn tool-using agents |
| `ctx.fs()` | 0.3.1 | grant-scoped filesystem (watch / read / list) |
| `new ChiefApp()` | 0.3.1 | headless app access to Work Objects, Chief-mediated AI, Ceremony |

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
