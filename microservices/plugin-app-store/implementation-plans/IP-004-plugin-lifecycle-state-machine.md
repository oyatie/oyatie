---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-004-plugin-lifecycle-state-machine
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004-plugin-lifecycle-state-machine: plugin-lifecycle state machine (draft→submitted→vetting→published→deprecated→retired + revoked)

## Intent

Implement the version-level state machine governing every plugin's transition through lifecycle. Pure transition logic in domain; persistence in adapter-postgres; orchestration in usecase. State is event-sourced; every transition emits an audit-chain seal event.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-plugin-lifecycle-kernel`
- `oya-plugin-app-store-plugin-lifecycle-domain`
- `oya-plugin-app-store-plugin-lifecycle-usecase`
- `oya-plugin-app-store-plugin-lifecycle-api`
- `oya-plugin-app-store-plugin-lifecycle-adapter-postgres`
- `oya-plugin-app-store-plugin-lifecycle-worker`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-lifecycle-kernel/src/state.rs` | create | LifecycleState enum + Transition enum |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-lifecycle-kernel/src/ports.rs` | create | LifecycleStore, EventEmitter ports |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-lifecycle-domain/src/transitions.rs` | create | pure transition table: (state, event) -> Result<new_state> |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-lifecycle-usecase/src/orchestrator.rs` | create | PluginLifecycleOrchestrator |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-lifecycle-adapter-postgres/migrations/V1__lifecycle_events.sql` | create | lifecycle_events event log table |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-lifecycle-worker/src/main.rs` | create | background reaper: deprecated → retired transitions |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleState {
    Draft, Submitted, Vetting, Approved, Rejected, Published, Deprecated, Retired, Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transition {
    Submit, VettingPass, VettingReject, Publish, Deprecate, Retire, Revoke, ReSubmit,
}

pub fn next_state(current: LifecycleState, t: Transition) -> Result<LifecycleState, TransitionError> {
    use LifecycleState::*;
    use Transition::*;
    match (current, t) {
        (Draft, Submit) => Ok(Submitted),
        (Submitted, VettingPass) => Ok(Approved),
        (Submitted, VettingReject) => Ok(Rejected),
        (Approved, Publish) => Ok(Published),
        (Published, Deprecate) => Ok(Deprecated),
        (Deprecated, Retire) => Ok(Retired),
        (_, Revoke) if current != Retired => Ok(Revoked),
        (Rejected, ReSubmit) => Ok(Submitted),
        _ => Err(TransitionError::InvalidTransition { from: current, t }),
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
cargo check -p oya-plugin-app-store-plugin-lifecycle-kernel --all-features
cargo build -p oya-plugin-app-store-plugin-lifecycle-kernel --all-features
cargo clippy -p oya-plugin-app-store-plugin-lifecycle-kernel --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-plugin-lifecycle-kernel --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-plugin-lifecycle-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate port-location --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash

```

## Test Plan

| Test | Verifies |
|---|---|
| `test_valid_transitions` | Each (state, transition) in spec returns Ok |
| `test_invalid_transitions_rejected` | Each (state, transition) NOT in spec returns InvalidTransition |
| `test_revoke_from_any_non_terminal` | Revoke valid from any state except Retired |
| `test_retire_terminal` | No transitions valid from Retired |
| `test_event_sourced_replay_byte_equal` | Replay event log produces byte-equal final state |
| `test_audit_chain_seal_per_transition` | Every transition emits a seal event |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-004-plugin-lifecycle-state-machine/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Any transition not specified in the table accepted at runtime.
- Event log replay produces divergent state.
- Audit-chain seal missing for any transition.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-004-plugin-lifecycle-state-machine`
- `microservice`: `plugin-app-store`
- `milestone`: `M04-ecosystem-substrate`
- `phase`: `P01-plugin-app-store-substrate`
- `claim_paths`: every glob declared above
- `acceptance_lanes_green`: exhaustive list of CI lanes that ran and exited 0
- `test_count`: {unit, integration, e2e}
- `coverage_pct`: float
- `multispectrum_review_facets`: F1..F9 + A1..A7 + M1..M2 minimum
- `signature`: Ed25519 signing per ADR-0181

## Next IP

[`IP-005-plugin-install-kernel-domain-usecase`](IP-005-plugin-install-kernel-domain-usecase.md)

## References

- ADR-0110 (ChangeSet state machine — same shape)
- ADR-0028 (audit chain)
- PRD §plugin-lifecycle
