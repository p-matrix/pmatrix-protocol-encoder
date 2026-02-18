// validate.rs
// NON-NORMATIVE VALIDATION SCHEMA
// Normative authority: 4.0 TechRxiv Paper §5
// Canonical anchor: pmatrix.io/schema/4.0.json
//
// §5 schema field/type/range validation.
// FG-5: No scoring, threshold, or decision logic.

use crate::types::*;
use regex::Regex;

/// Validate a StateVector (§5.1). Returns list of errors.
pub fn validate_state_vector(sv: &StateVector) -> Vec<String> {
    let mut errors = Vec::new();

    // risk_info
    if sv.risk_info.r_t.is_nan() {
        errors.push("risk_info.r_t: NaN not allowed".into());
    } else if !(0.0..=1.0).contains(&sv.risk_info.r_t) {
        errors.push(format!("risk_info.r_t: {} out of range [0.0, 1.0]", sv.risk_info.r_t));
    }
    if sv.risk_info.s_t.is_nan() {
        errors.push("risk_info.s_t: NaN not allowed".into());
    } else if !(0.0..=1.0).contains(&sv.risk_info.s_t) {
        errors.push(format!("risk_info.s_t: {} out of range [0.0, 1.0]", sv.risk_info.s_t));
    }
    if !VALID_GATE_MODES.contains(&sv.risk_info.mode.as_str()) {
        errors.push(format!("risk_info.mode: '{}' invalid", sv.risk_info.mode));
    }
    if !VALID_RISK_LEVELS.contains(&sv.risk_info.risk_level.as_str()) {
        errors.push(format!("risk_info.risk_level: '{}' invalid", sv.risk_info.risk_level));
    }
    // mode ↔ risk_level consistency
    if let Some(expected) = expected_risk_level(&sv.risk_info.mode) {
        if sv.risk_info.risk_level != expected {
            errors.push(format!(
                "mode↔risk_level mismatch: {} expects {}, got {}",
                sv.risk_info.mode, expected, sv.risk_info.risk_level
            ));
        }
    }

    // lifecycle_info
    if sv.lifecycle_info.node_id.is_empty() {
        errors.push("lifecycle_info.node_id: must be non-empty".into());
    }

    // policy_digest: 64-char lowercase hex
    let hex_re = Regex::new(r"^[0-9a-f]{64}$").unwrap();
    if !hex_re.is_match(&sv.policy_digest) {
        errors.push("policy_digest: must be 64-char lowercase hex".into());
    }

    // freshness: at least one of timestamp/nonce/sequence must be non-zero/non-empty (§5.1)
    let has_ts = sv.freshness.timestamp > 0;
    let has_nonce = !sv.freshness.nonce.is_empty();
    let has_seq = sv.freshness.sequence > 0;
    if !has_ts && !has_nonce && !has_seq {
        errors.push("freshness: at least one of timestamp/nonce/sequence must be non-zero/non-empty".into());
    }

    // integrity: at least one of signature/signature_algo must be non-empty (§5.1)
    // Note: §5.1 requires "at least one of: digital signature, MAC, or attestation token"
    let has_sig = !sv.integrity.signature.is_empty();
    let has_algo = !sv.integrity.signature_algo.is_empty();
    if !has_sig && !has_algo {
        errors.push("integrity: at least one of signature/signature_algo must be non-empty".into());
    }

    errors
}

/// Validate a VerificationResult (§5.2).
pub fn validate_verification_result(vr: &VerificationResult) -> Vec<String> {
    let mut errors = Vec::new();

    if !VALID_VERIFICATION_STATUSES.contains(&vr.status.as_str()) {
        errors.push(format!("status: '{}' invalid", vr.status));
    }
    if vr.peer_node_id.is_empty() {
        errors.push("peer_node_id: must be non-empty".into());
    }

    errors
}

/// Validate a TransitionEvent (§5.3).
pub fn validate_transition_event(te: &TransitionEvent) -> Vec<String> {
    let mut errors = Vec::new();

    if !VALID_MODES.contains(&te.previous_mode) {
        errors.push(format!("previous_mode: {} invalid", te.previous_mode));
    }
    if !VALID_MODES.contains(&te.new_mode) {
        errors.push(format!("new_mode: {} invalid", te.new_mode));
    }
    if !VALID_TRANSITION_RULES.contains(&te.trigger_rule.as_str()) {
        errors.push(format!("trigger_rule: '{}' invalid", te.trigger_rule));
    }
    if !(1..=5).contains(&te.trigger_priority) {
        errors.push(format!("trigger_priority: {} must be 1-5", te.trigger_priority));
    }
    // rule ↔ priority consistency
    if let Some(expected) = rule_priority(&te.trigger_rule) {
        if te.trigger_priority != expected {
            errors.push(format!(
                "trigger_priority mismatch: {} expects {}, got {}",
                te.trigger_rule, expected, te.trigger_priority
            ));
        }
    }

    errors
}

