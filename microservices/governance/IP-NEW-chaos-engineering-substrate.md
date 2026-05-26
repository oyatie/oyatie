---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P15-resilience
impl_plan_id: IP-NEW-chaos-engineering-substrate
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability + council-architecture
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, chaos-engineering-catalog]
related_adrs:
  - ADR-0114
  - ADR-0121
  - ADR-0128
  - ADR-0139
  - ADR-0145
  - ADR-0148
  - ADR-0157
  - ADR-0158
  - ADR-0160
  - ADR-0165
related_crates:
  - oya-check-chaos-engineering-catalog
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md -->

# IP-NEW: wire `oya-check-chaos-engineering-catalog` into oya-dev-cli gate validate

## Intent

Activate the `oya-check-chaos-engineering-catalog` kernel (to be authored
at `crates/oya-check-chaos-engineering-catalog/`) as a fitness lane
`oya gate validate chaos-engineering-catalog`. The lane reads every
`microservices/<ms>/chaos/scenarios/*.yaml` and refuses the build when
a µservice that declares production SLOs lacks the minimum scenario set
required by ADR-0165 (pod-kill / network-delay-100ms / dependency-failure
/ disk-slow-1000ms / time-skew-30s where applicable).

The lane operationalizes ADR-0165 — Chaos Engineering Substrate (Chaos
Mesh 2.x). Without this CI gate, µservices can ship to production
without ever rehearsing the resilience invariants declared in ADR-0128
(INV-AT-LEAST-3-REPLICAS, INV-SHUFFLE-SHARDING) and ADR-0145 (retry +
circuit-breaker invariants).

## ChangeSet boundary

- Author `crates/oya-check-chaos-engineering-catalog/` kernel.
- Add as workspace dep of `oya-dev-cli`.
- Author `crates/oya-dev-cli/src/chaos_engineering_catalog_gate.rs`.
- Wire subcommand into `commands/gate/mod.rs`.
- Register in `AGGREGATED_VALIDATE_LANES`.
- Register in branch protection.
- Ship Chaos Mesh 2.x Helm chart skeleton at `microservices/cloud-iac/iac/helm/chaos-mesh/Chart.yaml`.
- Ship `.github/workflows/chaos-nightly.yml` workflow.
- Ship per-µservice minimum scenarios for the first 5 µservices (audit-chain,
  tenancy, api-gateway, cloud-k8s, foundry) as exemplar.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-check-chaos-engineering-catalog/Cargo.toml` | create |
| `crates/oya-check-chaos-engineering-catalog/src/lib.rs` | create — kernel + validator |
| `crates/oya-check-chaos-engineering-catalog/tests/catalog_validation.rs` | create — integration tests |
| `crates/oya-dev-cli/Cargo.toml` | edit — add dep |
| `crates/oya-dev-cli/src/chaos_engineering_catalog_gate.rs` | create — file-reading runner |
| `crates/oya-dev-cli/src/lib.rs` | edit — declare module |
| `crates/oya-dev-cli/src/commands/gate/mod.rs` | edit — add match arm |
| `crates/oya-governance-gate-catalog-domain/src/lib.rs` | edit — append `"chaos-engineering-catalog"` |
| `.github/branch-protection.yaml` | edit — add to dev required-status-checks |
| `microservices/governance/catalog/oya-check-chaos-engineering-catalog.yaml` | create — catalog entry |
| `microservices/cloud-iac/iac/helm/chaos-mesh/Chart.yaml` | create — Helm chart skeleton |
| `.github/workflows/chaos-nightly.yml` | create — nightly drill workflow |
| `microservices/audit-chain/chaos/scenarios/pod-kill.yaml` | create — exemplar |
| `microservices/tenancy/chaos/scenarios/pod-kill.yaml` | create — exemplar |
| `microservices/api-gateway/chaos/scenarios/network-delay-100ms.yaml` | create — exemplar |
| `microservices/cloud-k8s/chaos/scenarios/cross-cell-partition.yaml` | create — exemplar |
| `microservices/foundry/chaos/scenarios/dependency-failure-llm-provider.yaml` | create — exemplar |

## Validator shape

The `oya-check-chaos-engineering-catalog` kernel:

1. Reads every `microservices/<ms>/manifest.json#production_slos_declared`.
2. For each µservice with production SLOs, reads `microservices/<ms>/chaos/scenarios/*.yaml`.
3. Validates the minimum scenario set per ADR-0165:
   - `pod-kill` REQUIRED — kind: PodChaos.
   - `network-delay-100ms` REQUIRED — kind: NetworkChaos.
   - `dependency-failure-<dep>` REQUIRED per declared downstream — kind: HTTPChaos.
   - `disk-slow-1000ms` REQUIRED — kind: IOChaos.
   - `time-skew-30s` REQUIRED if `chaos_time_sensitive: true` — kind: TimeChaos.
4. Validates each scenario's `metrics:` block references an SLO query that
   exists in the µservice's `slos/` catalog.
5. Validates each scenario's target selector resolves to a real Deployment.
6. Returns `ChaosCatalogReport { microservices_checked, missing_scenarios,
   orphan_scenarios, slo_gate_missing }`.

## Acceptance gates

```bash
cargo check -p oya-dev-cli
cargo nextest run -p oya-check-chaos-engineering-catalog
cargo run -p oya-dev-cli -- gate validate chaos-engineering-catalog \
    --microservices-dir microservices
cargo run -p oya-dev-cli -- gate run-all
```

## Halt conditions

- Lane fires on existing µservices that have production SLOs but no chaos
  catalog → author the minimum scenario set before flipping the lane to
  BLOCKER. Order of operations:
  1. Land kernel + CLI gate as YELLOW (warn-only).
  2. Land exemplar scenarios across the 5 first µservices.
  3. Land Chaos Mesh Helm chart + per-cell installation.
  4. Land nightly workflow.
  5. Burn-in 30 days.
  6. Flip lane to BLOCKER once every µservice with production SLOs has the
     minimum scenario set.

## References

- ADR-0165 — Chaos Engineering Substrate (Chaos Mesh 2.x).
- Chaos Mesh documentation — https://chaos-mesh.org/docs/
- Netflix Simian Army — https://netflixtechblog.com/the-netflix-simian-army-16e57fbab116
- Google SRE Workbook Chapter 17 — Testing for Reliability.
- Principles of Chaos Engineering — https://principlesofchaos.org/
- `crates/oya-check-chaos-engineering-catalog` (to author).

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the explicit counterpart hook required by ADR-0328 D-20. Governance parity is evaluated against GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, and Renovate. The implementation must state which of those controls it closes or deliberately does not target before promotion.
