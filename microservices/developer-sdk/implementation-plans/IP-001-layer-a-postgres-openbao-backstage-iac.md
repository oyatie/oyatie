---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M06-ecosystem-developer-portal
phase: P01-developer-sdk-substrate
impl_plan_id: IP-001-layer-a-postgres-openbao-backstage-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001-layer-a-postgres-openbao-backstage-iac: Layer-A Postgres + OpenBao + Backstage IaC

## Intent

Stand up Layer-A substrate: Postgres for onboarding + payout + tax-form state; OpenBao for signing key issuance; Backstage for dev portal.

This IP advances PRD AC criteria per `microservices/developer-sdk/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `microservices/developer-sdk/iac/helm/postgres`
- `microservices/developer-sdk/iac/helm/openbao`
- `microservices/developer-sdk/iac/helm/backstage`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/developer-sdk/iac/helm/postgres/Chart.yaml` | create | Postgres 16 |
| `microservices/developer-sdk/iac/helm/openbao/Chart.yaml` | create | OpenBao secrets engine |
| `microservices/developer-sdk/iac/helm/backstage/Chart.yaml` | create | Backstage dev portal |

| `microservices/developer-sdk/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/developer-sdk/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
# Helm charts per workflow-engine IP-001 pattern
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
cargo check -p microservices/developer-sdk/iac/helm/postgres --all-features
cargo build -p microservices/developer-sdk/iac/helm/postgres --all-features
cargo clippy -p microservices/developer-sdk/iac/helm/postgres --all-features -- -D warnings
cargo nextest run -p microservices/developer-sdk/iac/helm/postgres --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p microservices/developer-sdk/iac/helm/postgres --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate port-location --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash

```

## Test Plan

| Test | Verifies |
|---|---|
| `helm-install-developer-sdk-postgres` | Chart renders + reconciles |
| `helm-install-openbao-secrets-engine-enabled` | PKI + transit engines mounted |
| `helm-install-backstage-ui-reachable` | /health responds 200 |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/developer-sdk/tests/fixtures/ip-001-layer-a-postgres-openbao-backstage-iac/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- OpenBao seals on first start (must be initialized + unsealed via cloud-secrets µservice handoff).

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/developer-sdk/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/developer-sdk/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/developer-sdk/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-001-layer-a-postgres-openbao-backstage-iac`
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

[`IP-002-developer-onboarding-kernel-domain`](IP-002-developer-onboarding-kernel-domain.md)

## References

- ADR-0202
- ADR-0170 (Backstage)
- OpenBao docs
