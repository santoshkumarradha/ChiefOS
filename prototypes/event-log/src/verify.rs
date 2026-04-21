//! Merkle chain verification for stored event records.

use crate::sign::{full_form, verify_signature};
use crate::{EventLog, StoredEvent, ZERO_HASH};
use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("chain break at position {position}: {reason}")]
pub struct ChainBreakError {
    pub position: u64,
    pub reason: String,
}

pub(crate) fn walk_chain(log: &EventLog) -> Result<(), ChainBreakError> {
    let height = log.height().map_err(|err| ChainBreakError {
        position: 0,
        reason: format!("could not read height: {err}"),
    })?;

    let mut previous_full_hash = ZERO_HASH;

    for position in 0..height {
        let stored = log
            .stored_at(position)
            .map_err(|err| chain_err(position, format!("could not read event: {err}")))?;

        verify_record(position, &stored, &previous_full_hash)?;
        previous_full_hash = stored.full_form_hash;
    }

    let last_root = log.last_root().map_err(|err| ChainBreakError {
        position: height.saturating_sub(1),
        reason: format!("could not read last root: {err}"),
    })?;

    if last_root != previous_full_hash {
        return Err(ChainBreakError {
            position: height.saturating_sub(1),
            reason: "metadata last_merkle_root does not match chain head".to_string(),
        });
    }

    Ok(())
}

fn verify_record(
    position: u64,
    stored: &StoredEvent,
    expected_prev: &[u8; 32],
) -> Result<(), ChainBreakError> {
    if stored.position != position {
        return Err(chain_err(
            position,
            format!("stored position {} does not match key", stored.position),
        ));
    }

    if &stored.prev_hash != expected_prev {
        return Err(chain_err(
            position,
            "prev_hash does not match previous full-form hash".to_string(),
        ));
    }

    let event_bytes = crate::schema::canonical_event_bytes(&stored.event)
        .map_err(|err| chain_err(position, format!("event serialization failed: {err}")))?;

    if event_bytes != stored.event_bytes {
        return Err(chain_err(
            position,
            "event_bytes do not match canonical event serialization".to_string(),
        ));
    }

    let full_form = full_form(
        &stored.prev_hash,
        &stored.event_bytes,
        stored.timestamp_nanos,
        &stored.device_id,
    );
    let full_hash = *blake3::hash(&full_form).as_bytes();

    if stored.full_form_hash != full_hash {
        return Err(chain_err(
            position,
            "full-form hash does not match stored Merkle root".to_string(),
        ));
    }

    let event_id = *blake3::hash(&full_form).as_bytes();
    if stored.event_id != event_id {
        return Err(chain_err(
            position,
            "event_id does not match full-form hash".to_string(),
        ));
    }

    verify_signature(&stored.device_pubkey, &full_form, &stored.signature)
        .map_err(|err| chain_err(position, format!("signature verification failed: {err}")))?;

    Ok(())
}

fn chain_err(position: u64, reason: String) -> ChainBreakError {
    ChainBreakError { position, reason }
}
