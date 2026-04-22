# HAX Inbox Prototype

Chief OS notification surface per ADR-0007. Single unified queue replaces all popups.

## Features

- **Append-only SQLite backend** with type-safe schema (InboxItem, BadgeVariant enum)
- **ratatui TUI** with 3-tab grouping: CEREMONY PENDING, NEEDS ATTENTION, HANDLED TODAY
- **Wayland layer-shell overlay** (Linux only) — top-right anchored 22% width × 68% height
- **CLI** to post items and launch TUI / overlay

## Building

```bash
cargo build -p hax-inbox
```

macOS builds skip layer-shell. Linux builds include full Wayland support.

## Usage

```bash
# Post an item
hax-inbox post needs-attention "Review draft" "Stanford PDF" \
  --badge review --source "law-briefer"

# Launch TUI
hax-inbox tui

# Launch Wayland overlay (Linux only)
hax-inbox overlay
```

### TUI Keybinds

- **Ctrl+I**: Switch tab
- **Ctrl+J** or **Enter**: Open selected item
- **↑↓**: Navigate within tab
- **Esc**: Close TUI or clear opened item

## Data Model

```rust
pub struct InboxItem {
    pub id: Uuid,
    pub kind: InboxKind,  // NeedsAttention | Informational | CeremonyPending
    pub title: String,
    pub snippet: String,
    pub source_agent: String,
    pub badge: BadgeVariant,  // NeedsYou | Review | Handled | Ceremony | Anchor
    pub timestamp: DateTime<Utc>,
}
```

## Persistence

Items stored in append-only SQLite table under `~/.chief/inbox.sqlite3` (default).
Schema auto-initializes on first write.

## Wayland Compositors

Tested on wlroots-based compositors (Hyprland, Sway, Niri). Requires `wlr-layer-shell` protocol.

## Integration

This prototype proves the substrate-spine mechanism. Real Chief UI integration (React surface in Tauri) is downstream.
