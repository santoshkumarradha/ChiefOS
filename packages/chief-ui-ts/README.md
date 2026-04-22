# @chief-os/ui

Chief UI v0 is the locked React primitive kit for Chief OS surfaces. Import
`@chief-os/ui/tokens.css` once in the Tauri host, then compose only primitives
from the public entry point.

```tsx
import "@chief-os/ui/tokens.css";
import { Badge, Box, CommandPill, DotTrail, Pane, Row, SectionHeader, SourceAvatar, TabBar, Widget } from "@chief-os/ui";
export function MorningBrief() {
  const chips = [{ id: "needs-you", label: "Needs you", count: 2 }];
  return (
    <Pane variant="content" tint="warm">
      <TabBar chips={chips} selected="needs-you" onSelect={() => {}} />
      <SectionHeader>TODAY / MORNING</SectionHeader>
      <Row
        avatar={<SourceAvatar color="amber" />}
        sender="Calendar Agent"
        subject="Dentist moved to 3pm"
        snippet="Rescheduled by Dr. Park's office"
        badge={<Badge variant="needs-you">Needs you</Badge>}
        timestamp="08:03"
      />
      <Widget title="TRUST">
        <DotTrail label="Calendar" filled={4} total={5} color="amber" />
        <DotTrail label="Email" filled={3} total={5} color="green" />
      </Widget>
      <Box flex={{ gap: "12px" }}>
        <Badge variant="review">Review</Badge>
        <Badge variant="handled">Handled</Badge>
      </Box>
      <CommandPill
        actions={[{ id: "chat", label: "Chat", onSelect: () => {} }]}
      />
    </Pane>
  );
}
```
