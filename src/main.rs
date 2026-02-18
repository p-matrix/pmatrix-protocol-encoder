// main.rs
// NON-NORMATIVE VALIDATION SCHEMA
// Normative authority: 4.0 TechRxiv Paper §5~§8
// Canonical anchor: pmatrix.io/schema/4.0.json
//
// P-MATRIX 4.0 Protocol Reference Encoder CLI.
// 3 commands: emit, validate, simulate.
// FG-5: No scoring, threshold, or decision logic.

mod audit;
mod emit;
mod invariants;
mod simulate;
mod types;
mod validate;

use clap::{Parser, Subcommand};
use std::io::Read;

#[derive(Parser)]
#[command(name = "pmatrix-protocol")]
#[command(about = "P-MATRIX 4.0 Protocol Reference Encoder CLI (Non-normative)")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Emit a synthetic §5.1 StateVector JSON
    Emit {
        /// Node identifier
        #[arg(long, default_value = "node-alpha")]
        node_id: String,

        /// Operating mode (1=unrestricted, 2=restricted, 3=isolated)
        #[arg(long, default_value_t = 1)]
        mode: u8,

        /// Risk score r(t) in [0.0, 1.0]
        #[arg(long, default_value_t = 0.1)]
        r_t: f64,

        /// Cycle ID
        #[arg(long, default_value_t = 1)]
        cycle_id: u64,
    },

    /// Validate a JSON message against §5 schema + §7 invariants
    Validate {
        /// JSON file path (reads stdin if omitted)
        #[arg(long)]
        file: Option<String>,

        /// Message type to validate
        #[arg(long, default_value = "state_vector")]
        msg_type: String,
    },

    /// Run §8 2-node deterministic scenario (playback only)
    Simulate,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Emit { node_id, mode, r_t, cycle_id } => {
            cmd_emit(&node_id, mode, r_t, cycle_id);
        }
        Commands::Validate { file, msg_type } => {
            cmd_validate(file, &msg_type);
        }
        Commands::Simulate => {
            cmd_simulate();
        }
    }
}

fn cmd_emit(node_id: &str, mode: u8, r_t: f64, cycle_id: u64) {
    if !types::VALID_MODES.contains(&mode) {
        eprintln!("Error: mode must be 1, 2, or 3");
        std::process::exit(1);
    }
    if r_t.is_nan() || !(0.0..=1.0).contains(&r_t) {
        eprintln!("Error: r_t must be in [0.0, 1.0]");
        std::process::exit(1);
    }

    let sv = emit::emit_state_vector(node_id, mode, r_t, cycle_id);

    // Validate the emitted vector
    let errors = validate::validate_state_vector(&sv);
    if !errors.is_empty() {
        eprintln!("Internal error: emitted vector failed validation:");
        for e in &errors {
            eprintln!("  - {}", e);
        }
        std::process::exit(1);
    }

    let json = serde_json::to_string_pretty(&sv).unwrap();
    println!("{}", json);
}

fn cmd_validate(file: Option<String>, msg_type: &str) {
    let input = match file {
        Some(path) => std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", path, e);
            std::process::exit(1);
        }),
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("Error reading stdin: {}", e);
                std::process::exit(1);
            });
            buf
        }
    };

    let mut all_errors: Vec<String> = Vec::new();

    // Structural invariants (FP-3/4, PI-2/4) — always checked
    all_errors.extend(invariants::check_all_structural_invariants());

    match msg_type {
        "state_vector" => {
            match serde_json::from_str::<types::StateVector>(&input) {
                Ok(sv) => {
                    all_errors.extend(validate::validate_state_vector(&sv));
                }
                Err(e) => all_errors.push(format!("JSON parse error: {}", e)),
            }
        }
        "verification_result" => {
            match serde_json::from_str::<types::VerificationResult>(&input) {
                Ok(vr) => all_errors.extend(validate::validate_verification_result(&vr)),
                Err(e) => all_errors.push(format!("JSON parse error: {}", e)),
            }
        }
        "transition_event" => {
            match serde_json::from_str::<types::TransitionEvent>(&input) {
                Ok(te) => {
                    all_errors.extend(validate::validate_transition_event(&te));
                    all_errors.extend(invariants::check_all_transition_invariants(&te));
                }
                Err(e) => all_errors.push(format!("JSON parse error: {}", e)),
            }
        }
        "recovery_checklist" => {
            match serde_json::from_str::<types::RecoveryChecklist>(&input) {
                Ok(rc) => all_errors.extend(validate::validate_recovery_checklist(&rc)),
                Err(e) => all_errors.push(format!("JSON parse error: {}", e)),
            }
        }
        "pep_action" => {
            match serde_json::from_str::<types::PEPAction>(&input) {
                Ok(pa) => all_errors.extend(validate::validate_pep_action(&pa)),
                Err(e) => all_errors.push(format!("JSON parse error: {}", e)),
            }
        }
        "propagation_message" => {
            match serde_json::from_str::<types::PropagationMessage>(&input) {
                Ok(pm) => all_errors.extend(validate::validate_propagation_message(&pm)),
                Err(e) => all_errors.push(format!("JSON parse error: {}", e)),
            }
        }
        "audit_entry" => {
            match serde_json::from_str::<types::AuditEntry>(&input) {
                Ok(ae) => all_errors.extend(validate::validate_audit_entry(&ae)),
                Err(e) => all_errors.push(format!("JSON parse error: {}", e)),
            }
        }
        "audit_chain" => {
            match serde_json::from_str::<Vec<types::AuditEntry>>(&input) {
                Ok(entries) => {
                    for (i, ae) in entries.iter().enumerate() {
                        let ae_errors = validate::validate_audit_entry(ae);
                        for e in ae_errors {
                            all_errors.push(format!("[{}] {}", i, e));
                        }
                    }
                    all_errors.extend(invariants::check_pi3_append_only_audit(&entries));
                }
                Err(e) => all_errors.push(format!("JSON parse error: {}", e)),
            }
        }
        _ => {
            eprintln!("Unknown message type: '{}'. Valid: state_vector, verification_result, \
                       transition_event, recovery_checklist, pep_action, propagation_message, \
                       audit_entry, audit_chain", msg_type);
            std::process::exit(1);
        }
    }

    if all_errors.is_empty() {
        println!("PASS");
    } else {
        eprintln!("FAIL ({} error(s)):", all_errors.len());
        for e in &all_errors {
            eprintln!("  - {}", e);
        }
        std::process::exit(1);
    }
}

fn cmd_simulate() {
    let log = simulate::run_simulation();
    let json = serde_json::to_string_pretty(&log).unwrap();
    println!("{}", json);
}