/// Validate a RecoveryChecklist (§5.4).
pub fn validate_recovery_checklist(rc: &RecoveryChecklist) -> Vec<String> {
    let mut errors = Vec::new();

    if rc.required_successes == 0 {
        errors.push("required_successes: must be > 0".into());
    }
    // recovery_target: 0=not recovered, 1=MODE_1 target (full), 2=MODE_2 target (conservative)
    // Per §5.4: if all conditions met + risk normal → 1, risk warning → 2, not met → 0
    if ![0, 1, 2].contains(&rc.recovery_target) {
        errors.push(format!("recovery_target: must be 0, 1, or 2, got {}", rc.recovery_target));
    }

    // all_satisfied consistency
    let success_met = rc.consecutive_successes >= rc.required_successes;
    let expected_all = rc.risk_normalized && rc.policy_aligned && success_met && rc.reject_ceased;
    if rc.all_satisfied != expected_all {
        errors.push(format!(
            "all_satisfied inconsistency: computed {}, got {}",
            expected_all, rc.all_satisfied
        ));
    }

    // recovery_target consistency
    if !rc.all_satisfied && rc.recovery_target != 0 {
        errors.push("recovery_target: must be 0 when all_satisfied is false".into());
    }
    if rc.all_satisfied && rc.recovery_target == 0 {
        errors.push("recovery_target: must be 1 or 2 when all_satisfied is true".into());
    }

    errors
}

/// Validate a PEPAction (§5.5).
pub fn validate_pep_action(pa: &PEPAction) -> Vec<String> {
    let mut errors = Vec::new();

    if !VALID_PEP_ACTION_TYPES.contains(&pa.action_type.as_str()) {
        errors.push(format!("action_type: '{}' invalid", pa.action_type));
    }

    errors
}

/// Validate a PropagationMessage (§5.6).
pub fn validate_propagation_message(pm: &PropagationMessage) -> Vec<String> {
    let mut errors = Vec::new();

    let uuid_re = Regex::new(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ).unwrap();
    if !uuid_re.is_match(&pm.propagation_id.to_lowercase()) {
        errors.push("propagation_id: invalid UUID format".into());
    }

    if pm.source_node_id.is_empty() {
        errors.push("source_node_id: must be non-empty".into());
    }
    if !VALID_MODES.contains(&pm.previous_mode) {
        errors.push(format!("previous_mode: {} invalid", pm.previous_mode));
    }
    if !VALID_MODES.contains(&pm.new_mode) {
        errors.push(format!("new_mode: {} invalid", pm.new_mode));
    }

    if let Some(ref rule) = pm.trigger_rule {
        if !VALID_TRANSITION_RULES.contains(&rule.as_str()) {
            errors.push(format!("trigger_rule: '{}' invalid", rule));
        }
    }

    // Nested state vector validation
    if let Some(ref sv) = pm.state_vector {
        let sv_errors = validate_state_vector(sv);
        for e in sv_errors {
            errors.push(format!("state_vector.{}", e));
        }
    }

    errors
}

/// Validate an AuditEntry (§7 PI-3).
pub fn validate_audit_entry(ae: &AuditEntry) -> Vec<String> {
    let mut errors = Vec::new();
    let hex_re = Regex::new(r"^[0-9a-f]{64}$").unwrap();

    if !VALID_AUDIT_EVENT_TYPES.contains(&ae.event_type.as_str()) {
        errors.push(format!("event_type: '{}' invalid", ae.event_type));
    }
    if ae.entry_data.is_empty() {
        errors.push("entry_data: must be non-empty".into());
    }
    if !hex_re.is_match(&ae.prev_hash) {
        errors.push("prev_hash: must be 64-char lowercase hex".into());
    }
    if !hex_re.is_match(&ae.entry_hash) {
        errors.push("entry_hash: must be 64-char lowercase hex".into());
    }

    // Genesis check
    if ae.chain_index == 0 && ae.prev_hash != GENESIS_HASH {
        errors.push(format!("chain_index 0: prev_hash must be genesis ({})", GENESIS_HASH));
    }

    // Hash chain integrity
    if !ae.entry_data.is_empty() && hex_re.is_match(&ae.prev_hash) && hex_re.is_match(&ae.entry_hash) {
        let expected = crate::audit::compute_entry_hash(&ae.prev_hash, &ae.entry_data);
        if ae.entry_hash != expected {
            errors.push(format!("hash chain broken: expected {}, got {}", expected, ae.entry_hash));
        }
    }

    errors
}
