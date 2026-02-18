// types.rs
// NON-NORMATIVE VALIDATION SCHEMA
// Normative authority: 4.0 TechRxiv Paper §5
// Canonical anchor: pmatrix.io/schema/4.0.json
//
// 7 protocol schema structs covering §5.1–§5.6 + §7 PI-3.
// FG-5: No scoring, threshold, or decision logic.

use serde::{Deserialize, Serialize};

// ── §5.1 State Vector Exchange Format ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateVector {
    pub risk_info: RiskInfo,
    pub lifecycle_info: LifecycleInfo,
    pub policy_digest: String,
    pub freshness: Freshness,
    pub integrity: Integrity,
    pub is_propagation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskInfo {
    pub r_t: f64,
    pub s_t: f64,
    pub mode: String,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LifecycleInfo {
    pub node_id: String,
    pub cycle_id: u64,
    pub uptime_cycles: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Freshness {
    pub timestamp: u64,
    pub nonce: String,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Integrity {
    pub signature: String,
    pub signature_algo: String,
}

// ── §5.2 Verification Result Structure ───────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationResult {
    pub status: String,
    pub peer_node_id: String,
    pub verified_at: u64,
    pub details: String,
}

// ── §5.3 Transition Event ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionEvent {
    pub previous_mode: u8,
    pub new_mode: u8,
    pub trigger_rule: String,
    pub trigger_priority: u8,
    pub policy_digest_mismatch: bool,
    pub timestamp: u64,
}

// ── §5.4 Recovery Checklist ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecoveryChecklist {
    pub risk_normalized: bool,
    pub policy_aligned: bool,
    pub consecutive_successes: u32,
    pub required_successes: u32,
    pub reject_ceased: bool,
    pub all_satisfied: bool,
    pub recovery_target: u8,
    pub timestamp: u64,
}

// ── §5.5 PEP Action ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PEPAction {
    pub action_type: String,
    pub blocked: bool,
    pub detail: String,
}

// ── §5.6 Propagation Message ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropagationMessage {
    pub propagation_id: String,
    pub source_node_id: String,
    pub sequence_number: u64,
    pub monotonic_counter: u64,
    pub previous_mode: u8,
    pub new_mode: u8,
    pub trigger_rule: Option<String>,
    pub state_vector: Option<StateVector>,
    pub timestamp: u64,
}

// ── §7 PI-3 Audit Entry ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    pub chain_index: u64,
    pub event_type: String,
    pub entry_data: String,
    pub prev_hash: String,
    pub entry_hash: String,
    pub timestamp: u64,
}

// ── Constants ────────────────────────────────────────────────────────

pub const VALID_MODES: [u8; 3] = [1, 2, 3];
pub const VALID_GATE_MODES: [&str; 5] = ["A+1", "A+0", "A-1", "A-2", "A-0"];
pub const VALID_RISK_LEVELS: [&str; 5] = ["L1", "L2", "L3", "L4", "L5"];

pub const VALID_VERIFICATION_STATUSES: [&str; 6] = [
    "passed",
    "no_peer",
    "failed_freshness",
    "failed_integrity",
    "failed_risk_range",
    "failed_policy_mismatch",
];

pub const VALID_TRANSITION_RULES: [&str; 5] = [
    "REJECT_SIGNAL",
    "VERIFICATION_FAILURE",
    "COMMUNICATION_TIMEOUT",
    "ADDITIONAL_CONDITIONS",
    "MONOTONICITY",
];

pub const VALID_PEP_ACTION_TYPES: [&str; 6] = [
    "block_output",
    "block_tool_call",
    "block_network",
    "block_file_write",
    "block_service_call",
    "permit_all",
];

pub const VALID_AUDIT_EVENT_TYPES: [&str; 6] = [
    "cycle",
    "propagation",
    "policy_sync",
    "reject",
    "recovery",
    "error",
];

pub const GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Mode ↔ risk_level mapping
pub fn expected_risk_level(mode: &str) -> Option<&'static str> {
    match mode {
        "A+1" => Some("L1"),
        "A+0" => Some("L2"),
        "A-1" => Some("L3"),
        "A-2" => Some("L4"),
        "A-0" => Some("L5"),
        _ => None,
    }
}

/// Transition rule → priority mapping
pub fn rule_priority(rule: &str) -> Option<u8> {
    match rule {
        "REJECT_SIGNAL" => Some(1),
        "VERIFICATION_FAILURE" => Some(2),
        "COMMUNICATION_TIMEOUT" => Some(3),
        "ADDITIONAL_CONDITIONS" => Some(4),
        "MONOTONICITY" => Some(5),
        _ => None,
    }
}
