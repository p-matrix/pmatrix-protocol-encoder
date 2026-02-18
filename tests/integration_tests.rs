// tests/integration_tests.rs
// NON-NORMATIVE VALIDATION SCHEMA
// Normative authority: 4.0 TechRxiv Paper §5~§8

use std::process::{Command, Output};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pmatrix-protocol"))
}

fn validate_json(msg_type: &str, json: &str) -> Output {
    let tmp = std::env::temp_dir().join(format!(
        "pmt_{}_{}_{}.json", msg_type, std::process::id(),
        std::thread::current().name().unwrap_or("x").replace("::", "_")
    ));
    std::fs::write(&tmp, json).unwrap();
    let out = cli()
        .args(["validate", "--msg-type", msg_type, "--file", tmp.to_str().unwrap()])
        .output()
        .unwrap();
    std::fs::remove_file(tmp).ok();
    out
}

fn sim() -> serde_json::Value {
    let o = cli().args(["simulate"]).output().unwrap();
    assert!(o.status.success());
    serde_json::from_slice(&o.stdout).unwrap()
}

// ── emit (9) ─────────────────────────────────────────────────────────

#[test]
fn emit_default() {
    let o = cli().args(["emit"]).output().unwrap();
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["risk_info"]["mode"], "A+1");
    assert_eq!(v["risk_info"]["risk_level"], "L1");
}

#[test]
fn emit_mode_1() {
    let o = cli().args(["emit", "--mode", "1", "--r-t", "0.2"]).output().unwrap();
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["risk_info"]["r_t"], 0.2);
}

#[test]
fn emit_mode_2() {
    let o = cli().args(["emit", "--mode", "2"]).output().unwrap();
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["risk_info"]["mode"], "A-1");
    assert_eq!(v["risk_info"]["risk_level"], "L3");
}

#[test]
fn emit_mode_3() {
    let o = cli().args(["emit", "--mode", "3"]).output().unwrap();
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["risk_info"]["mode"], "A-0");
    assert_eq!(v["risk_info"]["risk_level"], "L5");
}

#[test]
fn emit_invalid_mode() {
    assert!(!cli().args(["emit", "--mode", "5"]).output().unwrap().status.success());
}

#[test]
fn emit_invalid_r_t() {
    assert!(!cli().args(["emit", "--r-t", "1.5"]).output().unwrap().status.success());
}

