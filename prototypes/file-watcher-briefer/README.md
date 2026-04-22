# File-Watcher Briefer

Second dogfood pack for Chief OS. Proves SDK scales beyond HN briefer's `net.http` shape.

## Purpose

Every morning, scan `~/Documents` for files modified in the last 24 hours, summarize each with `ctx.ai().tier(Tier::Fast)`, persist summaries to Memory Graph, and emit one aggregate Brief card to `morning-brief` surface.

## Grants Declared

1. **fs.watch** — Watch `~/Documents/**/*.md` and `~/Documents/**/*.txt` for changes
2. **fs.read** — Read all `~/Documents/**` files to extract content
3. **mem.write** — Write `thought` and `card` nodes to Memory Graph
4. **surface.pane** — Render aggregate card on `morning-brief` surface
5. **llm.ai** — Summarize files with fast-tier inference (max 500 tokens)

## Example Card

```json
{
  "type": "card",
  "source": "File Watcher",
  "sender": "File Watcher",
  "subject": "Documents changed: 3 files",
  "snippet": "Summarized 3 files modified in the last 24 hours",
  "badge": "handled",
  "timestamp": "2026-04-22T14:30:00Z",
  "source_color": "green",
  "surface": "morning-brief",
  "items": [
    {
      "source": "Documents",
      "item_id": "file-summary-...",
      "path": "~/Documents/notes.md",
      "modified_at": "2026-04-22T10:15:00Z",
      "summary": "Brief one-sentence summary of the file content..."
    }
  ]
}
```

## Install

```bash
cargo build -p file-watcher-briefer
```

Tests:
```bash
cargo test -p file-watcher-briefer
```
