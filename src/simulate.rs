// simulate.rs
// NON-NORMATIVE VALIDATION SCHEMA
// Normative authority: 4.0 TechRxiv Paper §8
// Canonical anchor: pmatrix.io/schema/4.0.json
//
// DETERMINISTIC PLAYBACK ONLY.
// This is a "recorded scenario player", NOT an engine.
// Condition #1: Fixed sequence JSON generation only. No dynamic simulation.
// Condition #2: Recovery condition calculation logic PROHIBITED.
//               "4 conditions satisfied" is assumed in the log.
// Condition #3: PEP decision logic PROHIBITED.
//               "Block occurred" log generation is permitted.
// FG-5: No scoring, threshold, or decision logic.

use serde::Serialize;

use crate::audit;
use crate::emit::emit_state_vector;
use crate::types::*;

// ── Simulation output types ──────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SimulationLog {
    pub metadata: SimulationMetadata,
    pub cycles: Vec<CycleRecord>,
}

#[derive(Debug, Serialize)]
pub struct SimulationMetadata {
    pub spec: String,
    pub description: String,
    pub node_a: String,
    pub node_b: String,
    pub disclaimer: String,
}

#[derive(Debug, Serialize)]
pub struct CycleRecord {
    pub cycle: u32,
    pub stage: String,
    pub description: String,
    pub node_a: Option<NodeState>,
    pub node_b: Option<NodeState>,
    pub events: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct NodeState {
    pub mode: u8,
    pub r_t: f64,
    pub state_vector: Option<StateVector>,
    pub verification: Option<VerificationResult>,
    pub transition: Option<TransitionEvent>,
    pub pep_actions: Option<Vec<PEPAction>>,
    pub recovery: Option<RecoveryChecklist>,
    pub propagation: Option<PropagationMessage>,
}

// ── Deterministic playback ───────────────────────────────────────────

/// Run the §8 2-node 6-stage deterministic scenario.
///
/// This produces a FIXED sequence of JSON records.
/// All values are synthetic. No decision logic is executed.
pub fn run_simulation() -> SimulationLog {
    let node_a_id = "node-alpha";
    let node_b_id = "node-beta";

    let mut audit_chain: Vec<AuditEntry> = Vec::new();
    let mut cycles = Vec::new();
    let base_ts = 1707500000_u64;

    // ── Stage 1: Normal exchange (§8 step 1) ─────────────────────────
    // Both nodes MODE_1, mutual verification passed.
    let sv_a1 = emit_state_vector(node_a_id, 1, 0.1, 1);
    let sv_b1 = emit_state_vector(node_b_id, 1, 0.15, 1);

    let vr_a1 = VerificationResult {
        status: "passed".into(),
        peer_node_id: node_b_id.into(),
        verified_at: base_ts + 1_000_000,
        details: "All 4 verification types passed".into(),
    };
    let vr_b1 = VerificationResult {
        status: "passed".into(),
        peer_node_id: node_a_id.into(),
        verified_at: base_ts + 1_000_000,
        details: "All 4 verification types passed".into(),
    };

    let ae1 = audit::build_genesis_entry(
        "cycle",
        &serde_json::to_string(&serde_json::json!({
            "cycle": 1, "node_a_mode": 1, "node_b_mode": 1, "status": "normal"
        })).unwrap(),
        base_ts + 1_000_000,
    );
    audit_chain.push(ae1);

    cycles.push(CycleRecord {
        cycle: 1,
        stage: "normal_exchange".into(),
        description: "§8 Step 1: Both nodes exchange valid state vectors. Both remain MODE_1.".into(),
        node_a: Some(NodeState {
            mode: 1, r_t: 0.1,
            state_vector: Some(sv_a1),
            verification: Some(vr_a1),
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        node_b: Some(NodeState {
            mode: 1, r_t: 0.15,
            state_vector: Some(sv_b1),
            verification: Some(vr_b1),
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        events: vec![serde_json::json!({"audit": audit_chain.last().unwrap()})],
    });

    // ── Stage 2: Risk escalation (§8 step 2) ─────────────────────────
    // Node B r_t = 0.85 (exceeds warning). Node A detects, issues reject.
    let sv_a2 = emit_state_vector(node_a_id, 1, 0.1, 2);
    let sv_b2 = emit_state_vector(node_b_id, 1, 0.85, 2);

    let vr_a2 = VerificationResult {
        status: "failed_risk_range".into(),
        peer_node_id: node_b_id.into(),
        verified_at: base_ts + 2_000_000,
        details: "Node B r_t=0.85 exceeds acceptable range. Reject signal issued.".into(),
    };

    let ae2 = audit::append_to_chain(
        &audit_chain, "reject",
        &serde_json::to_string(&serde_json::json!({
            "cycle": 2, "rejector": node_a_id, "target": node_b_id,
            "reason": "risk_range_exceeded", "r_t": 0.85
        })).unwrap(),
        base_ts + 2_000_000,
    );
    audit_chain.push(ae2);

    cycles.push(CycleRecord {
        cycle: 2,
        stage: "risk_escalation".into(),
        description: "§8 Step 2: Node B r_t=0.85 exceeds threshold. Node A issues reject signal.".into(),
        node_a: Some(NodeState {
            mode: 1, r_t: 0.1,
            state_vector: Some(sv_a2),
            verification: Some(vr_a2),
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        node_b: Some(NodeState {
            mode: 1, r_t: 0.85,
            state_vector: Some(sv_b2),
            verification: None,
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        events: vec![serde_json::json!({"audit": audit_chain.last().unwrap()})],
    });

    // ── Stage 3: Reject-triggered escalation (§8 step 3) ─────────────
    // Node B receives reject → MODE_3 (PI-1: reject overrides self-assessment)
    let te_b3 = TransitionEvent {
        previous_mode: 1,
        new_mode: 3,
        trigger_rule: "REJECT_SIGNAL".into(),
        trigger_priority: 1,
        policy_digest_mismatch: false,
        timestamp: base_ts + 3_000_000,
    };

    let ae3 = audit::append_to_chain(
        &audit_chain, "cycle",
        &serde_json::to_string(&serde_json::json!({
            "cycle": 3, "node_b_transition": "MODE_1→MODE_3",
            "trigger": "REJECT_SIGNAL", "pi1": "reject_dominance_applied"
        })).unwrap(),
        base_ts + 3_000_000,
    );
    audit_chain.push(ae3);

    cycles.push(CycleRecord {
        cycle: 3,
        stage: "reject_triggered_escalation".into(),
        description: "§8 Step 3: Node B receives reject. PI-1 applied: MODE_1→MODE_3.".into(),
        node_a: Some(NodeState {
            mode: 1, r_t: 0.1,
            state_vector: None, verification: None,
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        node_b: Some(NodeState {
            mode: 3, r_t: 0.85,
            state_vector: None, verification: None,
            transition: Some(te_b3),
            pep_actions: None, recovery: None, propagation: None,
        }),
        events: vec![serde_json::json!({"audit": audit_chain.last().unwrap()})],
    });

    // ── Stage 4: Enforcement (§8 step 4) ─────────────────────────────
    // Node B PEP restricts all 5 external effect paths. Audit record appended.
    // Condition #3: No PEP decision logic. "Block occurred" log only.
    let pep_actions = vec![
        PEPAction { action_type: "block_output".into(), blocked: true, detail: "MODE_3: output generation blocked".into() },
        PEPAction { action_type: "block_tool_call".into(), blocked: true, detail: "MODE_3: tool invocation blocked".into() },
        PEPAction { action_type: "block_network".into(), blocked: true, detail: "MODE_3: network transmission blocked".into() },
        PEPAction { action_type: "block_file_write".into(), blocked: true, detail: "MODE_3: file system writes blocked".into() },
        PEPAction { action_type: "block_service_call".into(), blocked: true, detail: "MODE_3: microservice calls blocked".into() },
    ];

    let ae4 = audit::append_to_chain(
        &audit_chain, "cycle",
        &serde_json::to_string(&serde_json::json!({
            "cycle": 4, "enforcement": "all_5_paths_blocked",
            "node_b_mode": 3, "fp4": "non_bypass_enforcement"
        })).unwrap(),
        base_ts + 4_000_000,
    );
    audit_chain.push(ae4);

    cycles.push(CycleRecord {
        cycle: 4,
        stage: "enforcement".into(),
        description: "§8 Step 4: Node B PEP blocks all 5 external effect paths. FP-4 satisfied.".into(),
        node_a: Some(NodeState {
            mode: 1, r_t: 0.1,
            state_vector: None, verification: None,
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        node_b: Some(NodeState {
            mode: 3, r_t: 0.85,
            state_vector: None, verification: None, transition: None,
            pep_actions: Some(pep_actions),
            recovery: None, propagation: None,
        }),
        events: vec![serde_json::json!({"audit": audit_chain.last().unwrap()})],
    });

    // ── Stage 5: State propagation (§8 step 5) ──────────────────────
    // Node B generates MODE_3 state vector and propagates to Node A.
    let sv_b5 = emit_state_vector(node_b_id, 3, 0.85, 5);
    let prop_msg = PropagationMessage {
        propagation_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        source_node_id: node_b_id.into(),
        sequence_number: 5,
        monotonic_counter: 5,
        previous_mode: 1,
        new_mode: 3,
        trigger_rule: Some("REJECT_SIGNAL".into()),
        state_vector: Some(sv_b5.clone()),
        timestamp: base_ts + 5_000_000,
    };

    let ae5 = audit::append_to_chain(
        &audit_chain, "propagation",
        &serde_json::to_string(&serde_json::json!({
            "cycle": 5, "source": node_b_id, "propagated_mode": 3,
            "propagation_id": &prop_msg.propagation_id
        })).unwrap(),
        base_ts + 5_000_000,
    );
    audit_chain.push(ae5);

    cycles.push(CycleRecord {
        cycle: 5,
        stage: "state_propagation".into(),
        description: "§8 Step 5: Node B propagates MODE_3 state vector to Node A.".into(),
        node_a: Some(NodeState {
            mode: 1, r_t: 0.1,
            state_vector: None, verification: None,
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        node_b: Some(NodeState {
            mode: 3, r_t: 0.85,
            state_vector: Some(sv_b5),
            verification: None, transition: None, pep_actions: None, recovery: None,
            propagation: Some(prop_msg),
        }),
        events: vec![serde_json::json!({"audit": audit_chain.last().unwrap()})],
    });

    // ── Stage 6: Recovery (§8 step 6) ────────────────────────────────
    // Condition #2: No recovery condition calculation.
    // "4 conditions satisfied" is ASSUMED. Deterministic playback only.
    //
    // Recovery path: MODE_3→MODE_2→MODE_1 (FP-1 satisfied: no direct MODE_3→MODE_1)

    // 6a: MODE_3→MODE_2 (partial recovery assumed)
    let te_b6a = TransitionEvent {
        previous_mode: 3,
        new_mode: 2,
        trigger_rule: "MONOTONICITY".into(),
        trigger_priority: 5,
        policy_digest_mismatch: false,
        timestamp: base_ts + 6_000_000,
    };

    let rc_b6a = RecoveryChecklist {
        risk_normalized: true,
        policy_aligned: true,
        consecutive_successes: 3,
        required_successes: 3,
        reject_ceased: true,
        all_satisfied: true,
        recovery_target: 2, // Conservative re-entry → MODE_2
        timestamp: base_ts + 6_000_000,
    };

    let ae6a = audit::append_to_chain(
        &audit_chain, "recovery",
        &serde_json::to_string(&serde_json::json!({
            "cycle": 6, "phase": "partial_recovery",
            "node_b_transition": "MODE_3→MODE_2",
            "all_4_conditions": "assumed_satisfied",
            "recovery_target": 2
        })).unwrap(),
        base_ts + 6_000_000,
    );
    audit_chain.push(ae6a);

    let pep_b6a = vec![
        PEPAction { action_type: "block_output".into(), blocked: false, detail: "MODE_2: output partially permitted".into() },
        PEPAction { action_type: "block_network".into(), blocked: true, detail: "MODE_2: network still restricted".into() },
        PEPAction { action_type: "permit_all".into(), blocked: false, detail: "MODE_2: partial restrictions".into() },
    ];

    cycles.push(CycleRecord {
        cycle: 6,
        stage: "recovery_partial".into(),
        description: "§8 Step 6a: 4 recovery conditions assumed satisfied. MODE_3→MODE_2 (FP-1 compliant).".into(),
        node_a: Some(NodeState {
            mode: 1, r_t: 0.1,
            state_vector: None, verification: None,
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        node_b: Some(NodeState {
            mode: 2, r_t: 0.3,
            state_vector: None, verification: None,
            transition: Some(te_b6a),
            pep_actions: Some(pep_b6a),
            recovery: Some(rc_b6a),
            propagation: None,
        }),
        events: vec![serde_json::json!({"audit": audit_chain.last().unwrap()})],
    });

    // 6b: MODE_2→MODE_1 (full recovery assumed, risk normalized)
    let te_b6b = TransitionEvent {
        previous_mode: 2,
        new_mode: 1,
        trigger_rule: "MONOTONICITY".into(),
        trigger_priority: 5,
        policy_digest_mismatch: false,
        timestamp: base_ts + 7_000_000,
    };

    let rc_b6b = RecoveryChecklist {
        risk_normalized: true,
        policy_aligned: true,
        consecutive_successes: 5,
        required_successes: 3,
        reject_ceased: true,
        all_satisfied: true,
        recovery_target: 1, // Full recovery → MODE_1
        timestamp: base_ts + 7_000_000,
    };

    let ae6b = audit::append_to_chain(
        &audit_chain, "recovery",
        &serde_json::to_string(&serde_json::json!({
            "cycle": 7, "phase": "full_recovery",
            "node_b_transition": "MODE_2→MODE_1",
            "all_4_conditions": "assumed_satisfied",
            "risk_normalized": true, "recovery_target": 1
        })).unwrap(),
        base_ts + 7_000_000,
    );
    audit_chain.push(ae6b);

    let pep_b6b = vec![
        PEPAction { action_type: "permit_all".into(), blocked: false, detail: "MODE_1: all paths unrestricted".into() },
    ];

    cycles.push(CycleRecord {
        cycle: 7,
        stage: "recovery_full".into(),
        description: "§8 Step 6b: Full recovery. MODE_2→MODE_1. All paths unrestricted.".into(),
        node_a: Some(NodeState {
            mode: 1, r_t: 0.1,
            state_vector: None, verification: None,
            transition: None, pep_actions: None, recovery: None, propagation: None,
        }),
        node_b: Some(NodeState {
            mode: 1, r_t: 0.1,
            state_vector: None, verification: None,
            transition: Some(te_b6b),
            pep_actions: Some(pep_b6b),
            recovery: Some(rc_b6b),
            propagation: None,
        }),
        events: vec![serde_json::json!({"audit": audit_chain.last().unwrap()})],
    });

    SimulationLog {
        metadata: SimulationMetadata {
            spec: "P-MATRIX 4.0 TechRxiv Paper §8".into(),
            description: "2-node mutual verification scenario — deterministic playback. \
                          NON-NORMATIVE. All values are synthetic.".into(),
            node_a: node_a_id.into(),
            node_b: node_b_id.into(),
            disclaimer: "This simulator uses synthetic risk values and fixed thresholds. \
                         The evaluation logic exists solely to populate required fields for \
                         protocol flow illustration. No claim is made regarding suitability \
                         for production deployment.".into(),
        },
        cycles,
    }
}
