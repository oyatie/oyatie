---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-014-branch-protection-and-hyperscaler-gates
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014-branch-protection-and-hyperscaler-gates: branch-protection + hyperscaler-gates HG-PAS registration

## Intent

Update `.github/branch-protection.yaml` with vetting + per-plugin permission + rate-limit BLOCKER lanes on dev + staging; register pattern protection for `release/plugin-app-store/{staging,production}`; register HG-PAS in `/specs/hyperscaler-gates.json` per ADR-0123. **DEFERRED to parent-wiring-todo per scope-lock; this IP authors the diff preview only.**

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `.github/branch-protection.yaml (DEFERRED)`
- `/specs/hyperscaler-gates.json (DEFERRED)`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `evidence/parent-wiring-todo-plugin-app-store-batch.json#hg-pas-registration` | TODO | HG-PAS hyperscaler-gate row to register |
| `evidence/parent-wiring-todo-plugin-app-store-batch.json#branch-protection-required-checks` | TODO | list of required status checks to add |
| `evidence/parent-wiring-todo-plugin-app-store-batch.json#release-pattern-protection` | TODO | release/plugin-app-store/{staging,production} pattern |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
# scope-locked: this IP emits the diff preview only; parent-wiring agent applies
# DIFF PREVIEW for .github/branch-protection.yaml:
#   dev.required_status_checks += [
#     oya-governance-vetting-pipeline-correctness,
#     oya-governance-per-plugin-permission-enforcement,
#     oya-governance-per-plugin-rate-limit-correctness
#   ]
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
cargo check -p .github/branch-protection.yaml (DEFERRED) --all-features
cargo build -p .github/branch-protection.yaml (DEFERRED) --all-features
cargo clippy -p .github/branch-protection.yaml (DEFERRED) --all-features -- -D warnings
cargo nextest run -p .github/branch-protection.yaml (DEFERRED) --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p .github/branch-protection.yaml (DEFERRED) --no-deps
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
| `branch-protection-yaml-schema-valid` | After parent-wiring agent applies, YAML parses |
| `hyperscaler-gates-json-schema-valid` | After parent-wiring agent applies, JSON parses |
| `hg-pas-gate-discoverable` | oya gate validate hyperscaler-maturity-claims lists HG-PAS |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-014-branch-protection-and-hyperscaler-gates/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Parent-wiring agent applies the diff and CI lane fails on schema validation.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-014-branch-protection-and-hyperscaler-gates`
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

[`IP-015-discovery-install-leptos-app`](IP-015-discovery-install-leptos-app.md)

## References

- ADR-0123
- ADR-0213
- /specs/hyperscaler-gates.json
