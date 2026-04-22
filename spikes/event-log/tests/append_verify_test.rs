use chief_event_log_proto::schema::Event;
use chief_event_log_proto::EventLog;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tempfile::tempdir;

fn h(seed: u8) -> [u8; 32] {
    [seed; 32]
}

fn all_events() -> Vec<Event> {
    vec![
        Event::SyscallWrap {
            syscall: "openat".to_string(),
            fd: 7,
            args_hash: h(1),
        },
        Event::ToolCall {
            agent: "calendar-agent".to_string(),
            tool: "calendar.create".to_string(),
            args_hash: h(2),
            result_hash: h(3),
        },
        Event::AgentDecision {
            agent: "finance-agent".to_string(),
            question: "approve transfer?".to_string(),
            choice: "escalate".to_string(),
            rationale_hash: h(4),
        },
        Event::UIAction {
            surface: "Morning Reveal".to_string(),
            action: "approve".to_string(),
            payload_hash: h(5),
        },
        Event::CapabilityIssued {
            grant: "grant:calendar:write".to_string(),
        },
        Event::CapabilityRevoked {
            grant_id: "grant-123".to_string(),
        },
        Event::CapabilityCheck {
            principal: "agent:calendar".to_string(),
            op: "calendar.write".to_string(),
            allowed: true,
        },
    ]
}

#[test]
fn happy_path_per_event_kind() {
    let dir = tempdir().unwrap();
    let log = EventLog::open(dir.path()).unwrap();

    for event in all_events() {
        let id = log.append(event.clone()).unwrap();
        assert_eq!(log.read(id).unwrap(), event);
    }

    log.verify_chain().unwrap();
}

#[test]
fn iter_since_filters_by_timestamp() {
    let dir = tempdir().unwrap();
    let log = EventLog::open(dir.path()).unwrap();
    let before = Utc::now() - Duration::seconds(1);

    for event in all_events() {
        log.append(event).unwrap();
    }

    let events: Vec<_> = log.iter_since(before).unwrap().collect();
    assert_eq!(events.len(), 7);
}

#[test]
fn merkle_chain_integrity_detects_tampering() {
    let dir = tempdir().unwrap();
    let log = EventLog::open(dir.path()).unwrap();

    for event in all_events() {
        log.append(event).unwrap();
    }

    log.verify_chain().unwrap();
    log.corrupt_event_byte_for_test(2, 12).unwrap();

    let err = log.verify_chain().unwrap_err();
    assert_eq!(err.position, 2);
}

#[test]
fn concurrent_append_safety() {
    let dir = tempdir().unwrap();
    let log = Arc::new(EventLog::open(dir.path()).unwrap());
    let mut handles = Vec::new();

    for worker in 0..8 {
        let log = Arc::clone(&log);
        handles.push(std::thread::spawn(move || {
            for i in 0..100 {
                log.append(Event::CapabilityCheck {
                    principal: format!("agent:{worker}"),
                    op: format!("op:{i}"),
                    allowed: i % 2 == 0,
                })
                .unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    log.verify_chain().unwrap();
    let events: Vec<_> = log
        .iter_since(Utc::now() - Duration::minutes(5))
        .unwrap()
        .collect();
    assert_eq!(events.len(), 800);
}
