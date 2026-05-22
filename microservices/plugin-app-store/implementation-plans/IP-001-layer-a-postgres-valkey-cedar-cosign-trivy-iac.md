---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-001-layer-a-postgres-valkey-cedar-cosign-trivy-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001-layer-a-postgres-valkey-cedar-cosign-trivy-iac: Layer-A Postgres + Valkey + Cedar evaluator + Cosign + Trivy + Wasmtime IaC

## Intent

Stand up the Layer-A substrate dependencies as Helm/Kustomize charts under `microservices/plugin-app-store/iac/helm/`. Provides authoritative state (Postgres), ephemeral lease + rate-limit (Valkey), Cedar policy evaluator binding, Cosign verification binary, Trivy vulnerability scanner, and Wasmtime runtime for per-plugin sandbox execution.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `microservices/plugin-app-store/iac/helm/postgres`
- `microservices/plugin-app-store/iac/helm/valkey`
- `microservices/plugin-app-store/iac/helm/cedar-evaluator`
- `microservices/plugin-app-store/iac/helm/cosign`
- `microservices/plugin-app-store/iac/helm/trivy`
- `microservices/plugin-app-store/iac/helm/wasmtime-runtime`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/iac/helm/postgres/Chart.yaml` | create | Postgres 16 LTS Helm chart |
| `microservices/plugin-app-store/iac/helm/postgres/values.yaml` | create | tenant-isolated DB per Cilium NetworkPolicy |
| `microservices/plugin-app-store/iac/helm/postgres/templates/statefulset.yaml` | create | 3-replica statefulset with pgbackrest sidecar |
| `microservices/plugin-app-store/iac/helm/valkey/Chart.yaml` | create | Valkey 8.1 Helm chart (Sentinel HA) |
| `microservices/plugin-app-store/iac/helm/cedar-evaluator/Chart.yaml` | create | Cedar 4.x evaluator sidecar |
| `microservices/plugin-app-store/iac/helm/cosign/Chart.yaml` | create | Cosign 2.x verification binary |
| `microservices/plugin-app-store/iac/helm/trivy/Chart.yaml` | create | Trivy 0.50.x scanner pod |
| `microservices/plugin-app-store/iac/helm/wasmtime-runtime/Chart.yaml` | create | Wasmtime engine pod pool |
| `microservices/plugin-app-store/iac/helm/wasmtime-runtime/templates/deployment.yaml` | create | per-tenant Wasmtime engine deployment |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
# Chart.yaml shape (Postgres example)
apiVersion: v2
name: plugin-app-store-postgres
description: Plugin-app-store authoritative state store
version: 0.1.0
appVersion: "16.0"
type: application
dependencies:
  - name: postgres-operator
    version: 1.10.x
    repository: oci://oyatie-internal/charts
maintainers:
  - name: axis-ecosystem
    email: axis-ecosystem@oyatie.dev
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
cargo check -p microservices/plugin-app-store/iac/helm/postgres --all-features
cargo build -p microservices/plugin-app-store/iac/helm/postgres --all-features
cargo clippy -p microservices/plugin-app-store/iac/helm/postgres --all-features -- -D warnings
cargo nextest run -p microservices/plugin-app-store/iac/helm/postgres --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p microservices/plugin-app-store/iac/helm/postgres --no-deps
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
| `helm-install-postgres-chart` | Chart renders without error; statefulset reconciles |
| `helm-test-valkey-sentinel-failover` | Sentinel promotes replica within 10s on primary kill |
| `helm-test-cedar-evaluator-policy-evaluation` | Cedar evaluator returns Allow/Deny for canonical test policy |
| `helm-test-cosign-verify` | Cosign verifies signed artifact against expected key |
| `helm-test-trivy-scan` | Trivy detects known CVE in test fixture image |
| `helm-test-wasmtime-runtime-cold-start` | Wasmtime engine cold-starts a no-op WASM in ≤ 300ms p99 |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-001-layer-a-postgres-valkey-cedar-cosign-trivy-iac/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Postgres replication lag exceeds 5s in steady state.
- Valkey Sentinel split-brain detected.
- Cedar evaluator returns Allow for default-deny policy.
- Cosign verifies an unsigned artifact (false-pass).
- Trivy misses a known CVE in test fixture.
- Wasmtime cold-start exceeds 1s p99.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-001-layer-a-postgres-valkey-cedar-cosign-trivy-iac`
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

[`IP-002-plugin-catalog-kernel-domain`](IP-002-plugin-catalog-kernel-domain.md)

## References

- ADR-0147 (Wasmtime sandbox baseline)
- ADR-0181 (Cosign signing)
- ADR-0200 (Wasmtime canonical)
- ADR-0202 (GitOps IaC three-tier)
- Postgres 16 LTS — postgresql.org/docs/16
- Valkey 8.1 — valkey.io/docs
- Cedar 4.x — cedarpolicy.com/docs
- Trivy — github.com/aquasecurity/trivy
