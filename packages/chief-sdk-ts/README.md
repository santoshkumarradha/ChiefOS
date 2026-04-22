# @chief-os/sdk

TypeScript bindings for Chief OS pack authors.

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
