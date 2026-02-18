// invariants.rs
// NON-NORMATIVE VALIDATION SCHEMA
// Normative authority: 4.0 TechRxiv Paper §6 (FP-1~5), §7 (PI-1~4)
// Canonical anchor: pmatrix.io/schema/4.0.json
//
// Structural property and protocol invariant verification.
// Separated from validate.rs per condition #4.
// FG-5: No scoring, threshold, or decision logic.

use crate::types::*;

/// FP-1: Monotonic Escalation — MODE_3→MODE_1 direct transition prohibited.
/// If state(t) = MODE_3 ∧ ¬recovery_flag(t), then state(t+1) ≠ MODE_1.
pub fn check_fp1_monotonic_escalation(te: &TransitionEvent) -> Option<String> {
    if te.previous_mode == 3 && te.new_mode == 1 {
        Some("FP-1 violation: MODE_3→MODE_1 direct transition prohibited".into())
    } else {
        None
    }
}

/// FP-2: Reject Priority Invariance — reject signal = highest priority (1).
/// ∀t: reject_signal(t) ⇒ priority(reject) = max(priority_set).
pub fn check_fp2_reject_priority(te: &TransitionEvent) -> Option<String> {
    if te.trigger_rule == "REJECT_SIGNAL" && te.trigger_priority != 1 {
        Some(format!(
            "FP-2 violation: REJECT_SIGNAL must have priority 1, got {}",
            te.trigger_priority
        ))
    } else {
        None
    }
}

/// FP-3: Verification–Decision Separation (structural check).
/// This is an architectural property; at schema level, we verify that
/// verification results and transition events are distinct message types.
pub fn check_fp3_separation() -> Option<String> {
    // Structural: VerificationResult and TransitionEvent are distinct types.
    // This is guaranteed by the type system at compile time.
    None
}

/// FP-4: Non-bypass Enforcement — all 5 external effect paths must be
/// representable in PEP actions.
pub fn check_fp4_non_bypass() -> Option<String> {
    // Verify all 5 effect paths + permit_all exist in VALID_PEP_ACTION_TYPES
    let required = [
        "block_output",
        "block_tool_call",
        "block_network",
        "block_file_write",
        "block_service_call",
    ];
    for path in &required {
        if !VALID_PEP_ACTION_TYPES.contains(path) {
            return Some(format!("FP-4 violation: missing effect path '{}'", path));
        }
    }
    None
}

/// FP-5: Safety Overrides Liveness — reject → restrict regardless of availability.
/// At schema level: reject signal always maps to escalation (MODE_2 or MODE_3).
pub fn check_fp5_safety_overrides_liveness(te: &TransitionEvent) -> Option<String> {
    if te.trigger_rule == "REJECT_SIGNAL" && te.new_mode < te.previous_mode {
        Some(format!(
            "FP-5 violation: REJECT_SIGNAL must escalate or maintain, got MODE_{}→MODE_{}",
            te.previous_mode, te.new_mode
        ))
    } else {
        None
    }
}

/// PI-1: Reject Dominance — peer rejection > local self-assessment.
/// At schema level: REJECT_SIGNAL priority = 1 (checked via FP-2).
pub fn check_pi1_reject_dominance(te: &TransitionEvent) -> Option<String> {
    check_fp2_reject_priority(te)
}

/// PI-2: Verification–Decision Separation (same as FP-3).
pub fn check_pi2_verification_decision_separation() -> Option<String> {
    check_fp3_separation()
}

/// PI-3: Append-Only Audit — validate chain linkage.
pub fn check_pi3_append_only_audit(entries: &[AuditEntry]) -> Vec<String> {
    let mut errors = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        if entry.chain_index != i as u64 {
            errors.push(format!(
                "[{}] chain_index: expected {}, got {}",
                i, i, entry.chain_index
            ));
        }
        if i > 0 {
            let expected_prev = &entries[i - 1].entry_hash;
            if entry.prev_hash != *expected_prev {
                errors.push(format!(
                    "[{}] prev_hash linkage broken: expected {}, got {}",
                    i, expected_prev, entry.prev_hash
                ));
            }
        }
    }

    errors
}

/// PI-4: Non-bypass Enforcement (same as FP-4).
pub fn check_pi4_non_bypass() -> Option<String> {
    check_fp4_non_bypass()
}

/// Run all structural invariant checks on a TransitionEvent.
pub fn check_all_transition_invariants(te: &TransitionEvent) -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(e) = check_fp1_monotonic_escalation(te) {
        errors.push(e);
    }
    if let Some(e) = check_fp2_reject_priority(te) {
        errors.push(e);
    }
    if let Some(e) = check_fp5_safety_overrides_liveness(te) {
        errors.push(e);
    }
    if let Some(e) = check_pi1_reject_dominance(te) {
        // PI-1 overlaps FP-2; include for completeness
        if !errors.iter().any(|x| x.contains("FP-2")) {
            errors.push(e);
        }
    }
    errors
}

/// Run all structural (non-message) invariant checks.
/// FP-3/PI-2: compile-time guarantee (type separation).
/// FP-4/PI-4: schema completeness (5 effect paths present).
pub fn check_all_structural_invariants() -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(e) = check_fp3_separation() {
        errors.push(e);
    }
    if let Some(e) = check_fp4_non_bypass() {
        errors.push(e);
    }
    if let Some(e) = check_pi2_verification_decision_separation() {
        errors.push(e);
    }
    if let Some(e) = check_pi4_non_bypass() {
        errors.push(e);
    }
    errors
}
