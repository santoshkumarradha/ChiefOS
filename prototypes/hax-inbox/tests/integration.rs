use std::str::FromStr;

use anyhow::Result;
use chrono::Utc;
use hax_inbox::inbox::{BadgeVariant, InboxBackend, InboxItem, InboxKind, NewInboxItem};
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn roundtrip_post_list_read() -> Result<()> {
    let backend = InboxBackend::open_memory()?;
    let posted = backend.post(fixture_new_item("Review plan"))?;

    let listed = backend.list()?;
    let read = backend.read(posted.id)?;

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, posted.id);
    assert_eq!(read.as_ref().map(|item| item.id), Some(posted.id));
    Ok(())
}

#[test]
fn persistence_across_backend_restart() -> Result<()> {
    let dir = tempdir()?;
    let db = dir.path().join("inbox.sqlite3");

    let posted = InboxBackend::open(&db)?.post(fixture_new_item("Persist me"))?;
    let reopened = InboxBackend::open(&db)?;

    assert_eq!(
        reopened.list()?.first().map(|item| item.id),
        Some(posted.id)
    );
    Ok(())
}

#[test]
fn badge_variant_is_closed_enum() {
    let error = BadgeVariant::from_str("urgent-toast").expect_err("invalid badge must fail");
    assert!(error.to_string().contains("invalid badge variant"));
}

#[test]
fn tui_snapshot_with_fixture() -> Result<()> {
    let snapshot = hax_inbox::tui::render_snapshot(fixture_items())?;

    assert!(snapshot.contains("CEREMONY PENDING"));
    assert!(snapshot.contains("NEEDS ATTENTION"));
    assert!(snapshot.contains("HANDLED TODAY"));
    assert!(snapshot.contains("Approve bank transfer"));
    assert!(snapshot.contains("Ctrl-I tab"));
    Ok(())
}

fn fixture_new_item(title: &str) -> NewInboxItem {
    NewInboxItem {
        kind: InboxKind::NeedsAttention,
        title: title.to_string(),
        snippet: "Human review requested".to_string(),
        source_agent: "fixture-agent".to_string(),
        badge: BadgeVariant::Review,
    }
}

fn fixture_items() -> Vec<InboxItem> {
    vec![
        fixture_item(
            InboxKind::CeremonyPending,
            "Approve bank transfer",
            "ACH batch needs ceremony",
            BadgeVariant::Ceremony,
        ),
        fixture_item(
            InboxKind::NeedsAttention,
            "Review Stanford draft",
            "Cites page 14 paragraph 3",
            BadgeVariant::NeedsYou,
        ),
        fixture_item(
            InboxKind::Informational,
            "Invoices filed",
            "Three receipts attached",
            BadgeVariant::Handled,
        ),
    ]
}

fn fixture_item(kind: InboxKind, title: &str, snippet: &str, badge: BadgeVariant) -> InboxItem {
    InboxItem {
        id: Uuid::new_v4(),
        kind,
        title: title.to_string(),
        snippet: snippet.to_string(),
        source_agent: "fixture-agent".to_string(),
        badge,
        timestamp: Utc::now(),
    }
}
