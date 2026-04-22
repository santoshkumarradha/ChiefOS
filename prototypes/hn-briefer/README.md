# HN + Substack Briefer Pack

First dogfood pack validating the public `@chief-os/sdk` surface.

## Purpose

Every morning, fetch Hacker News top stories and the user's subscribed Substack feeds, score them via on-device LLM, persist top 3-5 as memory nodes, and render one brief card in the Brief surface.

## Grants Declared

All 5 required grants with usage reasons:

1. **net.http** (HN): `Fetch HN top-stories feed and Algolia API`
2. **net.http** (Substack): `Fetch user subscribed Substack feeds`
3. **mem.write**: `Persist briefed items and daily digest card to Memory Graph`
4. **surface.pane**: `Render daily HN + Substack card in Brief surface`
5. **llm.generate**: `Summarize and score stories for relevance filtering`

## Example Card Output

```json
{
  "type": "card",
  "source": "HN Briefer",
  "sender": "HN Briefer",
  "subject": "Daily digest: 5 stories",
  "snippet": "Top 5 items from HN and Substack",
  "badge": "handled",
  "timestamp": "2026-04-21T10:00:00Z",
  "source_color": "amber",
  "items": [
    {
      "source": "HN",
      "item_id": "12345",
      "title": "Show HN: A different approach to building software",
      "url": "https://example.com/1",
      "snippet": "A new paradigm for software development…",
      "score": 87.5
    }
  ]
}
```

## Installation (Nix Flake stub)

```bash
# Real flake integration is a follow-up task.
# For now, build locally:
cargo build -p hn-briefer
cargo test -p hn-briefer
```

## Implementation Notes

- **V0 scope**: Hardcoded Substack URLs; user-configurable feeds are a later task.
- **OAuth**: Not needed — HN and Substack RSS are public.
- **LLM routing**: Uses SDK's `ctx.llm()` stub; real model selection is kernel concern.
- **UI rendering**: Pack emits Card data; Brief consumer renders via chief-ui primitives.
- **Public API discipline**: Consumes ONLY `chief_sdk::*` exports — validates ADR-0010 and ADR-0011.
