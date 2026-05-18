---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-013-observability-slo-manifests
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013-observability-slo-manifests: plugin-app-store OpenSLO manifests + observability self-SLOs

## Intent

Author 9 OpenSLO manifests under `microservices/plugin-app-store/slos/`; each rendered into the observability µservice's promotion gate per ADR-0139.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `microservices/plugin-app-store/slos/*.openslo.yaml`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/slos/catalog-browse-latency.openslo.yaml` | create | p95 ≤ 200ms |
| `microservices/plugin-app-store/slos/catalog-browse-availability.openslo.yaml` | create | 99.99% |
| `microservices/plugin-app-store/slos/plugin-install-availability.openslo.yaml` | create | 99.95% |
| `microservices/plugin-app-store/slos/plugin-install-latency.openslo.yaml` | create | p99 ≤ 15s |
| `microservices/plugin-app-store/slos/plugin-revoke-latency.openslo.yaml` | create | p99 ≤ 30s |
| `microservices/plugin-app-store/slos/vetting-pipeline-throughput.openslo.yaml` | create | 95% decisions/submitted hourly |
| `microservices/plugin-app-store/slos/vetting-pipeline-correctness.openslo.yaml` | create | no false-approves |
| `microservices/plugin-app-store/slos/per-plugin-rate-limit-correctness.openslo.yaml` | create | no bypasses |
| `microservices/plugin-app-store/slos/subscription-billing-correctness.openslo.yaml` | create | byte-equal aggregation |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
apiVersion: openslo/v1
kind: SLO
metadata:
  name: oya-plugin-app-store-catalog-browse-latency
spec:
  service: oya-plugin-app-store-plugin-catalog-rest
  objectives:
    - target: 0.95
      displayName: 95% of browse requests ≤ 200ms
  timeWindow:
    - duration: 30d
      isRolling: true
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
cargo check -p microservices/plugin-app-store/slos/*.openslo.yaml --all-features
cargo build -p microservices/plugin-app-store/slos/*.openslo.yaml --all-features
cargo clippy -p microservices/plugin-app-store/slos/*.openslo.yaml --all-features -- -D warnings
cargo nextest run -p microservices/plugin-app-store/slos/*.openslo.yaml --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p microservices/plugin-app-store/slos/*.openslo.yaml --no-deps
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
| `openslo-yaml-syntactic-valid` | Each manifest parses as OpenSLO v1 |
| `burn-rate-alert-policy-present` | Each manifest defines burn-rate alert |
| `observability-promotion-gate-consumes` | observability µservice ingests the manifest |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-013-observability-slo-manifests/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Manifest fails OpenSLO schema validation.
- observability promotion gate refuses the manifest.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-013-observability-slo-manifests`
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

[`IP-014-branch-protection-and-hyperscaler-gates`](IP-014-branch-protection-and-hyperscaler-gates.md)

## References

- ADR-0139
- ADR-0210
- OpenSLO v1 spec
