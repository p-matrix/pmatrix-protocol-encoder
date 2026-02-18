// audit.rs
// NON-NORMATIVE VALIDATION SCHEMA
// Normative authority: 4.0 TechRxiv Paper §7 PI-3
// Canonical anchor: pmatrix.io/schema/4.0.json
//
// SHA-256 hash chain computation for append-only audit.
// Rule #9: SHA computation allowed in encoder (serialization integrity).
// FG-5: No scoring, threshold, or decision logic.

use sha2::{Digest, Sha256};

use crate::types::{AuditEntry, GENESIS_HASH};

/// Compute SHA-256(prev_hash_bytes + entry_data_bytes).
pub fn compute_entry_hash(prev_hash: &str, entry_data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(entry_data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Build an AuditEntry with correct hash chain linkage.
pub fn build_audit_entry(
    chain_index: u64,
    event_type: &str,
    entry_data: &str,
    prev_hash: &str,
    timestamp: u64,
) -> AuditEntry {
    let entry_hash = compute_entry_hash(prev_hash, entry_data);

    AuditEntry {
        chain_index,
        event_type: event_type.to_string(),
        entry_data: entry_data.to_string(),
        prev_hash: prev_hash.to_string(),
        entry_hash,
        timestamp,
    }
}

/// Build genesis entry (chain_index = 0).
pub fn build_genesis_entry(event_type: &str, entry_data: &str, timestamp: u64) -> AuditEntry {
    build_audit_entry(0, event_type, entry_data, GENESIS_HASH, timestamp)
}

/// Append a new entry to an existing chain.
pub fn append_to_chain(
    chain: &[AuditEntry],
    event_type: &str,
    entry_data: &str,
    timestamp: u64,
) -> AuditEntry {
    let (index, prev_hash) = if chain.is_empty() {
        (0, GENESIS_HASH.to_string())
    } else {
        let last = chain.last().unwrap();
        (last.chain_index + 1, last.entry_hash.clone())
    };

    build_audit_entry(index, event_type, entry_data, &prev_hash, timestamp)
}
