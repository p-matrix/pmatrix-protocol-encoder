// emit.rs
// NON-NORMATIVE VALIDATION SCHEMA
// Normative authority: 4.0 TechRxiv Paper §5.1
// Canonical anchor: pmatrix.io/schema/4.0.json
//
// Synthetic StateVector generation for schema conformance illustration.
// FG-5: Uses fixed synthetic values only. No scoring or threshold logic.

use crate::types::*;

/// Map operating mode (1/2/3) to a synthetic gate mode + risk level.
/// These are NON-NORMATIVE synthetic mappings for illustration only.
///
/// Note: 3.5 gate mode labels (A+1/A-1/A-0) are used as synthetic display
/// labels only. 4.0 Protocol operating modes are MODE_1/MODE_2/MODE_3.
fn synthetic_gate_mode(mode: u8) -> (&'static str, &'static str) {
    match mode {
        1 => ("A+1", "L1"),
        2 => ("A-1", "L3"),
        3 => ("A-0", "L5"),
        _ => ("A+0", "L2"), // fallback
    }
}

/// Emit a synthetic StateVector (§5.1).
///
/// All values are synthetic for schema conformance illustration.
/// No scoring, threshold, or evaluation logic is applied.
pub fn emit_state_vector(
    node_id: &str,
    mode: u8,
    r_t: f64,
    cycle_id: u64,
) -> StateVector {
    let (gate_mode, risk_level) = synthetic_gate_mode(mode);

    // Synthetic s_t: complement of r_t (NON-NORMATIVE, illustration only)
    let s_t = 1.0 - r_t;

    // Synthetic policy digest (fixed for illustration)
    let policy_digest = "a".repeat(64);

    // Synthetic timestamp (microseconds)
    let timestamp = 1707500000_u64 + cycle_id * 1_000_000;

    StateVector {
        risk_info: RiskInfo {
            r_t,
            s_t,
            mode: gate_mode.to_string(),
            risk_level: risk_level.to_string(),
        },
        lifecycle_info: LifecycleInfo {
            node_id: node_id.to_string(),
            cycle_id,
            uptime_cycles: cycle_id,
        },
        policy_digest,
        freshness: Freshness {
            timestamp,
            nonce: format!("nonce-{}-{}", node_id, cycle_id),
            sequence: cycle_id,
        },
        integrity: Integrity {
            signature: format!("sig-{}-{}", node_id, cycle_id),
            signature_algo: "ed25519".to_string(),
        },
        is_propagation: false,
    }
}
