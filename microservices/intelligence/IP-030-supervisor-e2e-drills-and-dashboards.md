---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-015-e2e-drills-and-dashboards
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane + ops-sre-reliability
acceptance_lanes: [cargo-nextest, oya-check-dashboard-coverage, oya-check-runbook-coverage]
depends_on: [IP-014]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: End-to-end drills + Grafana dashboards

## Intent

Wire all AC-01..AC-10 acceptance criteria via e2e tests + Grafana dashboards. Validate phase exit gate. Quarterly chaos drills.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/intelligence/tests/e2e/kill_switch_latency.rs` | create (AC-02) |
| `microservices/intelligence/tests/e2e/canary_rollout_gated.rs` | create (AC-03) |
| `microservices/intelligence/tests/e2e/autonomy_level_refusal.rs` | create (AC-04) |
| `microservices/intelligence/tests/e2e/supervision_event_lag.rs` | create (AC-05) |
| `microservices/intelligence/tests/e2e/drain_no_loss.rs` | create (AC-06) |
| `microservices/intelligence/tests/e2e/postgres_failover.rs` | create (AC-07) |
| `microservices/intelligence/tests/e2e/redis_failover.rs` | create (AC-08) |
| `microservices/intelligence/dashboards/` | already created (Slice C; verify deployment + JSON conformance) |

## Acceptance Gates

```bash
# All e2e tests pass
cargo nextest run --workspace --test '*e2e*' --all-features

# Dashboards deployed + load
cargo run -p oya-dev-cli -- gate validate dashboard-coverage --microservice foundry-supervisor

# Runbook coverage check (every FM-ID has matching runbook)
cargo run -p oya-dev-cli -- gate validate runbook-coverage --microservice foundry-supervisor

# Full phase exit gate validation
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-supervisor
cargo run -p oya-dev-cli -- gate validate authority-cohesion # HG-FND-SUP registers green
```

## Drills

| Scenario | Frequency | Pass criterion |
|---|---|---|
| AC-02 kill-switch latency drill | per-release + quarterly chaos | p99 ≤ 1 s; p999 ≤ 2 s; sample 100k engages |
| AC-03 canary rollout gated | per-release | rollout pauses at held verdict; rolls back on rollback verdict |
| AC-04 autonomy tier refusal | per-release | Cedar denies; AutonomyViolated audit-chain seal |
| AC-06 drain no loss | per-release | drain completes; in-flight reach success |
| AC-07 Postgres failover chaos | quarterly | control-plane available ≤ 30 s |
| AC-08 Valkey failover chaos | quarterly | kill-switch p99 stays ≤ 1 s during failover |
| EU AI Act Art. 60 post-market monitoring | quarterly | supervision-event + audit-chain + dashboards verified end-to-end |

## Halt Conditions

- Any AC fails.
- Dashboard JSON conformance violation.
- Runbook missing for any FM-ID.

## Phase exit

PHASE-01 exit gate is achieved when:
- All 15 IPs merged.
- All ACs green.
- HG-FND-SUP registered green in `/specs/hyperscaler-gates.json`.
- `cargo nextest run --workspace --all-features` exits 0.
- Per-changeset evidence (multispectrum) committed.

## References

- PRD §"Acceptance Criteria".
- `PHASE-01-CONTROL-PLANE-LANDING.md` §"End-to-end drill gates".
- `failure-modes.md` (all FMs covered by runbooks).
- `dashboards/{deployment-rate,kill-switch-coverage,autonomy-violation-rate}.json`.
- ADR-0123 + ADR-0133.

## Wave 15 counterpart anchor

- Counterparts: Palantir AIP Operator, Azure AI Foundry deployments, and GitHub merge-queue controls.
- Gap closure: this IP closes fleet control, kill-switch propagation, and deployability evidence with tenant-scoped policy enforcement.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
