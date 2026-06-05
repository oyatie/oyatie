---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M06-ecosystem-developer-portal
phase: P01-developer-sdk-substrate
impl_plan_id: IP-002-developer-onboarding-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion, payout-settlement-correctness, kyc-pipeline-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002-developer-onboarding-kernel-domain: developer-onboarding kernel + domain (KYC + AML state machine)

## Intent

Pure kernel + domain for developer onboarding: Developer entity, KYC state machine (pending → id-collecting → liveness-checking → sanctions-screening → approved/rejected), AML check policy, bank account verification micro-payment domain logic.

This IP advances PRD AC criteria per `microservices/developer-sdk/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-developer-sdk-developer-onboarding-kernel`
- `oya-developer-sdk-developer-onboarding-domain`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/developer-sdk/src/crates/oya-developer-sdk-developer-onboarding-kernel/src/entities.rs` | create | Developer, KycRecord, AmlCheck, BankAccount |
| `microservices/developer-sdk/src/crates/oya-developer-sdk-developer-onboarding-kernel/src/ports.rs` | create | DeveloperRepo, KycVerifier, AmlChecker, BankVerifier ports |
| `microservices/developer-sdk/src/crates/oya-developer-sdk-developer-onboarding-domain/src/kyc_state_machine.rs` | create | Pure transitions |
| `microservices/developer-sdk/src/crates/oya-developer-sdk-developer-onboarding-domain/src/aml_policy.rs` | create | OFAC + EU sanctions + FATF policy logic |

| `microservices/developer-sdk/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/developer-sdk/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KycState {
    Pending, IdCollecting, LivenessChecking, SanctionsScreening, Approved, Rejected,
}

pub fn next_state(s: KycState, evt: KycEvent) -> Result<KycState, KycError> {
    use KycState::*; use KycEvent::*;
    match (s, evt) {
        (Pending, StartIdCollection) => Ok(IdCollecting),
        (IdCollecting, IdSubmitted) => Ok(LivenessChecking),
        (LivenessChecking, LivenessPass) => Ok(SanctionsScreening),
        (SanctionsScreening, NoMatch) => Ok(Approved),
        (SanctionsScreening, Match) => Ok(Rejected),
        (LivenessChecking, LivenessFail) => Ok(Rejected),
        _ => Err(KycError::InvalidTransition),
    }
}
```

Layer assignment compliance (per ADR-0105 13-layer enum):
- `*-kernel` crates declare port traits + value types only; no dependencies on other project crates.
- `*-domain` crates implement pure domain logic; depend on `*-kernel` only.
- `*-usecase` crates orchestrate domain calls; depend on `*-kernel` + `*-domain` only.
- `*-adapter*` crates implement port traits against concrete backends; depend on `*-kernel` + `*-domain` + `*-usecase`; NEVER imported directly by `*-rest` or `*-app`.
- `*-rest` crates expose HTTP routes; depend on `*-kernel` + `*-api` + `*-usecase`.
- `*-worker` crates run long-lived loops; same dependency rules as `*-rest`.
- `*-app` crates are composition roots; the only crates allowed to wire concrete `*-adapter*` instances to `*-usecase` ports.

Port-in-kernel rule (per ADR-0064 SWEEP-I) is enforced by the `port-location` CI lane.

## Acceptance Gates

All gates must exit 0 before this IP is `verified`:

```bash
cargo check -p oya-developer-sdk-developer-onboarding-kernel --all-features
cargo build -p oya-developer-sdk-developer-onboarding-kernel --all-features
cargo clippy -p oya-developer-sdk-developer-onboarding-kernel --all-features -- -D warnings
cargo nextest run -p oya-developer-sdk-developer-onboarding-kernel --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-developer-sdk-developer-onboarding-kernel --no-deps
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=lean-a2 --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash
buck2 build //:quality-lane-registry-authority-check # lane=payout-settlement-correctness --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=kyc-pipeline-correctness --microservice developer-sdk
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_kyc_state_machine_valid_transitions` | Each spec'd transition Ok |
| `test_kyc_state_machine_invalid_rejected` | Each non-spec transition Err |
| `test_aml_policy_ofac_match_rejects` | OFAC SDN match → reject |
| `test_aml_policy_eu_sanctions_match_rejects` | EU sanctions list match → reject |
| `test_bank_verification_micropayment_amount` | Micropayment amount ≤ $1 enforced |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/developer-sdk/tests/fixtures/ip-002-developer-onboarding-kernel-domain/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- KYC state machine permits any transition outside the spec.
- AML policy passes a known OFAC match.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/developer-sdk/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/developer-sdk/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/developer-sdk/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-002-developer-onboarding-kernel-domain`
- `microservice`: `developer-sdk`
- `milestone`: `M06-ecosystem-developer-portal`
- `phase`: `P01-developer-sdk-substrate`
- `claim_paths`: every glob declared above
- `acceptance_lanes_green`: exhaustive list of CI lanes that ran and exited 0
- `test_count`: {unit, integration, e2e}
- `coverage_pct`: float
- `multispectrum_review_facets`: F1..F9 + A1..A7 + M1..M2 minimum
- `signature`: Ed25519 signing per ADR-0181

## Next IP

[`IP-003-developer-onboarding-usecase-api-adapter-rest-app`](IP-003-developer-onboarding-usecase-api-adapter-rest-app.md)

## References

- US BSA
- EU AML5
- FATF Recommendations
- OFAC SDN list