#[test]
fn emit_custom_node() {
    let o = cli().args(["emit", "--node-id", "my-node"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["lifecycle_info"]["node_id"], "my-node");
}

#[test]
fn emit_r_t_zero() {
    assert!(cli().args(["emit", "--r-t", "0.0"]).output().unwrap().status.success());
}

#[test]
fn emit_r_t_one() {
    assert!(cli().args(["emit", "--r-t", "1.0"]).output().unwrap().status.success());
}

// ── validate (12) ────────────────────────────────────────────────────

#[test]
fn validate_emitted_sv() {
    let e = cli().args(["emit"]).output().unwrap();
    let o = validate_json("state_vector", std::str::from_utf8(&e.stdout).unwrap());
    assert!(o.status.success());
}

#[test]
fn validate_bad_json() {
    assert!(!validate_json("state_vector", "not json").status.success());
}

#[test]
fn validate_fp1_violation() {
    let o = validate_json("transition_event",
        r#"{"previous_mode":3,"new_mode":1,"trigger_rule":"MONOTONICITY","trigger_priority":5,"policy_digest_mismatch":false,"timestamp":100}"#);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("FP-1"));
}

#[test]
fn validate_fp2_wrong_priority() {
    let o = validate_json("transition_event",
        r#"{"previous_mode":1,"new_mode":3,"trigger_rule":"REJECT_SIGNAL","trigger_priority":3,"policy_digest_mismatch":false,"timestamp":100}"#);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("priority"));
}

#[test]
fn validate_valid_transition() {
    assert!(validate_json("transition_event",
        r#"{"previous_mode":1,"new_mode":3,"trigger_rule":"REJECT_SIGNAL","trigger_priority":1,"policy_digest_mismatch":false,"timestamp":100}"#).status.success());
}

#[test]
fn validate_recovery_inconsistency() {
    let o = validate_json("recovery_checklist",
        r#"{"risk_normalized":true,"policy_aligned":true,"consecutive_successes":1,"required_successes":3,"reject_ceased":true,"all_satisfied":true,"recovery_target":1,"timestamp":100}"#);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("inconsistency"));
}

#[test]
fn validate_pep_valid() {
    assert!(validate_json("pep_action", r#"{"action_type":"block_output","blocked":true,"detail":"t"}"#).status.success());
}

#[test]
fn validate_pep_invalid() {
    assert!(!validate_json("pep_action", r#"{"action_type":"bad","blocked":true,"detail":"t"}"#).status.success());
}

#[test]
fn validate_vr_valid() {
    assert!(validate_json("verification_result", r#"{"status":"passed","peer_node_id":"n","verified_at":1,"details":"d"}"#).status.success());
}

#[test]
fn validate_vr_invalid() {
    assert!(!validate_json("verification_result", r#"{"status":"bad","peer_node_id":"n","verified_at":1,"details":"d"}"#).status.success());
}

#[test]
fn validate_unknown_type() {
    assert!(!validate_json("unknown", "{}").status.success());
}

#[test]
fn validate_all_6_statuses() {
    for s in ["passed", "no_peer", "failed_freshness", "failed_integrity", "failed_risk_range", "failed_policy_mismatch"] {
        let j = format!(r#"{{"status":"{}","peer_node_id":"n","verified_at":1,"details":"d"}}"#, s);
        assert!(validate_json("verification_result", &j).status.success(), "status '{}' should be valid", s);
    }
}

// ── simulate (14) ────────────────────────────────────────────────────

#[test]
fn sim_valid_json() { assert!(sim()["cycles"].is_array()); }

#[test]
fn sim_7_cycles() { assert_eq!(sim()["cycles"].as_array().unwrap().len(), 7); }

#[test]
fn sim_s1_normal() {
    let l = sim();
    let c = &l["cycles"][0];
    assert_eq!(c["stage"], "normal_exchange");
    assert_eq!(c["node_a"]["mode"], 1);
    assert_eq!(c["node_b"]["mode"], 1);
}

#[test]
fn sim_s2_risk() {
    let l = sim();
    assert_eq!(l["cycles"][1]["stage"], "risk_escalation");
    assert_eq!(l["cycles"][1]["node_a"]["verification"]["status"], "failed_risk_range");
}

#[test]
fn sim_s3_reject() {
    let l = sim();
    let c = &l["cycles"][2];
    assert_eq!(c["node_b"]["mode"], 3);
    assert_eq!(c["node_b"]["transition"]["trigger_rule"], "REJECT_SIGNAL");
    assert_eq!(c["node_b"]["transition"]["trigger_priority"], 1);
}

#[test]
fn sim_s4_enforcement() {
    let l = sim();
    let c = &l["cycles"][3];
    assert_eq!(c["stage"], "enforcement");
    let peps = c["node_b"]["pep_actions"].as_array().unwrap();
    assert_eq!(peps.len(), 5);
    for p in peps { assert_eq!(p["blocked"], true); }
}

#[test]
fn sim_s5_propagation() {
    let l = sim();
    let c = &l["cycles"][4];
    assert_eq!(c["stage"], "state_propagation");
    assert!(c["node_b"]["propagation"]["propagation_id"].is_string());
}

#[test]
fn sim_s6_recovery_fp1_compliant() {
    let l = sim();
    // MODE_3→MODE_2
    assert_eq!(l["cycles"][5]["node_b"]["transition"]["previous_mode"], 3);
    assert_eq!(l["cycles"][5]["node_b"]["transition"]["new_mode"], 2);
    // MODE_2→MODE_1
    assert_eq!(l["cycles"][6]["node_b"]["transition"]["previous_mode"], 2);
    assert_eq!(l["cycles"][6]["node_b"]["transition"]["new_mode"], 1);
}

#[test]
fn sim_s6_recovery_assumed() {
    let l = sim();
    assert_eq!(l["cycles"][5]["node_b"]["recovery"]["all_satisfied"], true);
    assert_eq!(l["cycles"][5]["node_b"]["recovery"]["recovery_target"], 2);
}

#[test]
fn sim_s7_full_recovery() {
    let l = sim();
    assert_eq!(l["cycles"][6]["node_b"]["recovery"]["recovery_target"], 1);
    assert_eq!(l["cycles"][6]["node_b"]["mode"], 1);
}

#[test]
fn sim_audit_in_events() {
    let l = sim();
    for c in l["cycles"].as_array().unwrap() {
        assert!(!c["events"].as_array().unwrap().is_empty());
    }
}

#[test]
fn sim_metadata_disclaimer() {
    assert!(sim()["metadata"]["disclaimer"].as_str().unwrap().contains("synthetic"));
}

#[test]
fn sim_deterministic() {
    let o1 = cli().args(["simulate"]).output().unwrap();
    let o2 = cli().args(["simulate"]).output().unwrap();
    assert_eq!(o1.stdout, o2.stdout);
}

#[test]
fn sim_audit_chain_integrity() {
    let l = sim();
    let mut prev = String::new();
    for (i, c) in l["cycles"].as_array().unwrap().iter().enumerate() {
        for ev in c["events"].as_array().unwrap() {
            if let Some(a) = ev.get("audit") {
                let ph = a["prev_hash"].as_str().unwrap();
                let eh = a["entry_hash"].as_str().unwrap();
                if i == 0 && prev.is_empty() {
                    assert_eq!(ph, "0000000000000000000000000000000000000000000000000000000000000000");
                } else if !prev.is_empty() {
                    assert_eq!(ph, prev, "chain broken at cycle {}", i);
                }
                prev = eh.to_string();
            }
        }
    }
    assert!(!prev.is_empty());
}

// ── M-1 보완: invariants 커버리지 (5) ────────────────────────────────

#[test]
fn invariant_fp3_separation_passes() {
    // FP-3/PI-2: compile-time guarantee, runtime check always None
    let out = validate_json("state_vector",
        &String::from_utf8_lossy(&cli().args(["emit"]).output().unwrap().stdout));
    assert!(out.status.success()); // structural invariants run on every validate
}

#[test]
fn invariant_fp4_all_5_effect_paths() {
    // FP-4 checks 5 paths exist in VALID_PEP_ACTION_TYPES
    for at in ["block_output","block_tool_call","block_network","block_file_write","block_service_call"] {
        let j = format!(r#"{{"action_type":"{}","blocked":true,"detail":"t"}}"#, at);
        assert!(validate_json("pep_action", &j).status.success(), "FP-4: {} should be valid", at);
    }
}

#[test]
fn invariant_pi3_audit_chain_valid() {
    // Build a 3-entry chain via simulate, extract audit entries, validate as chain
    let sim_out = cli().args(["simulate"]).output().unwrap();
    let log: serde_json::Value = serde_json::from_slice(&sim_out.stdout).unwrap();
    let mut chain = Vec::new();
    for c in log["cycles"].as_array().unwrap() {
        for ev in c["events"].as_array().unwrap() {
            if let Some(a) = ev.get("audit") { chain.push(a.clone()); }
        }
    }
    let chain_json = serde_json::to_string(&chain).unwrap();
    let out = validate_json("audit_chain", &chain_json);
    assert!(out.status.success(), "PI-3 audit chain should pass: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn invariant_pi3_audit_chain_broken() {
    // Tamper with prev_hash to break chain
    let sim_out = cli().args(["simulate"]).output().unwrap();
    let log: serde_json::Value = serde_json::from_slice(&sim_out.stdout).unwrap();
    let mut chain = Vec::new();
    for c in log["cycles"].as_array().unwrap() {
        for ev in c["events"].as_array().unwrap() {
            if let Some(a) = ev.get("audit") { chain.push(a.clone()); }
        }
    }
    // Tamper entry [1] prev_hash
    if chain.len() >= 2 {
        chain[1].as_object_mut().unwrap().insert("prev_hash".into(), serde_json::json!("ff".repeat(32)));
    }
    let chain_json = serde_json::to_string(&chain).unwrap();
    let out = validate_json("audit_chain", &chain_json);
    assert!(!out.status.success());
}

#[test]
fn invariant_fp5_reject_must_escalate() {
    // REJECT_SIGNAL with de-escalation should fail FP-5
    let out = validate_json("transition_event",
        r#"{"previous_mode":2,"new_mode":1,"trigger_rule":"REJECT_SIGNAL","trigger_priority":1,"policy_digest_mismatch":false,"timestamp":100}"#);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("FP-5") || stderr.contains("FP-2") || stderr.contains("priority"));
}
