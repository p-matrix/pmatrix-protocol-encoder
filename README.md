# pmatrix-protocol-encoder

**P-MATRIX Protocol Reference Encoder** — Non-normative schema validation tool.

> **NON-NORMATIVE.** Normative authority: P-MATRIX: A Mutual Verification Protocol for Runtime Governance Across Autonomous Nodes (DOI pending — Zenodo) §5–§8.  
> Canonical anchor: `pmatrix.io/schema/4.0.json`

## Overview

Rust CLI that validates P-MATRIX mutual verification protocol messages against §5 schema definitions and §7 protocol invariants. Produces synthetic state vectors and replays the §8 two-node mutual verification scenario.

**This encoder is intentionally non-normative.** It demonstrates schema conformance only and carries no authority over how runtime state values are computed in production systems.

## Commands

| Command | Description | Paper §  |
|---------|-------------|----------|
| `emit`  | Generate synthetic StateVector JSON | §5.1 |
| `validate` | Validate JSON against §5 schema + invariants | §5–§7 |
| `simulate` | Replay §8 two-node scenario (deterministic) | §8 |

## Usage

```bash
# Emit a synthetic state vector
pmatrix-protocol emit --node-id node-alpha --mode 1 --r-t 0.15

# Validate (file or stdin)
pmatrix-protocol emit | pmatrix-protocol validate --msg-type state_vector
pmatrix-protocol validate --msg-type transition_event --file event.json

# Run §8 scenario
pmatrix-protocol simulate
pmatrix-protocol simulate | jq '.cycles[].stage'
```

### Message types for validate

`state_vector`, `verification_result`, `transition_event`, `recovery_checklist`, `pep_action`, `propagation_message`, `audit_entry`, `audit_chain`

## Schema Coverage

| Section | Struct | Builder | Validator |
|---------|--------|---------|-----------|
| §5.1 State Vector | `StateVector` | `emit` | ✅ |
| §5.2 Verification | `VerificationResult` | — | ✅ |
| §5.3 Transition | `TransitionEvent` | — | ✅ + FP-1/FP-2/FP-5 |
| §5.4 Recovery | `RecoveryChecklist` | — | ✅ |
| §5.5 PEP | `PEPAction` | — | ✅ |
| §5.6 Propagation | `PropagationMessage` | — | ✅ |
| §7 PI-3 Audit | `AuditEntry` | `audit.rs` | ✅ (entry + chain) |

### Implementation Notes (§5.1 Private Boundary)

The CLI enforces specific validation schemes for illustration. The paper leaves these implementation-private:

- **policy_digest**: CLI enforces 64-char lowercase hex (SHA-256). The paper requires presence only; hash algorithm is implementation-private.
- **freshness**: CLI requires at least one of timestamp/nonce/sequence to be non-zero/non-empty. The paper requires "at least one of: timestamp, nonce, sequence number, or authenticated monotonic counter."
- **integrity**: CLI requires at least one of signature/signature_algo to be non-empty. The paper requires "at least one of: digital signature, MAC, or attestation token."

## Invariants

| Property | Checked | Method |
|----------|:-------:|--------|
| FP-1 Monotonic Escalation | ✅ | MODE_3→MODE_1 blocked |
| FP-2 Reject Priority | ✅ | priority=1 enforced |
| FP-3 V/D Separation | ✅ | Rust type system (compile-time) |
| FP-4 Non-bypass | ✅ | 5 effect paths schema check |
| FP-5 Safety > Liveness | ✅ | reject→escalate |
| PI-1 Reject Dominance | ✅ | via FP-2 |
| PI-2 V/D Separation | ✅ | Rust type system (compile-time) |
| PI-3 Append-Only Audit | ✅ | SHA-256 chain + `audit_chain` validate |
| PI-4 Non-bypass | ✅ | via FP-4 |

> **FP-3/PI-2 (Verification–Decision Separation)** is guaranteed by the Rust type system at compile time: `VerificationResult` and `TransitionEvent` are distinct structs with no mutual access paths. Schema-level runtime verification is not applicable.

## Build

```bash
cargo build --release
cargo test
```

## License

Apache-2.0. Copyright © 2026 Dong Hun Lee.

## References

- pmatrix-protocol-encoder: Reference encoder for mutual verification protocol schema conformance. https://github.com/p-matrix/pmatrix-protocol-encoder
- P-MATRIX: A Mutual Verification Protocol for Runtime Governance Across Autonomous Nodes, §5–§8 (normative authority)
- pmatrix-encoder (pattern reference)

